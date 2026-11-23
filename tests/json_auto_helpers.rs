use lobo_core::{compile_bundle, Artifact, SourceFile, Value};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

const SOURCE: &str = r#"module automatic;
use js {Json,stringify};
fn encode(name:Text)->Text {js.stringify(json {"name":name})}
fn empty()->Text {js.stringify(json {})}
identities {module:"auto",js.Json:"lobo.json.value",js.stringify:"lobo.json.stringify",encode:"auto.encode",empty:"auto.empty"}
"#;

fn sources(app: &str, library: &str) -> Vec<SourceFile> {
    vec![
        SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        SourceFile {
            name: "json".into(),
            text: library.into(),
            identities: None,
        },
        SourceFile {
            name: "http".into(),
            text: include_str!("../libraries/http/http.lobo").into(),
            identities: None,
        },
    ]
}
fn compile(app: &str) -> Artifact {
    compile_bundle("app", &sources(app, include_str!("../libraries/json/json.lobo"))).unwrap()
}

fn normalize_imports(mut value: Json) -> Json {
    for scope in value["program"]["scopes"].as_array_mut().unwrap() {
        let imports = scope["imports"].as_array_mut().unwrap();
        for import in imports.iter_mut() {
            match (import["name"].as_str().unwrap(), import["id"].as_str().unwrap()) {
                ("__lobo_json_object", "lobo.json.object") => import["name"] = json!("object"),
                ("__lobo_json_field", "lobo.json.field") => import["name"] = json!("field"),
                _ => {}
            }
        }
        imports.sort_by_key(|import| {
            (
                import["name"].as_str().unwrap().to_owned(),
                import["id"].as_str().unwrap().to_owned(),
            )
        });
    }
    value
}

#[test]
fn automatic_builders_preserve_checked_operations_native_json_and_auditable_dependencies() {
    let automatic = compile(SOURCE);
    let explicit = SOURCE
        .replace("{Json,stringify}", "{Json,stringify,object,field}")
        .replace(
            "encode:\"auto.encode\"",
            "js.object:\"lobo.json.object\",js.field:\"lobo.json.field\",encode:\"auto.encode\"",
        );
    assert_eq!(
        normalize_imports(serde_json::from_str(&automatic.to_json().unwrap()).unwrap()),
        normalize_imports(serde_json::from_str(&compile(&explicit).to_json().unwrap()).unwrap())
    );
    let encoded: Json = serde_json::from_str(&automatic.to_json().unwrap()).unwrap();
    let scope = encoded["program"]["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "auto")
        .unwrap();
    assert!(scope["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["name"] == "__lobo_json_object" && i["id"] == "lobo.json.object"));
    assert!(scope["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["name"] == "__lobo_json_field" && i["id"] == "lobo.json.field"));
    let builds = Builds::new(&Artifact::from_json(&automatic.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "auto.encode", &["Text:225c0ac3a9"], 10000, 10000);
        assert_eq!(
            serde_json::from_str::<Json>(result["value"]["value"].as_str().unwrap()).unwrap(),
            json!({"name":"\"\\\né"})
        );
        assert_eq!(
            builds.invoke(mode, "auto.empty", &[], 10000, 10000)["value"]["value"],
            "{}"
        );
    }
}

#[test]
fn ordinary_helper_names_cannot_hijack_literals_or_become_implicit_source_imports() {
    let local = SOURCE
        .replace("fn encode", "fn object()->I64 {7} fn field()->I64 {9} fn encode")
        .replace(
            "encode:\"auto.encode\"",
            "object:\"auto.object\",field:\"auto.field\",encode:\"auto.encode\"",
        );
    let artifact = compile(&local);
    assert_eq!(artifact.run("auto.object", vec![], 100).unwrap(), Value::I64(7));
    assert_eq!(
        artifact.run("auto.empty", vec![], 1000).unwrap(),
        Value::Text(lobo_core::BufferValue::from_bytes(b"{}".to_vec()))
    );
    for bad in [
        SOURCE.replace("js.stringify(json {})", "object()"),
        SOURCE.replace("js.stringify(json {})", "js.object()"),
        SOURCE.replace("js.stringify(json {})", "__lobo_json_object()"),
    ] {
        assert!(compile_bundle("app", &sources(&bad, include_str!("../libraries/json/json.lobo"))).is_err());
    }
    let imported = SOURCE
        .replace("fn encode", "use other {object,field}; fn encode")
        .replace(
            "encode:\"auto.encode\"",
            "other.object:\"other.object\",other.field:\"other.field\",encode:\"auto.encode\"",
        );
    let mut bundle = sources(&imported, include_str!("../libraries/json/json.lobo"));
    bundle.push(SourceFile{name:"other".into(),text:"module other; fn object()->I64 {7} fn field()->I64 {9} identities {module:\"other\",object:\"other.object\",field:\"other.field\"}".into(),identities:None});
    assert!(compile_bundle("app", &bundle).is_ok());
}

#[test]
fn generated_helpers_are_closed_minimal_typed_and_keep_action_checks() {
    let library = include_str!("../libraries/json/json.lobo");
    let missing = library.replace("\"lobo.json.field\"", "\"provider.field\"");
    let error = compile_bundle("app", &sources(SOURCE, &missing)).unwrap_err();
    assert_eq!(error.code, "compile.module.invalid");
    let empty="module m; use js {Json,stringify}; fn empty()->Text {js.stringify(json {})} identities {module:\"m\",js.Json:\"lobo.json.value\",js.stringify:\"lobo.json.stringify\",empty:\"empty\"}";
    assert!(
        compile_bundle("app", &sources(empty, &missing)).is_ok(),
        "empty objects require no field dependency"
    );
    let impure = library.replace(
        "fn field(document:JsonObject, name:Text, value:Json) -> JsonObject",
        "fn field(document:JsonObject, name:Text, value:Json) -> JsonObject actions {demo.observe}",
    );
    assert_eq!(
        compile_bundle("app", &sources(SOURCE, &impure)).unwrap_err().code,
        "compile.action.undeclared"
    );
    let bad = SOURCE.replace("\"name\":name", "\"name\":true");
    let error = compile_bundle("app", &sources(&bad, library)).unwrap_err();
    assert_eq!(error.code, "compile.type.mismatch");
    assert_eq!(
        serde_json::to_value(error).unwrap()["range"]["start"]["byte"],
        bad.find("true").unwrap()
    );
}
