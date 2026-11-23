use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
mod support;
use support::Builds;

fn same_outcome(actual: serde_json::Value, mut explicit: serde_json::Value, entry: &str, named: &str) {
    // Caller identities differ by construction. Callee fault identities must match exactly.
    if explicit["diagnostic"]["subject"] == named {
        assert_eq!(actual["diagnostic"]["subject"], entry);
        explicit["diagnostic"]["subject"] = entry.into();
    }
    assert_eq!(actual, explicit);
}

const SOURCE: &str = r#"module vectors @id("vectors");
record Vec2 @id("vec") { x: I64, y: I64 }
operator +(Vec2, Vec2) -> Vec2 = add;
operator *(Vec2, I64) -> Vec2 = scale;
function add @id("vector.add")(a: Vec2, b: Vec2) -> Vec2
requires { b.x >= 0 } { Vec2 { x: a.x + b.x, y: a.y + b.y } }
function scale @id("vector.scale")(a: Vec2, n: I64) -> Vec2 {
  Vec2 { x: a.x * n, y: a.y * n }
}
function symbolic @id("symbolic")(n: I64) -> I64 {
  (Vec2 { x: 2, y: 3 } + Vec2 { x: n, y: 4 } * 3).x
}
function explicit @id("explicit")(n: I64) -> I64 {
  add(Vec2 { x: 2, y: 3 }, scale(Vec2 { x: n, y: 4 }, 3)).x
}
function text @id("text")(name: Text) -> Text { "Hello, " + (name + "!") }
function text_explicit @id("text.explicit")(name: Text) -> Text {
  Text::concat("Hello, ", Text::concat(name, "!"))
}
"#;

#[test]
fn typed_bindings_and_text_sugar_lower_to_existing_checked_calls() {
    let artifact = compile(SOURCE).unwrap();
    let json = artifact.to_json().unwrap();
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let body = &data["program"]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == "symbolic")
        .unwrap()["body"]["value"]["base"];
    assert_eq!(body["kind"], "Call");
    assert_eq!(body["function"], "vector.add");
    assert_eq!(body["arguments"][1]["function"], "vector.scale");
    let functions = data["program"]["functions"].as_array().unwrap();
    for (symbolic, explicit) in [("symbolic", "explicit"), ("text", "text.explicit")] {
        let body = |id| &functions.iter().find(|f| f["id"] == id).unwrap()["body"];
        assert_eq!(body(symbolic), body(explicit));
    }
    assert!(data["program"].get("operators").is_none());
    let artifact = Artifact::from_json(&json).unwrap();
    assert_eq!(
        artifact.run("symbolic", vec![Value::I64(5)], 1000).unwrap(),
        Value::I64(17)
    );
    for mutation in ["target", "signature", "declaration"] {
        let mut changed = data.clone();
        match mutation {
            "target" => {
                let f = changed["program"]["functions"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|f| f["id"] == "symbolic")
                    .unwrap();
                f["body"]["value"]["base"]["function"] = "missing".into();
            }
            "signature" => changed["program"]["functions"][0]["output"] = "Bool".into(),
            _ => changed["program"]["operators"] = serde_json::json!([]),
        }
        assert!(Artifact::from_json(&changed.to_string()).is_err(), "{mutation}");
    }
}

#[test]
fn arithmetic_bindings_support_distinct_outputs_variants_and_concrete_generic_bodies() {
    let source = r#"module m @id("ops.all");
      variant Count @id("count") { N(I64) }
      operator +(Count, Count)->I64 = add;
      operator -(Count, I64)->Count = sub;
      operator *(Count, I64)->Count = mul;
      operator /(Count, I64)->Count = div;
      operator %(Count, I64)->Count = rem;
      function value @id("value")(a:Count)->I64 { match(a) { N(n) {n} } }
      function add @id("plus")(a:Count,b:Count)->I64 { value(a)+value(b) }
      function sub @id("minus")(a:Count,n:I64)->Count { Count::N(value(a)-n) }
      function mul @id("times")(a:Count,n:I64)->Count { Count::N(value(a)*n) }
      function div @id("divide")(a:Count,n:I64)->Count { Count::N(value(a)/n) }
      function rem @id("remainder")(a:Count,n:I64)->Count { Count::N(value(a)%n) }
      function generic<T> @id("generic.ops")(unused:T)->I64 {
        let v = Count::N(19) - 3;
        let vs = List::prepend(v * 2, List::empty<Count>());
        let q = if true { List::head(vs) / 5 } else { v };
        let r = while (s:Count = q; value(s)>3) { s - 1 };
        fold(i in 2, total:I64 = 0) { total + (r % 3 + Count::N(1)) }
      }
      function main @id("all")()->I64 { generic::<Text>("x") }
    "#;
    let a = compile(source).unwrap();
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "all", &[], 10000, 1000)["value"]["value"],
            "2"
        );
    }
}

