use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Harness(PathBuf);
impl Harness {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lobo-formal-replay-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        let harness = Self(path);
        for (name, flags) in [
            ("optimized", vec!["-O2"]),
            (
                "sanitized",
                vec![
                    "-O1",
                    "-g",
                    "-fsanitize=address,undefined",
                    "-fno-sanitize-recover=all",
                ],
            ),
        ] {
            let output = Command::new("clang")
                .args(["-std=c11", "-I"])
                .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
                .args(flags)
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/native/formal_trace_harness.c"
                ))
                .arg("-o")
                .arg(harness.0.join(name))
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        }
        harness
    }
    fn replay(&self, case: &str) {
        for mode in ["optimized", "sanitized"] {
            let output = Command::new(self.0.join(mode))
                .arg(case)
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(output.stderr.is_empty(), "{case}/{mode}: {output:?}");
            let text = String::from_utf8(output.stdout).unwrap();
            if case == "root" || case == "untracked-fault" {
                assert_eq!(output.status.code(), Some(1), "{case}/{mode}: {text}");
                let mut lines = text.lines();
                let diagnostic: serde_json::Value = serde_json::from_str(lines.next().unwrap()).unwrap();
                assert_eq!(
                    diagnostic["diagnostic"]["code"],
                    if case == "root" {
                        "invocation.native.authority"
                    } else {
                        "resource.fuel.exhausted"
                    }
                );
                if case == "untracked-fault" {
                    assert_eq!(lines.next(), Some("native-cleanup-on-fault-passed"));
                }
                assert!(lines.next().is_none());
            } else {
                assert!(output.status.success(), "{case}/{mode}: {text}");
                assert_eq!(
                    text,
                    if case == "publish" || case == "deadline" {
                        "native-adapter-case-passed\n"
                    } else {
                        "formal-trace-passed\n"
                    }
                );
            }
        }
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Formal C replay retained at {}", self.0.display());
        } else {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
}
#[test]
fn authority_counterexamples_replay_against_the_c_provider() {
    let harness = Harness::new();
    for case in [
        "root", "amplify", "stale", "kind", "denied", "quota", "wrap", "cleanup",
    ] {
        harness.replay(case);
    }
}
#[test]
fn operation_counterexamples_replay_against_the_c_provider() {
    let harness = Harness::new();
    for case in ["untracked-fault", "publish", "deadline"] {
        harness.replay(case);
    }
}
