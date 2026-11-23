use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

const SOURCE: &str = r#"module callbacks;
record Slot<T> { callback: Fn(T) actions { demo.read } -> T }
variant R<T> { Ok(T), Err(Text) }
fn observed(value:I64)->I64 actions { demo.read } requires {value>=0} ensures {result>value} {value+1}
fn identity<T>(value:T)->T actions {demo.read} {value}
fn invoke<T>(callback:Fn(T) actions {demo.read}->T,value:T)->T actions {demo.read} {callback(value)}
fn main()->I64 actions {demo.read} {let slot=Slot {callback:&observed}; invoke(slot.callback,41)+external(slot.callback,0)-1}
fn inferred()->Text actions {demo.read} {invoke(identity,"hello")}
fn reference()->Fn(I64) actions {demo.read}->I64 {&observed}
fn external(callback:Fn(I64) actions {demo.read}->I64,value:I64)->I64 actions {demo.read} {callback(value)}
fn pure(value:I64)->I64 {value}
fn lying(value:I64)->I64 actions {demo.read} ensures {false} {value}
fn stored()->List<Fn(I64) actions {demo.read}->I64> {List::prepend(&observed,List::empty<Fn(I64) actions {demo.read}->I64>())}
fn returns_pure()->Fn(I64)->I64 actions {demo.read} {&pure}
fn failed(value:I64)->R<I64> actions {demo.read} {Err("stopped")}
fn propagate(callback:Fn(I64) actions {demo.read}->R<I64>)->R<I64> actions {demo.read} {Ok(try callback(1))}
fn error()->R<I64> actions {demo.read} {propagate(failed)}
identities {module:"callbacks",Slot:"callbacks.slot",R:"callbacks.result",observed:"observed",identity:"identity",invoke:"invoke",main:"main",inferred:"inferred",reference:"reference",external:"external",pure:"pure",lying:"lying",stored:"stored",returns_pure:"returns_pure",failed:"failed",propagate:"propagate",error:"error"}
"#;

#[test]
fn callback_bounds_execute_with_contextual_generics_records_contracts_and_try() {
    let a = compile(SOURCE).unwrap();
    let explicit = SOURCE.replace("invoke(identity,\"hello\")", "invoke::<Text>(identity,\"hello\")");
    assert_eq!(
        a.to_json().unwrap(),
        compile(&explicit).unwrap().to_json().unwrap()
    );
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[], 10000, 10000)["value"]["value"],
            "42"
        );
        assert_eq!(
            builds.invoke(mode, "inferred", &[], 10000, 10000)["value"]["value"],
            "hello"
        );
        assert_eq!(
            builds.invoke(mode, "error", &[], 10000, 10000)["value"]["value"]["case"],
            "Err"
        );
        let value = builds.invoke(mode, "reference", &[], 10000, 10000);
        assert_eq!(
            value["value"]["value"]["signature"]["actions"],
            json!(["demo.read"])
        );
        assert_eq!(
            builds.invoke(mode, "external", &["Function:observed", "I64:5"], 10000, 10000)["value"]["value"],
            "6"
        );
        assert_eq!(
            builds.invoke(mode, "external", &["Function:observed", "I64:-1"], 10000, 10000)["diagnostic"]
                ["code"],
            "runtime.contract.precondition"
        );
        assert_eq!(
            builds.invoke(mode, "external", &["Function:pure", "I64:5"], 10000, 10000)["diagnostic"]["code"],
            "invocation.arguments.invalid"
        );
        let fault = builds.invoke(mode, "external", &["Function:lying", "I64:5"], 10000, 10000);
        assert_eq!(fault["diagnostic"]["code"], "runtime.contract.postcondition");
        assert_eq!(fault["diagnostic"]["subject"], "lying");
        let stored = builds.invoke(mode, "stored", &[], 10000, 10000);
        assert_eq!(
            stored["value"]["value"]["elements"][0]["value"]["signature"]["actions"],
            json!(["demo.read"])
        );
    }
    assert_eq!(a.run("main", vec![], 10000).unwrap(), Value::I64(42));
}