#[test]
fn operator_arguments_execute_once_in_left_to_right_order() {
    let source = format!(
        r#"{SOURCE}
      function left @id("left")()->Vec2 requires {{ false }} {{ Vec2 {{x:0,y:0}} }}
      function right @id("right")()->Vec2 requires {{ false }} {{ Vec2 {{x:0,y:0}} }}
      function order @id("order")()->Vec2 {{ left() + right() }}"#
    );
    let a = compile(&source).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "order", &[], 1000, 1000);
        assert_eq!(result["diagnostic"]["subject"], "left");
        assert_eq!(result["diagnostic"]["phase"], "precondition");
    }
}

#[test]
fn bindings_accept_exact_list_and_function_signatures_in_contracts() {
    let source = r#"module m @id("ops.signatures");
      record Box @id("ops.box") { value:I64 }
      operator *(Box, Fn(I64)->I64)->I64 = apply;
      operator +(Box, List<I64>)->List<I64> = prepend;
      operator -(Box, Box)->Bool = same;
      function inc @id("ops.inc")(n:I64)->I64 {n+1}
      function apply @id("ops.apply")(box:Box, callback:Fn(I64)->I64)->I64 {callback(box.value)}
      function prepend @id("ops.prepend")(box:Box, tail:List<I64>)->List<I64> {List::prepend(box.value,tail)}
      function same @id("ops.same")(a:Box,b:Box)->Bool {a.value==b.value}
      function checked @id("ops.checked")(box:Box)->I64
      requires {box - box}
      ensures {result == box * &inc}
      {List::head(box + List::empty<I64>()) + 1}
      function main @id("ops.main")()->I64 {checked(Box {value:5})}
    "#;
    let a = compile(source).unwrap();
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "ops.main", &[], 1000, 1000)["value"]["value"],
            "6"
        );
    }
}

#[test]
fn binding_admission_limits_and_old_checked_artifact_compatibility_are_explicit() {
    let too_many = format!("{SOURCE}{}", "operator +(Vec2,Vec2)->Vec2 = add;".repeat(64));
    assert_eq!(compile(&too_many).unwrap_err().code, "resource.frontend.limit");
    let generic = SOURCE.replace("record Vec2 @id", "record Vec2<T> @id");
    assert_eq!(compile(&generic).unwrap_err().code, "compile.operator.signature");
    let generic_target = SOURCE.replace("function add @id", "function add<T> @id");
    assert_eq!(
        compile(&generic_target).unwrap_err().code,
        "compile.operator.signature"
    );
    let original = compile("module m @id(\"compat\"); function f @id(\"f\")()->I64 {42}").unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&original.to_json().unwrap()).unwrap();
    data["language"] = "core/1.0".into();
    assert_eq!(
        Artifact::from_json(&data.to_string())
            .unwrap()
            .run("f", vec![], 100)
            .unwrap(),
        Value::I64(42)
    );
}

#[test]
fn native_operators_preserve_explicit_call_values_faults_and_budgets() {
    let a = compile(SOURCE).unwrap();
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        for n in [5, -1, i64::MAX] {
            let arg = format!("I64:{n}");
            for fuel in [0, 3, 20, 1000] {
                for heap in [0, 1000] {
                    same_outcome(
                        builds.invoke(mode, "symbolic", &[&arg], fuel, heap),
                        builds.invoke(mode, "explicit", &[&arg], fuel, heap),
                        "symbolic",
                        "explicit",
                    );
                }
            }
        }
        for fuel in [0, 2, 1000] {
            for heap in [0, 4, 1000] {
                same_outcome(
                    builds.invoke(mode, "text", &["Text:c3a9"], fuel, heap),
                    builds.invoke(mode, "text.explicit", &["Text:c3a9"], fuel, heap),
                    "text",
                    "text.explicit",
                );
            }
        }
    }
}

