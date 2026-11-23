//! Source-only elaboration. The existing parametric checker and concrete checker
//! validate the result; checked artifacts never run inference.
use super::*;

fn unresolved() -> Diagnostic {
    action_error(
        "compile.generic.inference",
        "arguments do not determine every type parameter; supply explicit type arguments",
    )
}

fn substitute(ty: &Type, bindings: &BTreeMap<String, Type>) -> Fallible<Type> {
    fn check(
        ty: &Type,
        bindings: Option<&BTreeMap<String, Type>>,
        depth: usize,
        nodes: &mut usize,
    ) -> Fallible<()> {
        *nodes += 1;
        if depth > 8 || *nodes > 8192 {
            return Err(action_error(
                "resource.frontend.limit",
                "inferred type exceeds nesting or node budget",
            ));
        }
        if let Type::Record(id) = ty {
            if let Some(actual) = bindings.and_then(|b| b.get(id)) {
                return check(actual, None, depth, nodes);
            }
        }
        match ty {
            Type::List(t) => check(t, bindings, depth + 1, nodes)?,
            Type::Applied { arguments, .. } => {
                for t in arguments {
                    check(t, bindings, depth + 1, nodes)?;
                }
            }
            Type::Function(s) => {
                for t in &s.parameters {
                    check(t, bindings, depth + 1, nodes)?;
                }
                check(&s.output, bindings, depth + 1, nodes)?;
            }
            _ => {}
        }
        Ok(())
    }
    check(ty, Some(bindings), 0, &mut 0)?;
    Ok(copy_substituted(ty, bindings))
}

fn copy_substituted(ty: &Type, bindings: &BTreeMap<String, Type>) -> Type {
    match ty {
        Type::Record(id) if bindings.contains_key(id) => bindings[id].clone(),
        Type::List(element) => Type::List(Box::new(copy_substituted(element, bindings))),
        Type::Applied { name, arguments } => Type::Applied {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|ty| copy_substituted(ty, bindings))
                .collect(),
        },
        Type::Function(signature) => Type::Function(FunctionType {
            actions: signature.actions.clone(),
            parameters: signature
                .parameters
                .iter()
                .map(|ty| copy_substituted(ty, bindings))
                .collect(),
            output: Box::new(copy_substituted(&signature.output, bindings)),
        }),
        _ => ty.clone(),
    }
}

fn constrain(
    pattern: &Type,
    actual: &Type,
    parameters: &BTreeSet<String>,
    bindings: &mut BTreeMap<String, Type>,
) -> Fallible<()> {
    if let Type::Record(id) = pattern {
        if parameters.contains(id) {
            if let Some(previous) = bindings.get(id) {
                return same(actual.clone(), previous.clone());
            }
            bindings.insert(id.clone(), actual.clone());
            return Ok(());
        }
    }
    match (pattern, actual) {
        (Type::List(a), Type::List(b)) => constrain(a, b, parameters, bindings),
        (
            Type::Applied {
                name: a,
                arguments: xs,
            },
            Type::Applied {
                name: b,
                arguments: ys,
            },
        ) if a == b && xs.len() == ys.len() => {
            for (x, y) in xs.iter().zip(ys) {
                constrain(x, y, parameters, bindings)?;
            }
            Ok(())
        }
        (Type::Function(a), Type::Function(b))
            if a.parameters.len() == b.parameters.len() && a.actions == b.actions =>
        {
            for (x, y) in a.parameters.iter().zip(&b.parameters) {
                constrain(x, y, parameters, bindings)?;
            }
            constrain(&a.output, &b.output, parameters, bindings)
        }
        _ => same(actual.clone(), pattern.clone()),
    }
}

struct Elaborator<'a> {
    updates: usize,
    source: &'a Program,
    functions: BTreeMap<String, &'a Function>,
    context: &'a modules::Context,
    nodes: &'a mut usize,
}

struct Constructor<'a> {
    name: &'a str,
    parameters: &'a [String],
    variant: bool,
}

