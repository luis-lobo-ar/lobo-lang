use lobo_core::{compile, Artifact};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

const SOURCE: &str = r#"module conveniences;
record State {left:I64,right:I64}
record Box<T> {value:T,flag:Bool}
variant R<T> {Ok(T),Err(Text)}
variant Maybe {None,Some(I64)}
record Data {text:Text,items:List<I64>,choice:Maybe}
fn base()->State {State {left:1,right:2}}
fn pass()->R<I64> {Ok(4)}
fn fail()->R<I64> {Err("stopped")}
fn update()->I64 {let original=base(); let changed=State { ...original,right:9 }; original.right*100+changed.left*10+changed.right}
fn copy()->I64 {(State { ...base() }).right}
fn nested()->I64 {(State { ...State { ...base(),right:7 },left:5 }).right}
fn generic<T>(value:Box<T>)->Box<T> {Box { ...value,flag:true }}
fn inferred()->I64 {let value=generic(Box {value:42,flag:false}); if value.flag {value.value} else {0}}
fn explicit()->I64 {(Box::<I64> { ...Box::<I64> {value:43,flag:false},flag:true }).value}
fn statement()->R<I64> {try pass(); let n=try pass(); try pass(); Ok(n)}
fn stopped()->R<I64> {try fail(); Ok(1/0)}
fn final_try()->R<I64> {(try pass()); Ok(try pass())}
fn looped()->R<I64> {let n=fold(i in 3,n:I64=0) {try pass(); n+1}; Ok(n)}
fn overrides()->R<State> {Ok(State { ...base(),left:try fail(),right:1/0 })}
fn empty_values()->I64 {
 let original=Data {text:"",items:List::empty<I64>(),choice:None};
 let copied=Data { ...original };
 let filled=Data { ...copied,text:"value" };
 let cleared=Data { ...filled,text:"" };
 if copied.text=="" && cleared.text=="" && List::length(copied.items)==0 && (match(copied.choice) {None=>true,Some(n)=>false}) {1} else {0}
}
fn wrong_base()->State ensures {result.left==0} {base()}
fn base_first()->State {State { ...wrong_base(),left:1/0,right:5 }}
fn order()->State {State { ...base(),right:1/0,left:9223372036854775807+1 }}
fn assured()->R<I64> ensures {match result {Ok(n)=>true,Err(e)=>false}} {try fail(); Ok(1)}
identities {module:"conveniences",State:"conv.state",Box:"conv.box",R:"conv.result",Maybe:"conv.maybe",Data:"conv.data",empty_values:"conv.empty_values",base:"conv.base",pass:"conv.pass",fail:"conv.fail",update:"conv.update",copy:"conv.copy",nested:"conv.nested",generic:"conv.generic",inferred:"conv.inferred",explicit:"conv.explicit",statement:"conv.statement",stopped:"conv.stopped",final_try:"conv.final_try",looped:"conv.looped",overrides:"conv.overrides",wrong_base:"conv.wrong_base",base_first:"conv.base_first",order:"conv.order",assured:"conv.assured"}"#;

