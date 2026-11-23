use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
mod support;
use support::Builds;

const SOURCE: &str = r#"module inference;
record Item { n:I64 }
fn identity<T>(x:T)->T {x}
fn apply<T,U>(f:Fn(T)->U,x:T)->U {f(x)}
fn extract(item:Item)->I64 {item.n}
fn main()->I64 {
  let item = identity(Item {n:41});
  apply(extract,item) + identity(1)
}
identities {module:"infer",Item:"infer.item",identity:"infer.identity",apply:"infer.apply",extract:"infer.extract",main:"infer.main"}
"#;

#[test]
fn inferred_calls_and_contextual_names_equal_explicit_checked_native_code() {
    let inferred = compile(SOURCE).unwrap();
    let explicit = SOURCE
        .replace("identity(Item", "identity::<Item>(Item")
        .replace("apply(extract,item)", "apply::<Item,I64>(&extract,item)")
        .replace("identity(1)", "identity::<I64>(1)");
    assert_eq!(
        inferred.to_json().unwrap(),
        compile(&explicit).unwrap().to_json().unwrap()
    );
    let builds = Builds::new(&Artifact::from_json(&inferred.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "infer.main", &[], 1000, 1000)["value"]["value"],
            "42"
        );
    }
}

#[test]
fn inferred_generic_bodies_remain_parametric_and_callbacks_can_use_expected_signatures() {
    let source = r#"module m;
fn identity<T>(x:T)->T {x}
fn forward<U>(x:U)->U {identity(x)}
fn apply<T>(x:T,f:Fn(T)->T)->T {f(x)}
fn main()->I64 {apply(forward(42),identity)}
identities {module:"m",identity:"identity",forward:"forward",apply:"apply",main:"main"}"#;
    assert_eq!(
        compile(source).unwrap().run("main", vec![], 1000).unwrap(),
        Value::I64(42)
    );
    assert_eq!(
        compile(&source.replace("{identity(x)}", "{identity(x)+1}"))
            .unwrap_err()
            .code,
        "compile.type.mismatch"
    );
}

#[test]
fn inference_rejects_unresolved_conflicting_and_impure_callbacks_with_original_spans() {
    let cases = [
        ("fn choose<T>(a:T,b:T)->T {a} fn main()->I64 {choose(1,true)}", "choose(1,true)", "compile.type.mismatch"),
        ("fn make<T>()->List<T> {List::empty<T>()} fn main()->List<I64> {make()}", "make()", "compile.generic.inference"),
        ("fn apply<T>(x:T,f:Fn(T)->T)->T {f(x)} fn observe(x:I64)->I64 actions {demo.read} {x} fn main()->I64 {apply(1,observe)}", "observe)", "compile.action.reference"),
        ("fn apply<T>(x:T,f:Fn(T)->T)->T {f(x)} fn wrong(x:Bool)->Bool {x} fn main()->I64 {apply(1,wrong)}", "apply(1,wrong)", "compile.type.mismatch"),
    ];
    for (body, needle, code) in cases {
        let names = if body.contains("choose") {
            "choose:\"choose\",main:\"main\""
        } else if body.contains("make") {
            "make:\"make\",main:\"main\""
        } else if body.contains("observe") {
            "apply:\"apply\",observe:\"observe\",main:\"main\""
        } else {
            "apply:\"apply\",wrong:\"wrong\",main:\"main\""
        };
        let source = format!("// 🐺\r\nmodule m; {body} identities {{module:\"m\",{names}}}");
        let error = serde_json::to_value(compile(&source).unwrap_err()).unwrap();
        assert_eq!(error["code"], code, "{error}");
        let start = error["range"]["start"]["byte"].as_u64().unwrap() as usize;
        assert!(start >= source.find(needle).unwrap(), "{error}");
    }
}

