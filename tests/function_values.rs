use lobo_core::{compile, Artifact};
mod support;
use support::Builds;

const SOURCE: &str = r#"module m @id("fn.module");
record Step<T,U> @id("fn.step") { apply: Fn(T) -> U }
function inc @id("inc")(x: I64) -> I64 { x + 1 }
function twice @id("twice")(x: I64) -> I64 { x * 2 }
function identity<T> @id("identity")(x: T) -> T { x }
function apply<T,U> @id("apply")(f: Fn(T) -> U, x: T) -> U { f(x) }
function choose @id("choose")(b: Bool, x: I64) -> I64 {
  let f = if b { &inc } else { &twice };
  let s = Step::<I64,I64> { apply: f };
  apply::<I64,I64>(s.apply, x)
}
function generic @id("generic")() -> Text { apply::<Text,Text>(&identity::<Text>, "é") }
function invoke @id("invoke")(f: Fn(I64) -> I64, x: I64) -> I64 { f(x) }
function reference @id("reference")() -> Fn(I64) -> I64 { &inc }
function direct @id("direct")(x: I64) -> I64 { Step::<I64,I64> { apply: &inc }.apply(x) }
"#;

#[test]
fn noncapturing_named_references_work_with_runtime_choices_and_generic_data() {
    let a = compile(SOURCE).unwrap();
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        for (b, value) in [("Bool:true", "6"), ("Bool:false", "10")] {
            assert_eq!(
                builds.invoke(mode, "choose", &[b, "I64:5"], 1000, 1000)["value"]["value"],
                value
            );
        }
        assert_eq!(
            builds.invoke(mode, "generic", &[], 1000, 1000)["value"]["value"],
            "é"
        );
        assert_eq!(
            builds.invoke(mode, "invoke", &["Function:inc", "I64:5"], 1000, 1000)["value"]["value"],
            "6"
        );
        assert_eq!(
            builds.invoke(mode, "reference", &[], 1000, 1000)["value"]["value"]["id"],
            "inc"
        );
        assert_eq!(
            builds.invoke(mode, "direct", &["I64:5"], 1000, 1000)["value"]["value"],
            "6"
        );
        for invalid in [
            "Function:missing",
            "Function:generic",
            "Function:inc,Function:twice",
        ] {
            assert_eq!(
                builds.invoke(mode, "invoke", &[invalid, "I64:5"], 1000, 1000)["diagnostic"]["code"],
                "invocation.arguments.invalid"
            );
        }
    }
}

#[test]
fn checked_references_and_programmatic_arguments_are_revalidated() {
    let a = compile(SOURCE).unwrap();
    let original: serde_json::Value = serde_json::from_str(&a.to_json().unwrap()).unwrap();
    for mode in 0..4 {
        let mut data = original.clone();
        let function = data["program"]["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["id"] == "reference")
            .unwrap();
        match mode {
            0 => function["body"]["value"]["function"] = "missing".into(),
            1 => function["body"]["value"]["type_arguments"] = serde_json::json!(["I64"]),
            2 => function["output"]["Function"]["output"] = "Bool".into(),
            _ => {
                function["body"]["value"] = serde_json::json!({"kind":"Literal","value":{"type":"Function","value":{"id":"missing","signature":{"parameters":["I64"],"output":"I64"}}}})
            }
        }
        assert!(Artifact::from_json(&data.to_string()).is_err());
    }
    let mut value = a.parse_argument("invoke", 0, "Function:inc").unwrap();
    if let lobo_core::Value::Function(reference) = &mut value {
        reference.id = "generic".into();
    }
    assert_eq!(
        a.run("invoke", vec![value, lobo_core::Value::I64(5)], 100)
            .unwrap_err()
            .code,
        "invocation.arguments.invalid"
    );
    let source = r#"module m @id("m"); function named @id("callback")(x: I64) -> I64 { x+10 }
      function inc @id("inc")(x: I64) -> I64 { x+1 }
      function f @id("f")(callback: Fn(I64) -> I64) -> I64 { callback(5) + named(5) }"#;
    let a = compile(source).unwrap();
    let value = a.parse_argument("f", 0, "Function:inc").unwrap();
    assert_eq!(a.run("f", vec![value], 100).unwrap(), lobo_core::Value::I64(21));
}

#[test]
fn function_references_enforce_purity_signatures_and_lexical_scope() {
    for source in [
        SOURCE.replace(
            "let f = if b { &inc } else { &twice };",
            "let f = if b { &inc } else { &identity::<Bool> };",
        ),
        SOURCE.replace(
            "function inc @id(\"inc\")(x: I64) -> I64 {",
            "function inc @id(\"inc\")(x: I64) -> I64 actions { demo.read } {",
        ),
        SOURCE.replace("let f = if b { &inc } else { &twice };", "let f = &x;"),
        SOURCE.replace("{ f(x) }", "{ f(true) }"),
    ] {
        assert!(compile(&source).is_err(), "invalid reference accepted");
    }
}
