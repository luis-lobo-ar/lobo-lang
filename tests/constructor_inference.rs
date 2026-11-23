use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
mod support;
use support::Builds;

const SOURCE: &str = r#"module inference;
record Box<T> {value:T}
variant Lookup<T> {Missing,Found(T)}
fn choose(flag:Bool)->Lookup<Box<I64>> {
  if flag {Found(Box {value:42})} else {Missing}
}
fn read(value:Lookup<Box<I64>>)->I64 {
  match value {Missing=>0,Found(box)=>box.value}
}
fn main(flag:Bool)->I64 {read(choose(flag))}
identities {module:"ctor",Box:"ctor.box",Lookup:"ctor.lookup",choose:"ctor.choose",read:"ctor.read",main:"ctor.main"}
"#;

#[test]
fn contextual_cases_and_records_equal_explicit_checked_native_code() {
    let inferred = compile(SOURCE).unwrap();
    let explicit = SOURCE
        .replace(
            "{Found(Box {value:42})}",
            "{Lookup::<Box<I64>>::Found(Box::<I64> {value:42})}",
        )
        .replace("else {Missing}", "else {Lookup::<Box<I64>>::Missing}");
    assert_eq!(
        inferred.to_json().unwrap(),
        compile(&explicit).unwrap().to_json().unwrap()
    );
    let builds = Builds::new(&Artifact::from_json(&inferred.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        for (flag, value) in [("true", "42"), ("false", "0")] {
            assert_eq!(
                builds.invoke(mode, "ctor.main", &[&format!("Bool:{flag}")], 1000, 1000)["value"]["value"],
                value
            );
        }
    }
}

#[test]
fn payloads_fields_annotations_and_call_arguments_determine_constructor_types() {
    let source = r#"module m;
record Pair<T> {left:T,right:T}
variant Maybe<T> {None,Some(T)}
fn select<T>(left:T,right:T)->T {left}
fn missing<T>()->Maybe<T> {None}
fn consume(value:Maybe<I64>)->I64 {match value {None=>0,Some(n)=>n}}
fn main()->I64 {
  let qualified=Maybe::Some(42);
  let pair=Pair {left:Maybe::None,right:qualified};
  let value:Maybe<I64> = None;
  let absent=select(Maybe::None,qualified);
  consume(select(Some(42),qualified)) + consume(pair.left) + consume(value) + consume(absent) + consume(missing::<I64>())
}
identities {module:"m",Pair:"pair",Maybe:"maybe",select:"select",missing:"missing",consume:"consume",main:"main"}"#;
    assert_eq!(
        compile(source).unwrap().run("main", vec![], 1000).unwrap(),
        Value::I64(42)
    );
}

#[test]
fn unknown_owners_unresolved_parameters_conflicts_and_shadowed_cases_fail_closed() {
    let prefix = "module m; variant Maybe<T> {None,Some(T)} record Pair<T> {left:T,right:T} ";
    for body in [
        "fn main()->I64 {let value=None; 0}",
        "fn main()->I64 {let value=Maybe::None; 0}",
        "fn main()->I64 {let pair=Pair {left:1,right:true}; 0}",
        "fn main()->Maybe<I64> {Some(true)}",
        "fn main()->Maybe<I64> {Some(1,2)}",
        "fn main()->Maybe<I64> {Unknown(1)}",
        "fn main(None:I64)->Maybe<I64> {None}",
    ] {
        let source =
            format!("{prefix}{body} identities {{module:\"m\",Maybe:\"maybe\",Pair:\"pair\",main:\"main\"}}");
        let error = compile(&source).unwrap_err();
        assert!(error.code.starts_with("compile."), "{body}: {error:?}");
        assert!(error.range.is_some(), "{error:?}");
    }
}

#[test]
fn expected_nominal_identity_selects_imported_cases_without_name_search() {
    let library="module models; variant Maybe<T> {None,Some(T)} variant Other<T> {None,Some(T)} identities {module:\"models\",Maybe:\"maybe\",Other:\"other\"}";
    let app="module app; use models {Maybe,Other}; fn create()->Maybe<I64> {Some(42)} fn main()->I64 {match create() {None=>0,Some(n)=>n}} identities {module:\"app\",models.Maybe:\"maybe\",models.Other:\"other\",create:\"create\",main:\"main\"}";
    let mut sources = vec![
        SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        SourceFile {
            name: "lib".into(),
            text: library.into(),
            identities: None,
        },
    ];
    let a = compile_bundle("app", &sources).unwrap();
    assert_eq!(a.run("main", vec![], 100).unwrap(), Value::I64(42));
    sources[0].text = app.replace("{Some(42)}", "{models.Maybe::<I64>::Some(42)}");
    assert_eq!(
        a.to_json().unwrap(),
        compile_bundle("app", &sources).unwrap().to_json().unwrap()
    );
    sources[0].text = app.replace("{Some(42)}", "{models.Other::Some(42)}");
    assert!(compile_bundle("app", &sources).is_err());
}

#[test]
fn generic_mvc_omits_constructor_qualifiers_without_changing_program_or_ids() {
    let source = include_str!("../examples/mvc.generic.lobo");
    let old = source
        .replace("Found(User {", "Lookup::<User>::Found(User {")
        .replace("\n    Missing\n", "\n    Lookup::<User>::Missing\n");
    let ids =
        lobo_core::identities::parse_manifest(include_str!("../examples/mvc.generic.lobo.ids.json")).unwrap();
    let sources = |text: &str| {
        vec![
            SourceFile {
                name: "mvc".into(),
                text: text.into(),
                identities: Some(ids.clone()),
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
        ]
    };
    assert_eq!(
        compile_bundle("mvc", &sources(source))
            .unwrap()
            .to_json()
            .unwrap(),
        compile_bundle("mvc", &sources(&old)).unwrap().to_json().unwrap()
    );
}

#[test]
fn constructor_context_reaches_loop_initializers_and_callbacks_remain_pure() {
    let source = r#"module m;
record Processor<T> {run:Fn(T)->I64}
variant Maybe<T> {None,Some(T)}
fn read(n:I64)->I64 {n}
fn main()->I64 {
  let processor=Processor {run:read};
  let absent=fold(i in 2,value:Maybe<I64> = None) {None};
  let value=while(state:Maybe<I64> = Some(42); false) {None};
  match value {None=>0,Some(n)=>processor.run(n)}
}

identities {module:"m",Processor:"processor",Maybe:"maybe",read:"read",main:"main"}"#;
    assert_eq!(
        compile(source).unwrap().run("main", vec![], 1000).unwrap(),
        Value::I64(42)
    );
    assert_eq!(
        compile(&source.replace(
            "fn read(n:I64)->I64 {n}",
            "fn read(n:I64)->I64 actions {demo.read} {n}"
        ))
        .unwrap_err()
        .code,
        "compile.action.reference"
    );
}

#[test]
fn inference_stays_source_only_and_legacy_constructors_stay_explicit() {
    let artifact = compile(SOURCE).unwrap();
    let mut encoded: serde_json::Value = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    let function = encoded["program"]["functions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| f["id"] == "ctor.choose")
        .unwrap();
    function["body"]["value"] = serde_json::json!({"kind":"Name","name":"Missing"});
    assert!(Artifact::from_json(&encoded.to_string()).is_err());
    let legacy="module m @id(\"m\"); variant Maybe<T> @id(\"maybe\") {None,Some(T)} function main @id(\"main\")()->Maybe<I64> {Maybe::None}";
    assert_eq!(compile(legacy).unwrap_err().code, "compile.generic.arguments");
}
