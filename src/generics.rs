use super::*;

const PREFIX: &str = "lobo.generic.";
const MAX_INSTANCES: usize = 256;
const MAX_TYPE_INSTANCES: usize = 128;
const MAX_NODES: usize = 262144;

#[derive(Clone)]
enum Nominal {
    Record(Record),
    Variant(Variant),
}
impl Nominal {
    fn id(&self) -> &String {
        match self {
            Self::Record(r) => &r.id,
            Self::Variant(v) => &v.id,
        }
    }
    fn parameters(&self) -> &[String] {
        match self {
            Self::Record(r) => &r.type_parameters,
            Self::Variant(v) => &v.type_parameters,
        }
    }
    fn specialize(&mut self, id: &str) -> Vec<String> {
        let name = format!("__lobo_generic_{}", id[PREFIX.len()..].replace('.', "_"));
        match self {
            Self::Record(r) => {
                r.id = id.into();
                r.name = name;
                std::mem::take(&mut r.type_parameters)
            }
            Self::Variant(v) => {
                v.id = id.into();
                v.name = name;
                std::mem::take(&mut v.type_parameters)
            }
        }
    }
}
struct PendingType {
    definition: Nominal,
    context: modules::Context,
    substitutions: BTreeMap<String, Type>,
    origin: Option<String>,
}

struct Pending {
    function: Function,
    context: modules::Context,
    substitutions: BTreeMap<String, Type>,
    origin: Option<String>,
}

struct Expansion {
    templates: BTreeMap<String, Function>,
    type_templates: BTreeMap<String, Nominal>,
    type_requests: BTreeMap<String, (String, Vec<Type>)>,
    pending_types: std::collections::VecDeque<PendingType>,
    contexts: BTreeMap<String, std::rc::Rc<modules::Context>>,
    requests: BTreeMap<String, (String, Vec<Type>)>,
    pending: std::collections::VecDeque<Pending>,
    nodes: usize,
    source_names: BTreeSet<String>,
    type_names: BTreeMap<String, bool>,
}

fn limit(message: &str) -> Diagnostic {
    action_error("resource.frontend.limit", message)
}

// A deterministic non-cryptographic identity digest. The full key is retained and
// compared on every hit, so a collision fails closed rather than aliases code.
fn identity(key: &str) -> String {
    let mut hash = 0x6c62272e07bb014262b821756295c58du128;
    for byte in key.bytes() {
        hash ^= u128::from(byte);
        hash = hash.wrapping_mul(0x0000000001000000000000000000013b);
    }
    format!("{PREFIX}{hash:032x}")
}

impl Expansion {
    fn request(&mut self, function: &str, arguments: &[Type]) -> Fallible<String> {
        let Some(template) = self.templates.get(function) else {
            if arguments.is_empty() {
                return Ok(function.into());
            }
            return Err(action_error(
                "compile.generic.arguments",
                "ordinary function takes no type arguments",
            ));
        };
        if template.type_parameters.len() != arguments.len() {
            return Err(action_error(
                "compile.generic.arguments",
                "explicit type arguments must match the function's type parameters",
            ));
        }
        for argument in arguments {
            self.check_argument(argument)?;
        }
        let key = (function.to_string(), arguments.to_vec());
        let id = identity(&serde_json::to_string(&key).expect("type keys serialize"));
        if let Some(previous) = self.requests.get(&id) {
            if previous != &key {
                return Err(action_error(
                    "compile.identity.duplicate",
                    "generic specialization identity collision",
                ));
            }
            return Ok(id);
        }
        if self.requests.len() == MAX_INSTANCES {
            return Err(limit("at most 256 function specializations per bundle"));
        }
        let mut instance = template.clone();
        instance.id = id.clone();
        instance.name = format!("__lobo_generic_{}", &id[PREFIX.len()..]);
        let substitutions = std::mem::take(&mut instance.type_parameters)
            .into_iter()
            .zip(arguments.iter().cloned())
            .map(|(name, ty)| (format!("@{name}"), ty))
            .collect();
        self.pending.push_back(Pending {
            function: instance,
            context: (*self.contexts[function]).clone(),
            substitutions,
            origin: Some(function.into()),
        });
        self.requests.insert(id.clone(), key);
        Ok(id)
    }

