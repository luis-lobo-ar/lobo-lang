#[test]
fn raw_loopback_keeps_try_and_explicit_cleanup() {
    let loopback = include_str!("../examples/native/loopback.lobo");
    assert!(loopback.contains("try Native::connect"));
    assert!(loopback.contains("try pair(Native::capability())"));
    let read = loopback.find("let received:").unwrap();
    let close = loopback.find("let close_client=").unwrap();
    assert!(read < close && loopback[close..].contains("received"));
}

#[test]
fn maintained_sources_have_no_remaining_checked_equivalent_simplifications() {
    let out = std::env::temp_dir().join(format!(
        "lobo-source-audit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = std::process::Command::new("node")
        .args([
            "scripts/modernize-source.cjs",
            "--audit",
            "--out",
            out.to_str().unwrap(),
            "--compiler",
            env!("CARGO_BIN_EXE_lobo"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "pass");
    assert_eq!(result["changed"], 0, "See {}", out.display());
    assert_eq!(result["rewrites"], 0);
    std::fs::remove_dir_all(out).unwrap();
}

#[test]
fn every_maintained_lobo_source_has_a_reviewed_http_library_boundary() {
    let out = std::env::temp_dir().join(format!(
        "lobo-http-audit-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = std::process::Command::new("node")
        .args(["scripts/check-http-refactor.cjs", "--out", out.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}; retained {}", out.display());
    let result: serde_json::Value = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
    assert_eq!(result["status"], "pass");
    assert!(result["errors"].as_array().unwrap().is_empty());
    std::fs::remove_file(out).unwrap();
}
