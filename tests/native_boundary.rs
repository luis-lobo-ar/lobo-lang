use lobo_core::{compile, Artifact};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

const SOURCE: &str = r#"module boundary;
fn integer(n:I64)->I64 {Native::roundtrip_i64(n)}
fn boolean(b:Bool)->Bool {Native::roundtrip_bool(b)}
fn bytes(b:Bytes)->Bytes {Native::copy_bytes(b)}
fn version()->I64 {Native::abi_version()}
identities {module:"boundary",integer:"boundary.integer",boolean:"boundary.boolean",bytes:"boundary.bytes",version:"boundary.version"}
"#;

#[test]
fn native_abi_preserves_scalar_and_buffer_values_on_every_backend() {
    let artifact = compile(SOURCE).unwrap();
    let builds = Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let corpus: Json = serde_json::from_str(include_str!("upstream/native/adapted-cases.json")).unwrap();
        for input in corpus["i64"].as_array().unwrap() {
            let n = input.as_str().unwrap().parse::<i64>().unwrap();
            let result = builds.invoke(mode, "boundary.integer", &[&format!("I64:{n}")], 1000, 10000);
            assert_eq!(result["value"], json!({"type":"I64","value":n.to_string()}));
        }
        for invalid in corpus["invalid_i64"].as_array().unwrap() {
            let atom = format!("I64:{}", invalid.as_str().unwrap());
            assert_eq!(
                builds.invoke(mode, "boundary.integer", &[&atom], 1000, 10000)["diagnostic"]["code"],
                "invocation.arguments.invalid"
            );
        }
        for b in [false, true] {
            assert_eq!(
                builds.invoke(mode, "boundary.boolean", &[&format!("Bool:{b}")], 1000, 10000)["value"],
                json!({"type":"Bool","value":b})
            );
        }
        for bytes in ["", "00", "00ff007f80", "225c0ac3a9f09f90ba"] {
            assert_eq!(
                builds.invoke(mode, "boundary.bytes", &[&format!("Bytes:{bytes}")], 1000, 10000)["value"],
                json!({"type":"Bytes","value":bytes})
            );
        }
        assert_eq!(
            builds.invoke(mode, "boundary.version", &[], 1000, 10000)["value"]["value"],
            "1"
        );
        assert_eq!(
            builds.invoke(mode, "boundary.bytes", &["Bytes:00ff"], 1000, 0)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
    }
}

#[test]
fn native_bindings_reject_unknown_operations_arities_and_types() {
    for (before, after, code) in [
        (
            "Native::roundtrip_i64(n)",
            "Native::system(n)",
            "compile.native.binding",
        ),
        (
            "Native::roundtrip_i64(n)",
            "Native::roundtrip_i64(true)",
            "compile.type.mismatch",
        ),
        (
            "Native::roundtrip_i64(n)",
            "Native::roundtrip_i64(n,n)",
            "compile.call.arity",
        ),
        (
            "Native::copy_bytes(b)",
            "Native::copy_bytes(true)",
            "compile.type.mismatch",
        ),
    ] {
        assert_eq!(compile(&SOURCE.replace(before, after)).unwrap_err().code, code);
    }
}

#[test]
fn checked_native_operations_pin_abi_and_cannot_be_retargeted() {
    let artifact = compile(SOURCE).unwrap();
    let mut data: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    let expression = &mut data["program"]["functions"][0]["body"]["value"];
    assert_eq!(expression["kind"], "NativeOp");
    assert_eq!(expression["binding"], "lobo.native.roundtrip_i64");
    assert_eq!(expression["abi"], 1);
    expression["abi"] = json!(2);
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "compile.native.binding"
    );
    expression_mutate(&artifact, "binding", json!("libc.system"));
    expression_mutate(&artifact, "output", json!("I64"));
    let mut old: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    old["language"] = json!("core/1.6");
    assert!(Artifact::from_json(&old.to_string()).is_err());
}
fn expression_mutate(artifact: &Artifact, key: &str, value: Json) {
    let mut data: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    data["program"]["functions"][0]["body"]["value"][key] = value;
    assert!(Artifact::from_json(&data.to_string()).is_err());
}