#[test]
fn every_indirect_call_requires_its_declared_actions_even_in_contracts_and_dead_code() {
    let head = "module m @id(\"m\"); function leaf @id(\"leaf\")(x:I64)->I64 actions {demo.read} {x}";
    for (clauses, body, code) in [
        ("", "f(1)", "compile.action.undeclared"),
        ("", "if false {f(1)} else {0}", "compile.action.undeclared"),
        ("actions {demo.other}", "f(1)", "compile.action.undeclared"),
        (
            "actions {demo.read} requires {f(1)==1}",
            "1",
            "compile.action.contract",
        ),
        (
            "actions {demo.read} ensures {f(1)==result}",
            "1",
            "compile.action.contract",
        ),
    ] {
        let source=format!("{head} function call @id(\"call\")(f:Fn(I64) actions {{demo.read}}->I64)->I64 {clauses} {{{body}}}");
        let error = compile(&source).unwrap_err();
        assert_eq!(error.code, code, "{source}");
        let r = error.range.unwrap();
        assert_eq!(&source[r.start.byte..r.end.byte], "f(1)");
    }
}

#[test]
fn signatures_are_exact_and_declared_action_sets_are_canonical_and_bounded() {
    let template="module m @id(\"m\"); function f @id(\"f\")(g:Fn(I64) actions {demo.read,demo.write}->I64)->I64 actions {demo.write,demo.read} {g(1)}";
    let a = compile(template).unwrap().to_json().unwrap();
    assert_eq!(
        a,
        compile(&template.replace("demo.read,demo.write", "demo.write, demo . read"))
            .unwrap()
            .to_json()
            .unwrap()
    );
    for (clause, code) in [
        ("demo.read,demo.read", "compile.action.duplicate"),
        ("single", "compile.action.invalid"),
        ("demo.if", "compile.action.invalid"),
    ] {
        assert_eq!(
            compile(&template.replace("demo.read,demo.write", clause))
                .unwrap_err()
                .code,
            code
        );
    }
    let many = (0..65)
        .map(|n| format!("demo.a{n}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        compile(&template.replace("demo.read,demo.write", &many))
            .unwrap_err()
            .code,
        "resource.frontend.limit"
    );
    let at_limit = (0..64)
        .map(|n| format!("demo.a{n}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(compile(
        &template
            .replace("demo.read,demo.write", &at_limit)
            .replace("demo.write,demo.read", &at_limit)
    )
    .is_ok());
    let pure = "module m @id(\"m\"); function f @id(\"f\")(g:Fn(I64)->I64)->I64 {g(1)}";
    assert_eq!(
        compile(pure).unwrap().to_json().unwrap(),
        compile(&pure.replace("Fn(I64)", "Fn(I64) actions {} "))
            .unwrap()
            .to_json()
            .unwrap()
    );
    let original = SOURCE.replace("invoke(identity,\"hello\")", "invoke::<Text>(identity,\"hello\")");
    for invalid in [
        original.replace("Slot {callback:&observed}", "Slot {callback:&pure}"),
        original.replace(
            "Fn(I64) actions {demo.read}->I64 {&observed}",
            "Fn(I64)->I64 {&observed}",
        ),
    ] {
        assert!(compile(&invalid).is_err());
    }
}

#[test]
fn loaded_and_programmatic_callbacks_cannot_forge_or_erase_actions_or_retag_old_languages() {
    let source = SOURCE.replace("invoke(identity,\"hello\")", "invoke::<Text>(identity,\"hello\")");
    let a = compile(&source).unwrap();
    let original: Json = serde_json::from_str(&a.to_json().unwrap()).unwrap();
    for actions in [
        json!([]),
        json!(["demo.read", "demo.read"]),
        json!(["single"]),
        json!(["demo.z", "demo.read"]),
    ] {
        let mut data = original.clone();
        let f = data["program"]["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["id"] == "external")
            .unwrap();
        f["parameters"][0]["ty"]["Function"]["actions"] = actions;
        assert!(Artifact::from_json(&data.to_string()).is_err());
    }
    for language in ["core/1.0", "core/1.9", "core/1.10"] {
        let mut data = original.clone();
        data["language"] = json!(language);
        assert_eq!(
            Artifact::from_json(&data.to_string()).unwrap_err().code,
            "invocation.artifact.version"
        );
    }
    let mut callback = a.parse_argument("external", 0, "Function:observed").unwrap();
    if let Value::Function(value) = &mut callback {
        value.id = "pure".into();
    }
    assert_eq!(
        a.run("external", vec![callback, Value::I64(1)], 1000)
            .unwrap_err()
            .code,
        "invocation.arguments.invalid"
    );
    let mut erased = a.parse_argument("external", 0, "Function:observed").unwrap();
    if let Value::Function(value) = &mut erased {
        value.signature.actions.clear();
    }
    assert_eq!(
        a.run("external", vec![erased, Value::I64(1)], 1000)
            .unwrap_err()
            .code,
        "invocation.arguments.invalid"
    );
    let nested = r#"module m @id("m"); record Holder @id("holder") {f:Fn(Fn(I64) actions {demo.read}->I64)->I64} function main @id("main")()->I64 {42}"#;
    let mut nested: Json = serde_json::from_str(&compile(nested).unwrap().to_json().unwrap()).unwrap();
    nested["language"] = json!("core/1.10");
    assert_eq!(
        Artifact::from_json(&nested.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
    let bare="module m @id(\"m\"); function leaf @id(\"leaf\")()->I64 actions {demo.read} {1} function main @id(\"main\")()->I64 {let f=&leaf; 1}";
    let mut data: Json = serde_json::from_str(&compile(bare).unwrap().to_json().unwrap()).unwrap();
    data["language"] = json!("core/1.10");
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
}

#[test]
fn actionful_native_callbacks_preserve_launch_grants_and_call_quotas() {
    let source = r#"module clock;
fn read(cap:Native::Capability)->Native::I64Result actions {clock.read} {Native::monotonic_ms(cap)}
fn invoke<C,T>(context:C,callback:Fn(C) actions {clock.read}->T)->T actions {clock.read} {callback(context)}
fn main()->Native::I64Result actions {native.authorize,clock.read} {invoke(Native::capability(),read)}
fn twice()->Native::I64Result actions {native.authorize,clock.read} {let cap=Native::capability(); try invoke(cap,read); invoke(cap,read)}
identities {module:"clock.callback",read:"clock.read",invoke:"clock.invoke",main:"clock.main",twice:"clock.twice"}"#;
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (entry, grant, calls, case, code) in [
            ("clock.main", "true", "1", "Ok", None),
            ("clock.main", "false", "1", "Err", Some("1")),
            ("clock.twice", "true", "1", "Err", Some("11")),
        ] {
            let mut command = std::process::Command::new(builds.0.join(mode));
            if grant == "true" {
                command.args(["--native-clock", "true"]);
            }
            let output = command
                .args([
                    "--entry",
                    entry,
                    "--native-calls",
                    calls,
                    "--fuel",
                    "100000",
                    "--heap-cells",
                    "100000",
                ])
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(
                output.status.success() && output.stderr.is_empty(),
                "{mode}: {output:?}"
            );
            let result: Json = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(result["value"]["value"]["case"], case, "{mode}: {result}");
            if let Some(code) = code {
                assert_eq!(result["value"]["value"]["payload"]["value"], code);
            }
        }
    }
}

#[test]
fn generic_callback_bodies_cannot_erase_actions_and_references_respect_module_visibility() {
    let invalid =
        r#"module m @id("m"); function apply<T> @id("apply")(f:Fn(T) actions {demo.read}->T,x:T)->T {f(x)}"#;
    assert_eq!(compile(invalid).unwrap_err().code, "compile.action.undeclared");
    let sources = [
        lobo_core::SourceFile {
            name: "a".into(),
            text: "module a @id(\"a\"); function f @id(\"f\")()->I64 {let hidden=&secret; 1}".into(),
            identities: None,
        },
        lobo_core::SourceFile {
            name: "b".into(),
            text: "module b @id(\"b\"); function secret @id(\"secret\")()->I64 actions {demo.read} {1}"
                .into(),
            identities: None,
        },
    ];
    assert!(lobo_core::compile_bundle("a", &sources).is_err());
}

#[test]
fn programming_skill_callback_example_executes_in_all_native_routes() {
    let source = lobo_core::SourceFile {
        name: "callbacks".into(),
        text: include_str!("../.agents/skills/lobo-programming/references/callbacks.lobo").into(),
        identities: Some(
            lobo_core::identities::parse_manifest(include_str!(
                "../.agents/skills/lobo-programming/references/callbacks.lobo.ids.json"
            ))
            .unwrap(),
        ),
    };
    let id = source.identities.as_ref().unwrap()["main"].clone();
    let a = lobo_core::compile_bundle("callbacks", &[source]).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, &id, &[], 10000, 10000)["value"]["value"],
            "42"
        );
    }
}
