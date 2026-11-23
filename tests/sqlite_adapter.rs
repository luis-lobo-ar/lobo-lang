use std::process::Command;

#[test]
fn granted_sqlite_database_opens_through_the_real_c_adapter() {
    let folder = std::env::temp_dir().join(format!("lobo-sqlite-adapter-{}", std::process::id()));
    std::fs::create_dir(&folder).unwrap();
    for (name, flags) in [
        ("native", vec!["-O1"]),
        (
            "sanitized",
            vec!["-O1", "-fsanitize=address,undefined", "-fno-sanitize-recover=all"],
        ),
    ] {
        let binary = folder.join(name);
        let output = Command::new("clang")
            .args(["-std=c11", "-g0"])
            .args(flags)
            .arg("tests/native/sqlite_harness.c")
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for case in [
            "open",
            "schema",
            "commit-busy",
            "memory",
            "symlink",
            "corrupt",
            "values",
            "empty-invalid",
            "transactions",
            "policy",
            "injection",
            "budgets",
            "stale",
            "busy",
            "handles",
            "generation",
            "deny",
            "options",
            "authority",
            "fuel-cleanup",
            "heap-cleanup",
        ] {
            let output = Command::new(&binary)
                .arg(case)
                .arg(folder.join(format!("{name}-{case}.sqlite")))
                .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            if ["authority", "fuel-cleanup", "heap-cleanup"].contains(&case) {
                assert_eq!(output.status.code(), Some(1), "{case}: {output:?}");
                let stdout = String::from_utf8(output.stdout.clone()).unwrap();
                let diagnostic: serde_json::Value =
                    serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
                let code = match case {
                    "authority" => "invocation.native.authority",
                    "fuel-cleanup" => "resource.fuel.exhausted",
                    _ => "resource.memory.exhausted",
                };
                assert_eq!(diagnostic["diagnostic"]["code"], code);
                if case != "authority" {
                    assert!(stdout.contains("sqlite-fault-cleanup-passed"), "{stdout}");
                }
            } else {
                assert!(
                    output.status.success(),
                    "{name}/{case}: {output:?}; reproducer {}",
                    folder.display()
                );
                assert_eq!(output.stdout, b"sqlite-adapter-case-passed\n");
            }
            assert!(output.stderr.is_empty(), "{case}: {output:?}");
        }
    }
    std::fs::remove_dir_all(folder).unwrap();
}
