use lobo_core::{compile_bundle, identities::parse_manifest, SourceFile};
use serde_json::Value as Json;
use std::collections::BTreeMap;
#[allow(dead_code)]
mod support;
use support::Builds;

fn fixture() -> lobo_core::Artifact {
    fixture_with_poll_limit(1024)
}

fn fixture_with_poll_limit(limit: i64) -> lobo_core::Artifact {
    let transport_ids =
        parse_manifest(include_str!("../libraries/http-server/transport.lobo.ids.json")).unwrap();
    let text = r#"module lifetime;
use io {serve,legacy_serve,serve_with,finish_connection,finish_listener,ServeState};
use http {Request,Response};
fn handle(request:Request)->Response {Response {status:200,content_type:"application/json",body:"{}"}}
fn handle_with(context:I64,request:Request)->Response actions {db.close,db.execute,db.read} {handle(request)}
fn zero(context:Bool)->Native::I64Result
 actions {native.authorize,clock.read,net.listen,net.inspect,net.accept,net.read,net.write,net.close,db.close,db.execute,db.read}
{
 let cap=Native::capability();
 let served=if context {serve_with(cap,0,0,42,handle_with)} else {serve(cap,0,0,handle)};
 try served;
 let listener=try Native::listen(cap,0);
 let port=Native::port(listener);
 try Native::close_listener(listener);
 port
}
fn pure()->Native::I64Result
 actions {native.authorize,clock.read,net.listen,net.accept,net.read,net.write,net.close}
{serve(Native::capability(),0,0,handle)}
fn quota(context:Bool)->Native::I64Result
 actions {native.authorize,clock.read,net.listen,net.accept,net.read,net.write,net.close,db.close,db.execute,db.read}
{
 let cap=Native::capability();
 if context {serve_with(cap,0,1,42,handle_with)} else {serve(cap,0,1,handle)}
}
fn borrowed(listener:Native::Listener,cap:Native::Capability,count:I64)->Native::I64Result
 actions {clock.read,net.accept,net.read,net.write,net.close}
{legacy_serve(listener,cap,count,handle)}
fn listener_close_error()->Native::I64Result actions {native.authorize,net.listen,net.close} {
 let cap=Native::capability();
 let listener=try Native::listen(cap,0);
 try Native::close_listener(listener);
 finish_listener(listener,Err(11))
}
fn close_error()->Native::I64Result
 actions {native.authorize,net.listen,net.inspect,net.connect,net.accept,net.close}
{
 let cap=Native::capability();
 let listener=try Native::listen(cap,0);
 let port=try Native::port(listener);
 let client=try Native::connect(cap,port,1000);
 let stream=try Native::accept(listener,1000);
 try Native::close_stream(stream);
 let state=finish_connection(stream,11,ServeState {handled:0,polls:0,error:0});
 try Native::close_stream(client);
 try Native::close_listener(listener);
 Ok(state.error)
}"#;
    let mut identities: BTreeMap<String, String> = [
        ("module", "lifetime"),
        ("handle", "lifetime.handle"),
        ("handle_with", "lifetime.handle_with"),
        ("zero", "lifetime.zero"),
        ("pure", "lifetime.pure"),
        ("quota", "lifetime.quota"),
        ("close_error", "lifetime.close_error"),
        ("borrowed", "lifetime.borrowed"),
        ("listener_close_error", "lifetime.listener_close_error"),
        ("http.Request", "lobo.http.server.Request"),
        ("http.Response", "example.web.response"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect();
    for name in [
        "serve",
        "serve_with",
        "finish_connection",
        "finish_listener",
        "ServeState",
    ] {
        identities.insert(format!("io.{name}"), transport_ids[name].clone());
    }
    identities.insert("io.legacy_serve".into(), transport_ids["serve_listener"].clone());
    let mut sources = vec![SourceFile {
        name: "fixture".into(),
        text: text.into(),
        identities: Some(identities),
    }];
    for (name, text, ids) in [
        (
            "io",
            include_str!("../libraries/http-server/transport.lobo"),
            Some(include_str!("../libraries/http-server/transport.lobo.ids.json")),
        ),
        (
            "server",
            include_str!("../libraries/http-server/server.lobo"),
            Some(include_str!("../libraries/http-server/server.lobo.ids.json")),
        ),
        (
            "access",
            include_str!("../libraries/native/permissions.lobo"),
            None,
        ),
        ("http", include_str!("../libraries/http/http.lobo"), None),
        ("json", include_str!("../libraries/json/json.lobo"), None),
    ] {
        sources.push(SourceFile {
            name: name.into(),
            text: if name == "io" {
                text.replace("state.polls < 1024", &format!("state.polls < {limit}"))
            } else {
                text.into()
            },
            identities: ids.map(|s| parse_manifest(s).unwrap()),
        });
    }
    compile_bundle("fixture", &sources).unwrap()
}

#[test]
fn pure_and_context_servers_close_listeners_before_return_and_keep_action_precision() {
    let artifact = fixture();
    assert!(!artifact.action_summary()["lifetime.pure"]
        .iter()
        .any(|a| a.starts_with("db.")));
    assert!(!artifact.action_summary()["lifetime.borrowed"]
        .iter()
        .any(|a| a == "net.listen" || a.starts_with("db.")));
    let builds = Builds::new(&artifact);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for context in ["Bool:false", "Bool:true"] {
            let out = std::process::Command::new(builds.0.join(mode))
                .args([
                    "--entry",
                    "lifetime.zero",
                    "--arg",
                    context,
                    "--native-listen",
                    "127.0.0.1:0",
                    "--native-handles",
                    "2",
                    "--fuel",
                    "1000000",
                    "--heap-cells",
                    "1000000",
                ])
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {out:?}");
            let value: Json = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(value["value"]["value"]["case"], "Ok", "{mode}: {value}");
            assert!(
                value["value"]["value"]["payload"]["value"]
                    .as_str()
                    .unwrap()
                    .parse::<u16>()
                    .unwrap()
                    > 0
            );
        }
    }
}

#[test]
fn quota_and_stream_close_precedence_match_for_the_shared_engine() {
    let builds = Builds::new(&fixture());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for context in ["Bool:false", "Bool:true"] {
            let out = std::process::Command::new(builds.0.join(mode))
                .args([
                    "--entry",
                    "lifetime.quota",
                    "--arg",
                    context,
                    "--native-listen",
                    "127.0.0.1:0",
                    "--native-calls",
                    "1",
                    "--fuel",
                    "1000000",
                    "--heap-cells",
                    "1000000",
                ])
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {out:?}");
            let value: Json = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(value["value"]["value"]["case"], "Err", "{mode}: {value}");
            assert_eq!(value["value"]["value"]["payload"]["value"], "11");
        }
        let listener_out = std::process::Command::new(builds.0.join(mode))
            .args([
                "--entry",
                "lifetime.listener_close_error",
                "--native-listen",
                "127.0.0.1:0",
                "--fuel",
                "1000000",
                "--heap-cells",
                "1000000",
            ])
            .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
            .env("UBSAN_OPTIONS", "halt_on_error=1")
            .output()
            .unwrap();
        assert!(
            listener_out.status.success() && listener_out.stderr.is_empty(),
            "{mode}: {listener_out:?}"
        );
        let listener_value: Json = serde_json::from_slice(&listener_out.stdout).unwrap();
        assert_eq!(listener_value["value"]["value"]["case"], "Err");
        assert_eq!(listener_value["value"]["value"]["payload"]["value"], "5");
        let out = std::process::Command::new(builds.0.join(mode))
            .args([
                "--entry",
                "lifetime.close_error",
                "--native-listen",
                "127.0.0.1:0",
                "--native-connect",
                "self",
                "--fuel",
                "1000000",
                "--heap-cells",
                "1000000",
            ])
            .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
            .env("UBSAN_OPTIONS", "halt_on_error=1")
            .output()
            .unwrap();
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {out:?}");
        let value: Json = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["value"]["value"]["case"], "Ok");
        assert_eq!(
            value["value"]["value"]["payload"]["value"], "5",
            "stream close error must override quota outcome 11: {value}"
        );
    }
}