    fn block(
        &mut self,
        block: &mut Block,
        types: &BTreeMap<String, Type>,
        needed: &mut BTreeSet<String>,
        depth: usize,
    ) -> Fallible<()> {
        for binding in &mut block.bindings {
            self.binding(&binding.name).map_err(|e| e.at(binding.span))?;
            if let Some(ty) = &mut binding.annotation {
                self.ty(ty, types, needed, 0)?;
            }
            self.expr(&mut binding.value, types, needed, depth)?;
        }
        self.expr(&mut block.value, types, needed, depth)
    }

    fn binding(&self, name: &str) -> Fallible<()> {
        if self.source_names.contains(name) {
            return Err(action_error(
                "compile.name.duplicate",
                "binding shadows a source declaration or type parameter",
            ));
        }
        Ok(())
    }

    fn check_argument(&self, ty: &Type) -> Fallible<()> {
        let valid = match ty {
            Type::Function(signature) => {
                for parameter in &signature.parameters {
                    self.check_argument(parameter)?;
                }
                return self.check_argument(&signature.output);
            }
            Type::List(element) => return self.check_argument(element),
            Type::Record(id) => self.type_names.get(id) == Some(&false),
            Type::Variant(id) => self.type_names.get(id) == Some(&true),
            Type::Applied { .. } => false,
            _ => true,
        };
        if !valid {
            return Err(action_error(
                "compile.type.unknown",
                "generic type argument must name a value type",
            ));
        }
        Ok(())
    }

    fn request_type(&mut self, name: &str, arguments: &[Type]) -> Fallible<Type> {
        let Some(template) = self.type_templates.get(name) else {
            if !arguments.is_empty() || !self.type_names.contains_key(name) {
                return Err(action_error(
                    "compile.generic.arguments",
                    "type takes no type arguments",
                ));
            }
            return Ok(if self.type_names[name] {
                Type::Variant(name.into())
            } else {
                Type::Record(name.into())
            });
        };
        if template.parameters().len() != arguments.len() {
            return Err(action_error(
                "compile.generic.arguments",
                "explicit type arguments must match the type's parameters",
            ));
        }
        for argument in arguments {
            self.check_argument(argument)?;
        }
        let key = (name.to_string(), arguments.to_vec());
        let id = format!(
            "{}type.{}",
            PREFIX,
            &identity(&serde_json::to_string(&key).expect("type keys serialize"))[PREFIX.len()..]
        );
        if let Some(previous) = self.type_requests.get(&id) {
            if previous != &key {
                return Err(action_error(
                    "compile.identity.duplicate",
                    "generic type identity collision",
                ));
            }
        } else {
            if self.type_requests.len() == MAX_TYPE_INSTANCES {
                return Err(limit("at most 128 nominal specializations per bundle"));
            }
            let mut definition = template.clone();
            let substitutions = definition
                .specialize(&id)
                .into_iter()
                .zip(arguments.iter().cloned())
                .map(|(name, ty)| (format!("@{name}"), ty))
                .collect();
            self.type_names
                .insert(id.clone(), matches!(definition, Nominal::Variant(_)));
            self.pending_types.push_back(PendingType {
                definition,
                context: (*self.contexts[name]).clone(),
                substitutions,
                origin: Some(name.into()),
            });
            self.type_requests.insert(id.clone(), key);
        }
        Ok(if self.type_names[&id] {
            Type::Variant(id)
        } else {
            Type::Record(id)
        })
    }

    fn ty(
        &mut self,
        ty: &mut Type,
        types: &BTreeMap<String, Type>,
        needed: &mut BTreeSet<String>,
        depth: usize,
    ) -> Fallible<()> {
        if depth > 8 {
            return Err(limit("specialized type nesting limit exceeded"));
        }
        if let Type::Record(id) = ty {
            if let Some(concrete) = types.get(id) {
                *ty = concrete.clone();
            }
        }
        match ty {
            Type::Function(signature) => {
                for parameter in &mut signature.parameters {
                    self.ty(parameter, types, needed, depth + 1)?;
                }
                self.ty(&mut signature.output, types, needed, depth + 1)?;
            }
            Type::List(element) => self.ty(element, types, needed, depth + 1)?,
            Type::Applied { name, arguments } => {
                for argument in arguments.iter_mut() {
                    self.ty(argument, types, needed, depth + 1)?;
                }
                *ty = self.request_type(name, arguments)?;
                self.ty(ty, types, needed, depth)?;
            }
            Type::Record(id) | Type::Variant(id) => {
                if self.type_templates.contains_key(id) {
                    return Err(action_error(
                        "compile.generic.arguments",
                        "generic type needs explicit type arguments",
                    ));
                }
                needed.insert(id.clone());
            }
            _ => (),
        }
        Ok(())
    }

