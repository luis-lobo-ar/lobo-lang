use lobo_core::compile;

const SMOKE: &str = r#"module sqlite;
fn main()->Native::I64Result actions {native.authorize,db.open,db.close} {
  let cap=Native::sqlite_capability();
  let db=try Native::sqlite_open(cap);
  let closed=try Native::sqlite_close(db);
  Ok(1)
}
identities {module:"sqlite.smoke",main:"sqlite.smoke.main"}"#;

#[test]
fn sqlite_native_catalog_compiles_without_user_defined_handles() {
    let artifact = compile(SMOKE).unwrap();
    let checked: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(checked["program"]["sqlite_abi"], 1);
    assert_eq!(checked["program"]["native_abi"], 1);
}

#[test]
fn sqlite_extensions_cannot_be_retagged_or_removed_from_checked_code() {
    let source = compile(SMOKE).unwrap().to_json().unwrap();
    for language in ["core/1.7", "core/1.8"] {
        let mut checked: serde_json::Value = serde_json::from_str(&source).unwrap();
        checked["language"] = language.into();
        assert_eq!(
            lobo_core::Artifact::from_json(&checked.to_string())
                .unwrap_err()
                .code,
            "invocation.artifact.version"
        );
    }
    let mut checked: serde_json::Value = serde_json::from_str(&source).unwrap();
    checked["program"].as_object_mut().unwrap().remove("sqlite_abi");
    assert!(lobo_core::Artifact::from_json(&checked.to_string()).is_err());
    checked["program"]["sqlite_abi"] = 2.into();
    assert!(lobo_core::Artifact::from_json(&checked.to_string()).is_err());
}

#[test]
fn sqlite_is_opt_in_and_literal_text_does_not_select_a_driver() {
    let normal =
        compile("module m; fn main()->I64 {Native::abi_version()} identities {module:\"m\",main:\"main\"}")
            .unwrap();
    let mut checked: serde_json::Value = serde_json::from_str(&normal.to_json().unwrap()).unwrap();
    assert!(checked["program"]["sqlite_abi"].is_null());
    assert_eq!(checked["program"]["variants"].as_array().unwrap().len(), 6);
    checked["language"] = "core/1.8".into();
    lobo_core::Artifact::from_json(&checked.to_string()).unwrap();
    let source =
        "module m; fn main()->Text {\"lobo.native.sqlite_open\"} identities {module:\"m\",main:\"main\"}";
    let checked: serde_json::Value =
        serde_json::from_str(&compile(source).unwrap().to_json().unwrap()).unwrap();
    assert!(checked["program"]["sqlite_abi"].is_null());
    assert!(checked["program"]["native_abi"].is_null());
}

#[test]
fn sqlite_operations_enforce_types_actions_and_native_only_evaluation() {
    for source in [
        SMOKE.replace("db.open,", ""),
        SMOKE.replace("sqlite_open(cap)", "sqlite_open(1)"),
        SMOKE.replace("sqlite_open(cap)", "sqlite_open(cap,\"anything\")"),
    ] {
        assert!(compile(&source).is_err());
    }
    let output = compile(SMOKE)
        .unwrap()
        .run("sqlite.smoke.main", vec![], 10000)
        .unwrap_err();
    assert_eq!(output.code, "invocation.native.required");
}

#[test]
fn sqlite_export_is_self_contained_and_non_database_programs_do_not_embed_the_engine() {
    let c = compile(SMOKE).unwrap().emit_c().unwrap();
    assert!(c.contains("#define SQLITE_VERSION        \"3.54.1\""));
    assert!(c.contains("lobo_sqlite_open_v1"));
    let c = compile("module m; fn main()->I64 {42} identities {module:\"m\",main:\"main\"}")
        .unwrap()
        .emit_c()
        .unwrap();
    assert!(!c.contains("sqlite3_prepare_v3"));
}

