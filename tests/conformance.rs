use lobo_core::{compile, Artifact, Value};
use serde_json::Value as Json;

fn program(body: &str, output: &str) -> String {
    format!("module demo @id(\"demo\"); function main @id(\"main\")() -> {output} {{ {body} }}")
}

#[test]
fn core_corpus_matches_the_specification() {
    let corpus: Json = serde_json::from_str(include_str!("../spec/conformance/core-cases.json")).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(ids.insert(id), "duplicate fixture {id}");
        let compiled = compile(case["source"].as_str().unwrap());
        let expected = &case["expected"];
        if expected["kind"] == "compile_error" {
            let error = compiled.expect_err(id);
            assert_eq!(error.kind(), "compile_error", "{id}");
            assert_eq!(error.code, expected["code"].as_str().unwrap(), "{id}");
            continue;
        }
        let artifact = compiled.unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
        for implementation in [&artifact, &loaded] {
            let arguments: Vec<Value> =
                serde_json::from_value(case["invocation"]["arguments"].clone()).unwrap();
            let result =
                implementation.run(case["invocation"]["entry"].as_str().unwrap(), arguments, 100_000);
            if expected["kind"] == "runtime_fault" {
                let error = result.expect_err(id);
                assert_eq!(error.kind(), "runtime_fault", "{id}");
                assert_eq!(error.code, expected["code"].as_str().unwrap(), "{id}");
            } else {
                let value = result.unwrap_or_else(|e| panic!("{id}: {e:?}"));
                assert_eq!(serde_json::to_value(value).unwrap(), expected["value"], "{id}");
            }
        }
    }
}

#[test]
fn arithmetic_boundaries_and_evaluation_order() {
    let values = [
        ("7 / -3", Value::I64(-2)),
        ("7 % -3", Value::I64(1)),
        ("-7 / 3", Value::I64(-2)),
        ("1 - 2 - 3", Value::I64(-4)),
        ("false || true && false", Value::Bool(false)),
        ("true || (1 / 0 == 0)", Value::Bool(true)),
        ("!false", Value::Bool(true)),
        ("if false { 1 / 0 } else { 8 }", Value::I64(8)),
        ("00042", Value::I64(42)),
    ];
    for (body, expected) in values {
        let ty = if matches!(expected, Value::Bool(_)) {
            "Bool"
        } else {
            "I64"
        };
        assert_eq!(
            compile(&program(body, ty))
                .unwrap()
                .run("main", vec![], 1000)
                .unwrap(),
            expected,
            "{body}"
        );
    }
    let faults = [
        ("-9223372036854775808 - 1", "runtime.integer.overflow"),
        ("9223372036854775807 * 2", "runtime.integer.overflow"),
        ("-(-9223372036854775808)", "runtime.integer.overflow"),
        ("-9223372036854775808 / -1", "runtime.integer.overflow"),
        ("-9223372036854775808 % -1", "runtime.integer.overflow"),
        ("42 % 0", "runtime.integer.zero_divisor"),
        (
            "(1 / 0) + (9223372036854775807 + 1)",
            "runtime.integer.zero_divisor",
        ),
        ("(9223372036854775807 + 1) + (1 / 0)", "runtime.integer.overflow"),
    ];
    for (body, expected) in faults {
        let error = compile(&program(body, "I64"))
            .unwrap()
            .run("main", vec![], 1000)
            .unwrap_err();
        assert_eq!(error.code, expected, "{body}");
    }
}

#[test]
fn scope_types_and_syntax_rejection() {
    let cases = [
        ("let x = x; x", "I64", "compile.name.unresolved"),
        ("let x = 1; let x = 2; x", "I64", "compile.name.duplicate"),
        ("let main = 1; main", "I64", "compile.name.duplicate"),
        ("let result = 1; result", "I64", "compile.name.reserved"),
        ("let x: Bool = 1; x", "Bool", "compile.type.mismatch"),
        (
            "if true { let x = 1; x } else { x }",
            "I64",
            "compile.name.unresolved",
        ),
        (
            "if true { let x = 1; x } else { let x = 2; x }",
            "Bool",
            "compile.type.mismatch",
        ),
        ("main(1)", "I64", "compile.call.arity"),
        ("1 < 2 < 3", "Bool", "compile.syntax.invalid"),
        ("1 == 2 == false", "Bool", "compile.syntax.invalid"),
        ("-(9223372036854775808)", "I64", "compile.literal.range"),
        ("result", "I64", "compile.name.unresolved"),
        ("1;", "I64", "compile.syntax.invalid"),
        ("if true { 1 }", "I64", "compile.syntax.invalid"),
    ];
    for (body, ty, expected) in cases {
        assert_eq!(compile(&program(body, ty)).unwrap_err().code, expected, "{body}");
    }
}