    fn constructor(
        &mut self,
        name: &mut String,
        arguments: &mut Vec<Type>,
        types: &BTreeMap<String, Type>,
        needed: &mut BTreeSet<String>,
    ) -> Fallible<()> {
        for argument in arguments.iter_mut() {
            self.ty(argument, types, needed, 0)?;
        }
        let ty = self.request_type(name, arguments)?;
        *name = match ty {
            Type::Record(id) | Type::Variant(id) => id,
            _ => unreachable!(),
        };
        needed.insert(name.clone());
        arguments.clear();
        Ok(())
    }

    fn is_template(&self, id: &str) -> bool {
        self.templates.contains_key(id) || self.type_templates.contains_key(id)
    }

    fn attach_scope(
        &self,
        program: &mut Program,
        id: &str,
        context: modules::Context,
        generated: bool,
        mut needed: BTreeSet<String>,
    ) {
        if generated {
            let mut imports: Vec<_> = context
                .aliases
                .into_iter()
                .filter(|(_, id)| !self.is_template(id))
                .map(|(name, id)| modules::Import { name, id, span: None })
                .collect();
            needed.remove(id);
            for (n, target) in needed.into_iter().enumerate() {
                if !imports.iter().any(|i| i.id == target) {
                    imports.push(modules::Import {
                        name: format!("__lobo_type_{n}"),
                        id: target,
                        span: None,
                    });
                }
            }
            program.scopes.push(modules::Scope {
                name: format!("__lobo_scope_{}", program.scopes.len()),
                id: format!("{id}.module"),
                declarations: vec![id.into()],
                imports,
            });
        } else {
            let scope = program
                .scopes
                .iter_mut()
                .find(|s| s.declarations.iter().any(|d| d == id))
                .expect("source declaration has scope");
            for target in needed.into_iter().filter(|i| i.starts_with(PREFIX)) {
                if !scope.imports.iter().any(|i| i.id == target) {
                    scope.imports.push(modules::Import {
                        name: format!("__lobo_generic_{}", target[PREFIX.len()..].replace('.', "_")),
                        id: target,
                        span: None,
                    });
                }
            }
        }
    }

    fn expr(
        &mut self,
        expr: &mut Expr,
        types: &BTreeMap<String, Type>,
        needed: &mut BTreeSet<String>,
        depth: usize,
    ) -> Fallible<()> {
        self.nodes += 1;
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            return Err(limit("generic expansion expression budget exceeded").at(expr.span()));
        }
        let span = expr.span();
        let depth = depth + 1;
        let checked = (|| {
            match expr {
                Expr::Reference {
                    function,
                    type_arguments,
                    ..
                } => {
                    for ty in type_arguments.iter_mut() {
                        self.ty(ty, types, needed, 0)?;
                    }
                    *function = self.request(function, type_arguments)?;
                    type_arguments.clear();
                    needed.insert(function.clone());
                }
                Expr::Invoke {
                    callee, arguments, ..
                } => {
                    self.expr(callee, types, needed, depth)?;
                    for argument in arguments {
                        self.expr(argument, types, needed, depth)?;
                    }
                }
                Expr::Call {
                    function,
                    type_arguments,
                    arguments,
                    ..
                } => {
                    for ty in type_arguments.iter_mut() {
                        self.ty(ty, types, needed, 0)?;
                    }
                    *function = self.request(function, type_arguments)?;
                    type_arguments.clear();
                    if function.starts_with(PREFIX) {
                        needed.insert(function.clone());
                    }
                    for arg in arguments {
                        self.expr(arg, types, needed, depth)?;
                    }
                }
                Expr::List { element, .. } => self.ty(element, types, needed, 0)?,
                Expr::NativeOp { arguments, .. }
                | Expr::BufferOp { arguments, .. }
                | Expr::ListOp { arguments, .. } => {
                    for arg in arguments {
                        self.expr(arg, types, needed, depth)?;
                    }
                }
                Expr::Variant {
                    variant,
                    type_arguments,
                    payload,
                    ..
                } => {
                    self.constructor(variant, type_arguments, types, needed)?;
                    if let Some(value) = payload {
                        self.expr(value, types, needed, depth)?;
                    }
                }
                Expr::Record {
                    record,
                    type_arguments,
                    fields,
                    ..
                } => {
                    self.constructor(record, type_arguments, types, needed)?;
                    for field in fields {
                        self.expr(&mut field.value, types, needed, depth)?;
                    }
                }
                Expr::Match { subject, arms, .. } => {
                    self.expr(subject, types, needed, depth)?;
                    for arm in arms {
                        if let Some(binding) = &arm.binding {
                            self.binding(binding).map_err(|e| e.at(arm.span))?;
                        }
                        self.block(&mut arm.body, types, needed, depth)?;
                    }
                }
                Expr::Field { base, .. } => self.expr(base, types, needed, depth)?,
                Expr::Unary { operand, .. } => self.expr(operand, types, needed, depth)?,
                Expr::Binary { left, right, .. } => {
                    self.expr(left, types, needed, depth)?;
                    self.expr(right, types, needed, depth)?;
                }
                Expr::If {
                    condition,
                    then_block,
                    else_block,
                    ..
                } => {
                    self.expr(condition, types, needed, depth)?;
                    self.block(then_block, types, needed, depth)?;
                    self.block(else_block, types, needed, depth)?;
                }
                Expr::Fold {
                    index,
                    accumulator,
                    count,
                    initial,
                    body,
                    ..
                } => {
                    self.binding(index)?;
                    self.binding(&accumulator.name)?;
                    self.ty(&mut accumulator.ty, types, needed, 0)?;
                    self.expr(count, types, needed, depth)?;
                    self.expr(initial, types, needed, depth)?;
                    self.block(body, types, needed, depth)?;
                }
                Expr::While {
                    state,
                    initial,
                    condition,
                    body,
                    ..
                } => {
                    self.binding(&state.name)?;
                    self.ty(&mut state.ty, types, needed, 0)?;
                    self.expr(initial, types, needed, depth)?;
                    self.expr(condition, types, needed, depth)?;
                    self.block(body, types, needed, depth)?;
                }
                Expr::Literal { .. } | Expr::Name { .. } => (),
            }
            Ok(())
        })();
        checked.map_err(|e: Diagnostic| e.at(span))
    }
}