#[test]
fn sql_library_reads_a_typed_bound_value_through_compiled_machine_code() {
    let app = r#"module consumer;
use sql {Cell,Row,query,no_params};
fn main()->I64 actions {native.authorize,db.open,db.execute,db.read,db.close} {
  let cap=Native::sqlite_capability();
  match Native::sqlite_open(cap) {
    Err(code)=> -1,
    Ok(db)=> {
      let value=match query(db,"SELECT 42",no_params()) {
        Err(error)=> -2,
        Ok(rows)=> {let cell=List::head(List::head(rows).values); match cell {Integer(n)=>n,Null=> -3,Decimal(n)=> -4,String(t)=> -5,Binary(b)=> -6}}
      };
      let closed=Native::sqlite_close(db);
      value
    }
  }
}
identities {module:"sqlite.consumer",sql.Cell:"lobo.sql.Cell",sql.Row:"lobo.sql.Row",sql.query:"lobo.sql.query",sql.no_params:"lobo.sql.no_params",main:"sqlite.consumer.main"}"#;
    let sources = [
        lobo_core::SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        lobo_core::SourceFile {
            name: "sql".into(),
            text: include_str!("../libraries/sql/sql.lobo").into(),
            identities: Some(
                lobo_core::identities::parse_manifest(include_str!("../libraries/sql/sql.lobo.ids.json"))
                    .unwrap(),
            ),
        },
    ];
    let artifact = lobo_core::compile_bundle("app", &sources).unwrap();
    let folder = std::env::temp_dir().join(format!("lobo-sql-library-{}", std::process::id()));
    std::fs::create_dir(&folder).unwrap();
    let exe = folder.join("sql");
    artifact.compile_native(&exe).unwrap();
    let output = std::process::Command::new(&exe)
        .args([
            "--entry",
            "sqlite.consumer.main",
            "--sqlite-file",
            folder.join("data.sqlite").to_str().unwrap(),
            "--sqlite-mode",
            "schema",
            "--fuel",
            "1000000",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result["value"]["value"],
        "42",
        "{result}; reproducer {}",
        folder.display()
    );
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn http_and_sql_libraries_compose_within_the_current_bundle_budget() {
    let names = [
        "examples/native/rest/app.lobo",
        "examples/native/rest/controller.lobo",
        "libraries/http-server/transport.lobo",
        "libraries/http-server/server.lobo",
        "libraries/http/http.lobo",
        "libraries/json/json.lobo",
        "libraries/sql/sql.lobo",
        "examples/native/sqlite/users.lobo",
    ];
    let sources = names
        .iter()
        .map(|name| {
            let sidecar = std::path::PathBuf::from(format!("{name}.ids.json"));
            lobo_core::SourceFile {
                name: (*name).into(),
                text: std::fs::read_to_string(name).unwrap(),
                identities: sidecar.exists().then(|| {
                    lobo_core::identities::parse_manifest(&std::fs::read_to_string(sidecar).unwrap()).unwrap()
                }),
            }
        })
        .collect::<Vec<_>>();
    lobo_core::compile_bundle(names[0], &sources).unwrap();
}

#[test]
fn structured_sql_errors_preserve_backend_codes_and_can_be_forwarded() {
    let source = lobo_core::SourceFile {
        name: "sql".into(),
        text: include_str!("../libraries/sql/sql.lobo").into(),
        identities: Some(
            lobo_core::identities::parse_manifest(include_str!("../libraries/sql/sql.lobo.ids.json"))
                .unwrap(),
        ),
    };
    let artifact = lobo_core::compile_bundle("sql", &[source]).unwrap();
    for raw in [
        1,
        2,
        3,
        4,
        5,
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        23 * 65536 + 1,
        1555 * 65536 + 8,
    ] {
        let decoded = artifact
            .run("lobo.sql.error_details", vec![lobo_core::Value::I64(raw)], 10000)
            .unwrap();
        assert_eq!(
            artifact.run("lobo.sql.error_code", vec![decoded], 10000).unwrap(),
            lobo_core::Value::I64(raw)
        );
    }
}

#[test]
fn old_user_identities_that_resemble_sqlite_results_keep_their_meaning() {
    let source="module old @id(\"old\"); variant Legacy @id(\"lobo.native.result.TextResult\") { Value(Text) } function f @id(\"old.f\")()->Legacy {Legacy::Value(\"old\")}";
    let artifact = compile(source).unwrap();
    for language in ["core/1.0", "core/1.7", "core/1.8"] {
        let mut wire: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
        wire["language"] = language.into();
        let old = lobo_core::Artifact::from_json(&wire.to_string()).unwrap();
        assert_eq!(
            old.run("old.f", vec![], 100).unwrap(),
            artifact.run("old.f", vec![], 100).unwrap()
        );
    }
}
