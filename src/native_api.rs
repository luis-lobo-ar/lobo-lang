use super::*;

pub(super) const ABI: u32 = 1;
pub(super) struct Signature {
    pub parameters: Vec<Type>,
    pub output: Type,
    pub actions: Vec<&'static str>,
    pub symbol: &'static str,
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceKind {
    Capability,
    Listener,
    Stream,
    SqliteCapability,
    Database,
    Statement,
}

const RESULTS: [(&str, Type); 6] = [
    ("CapabilityResult", Type::Resource(ResourceKind::Capability)),
    ("I64Result", Type::I64),
    ("BytesResult", Type::Bytes),
    ("BoolResult", Type::Bool),
    ("ListenerResult", Type::Resource(ResourceKind::Listener)),
    ("StreamResult", Type::Resource(ResourceKind::Stream)),
];
const SQLITE_RESULTS: [(&str, Type); 5] = [
    (
        "SqliteCapabilityResult",
        Type::Resource(ResourceKind::SqliteCapability),
    ),
    ("DatabaseResult", Type::Resource(ResourceKind::Database)),
    ("StatementResult", Type::Resource(ResourceKind::Statement)),
    ("TextResult", Type::Text),
    ("F64Result", Type::F64),
];
pub(super) fn sqlite_type_name(name: &str) -> bool {
    matches!(name, "SqliteCapability" | "Database" | "Statement")
        || SQLITE_RESULTS.iter().any(|(n, _)| *n == name)
}
fn sqlite_result(id: &str) -> bool {
    SQLITE_RESULTS.iter().any(|(name, _)| result_id(name) == id)
}
pub(super) fn result_id(name: &str) -> String {
    format!("lobo.native.result.{name}")
}
pub(super) fn result_ids() -> impl Iterator<Item = String> {
    RESULTS
        .iter()
        .chain(SQLITE_RESULTS.iter())
        .map(|(name, _)| result_id(name))
}
pub(super) fn is_result(id: &str) -> bool {
    id.strip_prefix("lobo.native.result.").is_some_and(|suffix| {
        RESULTS
            .iter()
            .chain(SQLITE_RESULTS.iter())
            .any(|(name, _)| *name == suffix)
    })
}
pub(super) fn named_type(name: &str) -> Fallible<Type> {
    Ok(match name {
        "Capability" => Type::Resource(ResourceKind::Capability),
        "Listener" => Type::Resource(ResourceKind::Listener),
        "Stream" => Type::Resource(ResourceKind::Stream),
        "SqliteCapability" => Type::Resource(ResourceKind::SqliteCapability),
        "Database" => Type::Resource(ResourceKind::Database),
        "Statement" => Type::Resource(ResourceKind::Statement),
        _ if RESULTS
            .iter()
            .chain(SQLITE_RESULTS.iter())
            .any(|(n, _)| *n == name) =>
        {
            Type::Variant(result_id(name))
        }
        _ => {
            return Err(action_error(
                "compile.native.binding",
                "unknown native resource/result type",
            ))
        }
    })
}
fn shapes(sqlite: bool) -> Vec<Variant> {
    RESULTS
        .iter()
        .chain(SQLITE_RESULTS.iter().filter(|_| sqlite))
        .map(|(name, ty)| Variant {
            name: format!("__lobo_native_{name}"),
            id: result_id(name),
            type_parameters: vec![],
            cases: vec![
                Case {
                    name: "Ok".into(),
                    payload: Some(ty.clone()),
                    span: None,
                },
                Case {
                    name: "Err".into(),
                    payload: Some(Type::I64),
                    span: None,
                },
            ],
            span: None,
        })
        .collect()
}
pub(super) fn reject_reserved(program: &Program) -> Fallible<()> {
    if program.native_abi.is_none() {
        return Ok(());
    }
    if program
        .scopes
        .iter()
        .map(|s| s.id.as_str())
        .chain(program.functions.iter().map(|f| f.id.as_str()))
        .chain(program.records.iter().map(|r| r.id.as_str()))
        .chain(program.variants.iter().map(|v| v.id.as_str()))
        .any(|id| id == "lobo.native" || id.starts_with("lobo.native."))
    {
        return Err(action_error(
            "compile.identity.reserved",
            "lobo.native. identities belong to the native ABI",
        ));
    }
    Ok(())
}
pub(super) fn inject(program: &mut Program) -> Fallible<()> {
    if program.native_abi.is_none() {
        return Ok(());
    }
    let variants = shapes(program.sqlite_abi.is_some());
    program.scopes.push(modules::Scope {
        name: "__lobo_native".into(),
        id: "lobo.native.runtime".into(),
        declarations: variants.iter().map(|v| v.id.clone()).collect(),
        imports: vec![],
    });
    program.variants.extend(variants);
    Ok(())
}
pub(super) fn validate_layouts(program: &Program) -> Fallible<()> {
    if (program.sqlite_abi.is_some() && (program.sqlite_abi != Some(1) || program.native_abi != Some(ABI)))
        || (program.sqlite_abi.is_none() && has_sqlite_nodes(program)?)
    {
        return Err(action_error(
            "compile.native.binding",
            "SQLite nodes require extension version 1",
        ));
    }
    if program.native_abi.is_none() {
        if has_new_nodes(program)? {
            return Err(action_error(
                "compile.native.binding",
                "native nodes require an explicit ABI marker",
            ));
        }
        return Ok(());
    }
    if program.native_abi != Some(ABI) {
        return Err(action_error(
            "compile.native.binding",
            "unsupported native ABI marker",
        ));
    }
    if program
        .functions
        .iter()
        .any(|f| f.id == "lobo.native" || f.id.starts_with("lobo.native."))
        || program
            .records
            .iter()
            .any(|r| r.id == "lobo.native" || r.id.starts_with("lobo.native."))
        || program.variants.iter().any(|v| {
            (v.id == "lobo.native" || v.id.starts_with("lobo.native."))
                && (!is_result(&v.id) || (sqlite_result(&v.id) && program.sqlite_abi.is_none()))
        })
        || program.scopes.iter().any(|s| {
            (s.id == "lobo.native" || s.id.starts_with("lobo.native.")) && s.id != "lobo.native.runtime"
        })
        || program.id == "lobo.native"
        || program.id.starts_with("lobo.native.")
    {
        return Err(action_error(
            "compile.native.binding",
            "reserved native identity impersonation",
        ));
    }
    for expected in shapes(program.sqlite_abi.is_some()) {
        let shape = program
            .variants
            .iter()
            .find(|v| v.id == expected.id)
            .ok_or_else(|| action_error("compile.native.binding", "missing native result layout"))?;
        if shape.name != expected.name
            || !shape.type_parameters.is_empty()
            || shape.cases.len() != 2
            || shape
                .cases
                .iter()
                .zip(&expected.cases)
                .any(|(a, b)| a.name != b.name || a.payload != b.payload)
        {
            return Err(action_error(
                "compile.native.binding",
                "inconsistent native result layout",
            ));
        }
    }
    let expected: Vec<_> = shapes(program.sqlite_abi.is_some())
        .into_iter()
        .map(|v| v.id)
        .collect();
    let scope = program
        .scopes
        .iter()
        .find(|s| s.id == "lobo.native.runtime")
        .ok_or_else(|| action_error("compile.native.binding", "missing native ABI scope"))?;
    if scope.name != "__lobo_native" || !scope.imports.is_empty() || scope.declarations != expected {
        return Err(action_error(
            "compile.native.binding",
            "inconsistent native ABI scope",
        ));
    }
    Ok(())
}
pub(super) fn signature(binding: &str, abi: u32) -> Fallible<Signature> {
    if abi != ABI {
        return Err(action_error(
            "compile.native.binding",
            "unsupported native ABI version",
        ));
    }
    let cap = Type::Resource(ResourceKind::Capability);
    let listener = Type::Resource(ResourceKind::Listener);
    let stream = Type::Resource(ResourceKind::Stream);
    let sqlcap = Type::Resource(ResourceKind::SqliteCapability);
    let db = Type::Resource(ResourceKind::Database);
    let stmt = Type::Resource(ResourceKind::Statement);
    let r = |name| Type::Variant(result_id(name));
    let (parameters, output, symbol, actions): (Vec<Type>, Type, &str, Vec<&str>) = match binding {
        "lobo.native.sqlite_capability" => (
            vec![],
            sqlcap,
            "lobo_sqlite_capability_v1",
            vec!["native.authorize"],
        ),
        "lobo.native.sqlite_restrict" => (
            vec![sqlcap, Type::I64],
            r("SqliteCapabilityResult"),
            "lobo_sqlite_restrict_v1",
            vec!["native.authorize"],
        ),
        "lobo.native.sqlite_open" => (
            vec![sqlcap],
            r("DatabaseResult"),
            "lobo_sqlite_open_v1",
            vec!["db.open"],
        ),
        "lobo.native.sqlite_close" => (
            vec![db],
            r("BoolResult"),
            "lobo_sqlite_close_v1",
            vec!["db.close"],
        ),
        "lobo.native.sqlite_prepare" => (
            vec![db, Type::Text],
            r("StatementResult"),
            "lobo_sqlite_prepare_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_bind_null" => (
            vec![stmt, Type::I64],
            r("BoolResult"),
            "lobo_sqlite_bind_null_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_bind_i64" => (
            vec![stmt, Type::I64, Type::I64],
            r("BoolResult"),
            "lobo_sqlite_bind_i64_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_bind_f64" => (
            vec![stmt, Type::I64, Type::F64],
            r("BoolResult"),
            "lobo_sqlite_bind_f64_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_bind_text" => (
            vec![stmt, Type::I64, Type::Text],
            r("BoolResult"),
            "lobo_sqlite_bind_text_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_bind_bytes" => (
            vec![stmt, Type::I64, Type::Bytes],
            r("BoolResult"),
            "lobo_sqlite_bind_bytes_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_step" => (
            vec![stmt],
            r("BoolResult"),
            "lobo_sqlite_step_v1",
            vec!["db.execute"],
        ),
        "lobo.native.sqlite_column_count" => (
            vec![stmt],
            r("I64Result"),
            "lobo_sqlite_column_count_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_column_type" => (
            vec![stmt, Type::I64],
            r("I64Result"),
            "lobo_sqlite_column_type_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_column_i64" => (
            vec![stmt, Type::I64],
            r("I64Result"),
            "lobo_sqlite_column_i64_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_column_f64" => (
            vec![stmt, Type::I64],
            r("F64Result"),
            "lobo_sqlite_column_f64_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_column_text" => (
            vec![stmt, Type::I64],
            r("TextResult"),
            "lobo_sqlite_column_text_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_column_bytes" => (
            vec![stmt, Type::I64],
            r("BytesResult"),
            "lobo_sqlite_column_bytes_v1",
            vec!["db.read"],
        ),
        "lobo.native.sqlite_finalize" => (
            vec![stmt],
            r("BoolResult"),
            "lobo_sqlite_finalize_v1",
            vec!["db.close"],
        ),
        "lobo.native.sqlite_begin" => (
            vec![db],
            r("BoolResult"),
            "lobo_sqlite_begin_v1",
            vec!["db.transaction"],
        ),
        "lobo.native.sqlite_commit" => (
            vec![db],
            r("BoolResult"),
            "lobo_sqlite_commit_v1",
            vec!["db.transaction"],
        ),
        "lobo.native.sqlite_rollback" => (
            vec![db],
            r("BoolResult"),
            "lobo_sqlite_rollback_v1",
            vec!["db.transaction"],
        ),
        "lobo.native.sqlite_changes" => (
            vec![db],
            r("I64Result"),
            "lobo_sqlite_changes_v1",
            vec!["db.read"],
        ),
        "lobo.native.abi_version" => (vec![], Type::I64, "lobo_native_abi_version_v1", vec![]),
        "lobo.native.roundtrip_i64" => (vec![Type::I64], Type::I64, "lobo_native_roundtrip_i64_v1", vec![]),
        "lobo.native.roundtrip_bool" => (
            vec![Type::Bool],
            Type::Bool,
            "lobo_native_roundtrip_bool_v1",
            vec![],
        ),
        "lobo.native.copy_bytes" => (
            vec![Type::Bytes],
            Type::Bytes,
            "lobo_native_copy_bytes_v1",
            vec![],
        ),
        "lobo.native.capability" => (vec![], cap, "lobo_native_capability_v1", vec!["native.authorize"]),
        "lobo.native.restrict_capability" => (
            vec![cap, Type::I64],
            r("CapabilityResult"),
            "lobo_native_restrict_capability_v1",
            vec!["native.authorize"],
        ),
        "lobo.native.monotonic_ms" => (
            vec![cap],
            r("I64Result"),
            "lobo_native_monotonic_ms_v1",
            vec!["clock.read"],
        ),
        "lobo.native.random_bytes" => (
            vec![cap, Type::I64],
            r("BytesResult"),
            "lobo_native_random_bytes_v1",
            vec!["random.read"],
        ),
        "lobo.native.listen" => (
            vec![cap, Type::I64],
            r("ListenerResult"),
            "lobo_native_listen_v1",
            vec!["net.listen"],
        ),
        "lobo.native.connect" => (
            vec![cap, Type::I64, Type::I64],
            r("StreamResult"),
            "lobo_native_connect_v1",
            vec!["net.connect"],
        ),
        "lobo.native.accept" => (
            vec![listener, Type::I64],
            r("StreamResult"),
            "lobo_native_accept_v1",
            vec!["net.accept"],
        ),
        "lobo.native.port" => (
            vec![listener],
            r("I64Result"),
            "lobo_native_port_v1",
            vec!["net.inspect"],
        ),
        "lobo.native.read" => (
            vec![stream, Type::I64, Type::I64],
            r("BytesResult"),
            "lobo_native_read_v1",
            vec!["net.read"],
        ),
        "lobo.native.write" => (
            vec![stream, Type::Bytes, Type::I64],
            r("I64Result"),
            "lobo_native_write_v1",
            vec!["net.write"],
        ),
        "lobo.native.shutdown_write" => (
            vec![stream],
            r("BoolResult"),
            "lobo_native_shutdown_write_v1",
            vec!["net.write"],
        ),
        "lobo.native.close_listener" => (
            vec![listener],
            r("BoolResult"),
            "lobo_native_close_listener_v1",
            vec!["net.close"],
        ),
        "lobo.native.close_stream" => (
            vec![stream],
            r("BoolResult"),
            "lobo_native_close_stream_v1",
            vec!["net.close"],
        ),
        _ => {
            return Err(action_error(
                "compile.native.binding",
                "native binding is not in the approved catalog",
            ))
        }
    };
    Ok(Signature {
        parameters,
        output,
        symbol,
        actions,
    })
}

pub(super) fn has_new_nodes(program: &Program) -> Fallible<bool> {
    scan_nodes(program, false)
}
pub(super) fn has_sqlite_nodes(program: &Program) -> Fallible<bool> {
    scan_nodes(program, true)
}
fn scan_nodes(program: &Program, sqlite: bool) -> Fallible<bool> {
    fn block(b: &Block, depth: usize, sqlite: bool) -> Fallible<bool> {
        let mut found = expr(&b.value, depth, sqlite)?;
        for binding in &b.bindings {
            found |= binding
                .annotation
                .as_ref()
                .is_some_and(|t| native_type(t, sqlite));
            found |= expr(&binding.value, depth, sqlite)?;
        }
        Ok(found)
    }
    fn expr(e: &Expr, depth: usize, sqlite: bool) -> Fallible<bool> {
        if depth > MAX_DEPTH {
            return Err(action_error(
                "resource.frontend.limit",
                "native expression scan depth exceeded",
            ));
        }
        let depth = depth + 1;
        let mut found = matches!(e, Expr::NativeOp { binding, .. } if !sqlite || binding.starts_with("lobo.native.sqlite_"));
        found |= match e {
            Expr::List { element, .. } => native_type(element, sqlite),
            Expr::Reference { type_arguments, .. }
            | Expr::Call { type_arguments, .. }
            | Expr::Record { type_arguments, .. }
            | Expr::Variant { type_arguments, .. } => type_arguments.iter().any(|t| native_type(t, sqlite)),
            Expr::Fold { accumulator, .. } => native_type(&accumulator.ty, sqlite),
            Expr::While { state, .. } => native_type(&state.ty, sqlite),
            Expr::Literal { value, .. } => native_type(&value.ty(), sqlite),
            _ => false,
        };
        match e {
            Expr::NativeOp { arguments, .. }
            | Expr::BufferOp { arguments, .. }
            | Expr::ListOp { arguments, .. }
            | Expr::Call { arguments, .. } => {
                for e in arguments {
                    found |= expr(e, depth, sqlite)?;
                }
            }
            Expr::Invoke {
                callee, arguments, ..
            } => {
                found |= expr(callee, depth, sqlite)?;
                for e in arguments {
                    found |= expr(e, depth, sqlite)?;
                }
            }
            Expr::Unary { operand, .. } => found |= expr(operand, depth, sqlite)?,
            Expr::Field { base, .. } => found |= expr(base, depth, sqlite)?,
            Expr::Binary { left, right, .. } => {
                found |= expr(left, depth, sqlite)?;
                found |= expr(right, depth, sqlite)?;
            }
            Expr::Variant { payload, .. } => {
                if let Some(e) = payload {
                    found |= expr(e, depth, sqlite)?;
                }
            }
            Expr::Record { fields, .. } => {
                for field in fields {
                    found |= expr(&field.value, depth, sqlite)?;
                }
            }
            Expr::Match { subject, arms, .. } => {
                found |= expr(subject, depth, sqlite)?;
                for arm in arms {
                    found |= block(&arm.body, depth, sqlite)?;
                }
            }
            Expr::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                found |= expr(condition, depth, sqlite)?;
                found |= block(then_block, depth, sqlite)?;
                found |= block(else_block, depth, sqlite)?;
            }
            Expr::Fold {
                count, initial, body, ..
            } => {
                found |= expr(count, depth, sqlite)?;
                found |= expr(initial, depth, sqlite)?;
                found |= block(body, depth, sqlite)?;
            }
            Expr::While {
                initial,
                condition,
                body,
                ..
            } => {
                found |= expr(initial, depth, sqlite)?;
                found |= expr(condition, depth, sqlite)?;
                found |= block(body, depth, sqlite)?;
            }
            Expr::Name { .. } | Expr::Literal { .. } | Expr::Reference { .. } | Expr::List { .. } => {}
        }
        Ok(found)
    }
    fn native_type(t: &Type, sqlite: bool) -> bool {
        match t {
            Type::Resource(kind) => {
                !sqlite
                    || matches!(
                        kind,
                        ResourceKind::SqliteCapability | ResourceKind::Database | ResourceKind::Statement
                    )
            }
            Type::Variant(_) => false,
            Type::List(e) => native_type(e, sqlite),
            Type::Applied { arguments, .. } => arguments.iter().any(|t| native_type(t, sqlite)),
            Type::Function(f) => {
                f.parameters.iter().any(|t| native_type(t, sqlite)) || native_type(&f.output, sqlite)
            }
            _ => false,
        }
    }
    let mut found = program
        .records
        .iter()
        .any(|r| r.fields.iter().any(|f| native_type(&f.ty, sqlite)))
        || program.variants.iter().any(|v| {
            v.cases
                .iter()
                .any(|c| c.payload.as_ref().is_some_and(|t| native_type(t, sqlite)))
        });
    for f in &program.functions {
        found |= native_type(&f.output, sqlite) || f.parameters.iter().any(|p| native_type(&p.ty, sqlite));
        found |= block(&f.body, 0, sqlite)?;
        for e in f.requires.iter().chain(&f.ensures) {
            found |= expr(e, 0, sqlite)?;
        }
    }
    Ok(found)
}
