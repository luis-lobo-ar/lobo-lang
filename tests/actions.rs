use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};
use std::io::Write;
use std::process::{Command, Stdio};

fn source(declaration: &str, body: &str) -> String {
    format!("module m @id(\"m\"); function main @id(\"main\")() -> I64 {declaration} {{ {body} }}")
}

fn leaf() -> &'static str {
    "function leaf @id(\"leaf\")() -> I64 actions { demo.observe } { 42 }"
}

#[test]
fn actions_are_contextual_canonical_bounds_and_run_as_native_code() {
    let text = format!(
        "{} {}",
        source("actions { demo.observe, demo.extra }", "leaf()"),
        leaf()
    );
    let artifact = compile(&text).unwrap();
    let encoded = artifact.to_json().unwrap();
    let data: Json = serde_json::from_str(&encoded).unwrap();
    assert_eq!(data["format"], "lobo-checked/1.0");
    assert_eq!(data["language"], lobo_core::LANGUAGE);
    assert_eq!(
        data["program"]["functions"][0]["actions"],
        json!(["demo.extra", "demo.observe"])
    );
    let reordered = text.replace(
        "demo.observe, demo.extra",
        "demo.extra, // comment\n demo . observe",
    );
    assert_eq!(encoded, compile(&reordered).unwrap().to_json().unwrap());
    let loaded = Artifact::from_json(&encoded).unwrap();
    for item in [&artifact, &loaded] {
        assert_eq!(item.run("main", vec![], 100).unwrap(), Value::I64(42));
        let output = item.run_native(&["--entry".into(), "main".into()]).unwrap();
        assert!(output.status.success(), "{output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["value"], json!({"type":"I64", "value":"42"}));
        assert!(!item.emit_c().unwrap().contains("demo.observe"));
    }
    let pure = compile(&source("actions {}", "42")).unwrap();
    assert_eq!(
        pure.to_json().unwrap(),
        compile(&source("", "42")).unwrap().to_json().unwrap()
    );
}

#[test]
fn every_call_edge_including_unreachable_expressions_requires_a_bound() {
    for body in [
        "leaf()",
        "if false { leaf() } else { 0 }",
        "fold(i in 0, a: I64 = 0) { leaf() }",
        "fold(i in leaf(), a: I64 = 0) { a }",
        "fold(i in 0, a: I64 = leaf()) { a }",
    ] {
        let text = format!("{} {}", source("", body), leaf());
        let error = compile(&text).unwrap_err();
        assert_eq!(error.code, "compile.action.undeclared", "{body}");
        assert_eq!(error.subject.as_deref(), Some("main"));
        let range = error.range.unwrap();
        assert_eq!(&text[range.start.byte..range.end.byte], "leaf()");
    }
    let transitive = format!(
        "{} function mid @id(\"mid\")() -> I64 actions {{ demo.observe }} {{ leaf() }} {}",
        source("", "mid()"),
        leaf()
    );
    assert_eq!(
        compile(&transitive).unwrap_err().code,
        "compile.action.undeclared"
    );
}

#[test]
fn recursive_bounds_are_checked_without_call_graph_unrolling() {
    let text = "module m @id(\"m\");
      function a @id(\"a\")(n: I64) -> I64 actions { demo.observe } { if n == 0 { 42 } else { b(n - 1) } }
      function b @id(\"b\")(n: I64) -> I64 actions { demo.observe } { a(n) }";
    assert_eq!(
        compile(text).unwrap().run("a", vec![Value::I64(2)], 100).unwrap(),
        Value::I64(42)
    );
    let broken = text.replacen("actions { demo.observe }", "actions {}", 1);
    assert_eq!(compile(&broken).unwrap_err().code, "compile.action.undeclared");
}

#[test]
fn contracts_cannot_call_actionful_functions_even_with_matching_body_bounds() {
    for contract in [
        "requires { leaf() == 42 }",
        "ensures { leaf() == result }",
        "requires { true || leaf() == 42 }",
    ] {
        let text = format!(
            "{} {}",
            source(&format!("actions {{ demo.observe }} {contract}"), "42"),
            leaf()
        );
        assert_eq!(compile(&text).unwrap_err().code, "compile.action.contract");
    }
    let text = format!(
        "{} function pure @id(\"pure\")() -> I64 {{ 42 }}",
        source(
            "actions { demo.observe } requires { pure() == 42 } ensures { pure() == result }",
            "42"
        )
    );
    assert_eq!(
        compile(&text).unwrap().run("main", vec![], 100).unwrap(),
        Value::I64(42)
    );
}

#[test]
fn action_declarations_reject_duplicates_malformed_ids_and_excess() {
    for (clause, code) in [
        (
            "actions { demo.observe, demo.observe }",
            "compile.action.duplicate",
        ),
        ("actions { single }", "compile.action.invalid"),
        ("actions { demo. }", "compile.syntax.invalid"),
        ("actions { demo.observe, }", "compile.syntax.invalid"),
        ("actions {} actions {}", "compile.syntax.invalid"),
    ] {
        assert_eq!(compile(&source(clause, "42")).unwrap_err().code, code, "{clause}");
    }
    let oversized = format!(
        "actions {{ {} }}",
        (0..65)
            .map(|n| format!("demo.a{n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(
        compile(&source(&oversized, "42")).unwrap_err().code,
        "resource.frontend.limit"
    );
    let long = format!("actions {{ demo.{} }}", "a".repeat(128));
    assert_eq!(
        compile(&source(&long, "42")).unwrap_err().code,
        "compile.action.invalid"
    );
    assert!(compile(&source(&oversized.replace(",demo.a64", ""), "42")).is_ok());
    assert_eq!(
        compile(&source("actions { demo.if }", "42")).unwrap_err().code,
        "compile.action.invalid"
    );
}

#[test]
fn executable_action_example_preserves_scalar_faults_and_emitted_code() {
    let text = include_str!("../examples/actions.lobo");
    let artifact = compile(text).unwrap();
    let unannotated = compile(&text.replace("actions { demo.observe }", "")).unwrap();
    assert_eq!(artifact.emit_c().unwrap(), unannotated.emit_c().unwrap());
    for (argument, expected) in [
        (41, Ok(Value::I64(42))),
        (i64::MAX, Err("runtime.contract.precondition")),
    ] {
        assert_eq!(
            artifact
                .run("example.actions.pipeline", vec![Value::I64(argument)], 100)
                .map_err(|e| e.code),
            expected.map_err(String::from)
        );
        let output = artifact
            .run_native(&[
                "--entry".into(),
                "example.actions.pipeline".into(),
                "--arg".into(),
                format!("I64:{argument}"),
            ])
            .unwrap();
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        if argument == 41 {
            assert!(output.status.success());
            assert_eq!(result["value"]["value"], "42");
        } else {
            assert!(!output.status.success());
            assert_eq!(result["diagnostic"]["code"], "runtime.contract.precondition");
        }
    }
}

#[test]
fn reloaded_artifacts_cannot_erase_bounds_or_smuggle_invalid_actions() {
    let text = format!("{} {}", source("actions { demo.observe }", "leaf()"), leaf());
    let original: Json = serde_json::from_str(&compile(&text).unwrap().to_json().unwrap()).unwrap();
    for (actions, code) in [
        (json!([]), "compile.action.undeclared"),
        (
            json!(["demo.observe", "demo.observe"]),
            "compile.action.duplicate",
        ),
        (json!(["demo.z", "demo.observe"]), "compile.action.invalid"),
        (json!(["single"]), "compile.action.invalid"),
    ] {
        let mut data = original.clone();
        data["program"]["functions"][0]["actions"] = actions;
        let error = Artifact::from_json(&data.to_string()).unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.range.is_none());
    }
    let mut data = original.clone();
    data["program"]["functions"][0]
        .as_object_mut()
        .unwrap()
        .remove("actions");
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "invocation.artifact.invalid"
    );
    let mut data = original;
    data["format"] = json!("lobo-checked/0.2");
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
}

#[test]
fn stdin_checks_expose_declared_action_sets_by_stable_id() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args(["check", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source("actions { demo.observe }", "42").as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["actions"], json!({"main": ["demo.observe"]}));
}
