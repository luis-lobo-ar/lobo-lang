use lobo_core::{compile, compile_bundle_json, Artifact, Value};
use serde_json::{json, Value as Json};
use std::process::Command;
mod support;
use support::Builds;

const CODE: &str = "module sample;\nfn increment(n:I64)->I64 {n+1}\n";

fn snapshot(source: &str, identities: Json) -> String {
    json!({"format":"lobo-source/0.2","root":"main","sources":[{"name":"main","text":source,"identities":identities}]}).to_string()
}

#[test]
fn explicit_managed_snapshots_compile_without_source_identity_syntax_or_io() {
    let source = snapshot(
        CODE,
        json!({"module":"stable.module","increment":"stable.increment"}),
    );
    let managed = compile_bundle_json(&source).unwrap();
    let inline = compile("module sample; fn increment(n:I64)->I64 {n+1} identities {module:\"stable.module\",increment:\"stable.increment\"}").unwrap();
    assert_eq!(managed.to_json().unwrap(), inline.to_json().unwrap());
    let restored = Artifact::from_json(&managed.to_json().unwrap()).unwrap();
    assert_eq!(
        restored
            .run("stable.increment", vec![Value::I64(41)], 100)
            .unwrap(),
        Value::I64(42)
    );
    assert!(
        compile(CODE).is_err(),
        "pure source checks must not create persistent identity state"
    );
    let duplicate = source.replace(
        "\"increment\":\"stable.increment\"",
        "\"increment\":\"stable.increment\",\"increment\":\"other\"",
    );
    assert!(compile_bundle_json(&duplicate).is_err());
    for bindings in [
        json!({"module":"stable.module"}),
        json!({"module":"stable.module","increment":"stable.increment","stale":"stale"}),
        json!({"module":"stable.module","increment":"bad id"}),
    ] {
        assert!(compile_bundle_json(&snapshot(CODE, bindings)).is_err());
    }
    let conflicting = format!("{CODE} identities {{module:\"stable.module\",increment:\"wrong.id\"}}");
    assert!(compile_bundle_json(&snapshot(
        &conflicting,
        json!({"module":"stable.module","increment":"stable.increment"})
    ))
    .is_err());
    assert!(compile_bundle_json(&source.replace("lobo-source/0.2", "lobo-source/0.1")).is_err());
    assert!(compile_bundle_json(&snapshot(
        "module sample @id(\"legacy\"); function f @id(\"f\")()->I64 {1}",
        json!({"module":"ignored"})
    ))
    .is_err());
}

