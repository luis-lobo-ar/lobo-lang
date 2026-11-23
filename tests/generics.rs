use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
mod support;
use support::Builds;

const LIB: &str = r#"module reusable @id("reuse.lib");
function copy<T> @id("reuse.copy")(xs: List<T>, suffix: List<T>) -> List<T>
ensures { List::length(result) == List::length(xs) + List::length(suffix) }
{ if List::length(xs) == 0 { suffix } else {
  List::prepend(List::head(xs), copy::<T>(List::tail(xs), suffix))
} }
function contains<T> @id("reuse.contains")(xs: List<T>, value: T) -> Bool {
  if List::length(xs) == 0 { false } else {
    if List::head(xs) == value { true } else { contains::<T>(List::tail(xs), value) }
  }
}
"#;
const APP: &str = r#"module app @id("reuse.app");
import copy @id("reuse.copy"); import contains @id("reuse.contains");
record Item @id("reuse.item") { key: I64, name: Text }
function ints @id("ints")(xs: List<I64>, ys: List<I64>) -> List<I64> { copy::<I64>(xs, ys) }
function texts @id("texts")(xs: List<Text>, ys: List<Text>) -> List<Text> { copy::<Text>(xs, ys) }
function items @id("items")(xs: List<Item>, x: Item) -> Bool { contains::<Item>(xs, x) }
function nested @id("nested")(xs: List<List<I64>>) -> List<List<I64>> {
  copy::<List<I64>>(xs, List::empty<List<I64>>())
}
"#;

fn sources(app: &str) -> Vec<SourceFile> {
    vec![
        SourceFile {
            identities: None,
            name: "app".into(),
            text: app.into(),
        },
        SourceFile {
            identities: None,
            name: "lib".into(),
            text: LIB.into(),
        },
    ]
}

#[test]
fn one_generic_library_handles_scalars_text_records_and_nested_lists_natively() {
    let artifact = compile_bundle("app", &sources(APP)).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(
            mode,
            "ints",
            &["List:Count:2,I64:1,I64:2", "List:Count:1,I64:3"],
            1000,
            1000,
        );
        assert_eq!(
            result["value"]["value"]["elements"],
            serde_json::json!([
                {"type":"I64","value":"1"},{"type":"I64","value":"2"},{"type":"I64","value":"3"}
            ]),
            "{mode}: {result}"
        );
        let result = builds.invoke(
            mode,
            "texts",
            &["List:Count:2,Text:6869,Text:c3a9", "List:Count:0"],
            1000,
            1000,
        );
        assert_eq!(result["value"]["value"]["elements"][1]["value"], "é", "{result}");
        for (xs, expected) in [
            ("List:Count:1,I64:7,Text:78", true),
            ("List:Count:1,I64:8,Text:78", false),
            ("List:Count:0", false),
        ] {
            let result = builds.invoke(
                mode,
                "items",
                &[xs, "Record:reuse.item:I64:7,Text:78"],
                1000,
                1000,
            );
            assert_eq!(result["value"]["value"], expected, "{mode}: {result}");
        }
        assert_eq!(
            builds.invoke(mode, "ints", &["List:Count:0", "List:Count:1,I64:3"], 1000, 1000)["status"],
            "ok"
        );
        assert_eq!(
            builds.invoke(mode, "nested", &["List:Count:1,Count:1,I64:5"], 1000, 1000)["status"],
            "ok"
        );
        assert_eq!(
            builds.invoke(mode, "ints", &["List:Count:1,I64:1", "List:Count:0"], 0, 1000)["diagnostic"]
                ["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "ints", &["List:Count:1,I64:1", "List:Count:0"], 1000, 4)["diagnostic"]
                ["code"],
            "resource.memory.exhausted"
        );
        for (heap, status) in [(7, "error"), (8, "ok")] {
            assert_eq!(
                builds.invoke(mode, "ints", &["List:Count:1,I64:1", "List:Count:0"], 1000, heap)["status"],
                status
            );
        }
        let long = format!("List:Count:40{}", ",I64:1".repeat(40));
        assert_eq!(
            builds.invoke(mode, "ints", &[&long, "List:Count:0"], 10000, 10000)["diagnostic"]["code"],
            "resource.stack.exhausted"
        );
    }
}