pub(super) fn lower(program: &mut Program) -> Fallible<BTreeMap<String, String>> {
    lower_mode(program, false)
}

pub(super) fn check_definitions(program: &Program) -> Fallible<()> {
    let mut abstract_program = program.clone();
    let origins = lower_mode(&mut abstract_program, true)?;
    validate_mode(&abstract_program, true, true).map_err(|mut e| {
        if let Some(origin) = e.subject.as_ref().and_then(|id| origins.get(id)) {
            e.subject = Some(origin.clone());
        }
        e
    })
}

fn lower_mode(program: &mut Program, abstract_types: bool) -> Fallible<BTreeMap<String, String>> {
    if program.records.len() + program.variants.len() > 64 {
        return Err(limit("at most 64 source nominal declarations per bundle"));
    }
    // Source cannot impersonate compiler-generated declarations or scopes.
    if program
        .scopes
        .iter()
        .any(|s| s.id.starts_with(PREFIX) || s.declarations.iter().any(|id| id.starts_with(PREFIX)))
    {
        return Err(action_error(
            "compile.identity.reserved",
            "lobo.generic. identities are reserved for specialization",
        ));
    }
    let contexts = modules::contexts(program)?;
    let mut expansion = Expansion {
        templates: BTreeMap::new(),
        type_templates: BTreeMap::new(),
        type_requests: BTreeMap::new(),
        pending_types: std::collections::VecDeque::new(),
        contexts,
        requests: BTreeMap::new(),
        pending: std::collections::VecDeque::new(),
        nodes: 0,
        source_names: BTreeSet::new(),
        type_names: program
            .records
            .iter()
            .map(|r| (r.id.clone(), false))
            .chain(program.variants.iter().map(|v| (v.id.clone(), true)))
            .collect(),
    };
    for definition in std::mem::take(&mut program.records)
        .into_iter()
        .map(Nominal::Record)
        .chain(
            std::mem::take(&mut program.variants)
                .into_iter()
                .map(Nominal::Variant),
        )
    {
        let id = definition.id().clone();
        if definition.parameters().is_empty() {
            expansion.pending_types.push_back(PendingType {
                definition,
                context: (*expansion.contexts[&id]).clone(),
                substitutions: BTreeMap::new(),
                origin: None,
            });
        } else {
            expansion.type_templates.insert(id, definition);
        }
    }
    for function in std::mem::take(&mut program.functions) {
        if function.type_parameters.is_empty() {
            let context = (*expansion.contexts[&function.id]).clone();
            expansion.pending.push_back(Pending {
                function,
                context,
                substitutions: BTreeMap::new(),
                origin: None,
            });
        } else {
            expansion.templates.insert(function.id.clone(), function);
        }
    }
    if abstract_types {
        // These are rigid symbolic types, not representative concrete values. Their
        // only supported operations are those valid uniformly for every value type.
        let functions: Vec<_> = expansion
            .templates
            .values()
            .map(|f| (f.id.clone(), f.type_parameters.len()))
            .collect();
        for (id, count) in functions {
            let args: Vec<_> = (0..count).map(|n| Type::Parameter(format!("{id}:{n}"))).collect();
            expansion.request(&id, &args)?;
        }
        let nominals: Vec<_> = expansion
            .type_templates
            .values()
            .map(|d| (d.id().clone(), d.parameters().len()))
            .collect();
        for (id, count) in nominals {
            let args: Vec<_> = (0..count).map(|n| Type::Parameter(format!("{id}:{n}"))).collect();
            expansion.request_type(&id, &args)?;
        }
    }
    for scope in &mut program.scopes {
        scope.declarations.retain(|id| !expansion.is_template(id));
        scope.imports.retain(|i| !expansion.is_template(&i.id));
    }
    let mut origins = BTreeMap::new();
    // New requests append to this work list; recursive instances are cached before
    // walking their bodies, avoiding host recursion even for type-growing calls.
    while !expansion.pending.is_empty() || !expansion.pending_types.is_empty() {
        if let Some(mut pending) = expansion.pending_types.pop_front() {
            let id = pending.definition.id().clone();
            let mut needed = BTreeSet::new();
            let checked = match &mut pending.definition {
                Nominal::Record(r) => r.fields.iter_mut().try_for_each(|f| {
                    expansion
                        .ty(&mut f.ty, &pending.substitutions, &mut needed, 0)
                        .map_err(|e| e.at(f.span))
                }),
                Nominal::Variant(v) => v.cases.iter_mut().try_for_each(|c| {
                    if let Some(ty) = &mut c.payload {
                        expansion
                            .ty(ty, &pending.substitutions, &mut needed, 0)
                            .map_err(|e| e.at(c.span))
                    } else {
                        Ok(())
                    }
                }),
            };
            checked.map_err(|mut e| {
                e.subject = Some(pending.origin.clone().unwrap_or_else(|| id.clone()));
                e
            })?;
            expansion.attach_scope(program, &id, pending.context, pending.origin.is_some(), needed);
            if let Some(origin) = pending.origin {
                origins.insert(id, origin);
            }
            match pending.definition {
                Nominal::Record(r) => program.records.push(r),
                Nominal::Variant(v) => program.variants.push(v),
            }
            continue;
        }
        let mut pending = expansion.pending.pop_front().expect("nonempty worklist");
        let function = &mut pending.function;
        expansion.source_names = pending
            .context
            .aliases
            .keys()
            .cloned()
            .chain(pending.substitutions.keys().map(|name| name[1..].to_string()))
            .collect();
        let mut needed = BTreeSet::new();
        let checked = (|| {
            expansion.ty(&mut function.output, &pending.substitutions, &mut needed, 0)?;
            for parameter in &mut function.parameters {
                expansion
                    .binding(&parameter.name)
                    .map_err(|e| e.at(parameter.span))?;
                expansion.ty(&mut parameter.ty, &pending.substitutions, &mut needed, 0)?;
            }
            // Definition checking needs ordinary signatures for static calls, but
            // their bodies belong to the concrete check. Do not instantiate their
            // generic calls here or charge them against the symbolic proof pass.
            if abstract_types && pending.origin.is_none() {
                return Ok(());
            }
            for expr in function.requires.iter_mut().chain(&mut function.ensures) {
                expansion.expr(expr, &pending.substitutions, &mut needed, 0)?;
            }
            expansion.block(&mut function.body, &pending.substitutions, &mut needed, 0)
        })();
        checked.map_err(|mut e| {
            e.subject = Some(pending.origin.clone().unwrap_or_else(|| function.id.clone()));
            e.at(function.span)
        })?;
        expansion.attach_scope(
            program,
            &function.id,
            pending.context,
            pending.origin.is_some(),
            needed,
        );
        if let Some(origin) = pending.origin {
            origins.insert(function.id.clone(), origin);
        }
        program.functions.push(pending.function);
    }
    operators::lower(program, abstract_types).map_err(|mut e| {
        if let Some(origin) = e.subject.as_ref().and_then(|id| origins.get(id)) {
            e.subject = Some(origin.clone());
        }
        e
    })?;
    Ok(origins)
}
