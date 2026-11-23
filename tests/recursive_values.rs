use lobo_core::{compile, Artifact, Value};
mod support;
use support::Builds;

const SOURCE: &str = r#"module recursive @id("recursive");
variant Node @id("node") { Num(I64), Sum(List<Node>) }
function echo @id("echo")(x: Node) -> Node { x }
function build @id("build")(n: I64) -> Node {
  fold(i in n, x: Node = Node::Num(7)) { Node::Sum(List::prepend(x, List::empty<Node>())) }
}
function shallow @id("shallow")(n: I64) -> I64 {
  match(build(n)) { Num(k) { k }, Sum(xs) { List::length(xs) } }
}
function same @id("same")(n: I64) -> Bool { build(n) == build(n) }
function contract @id("contract")(n: I64) -> I64 requires { build(n) == build(n) } { 0 }
"#;

#[test]
fn recursive_list_values_roundtrip_and_run_natively() {
    let artifact = compile(SOURCE).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let text = "Variant:node:Case:Sum,Count:2,Case:Num,I64:2,Case:Sum,Count:0";
    let value = artifact.parse_argument("echo", 0, text).unwrap();
    let expected = serde_json::to_value(&value).unwrap();
    assert_eq!(
        serde_json::to_value(artifact.run("echo", vec![value], 100).unwrap()).unwrap(),
        expected
    );
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "echo", &[text], 100, 10);
        assert_eq!(result["value"], expected, "{mode}: {result}");
        let result = builds.invoke(mode, "echo", &[text], 100, 9);
        assert_eq!(result["diagnostic"]["code"], "resource.memory.exhausted");
        for bad in [
            "Variant:node:Case:Sum,Count:1,Case:Num,Bool:true",
            "Variant:node:Case:Sum,Count:1,Case:Missing",
            "Variant:node:Case:Sum,Count:1",
            "Variant:node:Case:Sum,Count:0,I64:7",
        ] {
            assert_eq!(
                builds.invoke(mode, "echo", &[bad], 0, 0)["diagnostic"]["code"],
                "invocation.arguments.invalid"
            );
        }
    }
}

#[test]
fn list_guarding_allows_mutual_types_but_never_an_inline_cycle() {
    let source = r#"module m @id("m");
      record A @id("a") { children: List<B> }
      variant B @id("b") { Wrap(A) }
      function main @id("main")() -> B { B::Wrap(A { children: List::empty<B>() }) }"#;
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(builds.invoke(mode, "main", &[], 100, 0)["status"], "ok");
    }
    for bad in [
        "record A @id(\"a\") { same: A, children: List<A> }",
        "record A @id(\"a\") { b: B } variant B @id(\"b\") { Again(A) }",
    ] {
        let source = format!("module m @id(\"m\"); {bad} function f @id(\"f\")() -> I64 {{ 0 }}");
        assert_eq!(compile(&source).unwrap_err().code, "compile.record.cycle");
    }
}

#[test]
fn recursive_value_input_output_and_comparison_depth_are_classified() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    let too_deep = format!("Variant:node:{}Case:Num,I64:7", "Case:Sum,Count:1,".repeat(5));
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "echo", &[&too_deep], 0, 0)["diagnostic"]["code"],
            "resource.input.limit"
        );
        assert_eq!(
            builds.invoke(mode, "build", &["I64:4"], 1000, 1000)["diagnostic"]["code"],
            "resource.output.limit"
        );
        assert_eq!(
            builds.invoke(mode, "same", &["I64:31"], 10000, 1000)["value"]["value"],
            true
        );
        assert_eq!(
            builds.invoke(mode, "same", &["I64:32"], 10000, 1000)["diagnostic"]["code"],
            "resource.stack.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "same", &["I64:32"], 1, 1000)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        let result = builds.invoke(mode, "contract", &["I64:32"], 10000, 1000);
        assert_eq!(result["diagnostic"]["code"], "resource.stack.exhausted");
        assert_eq!(result["diagnostic"]["phase"], "precondition");
    }
}

