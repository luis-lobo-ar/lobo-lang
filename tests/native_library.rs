use lobo_core::{compile_bundle, SourceFile};
use serde_json::Value as Json;
use std::process::Command;
mod support;
use support::Builds;
const APP: &str = r#"module native_app;
use os {IoResult,IoError,monotonic_ms,random_bytes,from_error_code,to_error_code,listen,close_listener};
fn clock()->IoResult<I64> actions {native.authorize,clock.read} {let cap=Native::capability();os.monotonic_ms(cap)}
fn random(n:I64)->IoResult<Bytes> actions {native.authorize,random.read} {let cap=Native::capability();os.random_bytes(cap,n)}
fn error_roundtrip(code:I64)->I64 {os.to_error_code(os.from_error_code(code))}
fn listener()->IoResult<Bool> actions {native.authorize,net.listen,net.close} {
 let cap=Native::capability();
 match os.listen(cap,0) {Err(error)=>Err(error),Ok(socket)=>os.close_listener(socket)}
}
identities {module:"native.app",os.IoResult:"lobo.os.result",os.IoError:"lobo.os.error",os.monotonic_ms:"lobo.os.monotonic_ms",os.random_bytes:"lobo.os.random_bytes",os.from_error_code:"lobo.os.from_error_code",os.to_error_code:"lobo.os.to_error_code",os.listen:"lobo.os.listen",os.close_listener:"lobo.os.close_listener",clock:"native.app.clock",random:"native.app.random",error_roundtrip:"native.app.error_roundtrip",listener:"native.app.listener"}
"#;
fn artifact() -> lobo_core::Artifact {
    let library = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/libraries/native/native.lobo"
    ))
    .expect("the Lobo native library must exist");
    compile_bundle(
        "app",
        &[
            SourceFile {
                name: "app".into(),
                text: APP.into(),
                identities: None,
            },
            SourceFile {
                name: "access".into(),
                text: include_str!("../libraries/native/permissions.lobo").into(),
                identities: None,
            },
            SourceFile {
                name: "os".into(),
                text: library,
                identities: None,
            },
        ],
    )
    .unwrap()
}
fn invoke(builds: &Builds, mode: &str, entry: &str, options: &[&str]) -> Json {
    let output = Command::new(builds.0.join(mode))
        .args(["--entry", entry, "--fuel", "100000", "--heap-cells", "100000"])
        .args(options)
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert!(output.status.success() && output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn native_library_returns_typed_errors_without_ambient_grants() {
    let builds = Builds::new(&artifact());
    assert_eq!(
        builds.invoke("eval", "native.app.error_roundtrip", &["I64:6"], 10000, 10000)["value"]["value"],
        "6"
    );
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (entry, options) in [
            ("native.app.clock", vec![]),
            ("native.app.random", vec!["--arg", "I64:32"]),
            ("native.app.listener", vec![]),
        ] {
            let value = invoke(&builds, mode, entry, &options);
            assert_eq!(value["value"]["value"]["case"], "Err");
            assert_eq!(value["value"]["value"]["payload"]["value"]["case"], "Denied");
        }
    }
}
#[test]
fn native_library_converts_all_error_codes_and_preserves_unknown_codes() {
    let builds = Builds::new(&artifact());
    for mode in ["eval", "native", "sanitized"] {
        for code in [-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, i64::MAX] {
            assert_eq!(
                builds.invoke(
                    mode,
                    "native.app.error_roundtrip",
                    &[&format!("I64:{code}")],
                    100000,
                    100000
                )["value"]["value"],
                code.to_string()
            );
        }
    }
}
#[test]
fn native_library_uses_explicit_grants_for_real_os_calls() {
    let builds = Builds::new(&artifact());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let clock = invoke(&builds, mode, "native.app.clock", &["--native-clock", "true"]);
        assert_eq!(clock["value"]["value"]["case"], "Ok");
        let random = invoke(
            &builds,
            mode,
            "native.app.random",
            &["--native-random", "true", "--arg", "I64:32"],
        );
        assert_eq!(random["value"]["value"]["case"], "Ok");
        assert_eq!(
            random["value"]["value"]["payload"]["value"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
        let listener = invoke(
            &builds,
            mode,
            "native.app.listener",
            &["--native-listen", "127.0.0.1:0"],
        );
        assert_eq!(listener["value"]["value"]["case"], "Ok");
        assert_eq!(listener["value"]["value"]["payload"]["value"], true);
    }
}
