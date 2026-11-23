use std::process::Command;

#[test]
fn sqlite_source_correspondence_rejects_stale_fingerprints_even_under_python_o() {
    let checker = "scripts/check-sqlite-models.py";
    let output = Command::new("python3")
        .args(["-O", checker, "--verify-map"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let mut coverage: serde_json::Value =
        serde_json::from_str(include_str!("../verification/sqlite/coverage.json")).unwrap();
    coverage["files"]["src/native_sqlite.c"]["sha256"] = "stale".into();
    let path = std::env::temp_dir().join(format!("lobo-stale-sqlite-map-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&coverage).unwrap()).unwrap();
    let output = Command::new("python3")
        .args(["-O", checker, "--verify-map", "--coverage"])
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success(), "stale map passed: {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("stale reviewed SQLite input: src/native_sqlite.c")
    );
}