#[test]
fn unselected_native_runtime_is_not_enabled_for_pure_programs() {
    let artifact =
        compile("module plain; fn answer()->I64 {42} identities {module:\"plain\",answer:\"plain.answer\"}")
            .unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let output = std::process::Command::new(builds.0.join(mode))
            .args(["--entry", "plain.answer", "--native-clock", "true"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(
            serde_json::from_slice::<Json>(&output.stdout).unwrap()["diagnostic"]["code"],
            "invocation.cli.invalid"
        );
    }
}

#[test]
fn native_artifact_mutations_are_rejected_without_panics() {
    let good: Json = serde_json::from_str(&compile(SOURCE).unwrap().to_json().unwrap()).unwrap();
    let changes = [
        ("abi", json!(0)),
        ("abi", json!(2)),
        ("abi", json!(-1)),
        ("abi", json!(4294967296u64)),
        ("binding", json!("system")),
        ("binding", json!("lobo.native.roundtrip_i64;system")),
        ("arguments", json!([])),
        (
            "arguments",
            json!([{"kind":"Literal","value":{"type":"Bool","value":true}}]),
        ),
        ("symbol", json!("system")),
        ("library", json!("/tmp/untrusted.so")),
    ];
    for (field, value) in changes {
        let mut input = good.clone();
        input["program"]["functions"][0]["body"]["value"][field] = value;
        let result = std::panic::catch_unwind(|| Artifact::from_json(&input.to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().is_err(), "accepted {field}");
    }
    for marker in [json!(null), json!(0), json!(2)] {
        let mut input = good.clone();
        input["program"]["native_abi"] = marker;
        assert_eq!(
            Artifact::from_json(&input.to_string()).unwrap_err().code,
            "compile.native.binding"
        );
    }
    for index in 0..good["program"]["variants"].as_array().unwrap().len() {
        let mut input = good.clone();
        input["program"]["variants"][index]["cases"][0]["payload"] = json!("Bool");
        // BoolResult already has this payload, so mutate its Err arm instead.
        input["program"]["variants"][index]["cases"][1]["payload"] = json!("Bool");
        assert_eq!(
            Artifact::from_json(&input.to_string()).unwrap_err().code,
            "compile.native.binding"
        );
    }
    for version in [
        "core/1.0", "core/1.1", "core/1.2", "core/1.3", "core/1.4", "core/1.5", "core/1.6",
    ] {
        let mut input = good.clone();
        input["language"] = json!(version);
        assert_eq!(
            Artifact::from_json(&input.to_string()).unwrap_err().code,
            "invocation.artifact.version"
        );
    }
}

#[test]
fn native_types_in_unused_expressions_cannot_be_loaded_under_old_language_tags() {
    for body in [
        "let unused=List::empty<Native::Stream>();42",
        "let unused:List<Native::Capability>=List::empty<Native::Capability>();42",
        "let unused=while (state:List<Native::Stream>=List::empty<Native::Stream>();false) {state};42",
    ] {
        let source =
            format!("module m; fn answer()->I64 {{{body}}} identities {{module:\"m\",answer:\"answer\"}}");
        let mut value: Json = serde_json::from_str(&compile(&source).unwrap().to_json().unwrap()).unwrap();
        value["language"] = json!("core/1.6");
        assert_eq!(
            Artifact::from_json(&value.to_string()).unwrap_err().code,
            "invocation.artifact.version"
        );
    }
}

#[test]
fn checked_artifacts_cannot_impersonate_native_catalog_declarations() {
    let good: Json = serde_json::from_str(&compile(SOURCE).unwrap().to_json().unwrap()).unwrap();
    let mut input = good.clone();
    input["program"]["functions"][0]["id"] = json!("lobo.native.roundtrip_i64");
    let scope = input["program"]["scopes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"] == "boundary")
        .unwrap();
    let declarations = scope["declarations"].as_array_mut().unwrap();
    let id = declarations
        .iter_mut()
        .find(|id| **id == json!("boundary.integer"))
        .unwrap();
    *id = json!("lobo.native.roundtrip_i64");
    assert_eq!(
        Artifact::from_json(&input.to_string()).unwrap_err().code,
        "compile.native.binding"
    );
}

#[test]
fn earlier_artifacts_keep_their_preexisting_native_looking_user_identities() {
    let artifact = Artifact::from_json(include_str!("fixtures/native/legacy-native-names.loboc")).unwrap();
    let legacy_source = r#"module old @id("lobo.native.user"); variant Marker @id("lobo.native.result.I64Result") {Unset,Set(I64)} function main @id("lobo.native.previous")()->Marker {Marker::Set(42)}"#;
    let current = compile(legacy_source).unwrap();
    let previous: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    let current: Json = serde_json::from_str(&current.to_json().unwrap()).unwrap();
    assert_eq!(previous["program"], current["program"]);
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "lobo.native.previous", &[], 10000, 10000);
        assert_eq!(result["value"]["value"]["case"], "Set");
        assert_eq!(result["value"]["value"]["payload"]["value"], "42");
    }
}