fn cli(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn authoring_tools_assign_once_sync_explicitly_and_preserve_identity_after_rename_and_move() {
    let folder = std::env::temp_dir().join(format!(
        "lobo-managed-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&folder).unwrap();
    let source = folder.join("sample.lobo");
    std::fs::write(&source, CODE).unwrap();
    let name = source.to_str().unwrap();
    let result = cli(&["identities", "init", name]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(std::fs::read_to_string(&source).unwrap(), CODE);
    let sidecar = folder.join("sample.lobo.ids.json");
    let bytes = std::fs::read(&sidecar).unwrap();
    let first: Json = serde_json::from_slice(&bytes).unwrap();
    let id = first["identities"]["increment"].as_str().unwrap().to_owned();
    assert!(id.starts_with("lobo."), "{id}");
    assert!(cli(&["identities", "init", name]).status.success());
    assert_eq!(std::fs::read(&sidecar).unwrap(), bytes);
    let added = format!("{CODE}fn answer()->I64 {{increment(41)}}\n");
    std::fs::write(&source, &added).unwrap();
    assert!(!cli(&["check", name]).status.success());
    assert_eq!(std::fs::read(&sidecar).unwrap(), bytes, "checks never assign IDs");
    assert!(cli(&["identities", "sync", name]).status.success());
    let synced: Json = serde_json::from_slice(&std::fs::read(&sidecar).unwrap()).unwrap();
    assert_eq!(synced["identities"]["increment"], id);
    let renamed = added.replace("increment", "bump");
    std::fs::write(&source, &renamed).unwrap();
    let before_rename = std::fs::read(&sidecar).unwrap();
    assert!(
        !cli(&["identities", "sync", name]).status.success(),
        "sync must not guess that a stale identity is a new declaration"
    );
    assert_eq!(std::fs::read(&sidecar).unwrap(), before_rename);
    let result = cli(&["identities", "rename", name, "increment", "bump"]);
    assert!(result.status.success(), "{result:?}");
    let after: Json = serde_json::from_slice(&std::fs::read(&sidecar).unwrap()).unwrap();
    assert_eq!(after["identities"]["bump"], id);
    assert!(after["identities"].get("increment").is_none());
    assert_eq!(std::fs::read_to_string(&source).unwrap(), renamed);
    let moved = folder.join("moved.lobo");
    std::fs::rename(&source, &moved).unwrap();
    std::fs::rename(&sidecar, folder.join("moved.lobo.ids.json")).unwrap();
    let output = cli(&["eval", moved.to_str().unwrap(), "--entry", &id, "--arg", "I64:41"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        serde_json::from_slice::<Json>(&output.stdout).unwrap()["value"]["value"],
        "42"
    );
    std::fs::remove_file(moved).unwrap();
    std::fs::remove_file(folder.join("moved.lobo.ids.json")).unwrap();
    std::fs::remove_dir(folder).unwrap();
}

#[test]
fn managed_mvc_keeps_exact_ids_program_and_native_behavior() {
    let source = include_str!("../examples/mvc.compact.lobo")
        .split("// Persistent metadata.")
        .next()
        .unwrap();
    let bindings = serde_json::from_str::<Json>(include_str!("../examples/mvc.compact.lobo.ids.json"))
        .unwrap()["identities"]
        .clone();
    let mut sources = json!([
        {"name":"mvc","text":source,"identities":bindings},
        {"name":"http","text":include_str!("../libraries/http/http.lobo")},
        {"name":"json","text":include_str!("../libraries/json/json.lobo")}
    ]);
    let managed =
        compile_bundle_json(&json!({"format":"lobo-source/0.2","root":"mvc","sources":sources}).to_string())
            .unwrap();
    sources[0] = json!({"name":"mvc","text":include_str!("../examples/mvc.lobo")});
    let legacy =
        compile_bundle_json(&json!({"format":"lobo-source/0.1","root":"mvc","sources":sources}).to_string())
            .unwrap();
    assert_eq!(
        support::resolved_program(&managed),
        support::resolved_program(&legacy)
    );
    let builds = Builds::new(&Artifact::from_json(&managed.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        for (user, status, body) in [(1, "200", "{\"name\":\"Luis\"}"), (42, "404", "Not found\n")] {
            let result = builds.invoke(
                mode,
                "example.mvc.show_user",
                &[&format!("I64:{user}")],
                10000,
                10000,
            );
            assert_eq!(result["status"], "ok", "{result}");
            assert_eq!(result["value"]["value"]["fields"]["status"]["value"], status);
            assert_eq!(result["value"]["value"]["fields"]["body"]["value"], body);
        }
    }
}

#[test]
fn explicit_import_allocation_and_failed_edits_preserve_published_metadata() {
    let folder = std::env::temp_dir().join(format!(
        "lobo-managed-imports-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&folder).unwrap();
    let app = folder.join("app.lobo");
    let helper = folder.join("shared.lobo");
    std::fs::write(&app, "module app; use shared {step}; fn main()->I64 {step(41)}").unwrap();
    std::fs::write(&helper, "module shared; fn step(n:I64)->I64 {n+1}").unwrap();
    let app_name = app.to_str().unwrap();
    let helper_name = helper.to_str().unwrap();
    assert!(!cli(&["identities", "init", app_name]).status.success());
    let app_ids = lobo_core::identities::sidecar(&app);
    assert!(
        !app_ids.exists(),
        "an unresolved import must not publish fresh IDs"
    );
    let result = cli(&["identities", "init", app_name, "--source", helper_name]);
    assert!(result.status.success(), "{result:?}");
    let before = std::fs::read(&app_ids).unwrap();
    let bindings = lobo_core::identities::parse_manifest(std::str::from_utf8(&before).unwrap()).unwrap();
    let helper_ids = lobo_core::identities::sidecar(&helper);
    let exported =
        lobo_core::identities::parse_manifest(&std::fs::read_to_string(&helper_ids).unwrap()).unwrap();
    assert_eq!(bindings["shared.step"], exported["step"]);
    assert!(cli(&["check", app_name, "--source", helper_name])
        .status
        .success());
    let lock = app_ids.with_extension("json.lock");
    std::fs::write(&lock, "interrupted writer").unwrap();
    assert!(!cli(&["identities", "sync", app_name, "--source", helper_name])
        .status
        .success());
    assert!(!cli(&["check", app_name, "--source", helper_name])
        .status
        .success());
    assert_eq!(std::fs::read(&app_ids).unwrap(), before);
    std::fs::remove_file(lock).unwrap();
    std::fs::write(
        &app_ids,
        "{\"format\":\"lobo-identities/0.1\",\"identities\":{\"module\":\"one\",\"module\":\"two\"}}",
    )
    .unwrap();
    let broken = std::fs::read(&app_ids).unwrap();
    assert!(!cli(&["identities", "init", app_name, "--source", helper_name])
        .status
        .success());
    assert_eq!(std::fs::read(&app_ids).unwrap(), broken);
    for path in [app, helper, app_ids, helper_ids] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(folder).unwrap();
}
