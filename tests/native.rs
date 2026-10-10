use lobo_core::{compile, Artifact, Value};
use serde_json::Value as Json;
use std::process::Command;

fn native(artifact: &Artifact, entry: &str, arguments: &[Value], fuel: u64) -> Json {
    let mut options = vec!["--entry".into(), entry.into(), "--fuel".into(), fuel.to_string()];
    for value in arguments {
        options.push("--arg".into());
        options.push(match value {
            Value::I64(n) => format!("I64:{n}"),
            Value::Bool(b) => format!("Bool:{b}"),
        });
    }
    let output = artifact.run_native(&options).unwrap();
    let json: Json = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| panic!("{e}: {output:?}"));
    assert_eq!(output.status.success(), json["status"] == "ok");
    json
}

#[test]
fn native_core_corpus_agrees_with_specification_and_evaluator() {
    let corpus: Json = serde_json::from_str(include_str!("../spec/conformance/core-cases.json")).unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        if case["expected"]["kind"] == "compile_error" {
            continue;
        }
        let artifact = compile(case["source"].as_str().unwrap()).unwrap();
        let arguments: Vec<Value> = serde_json::from_value(case["invocation"]["arguments"].clone()).unwrap();
        let output = native(
            &artifact,
            case["invocation"]["entry"].as_str().unwrap(),
            &arguments,
            100000,
        );
        assert_eq!(output["kind"], case["expected"]["kind"], "{}", case["id"]);
        if output["kind"] == "value" {
            assert_eq!(output["value"], case["expected"]["value"], "{}", case["id"]);
        } else {
            assert_eq!(
                output["diagnostic"]["code"], case["expected"]["code"],
                "{}",
                case["id"]
            );
        }
    }
}

#[test]
fn native_boundaries_short_circuiting_recursion_and_host_errors() {
    let cases = [
        ("-9223372036854775808 / -1", "I64"),
        ("-9223372036854775808 % -1", "I64"),
        ("-(-9223372036854775808)", "I64"),
        ("-9223372036854775808 - 1", "I64"),
        ("9223372036854775807 * 2", "I64"),
        ("1 % 0", "I64"),
        ("7 / -3", "I64"),
        ("true || (1 / 0 == 0)", "Bool"),
        ("if false { 1 / 0 } else { let x = 8; x }", "I64"),
        ("(1 / 0) + (9223372036854775807 + 1)", "I64"),
    ];
    for (body, ty) in cases {
        let artifact = compile(&format!(
            "module demo @id(\"demo\"); function main @id(\"main\")() -> {ty} {{ {body} }}"
        ))
        .unwrap();
        let expected = artifact.run("main", vec![], 100000);
        let result = native(&artifact, "main", &[], 100000);
        match expected {
            Ok(value) => assert_eq!(result["value"], serde_json::to_value(value).unwrap(), "{body}"),
            Err(error) => assert_eq!(result["diagnostic"]["code"], error.code, "{body}"),
        }
    }
    let artifact = compile("module demo @id(\"demo\"); function fact @id(\"fact\")(n: I64) -> I64 requires { n >= 0 } ensures { result >= 1 } { if n == 0 { 1 } else { n * fact(n - 1) } }").unwrap();
    assert_eq!(
        native(&artifact, "fact", &[Value::I64(5)], 100000)["value"]["value"],
        "120"
    );
    assert_eq!(
        native(&artifact, "fact", &[Value::I64(-1)], 100000)["diagnostic"]["code"],
        "runtime.contract.precondition"
    );
    assert_eq!(
        native(&artifact, "fact", &[Value::I64(5)], 1)["kind"],
        "resource_exhausted"
    );
    assert_eq!(
        native(&artifact, "fact", &[Value::Bool(true)], 0)["kind"],
        "invocation_error"
    );
    assert_eq!(
        native(&artifact, "missing", &[], 0)["diagnostic"]["code"],
        "invocation.entry.unknown"
    );
    let looping =
        compile("module demo @id(\"demo\"); function main @id(\"main\")() -> I64 { main() }").unwrap();
    assert_eq!(
        native(&looping, "main", &[], 100000)["diagnostic"]["code"],
        "resource.stack.exhausted"
    );
}

#[test]
fn cli_default_produces_native_machine_code() {
    let cli = env!("CARGO_BIN_EXE_lobo");
    let source = format!("{}/examples/core.lobo", env!("CARGO_MANIFEST_DIR"));
    let directory = std::env::temp_dir().join(format!(
        "lobo-native-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let binary = directory.join("core");
    let output = Command::new(cli)
        .args(["compile", &source, "-o", binary.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let bytes = std::fs::read(&binary).unwrap();
    #[cfg(target_os = "linux")]
    assert_eq!(&bytes[..4], b"\x7fELF");
    let run = Command::new(&binary)
        .args(["--entry", "example.increment", "--arg", "I64:41"])
        .output()
        .unwrap();
    assert!(run.status.success());
    assert_eq!(
        serde_json::from_slice::<Json>(&run.stdout).unwrap()["value"]["value"],
        "42"
    );
    let refusal = Command::new(cli)
        .args(["compile", &source, "-o", binary.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!refusal.status.success());
    assert_eq!(std::fs::read(&binary).unwrap(), bytes);
    let convenience = Command::new(cli)
        .args(["run", &source, "--entry", "example.absolute", "--arg", "I64:-7"])
        .output()
        .unwrap();
    assert!(convenience.status.success(), "{convenience:?}");
    assert_eq!(
        serde_json::from_slice::<Json>(&convenience.stdout).unwrap()["value"]["value"],
        "7"
    );
    std::fs::remove_file(binary).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