#[test]
fn recursion_contracts_and_resource_limits() {
    let source = "module demo @id(\"demo\"); function fact @id(\"factorial\")(n: I64) -> I64 requires { n >= 0 } ensures { result >= 1 } { if n == 0 { 1 } else { n * fact(n - 1) } }";
    let artifact = compile(source).unwrap();
    assert_eq!(
        artifact.run("factorial", vec![Value::I64(5)], 1000).unwrap(),
        Value::I64(120)
    );
    assert_eq!(
        artifact
            .run("factorial", vec![Value::I64(-1)], 1000)
            .unwrap_err()
            .code,
        "runtime.contract.precondition"
    );
    assert_eq!(
        artifact
            .run("factorial", vec![Value::I64(5)], 1)
            .unwrap_err()
            .kind(),
        "resource_exhausted"
    );
    assert_eq!(
        artifact
            .run("factorial", vec![Value::Bool(true)], 1000)
            .unwrap_err()
            .kind(),
        "invocation_error"
    );
    assert_eq!(
        artifact.run("unknown", vec![], 1000).unwrap_err().kind(),
        "invocation_error"
    );
    assert_eq!(
        artifact.run("unknown", vec![], 0).unwrap_err().kind(),
        "invocation_error"
    );
    assert_eq!(
        artifact
            .run("factorial", vec![Value::Bool(true)], 0)
            .unwrap_err()
            .kind(),
        "invocation_error"
    );
    let looping = compile(&program("main()", "I64")).unwrap();
    assert_eq!(
        looping.run("main", vec![], 100_000).unwrap_err().kind(),
        "resource_exhausted"
    );

    let source = "module demo @id(\"demo\"); function main @id(\"main\")() -> I64 requires { 1 / 0 == 0 } ensures { false } { 1 }";
    let error = compile(source).unwrap().run("main", vec![], 1000).unwrap_err();
    assert_eq!(error.code, "runtime.integer.zero_divisor");
    assert_eq!(error.phase, "precondition");
    let source =
        "module demo @id(\"demo\"); function main @id(\"main\")() -> I64 ensures { 1 / 0 == 0 } { 1 }";
    let error = compile(source).unwrap().run("main", vec![], 1000).unwrap_err();
    assert_eq!(error.code, "runtime.integer.zero_divisor");
    assert_eq!(error.phase, "postcondition");
}

#[test]
fn artifacts_are_deterministic_resolved_and_revalidated() {
    let source = "module demo @id(\"demo\"); function main @id(\"main\")() -> I64 { other(3) } function other @id(\"stable.other\")(x: I64) -> I64 { x }";
    let a = compile(source).unwrap().to_json().unwrap();
    let b = compile(&format!("// comment\n{source}\n"))
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(a, b);
    assert!(a.contains("\"function\": \"stable.other\""));
    assert_eq!(Artifact::from_json(&a).unwrap().to_json().unwrap(), a);
    let mut json: Json = serde_json::from_str(&a).unwrap();
    json["format"] = "unsupported".into();
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
    json["format"] = lobo_core::FORMAT.into();
    json["program"]["functions"][0]["body"]["value"]["function"] = "missing.id".into();
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "compile.name.unresolved"
    );
    json["unexpected"] = true.into();
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "invocation.artifact.invalid"
    );
}

#[test]
fn frontend_depth_is_bounded_and_wider_syntax_is_rejected() {
    let deeply_nested = format!("{}1{}", "(".repeat(200), ")".repeat(200));
    assert_eq!(
        compile(&program(&deeply_nested, "I64")).unwrap_err().kind(),
        "resource_exhausted"
    );
    assert!(compile(include_str!("../examples/transfer.lobo")).is_err());
    let mut body = "1".to_string();
    for _ in 0..28 {
        body = format!("if true {{ {body} }} else {{ 0 }}");
    }
    let artifact = compile(&program(&body, "I64")).unwrap();
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(loaded.run("main", vec![], 1000).unwrap(), Value::I64(1));
}