#[test]
fn shared_serving_policy_keeps_request_poll_and_error_boundaries_in_every_route() {
    let artifact = fixture();
    let ids = parse_manifest(include_str!("../libraries/http-server/transport.lobo.ids.json")).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        for (state, count, expected) in [
            ("I64:0,I64:0,I64:0", "I64:0", false),
            ("I64:0,I64:1023,I64:0", "I64:1", true),
            ("I64:0,I64:1024,I64:0", "I64:1", false),
            ("I64:1,I64:1,I64:0", "I64:1", false),
            ("I64:0,I64:1,I64:11", "I64:1", false),
        ] {
            let argument = format!("Record:{}:{state}", ids["ServeState"]);
            let value = builds.invoke(mode, &ids["serving"], &[&argument, count], 100000, 100000);
            assert_eq!(value["status"], "ok", "{mode}: {value}");
            assert_eq!(value["value"]["value"], expected, "{mode}: {state}/{count}");
        }
    }
}

#[test]
fn shared_accept_loop_stops_after_1024_timeouts_and_the_off_by_one_mutant_fails() {
    for (limit, expected_status) in [(1024, 0), (1025, 42)] {
        let artifact = fixture_with_poll_limit(limit);
        let builds = Builds::new(&artifact);
        let original = std::fs::read_to_string(builds.0.join("program.c")).unwrap();
        let definition = "LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_native_accept_v1(";
        assert!(
            original.contains(definition),
            "provider definition to interpose must exist"
        );
        let interposed = original.replace(
            definition,
            "LOBO_NATIVE_NOINLINE lobo_native_result_v1 lobo_original_accept_v1(",
        );
        let probe = format!("#define main lobo_generated_main\n{interposed}\n#undef main\n")
            + r#"
static unsigned fake_accept_calls=0;
lobo_native_result_v1 lobo_native_accept_v1(uint64_t listener,int64_t timeout_ms) {
    (void)listener;
    if(timeout_ms!=2000) { fputs("unexpected-accept-timeout\n",stderr); exit(43); }
    ++fake_accept_calls;
    return lobo_native_error_v1(6);
}
int main(int argc,char **argv) {
    int result=lobo_generated_main(argc,argv);
    if(fake_accept_calls!=1024) { fputs("poll-bound-mismatch\n",stderr); return 42; }
    return result;
}
"#;
        std::fs::write(builds.0.join("probe.c"), probe).unwrap();
        for sanitized in [false, true] {
            let name = if sanitized { "probe-sanitized" } else { "probe" };
            let mut command = std::process::Command::new("clang");
            command.arg("-std=c11");
            if sanitized {
                command.args([
                    "-O1",
                    "-g",
                    "-fsanitize=address,undefined",
                    "-fno-sanitize-recover=all",
                ]);
            } else {
                command.arg("-O3");
            }
            let compiled = command
                .arg(builds.0.join("probe.c"))
                .arg("-o")
                .arg(builds.0.join(name))
                .output()
                .unwrap();
            assert!(
                compiled.status.success(),
                "baseline/mutant must build: {compiled:?}"
            );
            for context in ["Bool:false", "Bool:true"] {
                let output = std::process::Command::new(builds.0.join(name))
                    .args([
                        "--entry",
                        "lifetime.quota",
                        "--arg",
                        context,
                        "--native-listen",
                        "127.0.0.1:0",
                        "--native-handles",
                        "2",
                        "--fuel",
                        "1000000",
                        "--heap-cells",
                        "1000000",
                    ])
                    .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                    .env("UBSAN_OPTIONS", "halt_on_error=1")
                    .output()
                    .unwrap();
                println!(
                    "HTTP_POLL_CASE {}",
                    serde_json::json!({"poll_limit":limit,"sanitized":sanitized,"context":context,"exit_code":output.status.code(),"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)})
                );
                assert_eq!(
                    output.status.code(),
                    Some(expected_status),
                    "limit={limit}, sanitized={sanitized}, context={context}: {output:?}"
                );
                if limit == 1024 {
                    assert!(output.stderr.is_empty(), "{output:?}");
                    let value: Json = serde_json::from_slice(&output.stdout).unwrap();
                    assert_eq!(value["value"]["value"]["case"], "Ok");
                    assert_eq!(value["value"]["value"]["payload"]["value"], "0");
                } else {
                    assert_eq!(output.stderr, b"poll-bound-mismatch\n");
                }
            }
        }
    }
}