#[test]
fn binding_declarations_enforce_signature_purity_ownership_and_coherence() {
    for (source, code) in [
        (
            SOURCE.replace(
                "operator +(Vec2, Vec2) -> Vec2 = add;",
                "operator +(Vec2, Vec2) -> I64 = add;",
            ),
            "compile.operator.signature",
        ),
        (
            SOURCE.replace(
                "function add @id(\"vector.add\")(a: Vec2, b: Vec2) -> Vec2",
                "function add @id(\"vector.add\")(a: Vec2, b: Vec2) -> Vec2 actions { demo.read }",
            ),
            "compile.operator.purity",
        ),
        (
            SOURCE.replace(
                "operator +(Vec2, Vec2) -> Vec2 = add;",
                "operator +(I64, I64) -> I64 = add;",
            ),
            "compile.operator.ownership",
        ),
        (
            SOURCE.replace(
                "operator +(Vec2, Vec2) -> Vec2 = add;",
                "operator +(Vec2, Vec2) -> Vec2 = add; operator +(Vec2, Vec2) -> Vec2 = add;",
            ),
            "compile.operator.duplicate",
        ),
        (
            SOURCE.replace("operator +(Vec2, Vec2)", "operator ==(Vec2, Vec2)"),
            "compile.operator.symbol",
        ),
        (SOURCE.replace("= add;", "= missing;"), "compile.name.unresolved"),
        (
            SOURCE.replace("Vec2 { x: n, y: 4 } * 3", "3 * Vec2 { x: n, y: 4 }"),
            "compile.type.mismatch",
        ),
        (
            format!("{SOURCE} function bad<T> @id(\"bad\")(a:T,b:T)->T {{ a+b }}"),
            "compile.type.mismatch",
        ),
    ] {
        let e = compile(&source).unwrap_err();
        assert_eq!(e.code, code, "{}", e.message);
        assert!(e.range.is_some(), "{}", e.message);
    }
    for source in [
        "module m @id(\"m\"); function f @id(\"f\")()->Text { \"x\" + 1 }",
        "module m @id(\"m\"); function f @id(\"f\")()->Bool { false && (1/0==0) }",
    ] {
        let result = compile(source);
        if source.contains("false") {
            assert_eq!(result.unwrap().run("f", vec![], 100).unwrap(), Value::Bool(false));
        } else {
            assert!(result.is_err());
        }
    }
}

#[test]
fn importing_the_named_target_enables_its_binding_without_ambient_lookup() {
    let library = SourceFile {
        identities: None,
        name: "vectors.lobo".into(),
        text: SOURCE.into(),
    };
    let root = SourceFile {
        identities: None,
        name: "app.lobo".into(),
        text: r#"module app @id("app");
      import V @id("vec"); import sum @id("vector.add");
      function f @id("app.f")()->I64 { (V {x:2,y:0} + V {x:4,y:0}).x }"#
            .into(),
    };
    let artifact = compile_bundle("app.lobo", &[root.clone(), library.clone()]).unwrap();
    assert_eq!(artifact.run("app.f", vec![], 1000).unwrap(), Value::I64(6));
    let mut missing = root.clone();
    missing.text = missing.text.replace("import sum @id(\"vector.add\");", "");
    let e = compile_bundle("app.lobo", &[missing, library.clone()]).unwrap_err();
    assert_eq!(e.code, "compile.type.mismatch");
    assert_eq!(e.source.as_deref().map(|s| s.as_str()), Some("app.lobo"));
    let mut foreign = root;
    foreign.text = foreign.text.replace(
        "function f",
        "operator +(V,V)->V = own; function own @id(\"app.own\")(a:V,b:V)->V {a} function f",
    );
    assert_eq!(
        compile_bundle("app.lobo", &[foreign, library]).unwrap_err().code,
        "compile.operator.ownership"
    );
}
