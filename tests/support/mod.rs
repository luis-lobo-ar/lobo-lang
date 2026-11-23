use lobo_core::Artifact;
use serde_json::Value as Json;

pub struct Builds(pub std::path::PathBuf);
impl Builds {
    pub fn new(artifact: &Artifact) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("lobo-fixture-{}-{stamp}-{sequence}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let build = Self(root);
        std::fs::write(build.0.join("program.c"), artifact.emit_c().unwrap()).unwrap();
        std::fs::write(build.0.join("program.loboc"), artifact.to_json().unwrap()).unwrap();
        artifact.compile_native(&build.0.join("native")).unwrap();
        artifact
            .compile_native_with_backend(&build.0.join("llvm"), lobo_core::NativeBackend::Llvm)
            .unwrap();
        artifact
            .compile_llvm_sanitized(&build.0.join("llvm-sanitized"))
            .unwrap();
        let output = std::process::Command::new("clang")
            .args([
                "-std=c11",
                "-O1",
                "-g",
                "-fsanitize=address,undefined",
                "-fno-sanitize-recover=all",
            ])
            .arg(build.0.join("program.c"))
            .arg("-o")
            .arg(build.0.join("sanitized"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        build
    }
    pub fn invoke(&self, mode: &str, entry: &str, arguments: &[&str], fuel: u64, heap: u64) -> Json {
        let mut command = if mode == "eval" {
            let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lobo"));
            command.arg("eval").arg(self.0.join("program.loboc"));
            command
        } else {
            std::process::Command::new(self.0.join(mode))
        };
        command
            .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
            .env("UBSAN_OPTIONS", "halt_on_error=1")
            .args([
                "--entry",
                entry,
                "--fuel",
                &fuel.to_string(),
                "--heap-cells",
                &heap.to_string(),
            ]);
        for argument in arguments {
            command.args(["--arg", argument]);
        }
        let output = command.output().unwrap();
        assert!(output.stderr.is_empty(), "{mode}: {output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            output.status.success(),
            result["status"] == "ok",
            "{mode}: {result}"
        );
        if mode == "native" {
            for backend in ["llvm", "llvm-sanitized"] {
                assert_eq!(
                    result,
                    self.invoke(backend, entry, arguments, fuel, heap),
                    "C versus {backend}: entry={entry}, args={arguments:?}, fuel={fuel}, heap={heap}"
                );
            }
        }
        result
    }
}
impl Drop for Builds {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Native reproducer: {}", self.0.display());
            return;
        }
        for name in [
            "program.c",
            "program.loboc",
            "native",
            "sanitized",
            "llvm",
            "llvm-sanitized",
            "probe.c",
            "probe",
            "probe-sanitized",
        ] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

// Generated JSON builder aliases and duplicate explicit imports do not change resolved code.
// Compare every other checked program field exactly, including the set of dependency IDs.
#[allow(dead_code)]
pub fn resolved_program(artifact: &Artifact) -> Json {
    let mut checked: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    for scope in checked["program"]["scopes"].as_array_mut().unwrap() {
        let ids: std::collections::BTreeSet<String> = scope["imports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|import| import["id"].as_str().unwrap().to_owned())
            .collect();
        scope["imports"] = serde_json::to_value(ids).unwrap();
    }
    checked
}
