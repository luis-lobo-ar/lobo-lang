use lobo_core::{compile, Artifact};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

const SOURCE: &str = r#"module propagation;
variant Outcome<T,E> {Err(E),Ok(T)}
record Answer {number:I64}
fn first(flag:Bool)->Outcome<I64,Text> {if flag {Ok(41)} else {Err("first")}}
fn second(value:I64)->Outcome<I64,Text> {Ok(value+1)}
fn main(flag:Bool)->Outcome<Answer,Text> {
  let n=try first(flag);
  let next=try second(n);
  Ok(Answer {number:next})
}
fn skip()->Outcome<I64,Text> {let n=try first(false); Ok(n / 0)}
fn inner()->Outcome<I64,Text> {let n=try first(false); Ok(n)}
fn boundary()->Outcome<I64,Text> {let ignored=inner(); Ok(7)}
fn expression(flag:Bool)->Outcome<I64,Text> {Ok(1 + try first(flag))}
fn folded(flag:Bool)->Outcome<I64,Text> {
 let total=fold(i in 3,total:I64=0) {total + try first(flag)}; Ok(total)
}
fn looped(flag:Bool)->Outcome<I64,Text> {
 let total=while(total:I64=0;total<2) {let n=try first(flag); total+1}; Ok(total)
}
fn guarded(flag:Bool)->Outcome<I64,Text> {if flag && (try first(false))==41 {Ok(1)} else {Ok(0)}}
fn assured()->Outcome<I64,Text>
ensures {match result {Ok(n)=>true,Err(message)=>false}}
{let n=try first(false); Ok(n)}
fn fault()->Outcome<I64,Text> {let n=try second(1/0); Ok(n)}
fn unwrap<T,E>(value:Outcome<T,E>)->Outcome<T,E> {Ok(try value)}
fn generic()->Outcome<I64,Text> {unwrap(first(true))}
variant Foreign {Ok(I64),Err(Text)}
fn foreign()->Outcome<I64,Text> {let n=try Foreign::Err("foreign"); Ok(n)}
fn pair(a:I64,b:I64)->I64 {a+b}
fn arguments()->Outcome<I64,Text> {Ok(pair(try first(false),1/0))}
fn repeated()->Outcome<I64,Text> {let n=fold(i in 64,total:I64=0) {let ignored=inner(); total+1}; Ok(n)}
fn through(f:Fn(Bool)->Outcome<I64,Text>)->Outcome<I64,Text> {Ok(try f(false))}
fn callback()->Outcome<I64,Text> {through(first)}
fn boolean(flag:Bool)->Outcome<Bool,Text> {if flag {Ok(true)} else {Err("first")}}
fn in_condition(flag:Bool)->Outcome<I64,Text> {let n=while(n:I64=0;n<1 && try boolean(flag)) {n+1}; Ok(n)}
fn in_initial(flag:Bool)->Outcome<I64,Text> {let n=fold(i in 0,n:I64=try first(flag)) {n}; Ok(n)}
fn in_count(flag:Bool)->Outcome<I64,Text> {let n=fold(i in try first(flag),n:I64=0) {n+1}; Ok(n)}
identities {module:"try.demo",Outcome:"try.outcome",Answer:"try.answer",first:"try.first",second:"try.second",main:"try.main",skip:"try.skip",inner:"try.inner",boundary:"try.boundary",expression:"try.expression",folded:"try.folded",looped:"try.looped",guarded:"try.guarded",assured:"try.assured",fault:"try.fault",unwrap:"try.unwrap",generic:"try.generic",Foreign:"try.foreign_type",foreign:"try.foreign",pair:"try.pair",arguments:"try.arguments",repeated:"try.repeated",through:"try.through",callback:"try.callback",boolean:"try.boolean",in_condition:"try.condition",in_initial:"try.initial",in_count:"try.count"}
"#;
fn result(value: &Json) -> (&str, &Json) {
    let variant = &value["value"]["value"];
    (variant["case"].as_str().unwrap(), &variant["payload"])
}
#[test]
fn try_propagates_errors_across_expressions_and_loops_on_every_backend() {
    let artifact = compile(SOURCE).unwrap();
    let builds = Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "llvm", "sanitized", "llvm-sanitized"] {
        for entry in [
            "try.main",
            "try.expression",
            "try.folded",
            "try.looped",
            "try.condition",
            "try.initial",
            "try.count",
        ] {
            for flag in [true, false] {
                let out = builds.invoke(
                    mode,
                    entry,
                    &[if flag { "Bool:true" } else { "Bool:false" }],
                    100000,
                    100000,
                );
                assert_eq!(out["status"], "ok", "{mode} {entry}: {out}");
                let (case, payload) = result(&out);
                assert_eq!(case, if flag { "Ok" } else { "Err" });
                if !flag {
                    assert_eq!(payload["value"], "first");
                } else if entry == "try.main" {
                    assert_eq!(payload["value"]["fields"]["number"]["value"], "42");
                }
            }
        }
        for (entry, case, payload) in [
            ("try.skip", "Err", json!("first")),
            ("try.boundary", "Ok", json!("7")),
            ("try.generic", "Ok", json!("41")),
            ("try.foreign", "Err", json!("foreign")),
            ("try.arguments", "Err", json!("first")),
            ("try.repeated", "Ok", json!("64")),
            ("try.callback", "Err", json!("first")),
        ] {
            let out = builds.invoke(mode, entry, &[], 100000, 100000);
            let (actual, value) = result(&out);
            assert_eq!(actual, case);
            assert_eq!(value["value"], payload);
        }
        for (entry, code) in [
            ("try.assured", "runtime.contract.postcondition"),
            ("try.fault", "runtime.integer.zero_divisor"),
        ] {
            let out = builds.invoke(mode, entry, &[], 100000, 100000);
            assert_eq!(out["diagnostic"]["code"], code, "{mode}: {out}");
        }
        let out = builds.invoke(mode, "try.guarded", &["Bool:false"], 100000, 100000);
        assert_eq!(result(&out).1["value"], "0");
    }
}
#[test]
fn try_requires_result_shapes_exact_error_types_and_a_function_body() {
    for (declarations, body, code) in [
        (
            "variant R {Ok(I64),Err(Text)}",
            "fn main()->I64 {try R::Ok(1)}",
            "compile.try.output",
        ),
        (
            "variant R {Ok(I64),Err(Text)}",
            "fn main()->R {let n=try 1; R::Ok(n)}",
            "compile.try.operand",
        ),
        (
            "variant R {Ok(I64),Err(Text)} variant Other {Ok(I64),Err(I64)}",
            "fn main()->R {let n=try Other::Err(7); R::Ok(n)}",
            "compile.try.error",
        ),
        (
            "variant R {Ok(I64),Err(Text),Missing}",
            "fn main()->R {let n=try R::Ok(1); R::Ok(n)}",
            "compile.try.operand",
        ),
        (
            "variant R {Ok(I64),Err}",
            "fn main()->R {let n=try R::Ok(1); R::Ok(n)}",
            "compile.try.operand",
        ),
        (
            "variant R {Ok(I64),Err(Text)}",
            "fn main()->R requires {(try R::Ok(1))==1} {R::Ok(1)}",
            "compile.try.contract",
        ),
        (
            "variant R {Ok(I64),Err(Text)}",
            "fn main()->R ensures {(try result)==1} {R::Ok(1)}",
            "compile.try.contract",
        ),
    ] {
        let source = format!(
            "module app; {declarations} {body} identities {{module:\"app\",R:\"r\",{}main:\"main\"}}",
            if declarations.contains("Other") {
                "Other:\"other\","
            } else {
                ""
            }
        );
        let error = compile(&source).unwrap_err();
        assert_eq!(error.code, code, "{source}: {error:?}");
        assert!(error.range.is_some());
    }
}
#[test]
fn try_nodes_cannot_be_smuggled_into_older_checked_language_versions() {
    let artifact = compile(SOURCE).unwrap();
    assert_eq!(artifact.language, lobo_core::LANGUAGE);
    for language in ["core/1.0", "core/1.7"] {
        let mut wire: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
        wire["language"] = json!(language);
        assert_eq!(
            Artifact::from_json(&wire.to_string()).unwrap_err().code,
            "invocation.artifact.version"
        );
    }
}