#[test]
fn generic_calls_are_explicit_and_concrete_types_are_checked() {
    for (body, code) in [
        ("copy(xs, xs)", "compile.generic.arguments"),
        ("copy::<I64, Text>(xs, xs)", "compile.generic.arguments"),
        ("copy::<Text>(xs, xs)", "compile.type.mismatch"),
        ("copy::<Missing>(xs, xs)", "compile.type.unknown"),
        ("copy::<f>(xs, xs)", "compile.type.unknown"),
    ] {
        let app = format!("module a @id(\"a\"); import copy @id(\"reuse.copy\"); function f @id(\"f\")(xs: List<I64>) -> List<I64> {{ {body} }}");
        let error = compile_bundle("app", &sources(&app)).unwrap_err();
        assert_eq!(error.code, code, "{body}: {error:?}");
        assert!(error.range.is_some());
    }
    let error = compile("module m @id(\"m\"); function f @id(\"f\")(x: I64) -> I64 { x } function g @id(\"g\")() -> I64 { f::<I64>(1) }").unwrap_err();
    assert_eq!(error.code, "compile.generic.arguments");
}

#[test]
fn same_type_recursion_reuses_instances_and_type_growth_is_bounded() {
    let source = r#"module m @id("m");
    function even<T> @id("even")(x: T, n: I64) -> T { if n == 0 { x } else { odd::<T>(x,n-1) } }
    function odd<T> @id("odd")(x: T, n: I64) -> T { even::<T>(x,n) }
    function f @id("f")() -> I64 { even::<I64>(7,4) + even::<I64>(8,2) }"#;
    let artifact = compile(source).unwrap();
    assert_eq!(artifact.entries().len(), 3);
    assert_eq!(artifact.run("f", vec![], 1000).unwrap(), Value::I64(15));
    let growing = r#"module m @id("m"); function grow<T> @id("grow")(x: T) -> I64 { grow::<List<T>>(List::prepend(x,List::empty<T>())) } function f @id("f")() -> I64 { grow::<I64>(0) }"#;
    assert_eq!(compile(growing).unwrap_err().code, "resource.frontend.limit");
}

#[test]
fn generic_actions_and_contracts_keep_existing_guarantees() {
    let source = r#"module m @id("m");
    function observe<T> @id("observe")(x: T) -> T actions { demo.read } { x }
    function identity<T> @id("identity")(x: T) -> T ensures { result == x } { x }
    function bad<T> @id("bad")(x: T) -> T ensures { false } { x }
    function f @id("f")() -> I64 actions { demo.read } { identity::<I64>(observe::<I64>(7)) }
    function fail @id("fail")() -> I64 { bad::<I64>(7) }"#;
    let artifact = compile(source).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(builds.invoke(mode, "f", &[], 100, 0)["value"]["value"], "7");
        let failure = builds.invoke(mode, "fail", &[], 100, 0);
        assert_eq!(failure["diagnostic"]["code"], "runtime.contract.postcondition");
    }
    let missing = source.replace(
        "function f @id(\"f\")() -> I64 actions { demo.read }",
        "function f @id(\"f\")() -> I64",
    );
    assert_eq!(compile(&missing).unwrap_err().code, "compile.action.undeclared");
    let impure = source.replace("ensures { result == x }", "requires { observe::<T>(x) == x }");
    assert_eq!(compile(&impure).unwrap_err().code, "compile.action.contract");
}

#[test]
fn specialization_ids_are_order_independent_and_artifacts_reject_forgery() {
    let artifact = compile_bundle("app", &sources(APP)).unwrap();
    let mut reversed = sources(APP);
    reversed.reverse();
    let second = compile_bundle("app", &reversed).unwrap();
    assert_eq!(artifact.to_json().unwrap(), second.to_json().unwrap());
    let extended = APP.replace(
        "record Item",
        "function extra @id(\"extra\")() -> Bool { contains::<Bool>(List::empty<Bool>(),true) } record Item",
    );
    let extra = compile_bundle("app", &sources(&extended)).unwrap();
    for id in artifact.entries() {
        assert!(extra.entries().contains(&id));
    }
    let original: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    for mode in 0..4 {
        let mut forged = original.clone();
        match mode {
            0 => forged["program"]["functions"][0]["type_parameters"] = serde_json::json!(["T"]),
            1 => {
                forged["program"]["functions"][0]["body"]["value"]["type_arguments"] =
                    serde_json::json!(["Text"])
            }
            2 => {
                let scope = forged["program"]["scopes"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|s| {
                        s["id"].as_str().unwrap().starts_with("lobo.generic.")
                            && s["imports"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|i| i["id"] == "reuse.item")
                    })
                    .unwrap();
                scope["imports"] = serde_json::json!([]);
            }
            _ => {
                forged["format"] = serde_json::json!("lobo-checked/0.10");
            }
        }
        assert!(
            Artifact::from_json(&forged.to_string()).is_err(),
            "forgery {mode}"
        );
    }
}

