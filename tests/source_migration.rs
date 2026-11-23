use std::process::Command;

#[test]
fn maintained_sources_use_modern_syntax_and_preserve_their_checked_programs() {
    let output = Command::new("node")
        .args([
            "scripts/migrate-syntax.cjs",
            "--verify",
            "--compiler",
            env!("CARGO_BIN_EXE_lobo"),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "passed");
    assert_eq!(result["checked_bundles"], 23);
    assert_eq!(result["embedded_sources"], 3);
}