#[test]
fn try_preserves_contextual_names_and_validates_unused_generic_bodies() {
    let source = r#"module m; fn try(n:I64)->I64 {n} fn main()->I64 {let try_value=3; try(try_value)} identities {module:"m",try:"legacy.try",main:"main"}"#;
    assert_eq!(
        compile(source).unwrap().run("main", vec![], 1000).unwrap(),
        lobo_core::Value::I64(3)
    );
    let source = r#"module m; variant R<T,E> {Ok(T),Err(E)} fn bad<T>(value:R<T,Text>)->R<T,I64> {Ok(try value)} fn main()->I64 {0} identities {module:"m",R:"r",bad:"bad",main:"main"}"#;
    assert_eq!(compile(source).unwrap_err().code, "compile.try.error");
}

#[test]
fn try_keeps_action_checks_and_does_not_convert_runtime_faults_to_typed_errors() {
    let source = r#"module m; variant R {Ok(I64),Err(Text)} fn get()->R actions {demo.observe} {Ok(1)} fn main()->R {Ok(try get())} identities {module:"m",R:"r",get:"get",main:"main"}"#;
    assert_eq!(compile(source).unwrap_err().code, "compile.action.undeclared");
    let valid = source.replace("fn main()->R {", "fn main()->R actions {demo.observe} {");
    assert!(compile(&valid).is_ok());
}