#[test]
fn templates_are_checked_parametrically_and_preserve_definition_scope() {
    let source = r#"module m @id("m");
    function add<T> @id("add")(x: T) -> T { x + x }
    function f @id("f")() -> I64 { add::<I64>(3) }"#;
    assert_eq!(compile(source).unwrap_err().code, "compile.type.mismatch");
    assert_eq!(
        compile(&source.replace("-> I64 { add::<I64>(3)", "-> Bool { add::<Bool>(true)"))
            .unwrap_err()
            .code,
        "compile.type.mismatch"
    );
    // An unused template must satisfy its signature for every admissible T.
    assert!(compile(&source.replace("add::<I64>(3)", "0")).is_err());
    let library = format!("{LIB} function private @id(\"private\")(x: I64) -> I64 {{ x + 1 }} function wrap<T> @id(\"wrap\")(x: T) -> I64 {{ private(7) }}");
    let app = "module a @id(\"a\"); import wrap @id(\"wrap\"); function f @id(\"f\")() -> I64 { wrap::<Bool>(true) }";
    let sources = [
        SourceFile {
            identities: None,
            name: "app".into(),
            text: app.into(),
        },
        SourceFile {
            identities: None,
            name: "lib".into(),
            text: library,
        },
    ];
    let artifact = compile_bundle("app", &sources).unwrap();
    assert_eq!(artifact.run("f", vec![], 100).unwrap(), Value::I64(8));
    let mut bad = sources.clone();
    bad[0].text = app.replace("wrap::<Bool>(true)", "private(7)");
    assert_eq!(
        compile_bundle("app", &bad).unwrap_err().code,
        "compile.name.unresolved"
    );
    for source in [
        "module m @id(\"m\"); function f<T,T> @id(\"f\")(x: T) -> T { x }",
        "module m @id(\"m\"); record T @id(\"t\") { a: I64 } function f<T> @id(\"f\")(x: T) -> T { x }",
        "module m @id(\"m\"); function id<T> @id(\"id\")(x: T) -> T { x } function f @id(\"f\")() -> I64 { let id = 0; id }",
    ] { assert_eq!(compile(source).unwrap_err().code,"compile.name.duplicate"); }
}

#[test]
fn generic_local_annotations_and_finite_instantiation_budget_are_enforced() {
    let source = r#"module m @id("m");
    function repeat<T> @id("repeat")(x: T) -> List<T> {
      let ys: List<T> = fold(i in 2, xs: List<T> = List::empty<T>()) { List::prepend(x,xs) };
      let zs = while(state: List<T> = ys; List::length(state) > 1) { List::tail(state) };
      zs
    }
    function f @id("f")() -> List<Bool> { repeat::<Bool>(true) }"#;
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "f", &[], 1000, 8)["value"]["value"]["elements"],
            serde_json::json!([{"type":"Bool","value":true}])
        );
    }
    let types = [
        "I64",
        "Bool",
        "Text",
        "List<I64>",
        "List<Bool>",
        "List<Text>",
        "List<List<I64>>",
        "List<List<Bool>>",
        "List<List<Text>>",
    ];
    let value = |ty: &str| match ty {
        "I64" => "0".to_string(),
        "Bool" => "true".to_string(),
        "Text" => "\"\"".to_string(),
        _ => format!("List::empty<{}>()", &ty[5..ty.len() - 1]),
    };
    let mut body = String::new();
    for n in 0..65 {
        let a = types[n / 9];
        let b = types[n % 9];
        body.push_str(&format!(
            "let x{n} = choose::<{a},{b}>({},{});",
            value(a),
            value(b)
        ));
        if n == 63 {
            let source = format!("module m @id(\"m\"); function choose<A,B> @id(\"choose\")(a: A,b: B) -> A {{ a }} function f @id(\"f\")() -> I64 {{ {body} 0 }}");
            assert_eq!(compile(&source).unwrap().entries().len(), 65);
        }
    }
    let source = format!("module m @id(\"m\"); function choose<A,B> @id(\"choose\")(a: A,b: B) -> A {{ a }} function f @id(\"f\")() -> I64 {{ {body} 0 }}");
    // Version 1 raises the concrete function-instance budget from 64 to 256.
    assert_eq!(compile(&source).unwrap().entries().len(), 66);
}
