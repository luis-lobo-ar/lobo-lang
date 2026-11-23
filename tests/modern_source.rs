#[test]
fn maintained_native_flows_use_try_without_bypassing_explicit_cleanup() {
    let app = include_str!("../examples/native/rest/app.lobo");
    assert!(app.contains("try Native::restrict_capability"));
    assert!(app.contains("try Native::listen"));
    let work = app.find("let outcome=serve(").unwrap();
    let close = app.find("try Native::close_listener").unwrap();
    assert!(work < close, "cleanup must run after the handled serve result");
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
