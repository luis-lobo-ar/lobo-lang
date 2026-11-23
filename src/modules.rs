use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::identities::deserialize_optional"
    )]
    pub identities: Option<BTreeMap<String, String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceBundle {
    format: String,
    root: String,
    sources: Vec<SourceFile>,
}

pub fn compile_bundle_json(text: &str) -> Fallible<Artifact> {
    if text.len() > 4 * MAX_SOURCE {
        return Err(Diagnostic::new(
            "resource.frontend.limit",
            "load",
            "source bundle envelope size limit exceeded",
        ));
    }
    let bundle: SourceBundle = serde_json::from_str(text)
        .map_err(|e| Diagnostic::new("invocation.bundle.invalid", "load", e.to_string()))?;
    if !matches!(bundle.format.as_str(), "lobo-source/0.1" | "lobo-source/0.2") {
        return Err(Diagnostic::new(
            "invocation.bundle.version",
            "load",
            "unsupported source bundle version",
        ));
    }
    if bundle.format == "lobo-source/0.1" && bundle.sources.iter().any(|s| s.identities.is_some()) {
        return Err(Diagnostic::new(
            "invocation.bundle.version",
            "load",
            "managed identities require lobo-source/0.2",
        ));
    }
    compile_bundle(&bundle.root, &bundle.sources)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Import {
    pub name: String,
    pub id: String,
    #[serde(skip)]
    pub span: Option<ByteSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Scope {
    pub name: String,
    pub id: String,
    pub declarations: Vec<String>,
    pub imports: Vec<Import>,
}

#[derive(Clone)]
pub(super) struct Context {
    pub aliases: BTreeMap<String, String>,
    pub visible: BTreeSet<String>,
}

fn module_error(message: &str) -> Diagnostic {
    action_error("compile.module.invalid", message)
}

pub(super) fn contexts(program: &Program) -> Fallible<BTreeMap<String, std::rc::Rc<Context>>> {
    if program.scopes.is_empty() || program.scopes.len() > 1024 {
        return Err(module_error(
            "program needs 1..1024 source/generated module scopes",
        ));
    }
    let declarations = program
        .functions
        .len()
        .saturating_add(program.records.len())
        .saturating_add(program.variants.len());
    let slots = program.scopes.iter().fold(0usize, |sum, scope| {
        sum.saturating_add(scope.declarations.len())
            .saturating_add(scope.imports.len())
    });
    if declarations > 16384 || slots > 16384 {
        return Err(action_error(
            "resource.frontend.limit",
            "module declaration/import metadata limit exceeded",
        ));
    }
    check_name(&program.module).map_err(|e| e.at(program.module_span))?;
    if !valid_id(&program.id) || program.functions.is_empty() {
        return Err(syntax("invalid module identity or empty linked program").at(program.id_span));
    }
    let mut ids = BTreeSet::new();
    for scope in &program.scopes {
        check_name(&scope.name)?;
        if !valid_id(&scope.id) {
            return Err(syntax("invalid module identity").at(program.id_span));
        }
        if !ids.insert(scope.id.clone()) {
            return Err(action_error(
                "compile.identity.duplicate",
                "repeated module identity",
            ));
        }
    }
    if !program
        .scopes
        .iter()
        .any(|s| s.id == program.id && s.name == program.module)
    {
        return Err(module_error("root module is missing or inconsistent"));
    }
    let mut declarations = BTreeMap::new();
    for (id, name, span, id_span) in program
        .functions
        .iter()
        .map(|f| (&f.id, &f.name, f.span, f.id_span))
        .chain(program.records.iter().map(|r| (&r.id, &r.name, r.span, r.span)))
        .chain(program.variants.iter().map(|v| (&v.id, &v.name, v.span, v.span)))
    {
        check_name(name).map_err(|e| e.at(span))?;
        if !valid_id(id) {
            return Err(syntax("invalid declaration identity").at(id_span));
        }
        if !ids.insert(id.clone()) {
            return Err(
                action_error("compile.identity.duplicate", "repeated declaration identity").at(id_span),
            );
        }
        declarations.insert(id.clone(), (name.clone(), span));
    }
    let mut owners = BTreeMap::new();
    for scope in &program.scopes {
        for id in &scope.declarations {
            if !declarations.contains_key(id) || owners.insert(id.clone(), &scope.id).is_some() {
                return Err(module_error("unknown declaration or repeated owner"));
            }
        }
    }
    if owners.len() != declarations.len() {
        return Err(module_error("declaration has no owner"));
    }
    let mut result = BTreeMap::new();
    for scope in &program.scopes {
        let checked = (|| {
            let mut aliases = BTreeMap::new();
            for id in &scope.declarations {
                let (name, span) = &declarations[id];
                if aliases.insert(name.clone(), id.clone()).is_some() {
                    return Err(action_error(
                        "compile.name.duplicate",
                        "module declaration name is repeated",
                    )
                    .at(*span));
                }
            }
            for import in &scope.imports {
                check_name(&import.name).map_err(|e| e.at(import.span))?;
                if !declarations.contains_key(&import.id) || owners[&import.id] == &scope.id {
                    return Err(
                        module_error("import must target a declaration in another supplied module")
                            .at(import.span),
                    );
                }
                if aliases.insert(import.name.clone(), import.id.clone()).is_some() {
                    return Err(action_error(
                        "compile.name.duplicate",
                        "import alias conflicts with a visible declaration",
                    )
                    .at(import.span));
                }
            }
            let mut visible: BTreeSet<_> = aliases.values().cloned().collect();
            if program.native_abi == Some(native_api::ABI)
                && program.scopes.iter().any(|s| s.id == "lobo.native.runtime")
            {
                visible.extend(native_api::result_ids());
            }
            Ok(Context { visible, aliases })
        })()
        .map_err(|mut e: Diagnostic| {
            e.subject = Some(scope.id.clone());
            e
        })?;
        let checked = std::rc::Rc::new(checked);
        for id in &scope.declarations {
            result.insert(id.clone(), checked.clone());
        }
    }
    Ok(result)
}

pub(super) fn check_type(ty: &Type, visible: &BTreeSet<String>) -> Fallible<()> {
    match ty {
        Type::Function(signature) => {
            for parameter in &signature.parameters {
                check_type(parameter, visible)?;
            }
            check_type(&signature.output, visible)
        }
        Type::List(element) => check_type(element, visible),
        Type::Record(id) | Type::Variant(id) if !visible.contains(id) => Err(action_error(
            "compile.module.visibility",
            "explicit type is not declared or imported in this module",
        )),
        _ => Ok(()),
    }
}

pub fn compile_bundle(root: &str, sources: &[SourceFile]) -> Fallible<Artifact> {
    if sources.is_empty() || sources.len() > 16 {
        return Err(action_error(
            "resource.frontend.limit",
            "source bundle needs 1..16 sources",
        ));
    }
    let mut names = BTreeSet::new();
    let mut bytes = 0usize;
    for source in sources {
        if source.name.is_empty()
            || source.name.len() > 1024
            || source.name.contains('\0')
            || !names.insert(&source.name)
        {
            return Err(module_error("invalid or duplicate source label"));
        }
        bytes = bytes.saturating_add(source.text.len());
        if let Some(ids) = &source.identities {
            bytes = bytes.saturating_add(serde_json::to_vec(ids).unwrap().len());
        }
    }
    if bytes > MAX_SOURCE {
        return Err(action_error(
            "resource.frontend.limit",
            "bundle source size limit exceeded",
        ));
    }
    if !names.contains(&root.to_string()) {
        return Err(module_error("root source is missing"));
    }
    let mut ordered: Vec<_> = sources.iter().collect();
    ordered.sort_by_key(|s| &s.name);
    let mut tokens_total = 0;
    let mut parsed = Vec::new();
    for source in ordered {
        let parsed_source = (|| {
            let (tokens, spans) = lex(&source.text)?;
            if source.identities.is_some() && tokens.get(2) != Some(&Token::Symbol(";".into())) {
                return Err(Diagnostic::new(
                    "invocation.identity.invalid",
                    "load",
                    "external identities require modern source",
                ));
            }
            tokens_total += tokens.len().saturating_sub(1);
            if let Some(ids) = &source.identities {
                tokens_total += ids
                    .keys()
                    .map(|key| if key.contains('.') { 6 } else { 4 })
                    .sum::<usize>();
            }
            if tokens_total > MAX_BUNDLE_TOKENS {
                return Err(action_error(
                    "resource.frontend.limit",
                    "bundle token limit exceeded",
                ));
            }
            let mut parser = Parser {
                tokens,
                spans,
                position: 0,
                depth: 0,
                compact: None,
                managed: source.identities.clone(),
                statement_counter: 0,
            };
            let program = parser.program().map_err(|e| e.at(Some(parser.span())))?;
            tokens_total += parser
                .compact
                .as_ref()
                .map_or(0, |syntax| syntax.generated.len() * 8);
            if tokens_total > MAX_BUNDLE_TOKENS {
                return Err(action_error(
                    "resource.frontend.limit",
                    "bundle token limit exceeded including generated imports",
                ));
            }
            Ok(program)
        })()
        .map_err(|e| locate(e, source))?;
        parsed.push((source, parsed_source));
    }
    let root_program = &parsed.iter().find(|(s, _)| s.name == root).unwrap().1;
    let mut program = Program {
        sqlite_abi: parsed.iter().any(|(_, p)| p.sqlite_abi.is_some()).then_some(1),
        native_abi: parsed
            .iter()
            .any(|(_, p)| p.native_abi.is_some())
            .then_some(native_api::ABI),
        module: root_program.module.clone(),
        id: root_program.id.clone(),
        functions: vec![],
        records: vec![],
        variants: vec![],
        operators: vec![],
        scopes: vec![],
        module_span: root_program.module_span,
        id_span: root_program.id_span,
    };
    parsed.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    for (_, part) in &parsed {
        program.functions.extend(part.functions.clone());
        program.records.extend(part.records.clone());
        program.variants.extend(part.variants.clone());
        program.scopes.extend(part.scopes.clone());
    }
    native_api::reject_reserved(&program)?;
    let scopes = contexts(&program).map_err(|e| locate_link(e, &parsed))?;
    let variant_ids: BTreeSet<_> = program.variants.iter().map(|v| v.id.clone()).collect();
    // Resolve each module in its own namespace before merging; no name mangling or source concatenation.
    program.functions.clear();
    program.records.clear();
    program.variants.clear();
    for (source, mut part) in parsed.iter().cloned() {
        let aliases = part.scopes[0]
            .declarations
            .first()
            .and_then(|id| scopes.get(id))
            .map(|c| c.aliases.clone())
            .unwrap_or_default();
        let variants = aliases
            .iter()
            .filter(|(_, id)| variant_ids.contains(*id))
            .map(|(name, _)| name.clone())
            .collect();
        resolve_program(&mut part, &aliases, &variants).map_err(|e| locate(e, source))?;
        program.functions.extend(part.functions);
        program.records.extend(part.records);
        program.variants.extend(part.variants);
        program.operators.extend(part.operators);
    }
    operators::check_bindings(&program).map_err(|e| locate_link(e, &parsed))?;
    let mut inference_enabled = BTreeSet::new();
    for (source, part) in &parsed {
        if lex(&source.text)?.0.get(2) == Some(&Token::Symbol(";".into())) {
            inference_enabled.extend(part.functions.iter().map(|f| f.id.clone()));
        }
    }
    native_api::inject(&mut program)?;
    inference::elaborate(&mut program, &inference_enabled).map_err(|e| locate_link(e, &parsed))?;
    generics::check_definitions(&program).map_err(|e| locate_link(e, &parsed))?;
    let origins = generics::lower(&mut program).map_err(|e| locate_link(e, &parsed))?;
    validate(&program, true).map_err(|mut e| {
        if let Some(origin) = e.subject.as_ref().and_then(|id| origins.get(id)) {
            e.message = format!("{} (specialization of {origin})", e.message);
            e.subject = Some(origin.clone());
        }
        locate_link(e, &parsed)
    })?;
    Ok(Artifact {
        format: FORMAT.into(),
        language: LANGUAGE.into(),
        program,
    })
}

fn locate(mut error: Diagnostic, source: &SourceFile) -> Diagnostic {
    error = error.locate(&source.text);
    error.source = Some(Box::new(source.name.clone()));
    error
}

fn locate_link(mut error: Diagnostic, parsed: &[(&SourceFile, Program)]) -> Diagnostic {
    if parsed.len() == 1 {
        return locate(error, parsed[0].0);
    }
    if let Some((source, _)) = parsed.iter().find(|(_, p)| {
        error
            .subject
            .as_ref()
            .is_some_and(|id| &p.id == id || p.scopes[0].declarations.contains(id))
    }) {
        return locate(error, source);
    }
    error.range = None;
    error.byte_span = None;
    error
}
