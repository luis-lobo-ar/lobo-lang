use super::*;

#[derive(Debug, Clone)]
pub(super) struct Binding {
    pub module: String,
    pub symbol: String,
    pub left: Type,
    pub right: Type,
    pub output: Type,
    pub target: String,
    pub span: Option<ByteSpan>,
    pub target_span: Option<ByteSpan>,
}

pub(super) fn eligible(symbol: &str) -> bool {
    matches!(symbol, "+" | "-" | "*" | "/" | "%")
}

fn concrete(ty: &Type, program: &Program) -> bool {
    match ty {
        Type::Applied { .. } | Type::Parameter(_) => false,
        Type::Record(id) => program
            .records
            .iter()
            .any(|r| &r.id == id && r.type_parameters.is_empty()),
        Type::Variant(id) => program
            .variants
            .iter()
            .any(|v| &v.id == id && v.type_parameters.is_empty()),
        Type::List(element) => concrete(element, program),
        Type::Function(f) => {
            f.parameters.iter().all(|t| concrete(t, program)) && concrete(&f.output, program)
        }
        _ => true,
    }
}

pub(super) fn check_bindings(program: &Program) -> Fallible<()> {
    if program.operators.len() > 64 {
        return Err(action_error(
            "resource.frontend.limit",
            "at most 64 operator bindings per bundle",
        ));
    }
    let mut signatures = BTreeSet::new();
    for binding in &program.operators {
        let checked = (|| {
            let scope = program
                .scopes
                .iter()
                .find(|s| s.id == binding.module)
                .ok_or_else(|| action_error("compile.operator.ownership", "unknown binding module"))?;
            let owns_operand = [&binding.left, &binding.right].iter().any(|ty| match ty {
                Type::Record(id) | Type::Variant(id) => scope.declarations.contains(id),
                _ => false,
            });
            if !owns_operand {
                return Err(action_error(
                    "compile.operator.ownership",
                    "binding module must declare an operand's nominal type",
                ));
            }
            if ![&binding.left, &binding.right, &binding.output]
                .iter()
                .all(|t| concrete(t, program))
            {
                return Err(action_error(
                    "compile.operator.signature",
                    "operator bindings require concrete types without generic applications",
                ));
            }
            let target = program
                .functions
                .iter()
                .find(|f| f.id == binding.target)
                .ok_or_else(|| {
                    action_error("compile.operator.signature", "operator target must be a function")
                })?;
            if !scope.declarations.contains(&target.id) {
                return Err(action_error(
                    "compile.operator.ownership",
                    "operator target must belong to the binding module",
                ));
            }
            if !target.actions.is_empty() {
                return Err(action_error(
                    "compile.operator.purity",
                    "operator target must be pure",
                ));
            }
            if !target.type_parameters.is_empty()
                || target.parameters.len() != 2
                || target.parameters[0].ty != binding.left
                || target.parameters[1].ty != binding.right
                || target.output != binding.output
            {
                return Err(action_error(
                    "compile.operator.signature",
                    "operator target signature must match exactly",
                ));
            }
            if !signatures.insert((&binding.symbol, &binding.left, &binding.right)) {
                return Err(action_error(
                    "compile.operator.duplicate",
                    "repeated symbol and ordered operand types",
                ));
            }
            Ok(())
        })();
        checked.map_err(|mut e: Diagnostic| {
            e.subject = Some(binding.module.clone());
            e.at(binding.span)
        })?;
    }
    Ok(())
}

// Lower after specialization, including the rigid-type definition-checking pass.
// Checked artifacts and native backends consume ordinary calls and buffer operations.
pub(super) fn lower(program: &mut Program, abstract_types: bool) -> Fallible<()> {
    let contexts = modules::contexts(program)?;
    let signatures = program.functions.clone();
    let mut checker = Checker {
        functions: signatures.iter().map(|f| (f.id.clone(), f)).collect(),
        names: BTreeSet::new(),
        visible: BTreeSet::new(),
        depth: 0,
        actions: BTreeSet::new(),
        in_contract: false,
        output: Type::I64,
        records: records::RecordTable::new_mode(&program.records, &program.variants, true, abstract_types)?,
        record_cells: 0,
    };
    for function in &mut program.functions {
        if abstract_types && !function.id.starts_with("lobo.generic.") {
            continue;
        }
        checker.output = function.output.clone();
        checker.visible = contexts[&function.id].visible.clone();
        checker.names = contexts[&function.id].aliases.keys().cloned().collect();
        checker.actions = function.actions.iter().cloned().collect();
        let env = function
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let mut lower = Lower {
            checker: &mut checker,
            bindings: &program.operators,
        };
        let result = (|| {
            lower.checker.in_contract = true;
            for expression in &mut function.requires {
                lower.expr(expression, &env, 0)?;
            }
            lower.checker.in_contract = false;
            lower.block(&mut function.body, &env, 0)?;
            let mut post = env;
            post.insert("result".into(), function.output.clone());
            lower.checker.in_contract = true;
            for expression in &mut function.ensures {
                lower.expr(expression, &post, 0)?;
            }
            Ok(())
        })();
        result.map_err(|mut e: Diagnostic| {
            e.subject = Some(function.id.clone());
            e
        })?;
    }
    program.operators.clear();
    Ok(())
}

struct Lower<'a, 'b> {
    checker: &'a mut Checker<'b>,
    bindings: &'a [Binding],
}