#[test]
fn native_try_evaluates_once_and_callers_can_close_resources_after_a_propagated_error() {
    let source = r#"module native_try;
fn clock()->Native::I64Result actions {native.authorize,clock.read} {Ok(try Native::monotonic_ms(Native::capability()))}
fn work(listener:Native::Listener)->Native::I64Result actions {net.accept} {let stream=try Native::accept(listener,0); Ok(1)}
fn cleanup()->Native::I64Result actions {native.authorize,net.listen,net.accept,net.close} {
 let cap=Native::capability();
 let listener=try Native::listen(cap,0);
 let outcome=work(listener);
 let closed=try Native::close_listener(listener);
 let replacement=try Native::listen(cap,0);
 let closed_again=try Native::close_listener(replacement);
 outcome
}
identities {module:"native.try",clock:"native.try.clock",work:"native.try.work",cleanup:"native.try.cleanup"}
"#;
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (entry, options, expected) in [
            (
                "native.try.clock",
                vec!["--native-clock", "true", "--native-calls", "1"],
                "Ok",
            ),
            (
                "native.try.cleanup",
                vec!["--native-listen", "127.0.0.1:0", "--native-handles", "2"],
                "Err",
            ),
        ] {
            let out = std::process::Command::new(builds.0.join(mode))
                .args(["--entry", entry, "--fuel", "100000", "--heap-cells", "100000"])
                .args(options)
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {out:?}");
            let value: Json = serde_json::from_slice(&out.stdout).unwrap();
            let (case, payload) = result(&value);
            assert_eq!(case, expected, "{mode}: {value}");
            if entry.ends_with("cleanup") {
                assert_eq!(payload["value"], "6");
            }
        }
    }
}

#[test]
fn snapshot_uses_try_with_unchanged_identities_and_explicit_grants() {
    let identities =
        lobo_core::identities::parse_manifest(include_str!("../examples/native/snapshot.lobo.ids.json"))
            .unwrap();
    let artifact = lobo_core::compile_bundle(
        "app",
        &[
            lobo_core::SourceFile {
                name: "app".into(),
                text: include_str!("../examples/native/snapshot.lobo").into(),
                identities: Some(identities),
            },
            lobo_core::SourceFile {
                name: "access".into(),
                text: include_str!("../libraries/native/permissions.lobo").into(),
                identities: None,
            },
            lobo_core::SourceFile {
                name: "os".into(),
                text: include_str!("../libraries/native/native.lobo").into(),
                identities: None,
            },
        ],
    )
    .unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (options, case, error) in [
            (
                vec![
                    "--native-clock",
                    "true",
                    "--native-random",
                    "true",
                    "--native-calls",
                    "3",
                ],
                "Ok",
                None,
            ),
            (vec![], "Err", Some("Denied")),
            (
                vec![
                    "--native-clock",
                    "true",
                    "--native-random",
                    "true",
                    "--native-calls",
                    "2",
                ],
                "Err",
                Some("LimitExceeded"),
            ),
        ] {
            let out = std::process::Command::new(builds.0.join(mode))
                .args([
                    "--entry",
                    "native.snapshot.main",
                    "--fuel",
                    "100000",
                    "--heap-cells",
                    "100000",
                ])
                .args(options)
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{out:?}");
            let value: Json = serde_json::from_slice(&out.stdout).unwrap();
            let (actual, payload) = result(&value);
            assert_eq!(actual, case);
            if let Some(error) = error {
                assert_eq!(payload["value"]["case"], error);
            } else {
                assert_eq!(payload["value"]["id"], "native.snapshot.value");
                assert_eq!(
                    payload["value"]["fields"]["random_bytes"]["value"]
                        .as_str()
                        .unwrap()
                        .len(),
                    32
                );
            }
        }
    }
}

#[test]
fn existing_core_1_7_native_artifacts_remain_readable_without_try() {
    let source = r#"module m; fn main()->Native::I64Result actions {native.authorize,clock.read} {Native::monotonic_ms(Native::capability())} identities {module:"m",main:"main"}"#;
    let mut wire: Json = serde_json::from_str(&compile(source).unwrap().to_json().unwrap()).unwrap();
    wire["language"] = json!("core/1.7");
    Artifact::from_json(&wire.to_string()).unwrap();
}

#[test]
fn try_preserves_match_inference_for_an_imported_producers_unnamed_payload_type() {
    let library = r#"module models; record Hidden {value:I64} variant R<T> {Ok(T),Err(Text)} fn get()->R<Hidden> {Ok(Hidden {value:42})} identities {module:"models",Hidden:"hidden",R:"foreign.result",get:"get"}"#;
    let app = r#"module app; use models {get}; variant Reply {Ok(I64),Err(Text)} fn main()->Reply {let value=try get(); Ok(value.value)} identities {module:"app",models.get:"get",Reply:"reply",main:"main"}"#;
    let bundle = |text: &str| {
        lobo_core::compile_bundle(
            "app",
            &[
                lobo_core::SourceFile {
                    name: "app".into(),
                    text: text.into(),
                    identities: None,
                },
                lobo_core::SourceFile {
                    name: "models".into(),
                    text: library.into(),
                    identities: None,
                },
            ],
        )
    };
    let explicit = app.replace(
        "let value=try get(); Ok(value.value)",
        "match(get()) {Ok(value)=>Ok(value.value),Err(error)=>Err(error)}",
    );
    let expected = bundle(&explicit).unwrap().run("main", vec![], 1000).unwrap();
    assert_eq!(bundle(app).unwrap().run("main", vec![], 1000).unwrap(), expected);
}