#[test]
fn updates_and_try_statements_execute_in_reference_c_llvm_and_sanitizers() {
    let artifact = compile(SOURCE).unwrap();
    let builds = Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "llvm", "sanitized", "llvm-sanitized"] {
        for (entry, expected) in [
            ("update", "219"),
            ("copy", "2"),
            ("nested", "7"),
            ("inferred", "42"),
            ("explicit", "43"),
            ("empty_values", "1"),
        ] {
            let value = builds.invoke(mode, &format!("conv.{entry}"), &[], 100000, 100000);
            assert_eq!(value["value"]["value"], expected, "{mode}/{entry}: {value}");
        }
        for (entry, case, payload) in [
            ("statement", "Ok", "4"),
            ("stopped", "Err", "stopped"),
            ("final_try", "Ok", "4"),
            ("looped", "Ok", "3"),
            ("overrides", "Err", "stopped"),
        ] {
            let value = builds.invoke(mode, &format!("conv.{entry}"), &[], 100000, 100000);
            assert_eq!(value["value"]["value"]["case"], case, "{mode}/{entry}: {value}");
            assert_eq!(value["value"]["value"]["payload"]["value"], payload);
        }
        for (entry, code, subject) in [
            ("base_first", "runtime.contract.postcondition", "conv.wrong_base"),
            ("order", "runtime.integer.zero_divisor", "conv.order"),
            ("assured", "runtime.contract.postcondition", "conv.assured"),
        ] {
            let value = builds.invoke(mode, &format!("conv.{entry}"), &[], 100000, 100000);
            assert_eq!(value["diagnostic"]["code"], code, "{mode}/{entry}: {value}");
            assert_eq!(value["diagnostic"]["subject"], subject);
        }
    }
}

#[test]
fn invalid_updates_and_silent_discard_remain_rejected() {
    let head="module m; record A {x:I64,y:Bool} record B {x:I64,y:Bool} variant R {Ok(I64),Err(Text)} fn ok()->R {Ok(1)} ";
    let ids = " identities {module:\"m\",A:\"a\",B:\"b\",R:\"r\",ok:\"ok\",f:\"f\"}";
    for body in [
        "fn f(a:A,b:B)->A {A { ...b,x:1 }}",
        "fn f(a:A,b:B)->A {A { ...a,x:true }}",
        "fn f(a:A,b:B)->A {A { ...a,z:1 }}",
        "fn f(a:A,b:B)->A {A { ...a,x:1,x:2 }}",
        "fn f(a:A,b:B)->A {A { x:1,...a }}",
        "fn f(a:A,b:B)->A {A { ...a,...a }}",
        "fn f(a:A,b:B)->A {A { ..a }}",
        "fn f(a:A,b:B)->A {A { ...{} }}",
        "fn f(a:A,b:B)->I64 {ok(); 1}",
        "fn f(a:A,b:B)->R {try ok()+1; Ok(1)}",
        "fn f(a:A,b:B)->R {try ok();}",
        "fn f(a:A,b:B)->I64 {try ok(); 1}",
        "fn f(a:A,b:B)->R {try 1; Ok(2)}",
        "fn f(a:A,b:B)->A {let __lobo_update_base_0=a; a}",
    ] {
        assert!(compile(&format!("{head}{body}{ids}")).is_err(), "accepted {body}");
    }
}

#[test]
fn loaded_artifacts_reject_source_only_update_fields_and_admit_old_core19() {
    let artifact = compile(SOURCE).unwrap();
    let mut value: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    value["language"] = json!("core/1.9");
    Artifact::from_json(&value.to_string()).unwrap();
    fn inject(value: &mut Json) -> bool {
        if value.get("kind") == Some(&json!("Record")) {
            value["base"] = Json::Null;
            return true;
        }
        match value {
            Json::Object(v) => v.values_mut().any(inject),
            Json::Array(v) => v.iter_mut().any(inject),
            _ => false,
        }
    }
    assert!(inject(&mut value));
    assert!(Artifact::from_json(&value.to_string()).is_err());
}

#[test]
fn update_base_is_evaluated_exactly_once_under_a_native_call_quota() {
    let source = r#"module once;
record State {left:I64,right:I64}
variant R<T> {Ok(T),Err(I64)}
fn read(cap:Native::Capability)->R<State> actions {clock.read} {
  let now=try Native::monotonic_ms(cap); Ok(State {left:now,right:0})
}
fn main()->R<I64> actions {native.authorize,clock.read} {
  let cap=Native::capability();
  let value=State { ...try read(cap),right:5 };
  Ok(if value.left>=0 && value.right==5 {1} else {0})
}
identities {module:"once",State:"once.state",R:"once.result",read:"once.read",main:"once.main"}"#;
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let out = std::process::Command::new(builds.0.join(mode))
            .args([
                "--entry",
                "once.main",
                "--native-clock",
                "true",
                "--native-calls",
                "1",
                "--fuel",
                "100000",
                "--heap-cells",
                "100000",
            ])
            .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
            .env("UBSAN_OPTIONS", "halt_on_error=1")
            .output()
            .unwrap();
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {out:?}");
        let value: Json = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["value"]["value"]["case"], "Ok", "{mode}: {value}");
        assert_eq!(value["value"]["value"]["payload"]["value"], "1");
    }
}

