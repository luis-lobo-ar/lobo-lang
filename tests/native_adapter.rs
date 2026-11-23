use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Harness(PathBuf);
impl Harness {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lobo-native-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        let this = Self(path);
        for (name, flags) in [
            ("plain", vec!["-O2"]),
            (
                "sanitized",
                vec!["-O1", "-fsanitize=address,undefined", "-fno-sanitize-recover=all"],
            ),
        ] {
            let output = Command::new("clang")
                .args(["-std=c11", "-g"])
                .args(flags)
                .arg("-I")
                .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/native/adapter_harness.c"
                ))
                .arg("-o")
                .arg(this.0.join(name))
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        }
        this
    }
    fn case(&self, name: &str) {
        for binary in ["plain", "sanitized"] {
            let output = Command::new(self.0.join(binary))
                .arg(name)
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            if name == "null-buffer" || name == "fault-cleanup" || name == "formal-fault-before-publication" {
                assert_eq!(
                    output.status.code(),
                    Some(1),
                    "case {name} / {binary}: {output:?}"
                );
                assert!(output.stderr.is_empty(), "{output:?}");
                let stdout = String::from_utf8(output.stdout).unwrap();
                let mut lines = stdout.lines();
                let value: serde_json::Value = serde_json::from_str(lines.next().unwrap()).unwrap();
                let code = if name == "null-buffer" {
                    "invocation.native.buffer"
                } else {
                    "resource.fuel.exhausted"
                };
                assert_eq!(value["diagnostic"]["code"], code);
                if name == "fault-cleanup" || name == "formal-fault-before-publication" {
                    assert_eq!(lines.next(), Some("native-cleanup-on-fault-passed"));
                }
                assert!(lines.next().is_none());
                continue;
            }
            assert!(output.status.success(), "case {name} / {binary}: {output:?}");
            assert!(output.stderr.is_empty(), "case {name} / {binary}: {output:?}");
            assert_eq!(output.stdout, b"native-adapter-case-passed\n");
        }
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            std::fs::remove_dir_all(&self.0).unwrap();
        } else {
            eprintln!("Native adapter reproducer: {}", self.0.display());
        }
    }
}
#[test]
fn interrupted_and_spurious_syscalls_preserve_deadlines_and_error_classification() {
    let harness = Harness::new();
    for case in [
        "poll-eintr-deadline",
        "poll-ready-deadline",
        "recv-eintr-clock-failure",
    ] {
        harness.case(case);
    }
}
#[test]
fn native_registry_model_rejects_forged_stale_wrong_kind_and_wrapped_handles() {
    Harness::new().case("registry");
}
#[test]
fn native_fault_paths_close_descriptors_and_enforce_authority_before_syscalls() {
    let harness = Harness::new();
    for case in ["deny", "quota-cleanup", "bind-conflict"] {
        harness.case(case);
    }
}
#[test]
fn native_adapter_handles_forced_short_io_eof_and_interrupted_random_reads() {
    let harness = Harness::new();
    for case in ["short-io", "random-short-eintr"] {
        harness.case(case);
    }
}

#[test]
fn native_capability_matrix_never_amplifies_parent_rights() {
    Harness::new().case("cap-rights-matrix");
}
#[test]
fn native_abi_buffers_are_owned_and_malformed_null_views_are_classified() {
    let harness = Harness::new();
    for case in ["abi-owned-buffer", "null-buffer"] {
        harness.case(case);
    }
}
#[test]
fn native_cleanup_runs_before_process_exit_even_on_a_language_fault() {
    Harness::new().case("fault-cleanup");
}
#[test]
fn native_runtime_kind_checks_protect_syscall_entry_points() {
    Harness::new().case("bad-handles");
}

#[test]
fn interrupted_connects_cannot_publish_unconnected_stream_handles() {
    Harness::new().case("connect-eintr-unstarted");
}

#[test]
fn formal_counterexample_fault_before_handle_publication_closes_pending_descriptor() {
    Harness::new().case("formal-fault-before-publication");
}
