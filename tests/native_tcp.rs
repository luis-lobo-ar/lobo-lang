use lobo_core::compile;
use serde_json::Value as Json;
use std::process::Command;
mod support;
use support::Builds;
const SOURCE: &str = include_str!("../examples/native/loopback.lobo");
fn invoke(builds: &Builds, mode: &str, entry: &str, atoms: &[&str], extra: &[&str]) -> Json {
    let mut command = Command::new(builds.0.join(mode));
    command.args([
        "--entry",
        entry,
        "--fuel",
        "1000000",
        "--heap-cells",
        "200000",
        "--native-listen",
        "127.0.0.1:0",
        "--native-connect",
        "self",
    ]);
    for atom in atoms {
        command.args(["--arg", atom]);
    }
    let output = command
        .args(extra)
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert!(output.stderr.is_empty(), "{output:?}");
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{result}");
    result
}
#[test]
fn native_loopback_preserves_binary_data_with_short_io_and_eof() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    assert_eq!(
        builds.invoke("eval", "native.loopback.roundtrip", &["Bytes:"], 10000, 100000)["diagnostic"]["code"],
        "invocation.native.required"
    );
    let corpus: Json = serde_json::from_str(include_str!("upstream/native/adapted-cases.json")).unwrap();
    let payload = corpus["socket_payload"]["hex"].as_str().unwrap();
    let large = "00ff7f80".repeat(1024);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for bytes in ["", "00ff", payload, &large] {
            let atom = format!("Bytes:{bytes}");
            let result = invoke(&builds, mode, "native.loopback.roundtrip", &[&atom], &[]);
            assert_eq!(result["value"]["value"]["case"], "Ok");
            assert_eq!(result["value"]["value"]["payload"]["value"], bytes);
        }
    }
}
#[test]
fn socket_deadlines_shutdown_close_and_generation_reuse_are_classified() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (case, code) in [
            (0, 6),
            (1, 2),
            (2, 2),
            (3, 5),
            (4, 0),
            (5, 5),
            (6, 3),
            (7, 5),
            (8, 2),
            (9, 0),
        ] {
            let atom = format!("I64:{case}");
            assert_eq!(
                invoke(&builds, mode, "native.loopback.edge", &[&atom], &[])["value"]["value"],
                code.to_string(),
                "{mode} case {case}"
            );
        }
    }
}
#[test]
fn native_resource_quotas_fail_closed() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for quota in [["--native-handles", "3"], ["--native-calls", "1"]] {
            assert_eq!(
                invoke(&builds, mode, "native.loopback.edge", &["I64:4"], &quota)["value"]["value"],
                "11"
            );
        }
        let result = invoke(
            &builds,
            mode,
            "native.loopback.roundtrip",
            &["Bytes:0001"],
            &["--native-bytes", "1"],
        );
        assert_eq!(result["value"]["value"]["case"], "Err");
        assert_eq!(result["value"]["value"]["payload"]["value"], "11");
    }
}