#[test]
fn update_type_errors_point_to_the_original_override_expression() {
    let source = SOURCE.replace(
        "State { ...original,right:9 }",
        "State { ...original,right:false }",
    );
    let error = compile(&source).unwrap_err();
    assert_eq!(error.code, "compile.type.mismatch");
    let range = error.range.unwrap();
    assert_eq!(&source[range.start.byte..range.end.byte], "false");
}

#[test]
fn programming_skill_example_runs_in_every_execution_mode() {
    let source = lobo_core::SourceFile {
        name: "skill".into(),
        text: include_str!("../.agents/skills/lobo-programming/references/conveniences.lobo").into(),
        identities: Some(
            lobo_core::identities::parse_manifest(include_str!(
                "../.agents/skills/lobo-programming/references/conveniences.lobo.ids.json"
            ))
            .unwrap(),
        ),
    };
    let entry = source.identities.as_ref().unwrap()["main"].clone();
    let artifact = lobo_core::compile_bundle("skill", &[source]).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "llvm", "sanitized", "llvm-sanitized"] {
        let value = builds.invoke(mode, &entry, &[], 100000, 100000);
        let payload = &value["value"]["value"]["payload"]["value"]["fields"];
        assert_eq!(payload["timeout"]["value"], "10", "{mode}: {value}");
        assert_eq!(payload["enabled"]["value"], true);
    }
}

#[test]
fn whitespace_changes_do_not_change_the_checked_program() {
    let original = compile(SOURCE).unwrap().to_json().unwrap();
    let spaced = compile(&SOURCE.replace("...", "... "))
        .unwrap()
        .to_json()
        .unwrap();
    assert!(
        original == spaced,
        "formatting changed compiler-generated binding names"
    );
}

#[test]
fn imported_generic_update_bases_resolve_in_the_callers_closed_module_scope() {
    let app = r#"module app; use pkg {Pair,seed}; fn main()->I64 {(Pair { ...pkg.seed(),extra:8 }).value} identities {module:"app",pkg.Pair:"lib.pair",pkg.seed:"lib.seed",main:"app.main"}"#;
    let library = r#"module factory; record Pair<T> {value:T,extra:I64} fn seed()->Pair<I64> {Pair {value:42,extra:0}} identities {module:"lib",Pair:"lib.pair",seed:"lib.seed"}"#;
    let sources = [
        lobo_core::SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        lobo_core::SourceFile {
            name: "library".into(),
            text: library.into(),
            identities: None,
        },
    ];
    let artifact = lobo_core::compile_bundle("app", &sources).unwrap();
    let checked = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(
        checked.run("app.main", vec![], 100000).unwrap(),
        lobo_core::Value::I64(42)
    );
}

#[test]
fn nested_updates_keep_the_existing_frontend_depth_boundary() {
    fn source(depth: usize) -> String {
        let mut value = "State {left:1,right:2}".to_owned();
        for _ in 0..depth {
            value = format!("State {{ ...{value} }}");
        }
        format!("module m; record State {{left:I64,right:I64}} fn main()->I64 {{({value}).left}} identities {{module:\"m\",State:\"state\",main:\"main\"}}")
    }
    assert_eq!(compile(&source(40)).unwrap_err().code, "resource.frontend.limit");
    assert_eq!(
        compile(&source(24)).unwrap().run("main", vec![], 100000).unwrap(),
        lobo_core::Value::I64(1)
    );
}
