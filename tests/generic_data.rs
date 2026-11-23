use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
mod support;
use support::Builds;

const LIB: &str = r#"module lib @id("data.lib");
record Cursor<T> @id("data.cursor") { rest: List<T>, count: I64 }
variant Option<T> @id("data.option") { None, Some(T) }
variant Result<T,E> @id("data.result") { Ok(T), Err(E) }
record Node<T> @id("data.node") { value: T, children: List<Node<T>> }
function first<T> @id("data.first")(xs: List<T>) -> Option<T> {
  if List::length(xs) == 0 { Option::<T>::None }
  else { Option::<T>::Some(List::head(xs)) }
}
"#;

#[test]
fn generic_data_crosses_modules_and_is_concrete_in_checked_artifacts() {
    let app = r#"module app @id("data.app");
import Cursor @id("data.cursor"); import Option @id("data.option");
import Result @id("data.result"); import Node @id("data.node");
import first @id("data.first");
function f @id("f")(xs: List<I64>) -> I64 {
  let c: Cursor<I64> = Cursor::<I64> { rest: xs, count: 7 };
  match (first::<I64>(c.rest)) { None { 0 }, Some(x) { x + c.count } }
}
function text @id("text")() -> Text {
  let r = Result::<Text,I64>::Ok("é");
  match (r) { Ok(x) { x }, Err(e) { "bad" } }
}
function tree @id("tree")() -> I64 {
  let leaf = Node::<I64> { value: 9, children: List::empty<Node<I64>>() };
  let n = Node::<I64> { value: 99, children: List::prepend(leaf,List::empty<Node<I64>>()) };
  List::head(n.children).value
}
"#;
    let artifact = compile_bundle(
        "app",
        &[
            SourceFile {
                identities: None,
                name: "lib".into(),
                text: LIB.into(),
            },
            SourceFile {
                identities: None,
                name: "app".into(),
                text: app.into(),
            },
        ],
    )
    .unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(artifact.run("tree", vec![], 100).unwrap(), Value::I64(9));
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "f", &["List:Count:1,I64:5"], 1000, 1000)["value"]["value"],
            "12"
        );
        assert_eq!(
            builds.invoke(mode, "f", &["List:Count:0"], 1000, 1000)["value"]["value"],
            "0"
        );
        assert_eq!(
            builds.invoke(mode, "text", &[], 1000, 1000)["value"]["value"],
            "é"
        );
        assert_eq!(
            builds.invoke(mode, "tree", &[], 1000, 1000)["value"]["value"],
            "9"
        );
    }
}

#[test]
fn generic_data_requires_exact_arguments_and_rejects_inline_cycles() {
    for (ty, ctor, code) in [
        ("Box", "Box { value: 1 }", "compile.generic.arguments"),
        (
            "Box<I64,Text>",
            "Box::<I64,Text> { value: 1 }",
            "compile.generic.arguments",
        ),
        ("Box<I64>", "Box::<I64> { value: true }", "compile.type.mismatch"),
        (
            "Box<Missing>",
            "Box::<Missing> { value: 1 }",
            "compile.type.unknown",
        ),
    ] {
        let source = format!("module m @id(\"m\"); record Box<T> @id(\"box\") {{ value: T }} function f @id(\"f\")() -> {ty} {{ {ctor} }}");
        assert_eq!(compile(&source).unwrap_err().code, code, "{source}");
    }
    let source = "module m @id(\"m\"); record Loop<T> @id(\"loop\") { next: Loop<T> } function f @id(\"f\")(x: Loop<I64>) -> Loop<I64> { x }";
    assert!(compile(source).unwrap_err().code.starts_with("compile.record."));
}

#[test]
fn generic_type_annotations_allow_adjacent_assignment_like_list_annotations() {
    let source="module m @id(\"m\"); record Box<T> @id(\"box\") { value:T } function f @id(\"f\")() -> I64 { let b:Box<I64>=Box::<I64> { value:7 }; b.value }";
    assert_eq!(
        compile(source).unwrap().run("f", vec![], 100).unwrap(),
        Value::I64(7)
    );
}

#[test]
fn nominal_admission_preserves_source_bound_and_exact_instance_boundary() {
    let mut declarations = String::from("module m @id(\"m\");");
    for n in 0..65 {
        declarations.push_str(&format!("record R{n} @id(\"r{n}\") {{ value:I64 }}"));
    }
    assert_eq!(
        compile(&format!("{declarations} function f @id(\"f\")() -> I64 {{ 0 }}"))
            .unwrap_err()
            .code,
        "resource.frontend.limit"
    );
    let mut source = String::from("module m @id(\"m\"); record Pair<T,U> @id(\"pair\") { left:T,right:U }");
    for n in 0..12 {
        source.push_str(&format!("record R{n} @id(\"r{n}\") {{ value:I64 }}"));
    }
    source.push_str("function f @id(\"f\")() -> I64 {");
    for n in 0..129 {
        let (a, b) = (n / 12, n % 12);
        source.push_str(&format!(
            "let p{n} = Pair::<R{a},R{b}> {{ left:R{a} {{ value:0 }},right:R{b} {{ value:0 }} }};"
        ));
        if n == 127 {
            assert!(compile(&format!("{source} 0 }}")).is_ok());
        }
    }
    assert_eq!(
        compile(&format!("{source} 0 }}")).unwrap_err().code,
        "resource.frontend.limit"
    );
}

#[test]
fn checked_artifacts_reject_generic_data_templates_applications_and_arguments() {
    let a=compile("module m @id(\"m\"); record Box<T> @id(\"box\") { value:T } variant Option<T> @id(\"option\") { None,Some(T) } function f @id(\"f\")() -> Box<I64> { Box::<I64> { value:7 } } function g @id(\"g\")() -> Option<I64> { Option::<I64>::Some(7) }").unwrap();
    let original: serde_json::Value = serde_json::from_str(&a.to_json().unwrap()).unwrap();
    for mode in 0..6 {
        let mut value = original.clone();
        match mode {
            0 => value["program"]["records"][0]["type_parameters"] = serde_json::json!(["T"]),
            1 => value["program"]["variants"][0]["type_parameters"] = serde_json::json!(["T"]),
            2 => {
                value["program"]["functions"][0]["output"] =
                    serde_json::json!({"Applied":{"name":"box","arguments":["I64"]}})
            }
            3 => value["program"]["functions"][0]["output"] = serde_json::json!({"Parameter":"T"}),
            4 => {
                value["program"]["functions"][0]["body"]["value"]["type_arguments"] =
                    serde_json::json!(["I64"])
            }
            _ => {
                value["program"]["functions"][1]["body"]["value"]["type_arguments"] =
                    serde_json::json!(["I64"])
            }
        }
        assert!(Artifact::from_json(&value.to_string()).is_err(), "forgery {mode}");
    }
}