impl Lower<'_, '_> {
    fn infer(&mut self, expression: &Expr, env: &BTreeMap<String, Type>) -> Fallible<Type> {
        // These queries determine source call targets. The final complete checker
        // independently enforces per-function temporary, scope and action budgets.
        self.checker.record_cells = 0;
        self.checker.depth = 0;
        self.checker.expr(expression, env)
    }

    fn block(&mut self, block: &mut Block, outer: &BTreeMap<String, Type>, depth: usize) -> Fallible<()> {
        let mut env = outer.clone();
        for binding in &mut block.bindings {
            self.expr(&mut binding.value, &env, depth)?;
            let ty = self.infer(&binding.value, &env)?;
            env.insert(binding.name.clone(), ty);
        }
        self.expr(&mut block.value, &env, depth)
    }

    fn expr(&mut self, expression: &mut Expr, env: &BTreeMap<String, Type>, depth: usize) -> Fallible<()> {
        if depth >= MAX_DEPTH {
            return Err(action_error(
                "resource.frontend.limit",
                "operator lowering expression nesting limit exceeded",
            )
            .at(expression.span()));
        }
        let depth = depth + 1;
        let span = expression.span();
        let result = (|| {
            match expression {
                Expr::Binary { op, left, right, .. } => {
                    self.expr(left, env, depth)?;
                    self.expr(right, env, depth)?;
                    if eligible(op) {
                        let a = self.infer(left, env)?;
                        let b = self.infer(right, env)?;
                        if op == "+" && a == Type::Text && b == Type::Text {
                            let arguments = vec![(**left).clone(), (**right).clone()];
                            *expression = Expr::BufferOp {
                                family: "Text".into(),
                                op: "concat".into(),
                                arguments,
                                span,
                            };
                        } else if let Some(binding) = self.bindings.iter().find(|binding| {
                            binding.symbol == *op
                                && binding.left == a
                                && binding.right == b
                                && self.checker.visible.contains(&binding.target)
                        }) {
                            *expression = Expr::Call {
                                function: binding.target.clone(),
                                type_arguments: vec![],
                                arguments: vec![(**left).clone(), (**right).clone()],
                                span,
                                name_span: None,
                            };
                        }
                    }
                }
                Expr::Unary { operand, .. } => self.expr(operand, env, depth)?,
                Expr::Field { base, .. } => self.expr(base, env, depth)?,
                Expr::Call { arguments, .. }
                | Expr::NativeOp { arguments, .. }
                | Expr::BufferOp { arguments, .. }
                | Expr::ListOp { arguments, .. } => {
                    for argument in arguments {
                        self.expr(argument, env, depth)?;
                    }
                }
                Expr::Invoke {
                    callee, arguments, ..
                } => {
                    self.expr(callee, env, depth)?;
                    for argument in arguments {
                        self.expr(argument, env, depth)?;
                    }
                }
                Expr::Record { fields, .. } => {
                    for field in fields {
                        self.expr(&mut field.value, env, depth)?;
                    }
                }
                Expr::Variant { payload, .. } => {
                    if let Some(payload) = payload {
                        self.expr(payload, env, depth)?;
                    }
                }
                Expr::Match { subject, arms, .. } => {
                    self.expr(subject, env, depth)?;
                    let ty = self.infer(subject, env)?;
                    let Type::Variant(id) = ty else {
                        return Err(action_error("compile.match.invalid", "match requires a variant"));
                    };
                    let definition = self.checker.records.get_variant(&id)?;
                    if arms.len() != definition.cases.len() {
                        return Err(action_error(
                            "compile.match.invalid",
                            "match must cover each case exactly once",
                        ));
                    }
                    let cases = definition.cases.clone();
                    let mut seen = BTreeSet::new();
                    for arm in arms {
                        let mut inner = env.clone();
                        let case = cases.iter().find(|c| c.name == arm.case).ok_or_else(|| {
                            action_error("compile.match.invalid", "unknown match case").at(arm.span)
                        })?;
                        if !seen.insert(arm.case.clone()) || case.payload.is_some() != arm.binding.is_some() {
                            return Err(action_error(
                                "compile.match.invalid",
                                "duplicate case or wrong binding presence",
                            )
                            .at(arm.span));
                        }
                        if let (Some(name), Some(payload)) = (&arm.binding, &case.payload) {
                            inner.insert(name.clone(), payload.clone());
                        }
                        self.block(&mut arm.body, &inner, depth)?;
                    }
                }
                Expr::If {
                    condition,
                    then_block,
                    else_block,
                    ..
                } => {
                    self.expr(condition, env, depth)?;
                    self.block(then_block, env, depth)?;
                    self.block(else_block, env, depth)?;
                }
                Expr::Fold {
                    count,
                    initial,
                    index,
                    accumulator,
                    body,
                    ..
                } => {
                    self.expr(count, env, depth)?;
                    self.expr(initial, env, depth)?;
                    let mut inner = env.clone();
                    inner.insert(index.clone(), Type::I64);
                    inner.insert(accumulator.name.clone(), accumulator.ty.clone());
                    self.block(body, &inner, depth)?;
                }
                Expr::While {
                    initial,
                    condition,
                    state,
                    body,
                    ..
                } => {
                    self.expr(initial, env, depth)?;
                    let mut inner = env.clone();
                    inner.insert(state.name.clone(), state.ty.clone());
                    self.expr(condition, &inner, depth)?;
                    self.block(body, &inner, depth)?;
                }
                Expr::Literal { .. } | Expr::Name { .. } | Expr::List { .. } | Expr::Reference { .. } => (),
            }
            Ok(())
        })();
        result.map_err(|e: Diagnostic| e.at(span))
    }
}