impl Elaborator<'_> {
    fn contextual_case(&self, name: &str, expected: Option<&Type>) -> Option<(String, bool)> {
        if self.context.aliases.contains_key(name) {
            return None;
        }
        let id = match expected? {
            Type::Applied { name, .. } | Type::Variant(name) => name,
            _ => return None,
        };
        let variant = self.source.variants.iter().find(|v| &v.id == id)?;
        let case = variant.cases.iter().find(|case| case.name == name)?;
        Some((id.clone(), case.payload.is_some()))
    }

    fn construct(
        &mut self,
        shape: Constructor<'_>,
        arguments: &mut Vec<Type>,
        parts: &mut [(Type, &mut Expr)],
        expected: Option<&Type>,
        env: &BTreeMap<String, Type>,
        depth: usize,
    ) -> Fallible<Option<Type>> {
        let Constructor {
            name,
            parameters,
            variant,
        } = shape;
        let infer = arguments.is_empty() && !parameters.is_empty();
        let variables: BTreeSet<_> = parameters.iter().map(|name| format!("@{name}")).collect();
        let mut bindings: BTreeMap<_, _> = parameters
            .iter()
            .map(|name| format!("@{name}"))
            .zip(arguments.iter().cloned())
            .collect();
        if infer {
            if let Some(expected @ (Type::Applied { .. } | Type::Variant(_) | Type::Record(_))) = expected {
                if !matches!(expected,Type::Record(id) if id.starts_with('@')) {
                    let pattern = Type::Applied {
                        name: name.into(),
                        arguments: parameters.iter().map(|p| Type::Record(format!("@{p}"))).collect(),
                    };
                    constrain(&pattern, expected, &variables, &mut bindings)?;
                }
            }
        }
        fn known(ty: &Type, variables: &BTreeSet<String>, bindings: &BTreeMap<String, Type>) -> bool {
            match ty {
                Type::Record(id) => !variables.contains(id) || bindings.contains_key(id),
                Type::List(t) => known(t, variables, bindings),
                Type::Applied { arguments, .. } => arguments.iter().all(|t| known(t, variables, bindings)),
                Type::Function(s) => {
                    s.parameters.iter().all(|t| known(t, variables, bindings))
                        && known(&s.output, variables, bindings)
                }
                _ => true,
            }
        }
        let mut deferred = vec![];
        for (pattern, value) in parts.iter_mut() {
            let contextual = known(pattern, &variables, &bindings);
            let expected = if contextual {
                Some(substitute(pattern, &bindings)?)
            } else {
                None
            };
            let named =
                matches!(value,Expr::Name{name,..} if !env.contains_key(name) && self.target(name).is_some());
            let actual = if !contextual && named {
                match value {
                    Expr::Name { name, .. } => self
                        .target(name)
                        .filter(|f| f.type_parameters.is_empty())
                        .map(|f| Type::Function(f.signature_type())),
                    _ => None,
                }
            } else {
                self.expr(value, env, expected.as_ref(), depth)?
            };
            deferred.push(actual.is_none() || (named && !contextual));
            if infer {
                if let Some(actual) = actual {
                    constrain(pattern, &actual, &variables, &mut bindings)?;
                }
            }
        }
        if infer {
            if parameters
                .iter()
                .any(|name| !bindings.contains_key(&format!("@{name}")))
            {
                return Ok(None);
            }
            *arguments = parameters
                .iter()
                .map(|name| bindings[&format!("@{name}")].clone())
                .collect();
            for ((pattern, value), deferred) in parts.iter_mut().zip(deferred) {
                if deferred {
                    let expected = substitute(pattern, &bindings)?;
                    self.expr(value, env, Some(&expected), depth)?;
                }
            }
        }
        let ty = if arguments.is_empty() {
            if variant {
                Type::Variant(name.into())
            } else {
                Type::Record(name.into())
            }
        } else {
            Type::Applied {
                name: name.into(),
                arguments: arguments.clone(),
            }
        };
        Ok(Some(substitute(&ty, &BTreeMap::new())?))
    }
    fn target(&self, name: &str) -> Option<&Function> {
        self.context
            .aliases
            .get(name)
            .and_then(|id| self.functions.get(id))
            .copied()
    }

    fn reference(
        &self,
        target: &Function,
        arguments: &mut Vec<Type>,
        expected: Option<&Type>,
    ) -> Fallible<Option<Type>> {
        let signature = Type::Function(target.signature_type());
        if target.type_parameters.is_empty() {
            return Ok(Some(signature));
        }
        let parameters: BTreeSet<_> = target
            .type_parameters
            .iter()
            .map(|name| format!("@{name}"))
            .collect();
        if arguments.is_empty() {
            let Some(expected @ Type::Function(_)) = expected else {
                return Ok(None);
            };
            let mut bindings = BTreeMap::new();
            constrain(&signature, expected, &parameters, &mut bindings)?;
            *arguments = target
                .type_parameters
                .iter()
                .map(|name| bindings.get(&format!("@{name}")).cloned().ok_or_else(unresolved))
                .collect::<Fallible<_>>()?;
        }
        if arguments.len() != parameters.len() {
            return Ok(None);
        }
        let bindings = target
            .type_parameters
            .iter()
            .map(|name| format!("@{name}"))
            .zip(arguments.iter().cloned())
            .collect();
        Ok(Some(substitute(&signature, &bindings)?))
    }

    fn block(
        &mut self,
        block: &mut Block,
        outer: &BTreeMap<String, Type>,
        expected: Option<&Type>,
        depth: usize,
    ) -> Fallible<Option<Type>> {
        let mut env = outer.clone();
        for binding in &mut block.bindings {
            let actual = self.expr(&mut binding.value, &env, binding.annotation.as_ref(), depth)?;
            if let Some(ty) = binding.annotation.clone().or(actual) {
                env.insert(binding.name.clone(), ty);
            }
        }
        self.expr(&mut block.value, &env, expected, depth)
    }

    fn member(&self, ty: &Type, field: &str, variant: bool) -> Fallible<Option<Type>> {
        let result = (|| {
            let (id, args) = match ty {
                Type::Applied { name, arguments } => (name, arguments.as_slice()),
                Type::Record(id) | Type::Variant(id) => (id, &[][..]),
                _ => return None,
            };
            let (parameters, member) = if variant {
                let shape = self.source.variants.iter().find(|v| &v.id == id)?;
                (
                    &shape.type_parameters,
                    shape.cases.iter().find(|c| c.name == field)?.payload.as_ref()?,
                )
            } else {
                let shape = self.source.records.iter().find(|r| &r.id == id)?;
                (
                    &shape.type_parameters,
                    &shape.fields.iter().find(|f| f.name == field)?.ty,
                )
            };
            let bindings = parameters
                .iter()
                .map(|name| format!("@{name}"))
                .zip(args.iter().cloned())
                .collect();
            Some(substitute(member, &bindings))
        })();
        result.transpose()
    }

    fn expr(
        &mut self,
        expr: &mut Expr,
        env: &BTreeMap<String, Type>,
        expected: Option<&Type>,
        depth: usize,
    ) -> Fallible<Option<Type>> {
        *self.nodes += 1;
        if depth >= MAX_DEPTH || *self.nodes > 262144 {
            return Err(action_error(
                "resource.frontend.limit",
                "call inference expression budget exceeded",
            )
            .at(expr.span()));
        }
        let span = expr.span();
        self.inner(expr, env, expected, depth + 1).map_err(|e| e.at(span))
    }

    fn inner(
        &mut self,
        expr: &mut Expr,
        env: &BTreeMap<String, Type>,
        expected: Option<&Type>,
        depth: usize,
    ) -> Fallible<Option<Type>> {
        let case = match expr {
            Expr::Name { name, .. } if !env.contains_key(name) => self
                .contextual_case(name, expected)
                .map(|(id, _)| (id, name.clone(), None)),
            Expr::Invoke {
                callee, arguments, ..
            } => match callee.as_ref() {
                Expr::Name { name, .. } if !env.contains_key(name) => {
                    if let Some((id, payload)) = self.contextual_case(name, expected) {
                        if arguments.len() != usize::from(payload) {
                            return Err(action_error(
                                "compile.call.arity",
                                "wrong constructor argument count",
                            ));
                        }
                        Some((id, name.clone(), arguments.pop().map(Box::new)))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if let Some((variant, case, payload)) = case {
            *expr = Expr::Variant {
                variant,
                case,
                payload,
                type_arguments: vec![],
                span: expr.span(),
            };
        }
        match expr {
            Expr::Name { name, span } => {
                if let Some(ty) = env.get(name) {
                    return Ok(Some(ty.clone()));
                }
                if matches!(expected, Some(Type::Function(_))) {
                    if let Some(target) = self.target(name).cloned() {
                        let mut type_arguments = vec![];
                        let ty = self.reference(&target, &mut type_arguments, expected)?;
                        *expr = Expr::Reference {
                            function: target.id,
                            type_arguments,
                            span: *span,
                        };
                        return Ok(ty);
                    }
                }
                Ok(None)
            }
            Expr::Reference {
                function,
                type_arguments,
                ..
            } => {
                let Some(target) = self.functions.get(function).copied() else {
                    return Ok(None);
                };
                self.reference(target, type_arguments, expected)
            }
            Expr::Call {
                function,
                type_arguments,
                arguments,
                ..
            } => {
                let Some(target) = self.functions.get(function).copied() else {
                    for arg in arguments {
                        self.expr(arg, env, None, depth)?;
                    }
                    return Ok(None);
                };
                let infer = !target.type_parameters.is_empty() && type_arguments.is_empty();
                if infer && arguments.len() != target.parameters.len() {
                    return Err(action_error("compile.call.arity", "wrong number of arguments"));
                }
                let parameters: BTreeSet<_> = target
                    .type_parameters
                    .iter()
                    .map(|name| format!("@{name}"))
                    .collect();
                let mut bindings: BTreeMap<_, _> = target
                    .type_parameters
                    .iter()
                    .map(|name| format!("@{name}"))
                    .zip(type_arguments.iter().cloned())
                    .collect();
                let mut deferred = Vec::with_capacity(arguments.len());
                for (index, arg) in arguments.iter_mut().enumerate() {
                    if !infer {
                        let expected = target
                            .parameters
                            .get(index)
                            .map(|p| substitute(&p.ty, &bindings))
                            .transpose()?;
                        self.expr(arg, env, expected.as_ref(), depth)?;
                        continue;
                    }
                    let named_function = matches!(arg,Expr::Name{name,..} if !env.contains_key(name) && self.target(name).is_some());
                    let actual = if let Expr::Name { name, .. } = arg {
                        if !env.contains_key(name) {
                            self.target(name)
                                .filter(|f| f.type_parameters.is_empty())
                                .map(|f| Type::Function(f.signature_type()))
                        } else {
                            env.get(name).cloned()
                        }
                    } else {
                        self.expr(arg, env, None, depth)?
                    };
                    deferred.push(named_function || actual.is_none());
                    if infer {
                        if let Some(actual) = actual {
                            constrain(&target.parameters[index].ty, &actual, &parameters, &mut bindings)
                                .map_err(|e| {
                                    if e.code == "compile.action.reference" {
                                        e.at(arg.span())
                                    } else {
                                        e
                                    }
                                })?;
                        }
                    }
                }
                if infer {
                    *type_arguments = target
                        .type_parameters
                        .iter()
                        .map(|name| bindings.get(&format!("@{name}")).cloned().ok_or_else(unresolved))
                        .collect::<Fallible<_>>()?;
                }
                if infer {
                    for ((arg, parameter), deferred) in
                        arguments.iter_mut().zip(&target.parameters).zip(deferred)
                    {
                        if deferred {
                            let ty = substitute(&parameter.ty, &bindings)?;
                            self.expr(arg, env, Some(&ty), depth)?;
                        }
                    }
                }
                Ok(Some(substitute(&target.output, &bindings)?))
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                let signature = self.expr(callee, env, None, depth)?;
                for (index, arg) in arguments.iter_mut().enumerate() {
                    let expected = match &signature {
                        Some(Type::Function(s)) => s.parameters.get(index),
                        _ => None,
                    };
                    self.expr(arg, env, expected, depth)?;
                }
                Ok(match signature {
                    Some(Type::Function(s)) => Some(*s.output),
                    _ => None,
                })
            }
            Expr::Literal { value, .. } => Ok(Some(value.ty())),
            Expr::List { element, .. } => Ok(Some(Type::List(Box::new(element.clone())))),
            Expr::Record {
                record,
                type_arguments,
                fields,
                base,
                span,
            } => {
                let Some(shape) = self.source.records.iter().find(|r| &r.id == record).cloned() else {
                    return Ok(None);
                };
                let updating = base.is_some();
                if updating {
                    let mut seen = BTreeSet::new();
                    for field in fields.iter() {
                        if !shape.fields.iter().any(|member| member.name == field.name)
                            || !seen.insert(&field.name)
                        {
                            return Err(records::field_error(
                                "record update has an unknown or repeated field",
                            )
                            .at(field.value.span()));
                        }
                    }
                }
                let mut parts = vec![];
                if let Some(value) = base.as_deref_mut() {
                    let pattern = if shape.type_parameters.is_empty() {
                        Type::Record(record.clone())
                    } else {
                        Type::Applied {
                            name: record.clone(),
                            arguments: shape
                                .type_parameters
                                .iter()
                                .map(|p| Type::Record(format!("@{p}")))
                                .collect(),
                        }
                    };
                    parts.push((pattern, value));
                }
                for field in fields.iter_mut() {
                    if let Some(member) = shape.fields.iter().find(|member| member.name == field.name) {
                        parts.push((member.ty.clone(), &mut field.value));
                    } else {
                        self.expr(&mut field.value, env, None, depth)?;
                    }
                }
                let output = self.construct(
                    Constructor {
                        name: record,
                        parameters: &shape.type_parameters,
                        variant: false,
                    },
                    type_arguments,
                    &mut parts,
                    expected,
                    env,
                    depth,
                )?;
                drop(parts);
                if !updating {
                    return Ok(output);
                }
                let ty = output.ok_or_else(unresolved)?;
                let anchor = self.updates;
                self.updates += 1;
                let owner = format!("__lobo_update_base_{anchor}");
                let mut bindings = vec![];
                let mut replacements = BTreeMap::new();
                for (index, field) in std::mem::take(fields).into_iter().enumerate() {
                    let name = format!("__lobo_update_field_{anchor}_{index}");
                    replacements.insert(field.name, (name.clone(), field.value.span()));
                    bindings.push(Binding {
                        name,
                        annotation: None,
                        span: field.value.span(),
                        value: field.value,
                    });
                }
                let members = shape
                    .fields
                    .iter()
                    .map(|field| FieldValue {
                        name: field.name.clone(),
                        value: if let Some((name, location)) = replacements.get(&field.name) {
                            Expr::Name {
                                name: name.clone(),
                                span: *location,
                            }
                        } else {
                            Expr::Field {
                                base: Box::new(Expr::Name {
                                    name: owner.clone(),
                                    span: *span,
                                }),
                                field: field.name.clone(),
                                span: *span,
                            }
                        },
                    })
                    .collect();
                let value = Expr::Record {
                    record: record.clone(),
                    type_arguments: type_arguments.clone(),
                    fields: members,
                    base: None,
                    span: *span,
                };
                // One existing fold iteration owns the base; override bindings preserve source order.
                *expr = Expr::Fold {
                    index: format!("__lobo_update_index_{anchor}"),
                    accumulator: Parameter {
                        name: owner,
                        ty: ty.clone(),
                        span: *span,
                    },
                    count: Box::new(Expr::Literal {
                        value: Value::I64(1),
                        span: *span,
                    }),
                    initial: base.take().unwrap(),
                    body: Block {
                        bindings,
                        value: Box::new(value),
                    },
                    span: *span,
                    index_span: None,
                };
                Ok(Some(ty))
            }
            Expr::Variant {
                variant,
                type_arguments,
                case,
                payload,
                ..
            } => {
                let Some(shape) = self.source.variants.iter().find(|v| &v.id == variant).cloned() else {
                    return Ok(None);
                };
                let mut parts = vec![];
                if let Some(value) = payload {
                    if let Some(ty) = shape
                        .cases
                        .iter()
                        .find(|member| &member.name == case)
                        .and_then(|member| member.payload.as_ref())
                    {
                        parts.push((ty.clone(), value.as_mut()));
                    } else {
                        self.expr(value, env, None, depth)?;
                    }
                }
                self.construct(
                    Constructor {
                        name: variant,
                        parameters: &shape.type_parameters,
                        variant: true,
                    },
                    type_arguments,
                    &mut parts,
                    expected,
                    env,
                    depth,
                )
            }
            Expr::Field { base, field, .. } => match self.expr(base, env, None, depth)? {
                Some(ty) => self.member(&ty, field, false),
                None => Ok(None),
            },
            Expr::Match { subject, arms, .. } => {
                let subject = self.expr(subject, env, None, depth)?;
                let mut output = None;
                for arm in arms {
                    let mut inner = env.clone();
                    if let (Some(name), Some(ty)) = (&arm.binding, &subject) {
                        if let Some(payload) = self.member(ty, &arm.case, true)? {
                            inner.insert(name.clone(), payload);
                        }
                    }
                    let ty = self.block(&mut arm.body, &inner, expected, depth)?;
                    if output.is_none() {
                        output = ty;
                    }
                }
                Ok(output)
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.expr(condition, env, None, depth)?;
                let left = self.block(then_block, env, expected, depth)?;
                let right = self.block(else_block, env, expected, depth)?;
                Ok(if left == right { left } else { None })
            }
            Expr::Fold {
                index,
                accumulator,
                count,
                initial,
                body,
                ..
            } => {
                self.expr(count, env, None, depth)?;
                self.expr(initial, env, Some(&accumulator.ty), depth)?;
                let mut inner = env.clone();
                inner.insert(index.clone(), Type::I64);
                inner.insert(accumulator.name.clone(), accumulator.ty.clone());
                self.block(body, &inner, Some(&accumulator.ty), depth)?;
                Ok(Some(accumulator.ty.clone()))
            }
            Expr::While {
                state,
                initial,
                condition,
                body,
                ..
            } => {
                self.expr(initial, env, Some(&state.ty), depth)?;
                let mut inner = env.clone();
                inner.insert(state.name.clone(), state.ty.clone());
                self.expr(condition, &inner, None, depth)?;
                self.block(body, &inner, Some(&state.ty), depth)?;
                Ok(Some(state.ty.clone()))
            }
            Expr::ListOp { op, arguments, .. } => {
                let mut last = None;
                for arg in arguments {
                    last = self.expr(arg, env, None, depth)?;
                }
                Ok(match op.as_str() {
                    "length" => Some(Type::I64),
                    "head" => match last {
                        Some(Type::List(t)) => Some(*t),
                        _ => None,
                    },
                    _ => last,
                })
            }
            Expr::NativeOp {
                binding,
                abi,
                arguments,
                ..
            } => {
                let signature = native_api::signature(binding, *abi)?;
                for (n, arg) in arguments.iter_mut().enumerate() {
                    self.expr(arg, env, signature.parameters.get(n), depth)?;
                }
                Ok(Some(signature.output))
            }
            Expr::BufferOp {
                family,
                op,
                arguments,
                ..
            } => {
                for arg in arguments {
                    self.expr(arg, env, None, depth)?;
                }
                Ok(buffers::signature(family, op).ok().map(|(_, output)| output))
            }
            Expr::Unary { op, operand, .. } => {
                let ty = self.expr(operand, env, None, depth)?;
                if op == "try" {
                    return match ty {
                        Some(ty) => self.member(&ty, "Ok", true),
                        None => Ok(None),
                    };
                }
                Ok(if op == "!" { Some(Type::Bool) } else { ty })
            }
            Expr::Binary { op, left, right, .. } => {
                let a = self.expr(left, env, None, depth)?;
                let b = self.expr(right, env, None, depth)?;
                if ["==", "!=", "<", "<=", ">", ">=", "&&", "||"].contains(&op.as_str()) {
                    return Ok(Some(Type::Bool));
                }
                let binding = self.source.operators.iter().find(|binding| {
                    binding.symbol == *op
                        && Some(&binding.left) == a.as_ref()
                        && Some(&binding.right) == b.as_ref()
                        && self.context.visible.contains(&binding.target)
                });
                Ok(binding
                    .map(|binding| binding.output.clone())
                    .or(if a == b { a } else { None }))
            }
        }
    }
}

pub(super) fn elaborate(program: &mut Program, enabled: &BTreeSet<String>) -> Fallible<()> {
    let source = program.clone();
    let contexts = modules::contexts(&source)?;
    let mut nodes = 0;
    for function in &mut program.functions {
        if !enabled.contains(&function.id) {
            continue;
        }
        let mut elaborator = Elaborator {
            updates: 0,
            source: &source,
            functions: source.functions.iter().map(|f| (f.id.clone(), f)).collect(),
            context: &contexts[&function.id],
            nodes: &mut nodes,
        };
        let env = function
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let result = (|| {
            for requirement in &mut function.requires {
                elaborator.expr(requirement, &env, None, 0)?;
            }
            elaborator.block(&mut function.body, &env, Some(&function.output), 0)?;
            let mut post = env;
            post.insert("result".into(), function.output.clone());
            for assurance in &mut function.ensures {
                elaborator.expr(assurance, &post, None, 0)?;
            }
            Ok(())
        })();
        result.map_err(|mut e: Diagnostic| {
            e.subject = Some(function.id.clone());
            e
        })?;
    }
    Ok(())
}