#[test]
fn inferred_imports_retain_scope_and_stable_specialization_identity() {
    let app = "module app; use shared {identity}; fn main()->I64 {shared.identity(42)} identities {module:\"app\",main:\"main\",shared.identity:\"identity\"}";
    let lib = "module lib; fn identity<T>(x:T)->T {x} identities {module:\"lib\",identity:\"identity\"}";
    let sources = vec![
        SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        SourceFile {
            name: "lib".into(),
            text: lib.into(),
            identities: None,
        },
    ];
    let inferred = compile_bundle("app", &sources).unwrap();
    let mut explicit = sources.clone();
    explicit[0].text = app.replace("shared.identity(42)", "shared.identity::<I64>(42)");
    assert_eq!(
        inferred.to_json().unwrap(),
        compile_bundle("app", &explicit).unwrap().to_json().unwrap()
    );
    let mut reversed = sources;
    reversed.reverse();
    assert_eq!(
        inferred.to_json().unwrap(),
        compile_bundle("app", &reversed).unwrap().to_json().unwrap()
    );
}

#[test]
fn nested_explicit_calls_do_not_repeat_inference_work_exponentially() {
    let expression = format!("{}1{}", "identity::<I64>(".repeat(18), ")".repeat(18));
    let source=format!("module m; fn identity<T>(x:T)->T {{x}} fn main()->I64 {{{expression}}} identities {{module:\"m\",identity:\"identity\",main:\"main\"}}");
    assert_eq!(
        compile(&source).unwrap().run("main", vec![], 1000).unwrap(),
        Value::I64(1)
    );
}

#[test]
fn maintained_generic_mvc_matches_explicit_source_and_native_responses() {
    let source = include_str!("../examples/mvc.generic.lobo");
    let ids =
        lobo_core::identities::parse_manifest(include_str!("../examples/mvc.generic.lobo.ids.json")).unwrap();
    let entry = ids["show_user"].clone();
    let mut sources = vec![
        SourceFile {
            name: "mvc".into(),
            text: source.into(),
            identities: Some(ids),
        },
        SourceFile {
            name: "http".into(),
            text: include_str!("../libraries/http/http.lobo").into(),
            identities: None,
        },
        SourceFile {
            name: "json".into(),
            text: include_str!("../libraries/json/json.lobo").into(),
            identities: None,
        },
    ];
    let inferred = compile_bundle("mvc", &sources).unwrap();
    sources[0].text = source.replace(
        "respond_json(find_user(id), render_user)",
        "respond_json::<User>(find_user(id), &render_user)",
    );
    assert_eq!(
        inferred.to_json().unwrap(),
        compile_bundle("mvc", &sources).unwrap().to_json().unwrap()
    );
    let builds = Builds::new(&inferred);
    for mode in ["eval", "native", "sanitized"] {
        for (user, status, body) in [(1, "200", "{\"name\":\"Luis\"}"), (42, "404", "Not found\n")] {
            let result = builds.invoke(mode, &entry, &[&format!("I64:{user}")], 10000, 10000);
            assert_eq!(
                result["value"]["value"]["fields"]["status"]["value"], status,
                "{result}"
            );
            assert_eq!(
                result["value"]["value"]["fields"]["body"]["value"], body,
                "{result}"
            );
        }
    }
}

#[test]
fn inferred_function_signatures_are_bounded_before_substitution_expands_them() {
    for (depth, valid) in [(4, true), (12, false)] {
        let expression = format!("{}1{}", "grow(".repeat(depth), ")".repeat(depth));
        let source=format!("module m; fn choose<T>(a:T,b:T)->T {{a}} fn grow<T>(x:T)->Fn(T,T)->T {{&choose::<T>}} fn main()->I64 {{let value={expression}; 0}} identities {{module:\"m\",choose:\"choose\",grow:\"grow\",main:\"main\"}}");
        let result = compile(&source);
        if valid {
            assert_eq!(result.unwrap().run("main", vec![], 1000).unwrap(), Value::I64(0));
        } else {
            assert_eq!(result.unwrap_err().code, "resource.frontend.limit");
        }
    }
}