#[test]
fn recursive_ast_evaluation_and_rewriting_obey_frozen_values_and_preserve_order() {
    let original = include_str!("../examples/ast.lobo");
    let missing = original.replace(
        ",\n    Product(children) => evaluate_children(children, true)",
        "",
    );
    assert_ne!(
        missing, original,
        "the negative fixture must remove the Product arm"
    );
    assert_eq!(compile(&missing).unwrap_err().code, "compile.match.invalid");
    let artifact = compile(include_str!("../examples/ast.lobo")).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let builds = Builds::new(&artifact);
    let cases = [
        ("Case:Num,I64:7", 7, 8),
        ("Case:Sum,Count:0", 0, 0),
        ("Case:Product,Count:0", 1, 1),
        ("Case:Sum,Count:3,Case:Num,I64:2,Case:Product,Count:2,Case:Num,I64:3,Case:Num,I64:4,Case:Sum,Count:0", 14, 23),
        ("Case:Product,Count:2,Case:Num,I64:-2,Case:Sum,Count:2,Case:Num,I64:1,Case:Num,I64:4", -10, -7),
    ];
    for (contents, value, shifted) in cases {
        let input = format!("Variant:example.ast.node:{contents}");
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(
                builds.invoke(mode, "example.ast.evaluate", &[&input], 10000, 10000)["value"]["value"],
                value.to_string()
            );
            assert_eq!(
                builds.invoke(
                    mode,
                    "example.ast.shifted_value",
                    &[&input, "I64:1"],
                    10000,
                    10000
                )["value"]["value"],
                shifted.to_string()
            );
            let original = builds.invoke(mode, "example.ast.shift", &[&input, "I64:0"], 10000, 10000);
            let expected = artifact
                .parse_argument("example.ast.evaluate", 0, &input)
                .unwrap();
            assert_eq!(original["value"], serde_json::to_value(expected).unwrap());
        }
    }
    let input = "Variant:example.ast.node:Case:Sum,Count:3,Case:Num,I64:8,Case:Num,I64:2,Case:Num,I64:5";
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "example.ast.shift", &[input, "I64:1"], 10000, 10000);
        let elements = result["value"]["value"]["payload"]["value"]["elements"]
            .as_array()
            .unwrap();
        let actual: Vec<_> = elements
            .iter()
            .map(|n| n["value"]["payload"]["value"].as_str().unwrap())
            .collect();
        assert_eq!(actual, ["9", "3", "6"]);
        let too_big = "Variant:example.ast.node:Case:Num,I64:9223372036854775807";
        assert_eq!(
            builds.invoke(mode, "example.ast.shift", &[too_big, "I64:1"], 100, 100)["diagnostic"]["code"],
            "runtime.integer.overflow"
        );
    }
}

#[test]
fn recursive_artifacts_scope_and_forged_inline_cycles_are_revalidated() {
    let source = r#"module defs @id("defs"); variant Node @id("node") { Num(I64), Sum(List<Node>) }"#;
    let caller = r#"module caller @id("caller"); import Tree @id("node"); function f @id("f")() -> Tree { Tree::Sum(List::empty<Tree>()) }"#;
    let sources = [
        lobo_core::SourceFile {
            identities: None,
            name: "defs".into(),
            text: source.into(),
        },
        lobo_core::SourceFile {
            identities: None,
            name: "caller".into(),
            text: caller.into(),
        },
    ];
    let artifact = lobo_core::compile_bundle("caller", &sources).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(builds.invoke(mode, "f", &[], 100, 0)["status"], "ok");
    }
    let artifact = compile(SOURCE).unwrap();
    let mut wire = serde_json::to_value(&artifact).unwrap();
    let mut legacy = wire.clone();
    legacy["format"] = serde_json::json!("lobo-checked/0.9");
    legacy["language"] = serde_json::json!("core/0.9-draft");
    assert_eq!(
        Artifact::from_json(&legacy.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
    wire["program"]["variants"][0]["cases"][1]["payload"] = serde_json::json!({"Variant":"node"});
    assert_eq!(
        Artifact::from_json(&wire.to_string()).unwrap_err().code,
        "compile.record.cycle"
    );
}

#[test]
fn deeply_nested_internal_values_are_reclaimed_without_recursing_the_host_stack() {
    let artifact = compile(SOURCE).unwrap();
    assert_eq!(
        artifact
            .run_with_heap("shallow", vec![Value::I64(10000)], 500000, 50000)
            .unwrap(),
        Value::I64(1)
    );
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "shallow", &["I64:10000"], 500000, 50000)["value"]["value"],
            "1"
        );
        assert_eq!(
            builds.invoke(mode, "shallow", &["I64:10000"], 500000, 49999)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "build", &["I64:10000"], 500000, 50000)["diagnostic"]["code"],
            "resource.output.limit"
        );
    }
}
