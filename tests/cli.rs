use serde_json::Value;
use std::process::Command;

#[test]
fn source_and_compiled_artifact_have_identical_cli_behavior() {
    let binary = env!("CARGO_BIN_EXE_lobo");
    let source = format!("{}/examples/core.lobo", env!("CARGO_MANIFEST_DIR"));
    let directory = std::env::temp_dir().join(format!(
        "lobo-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let artifact = directory.join("core.loboc");
    let compiled = Command::new(binary)
        .args([
            "compile",
            &source,
            "-o",
            artifact.to_str().unwrap(),
            "--emit",
            "checked",
        ])
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{:?}", compiled);
    let mut outcomes = Vec::new();
    for file in [&source, &artifact.to_string_lossy().into_owned()] {
        let run = Command::new(binary)
            .args(["eval", file, "--entry", "example.increment", "--arg", "I64:41"])
            .output()
            .unwrap();
        assert!(run.status.success());
        let json: Value = serde_json::from_slice(&run.stdout).unwrap();
        assert_eq!(json["value"]["value"], "42");
        outcomes.push(json);
    }
    assert_eq!(outcomes[0], outcomes[1]);
    let retry = Command::new(binary)
        .args([
            "compile",
            &source,
            "-o",
            artifact.to_str().unwrap(),
            "--emit",
            "checked",
        ])
        .output()
        .unwrap();
    assert!(
        !retry.status.success(),
        "compile must not silently overwrite artifacts"
    );
    let fault = Command::new(binary)
        .args([
            "eval",
            &source,
            "--entry",
            "example.increment",
            "--arg",
            "I64:9223372036854775807",
        ])
        .output()
        .unwrap();
    assert!(!fault.status.success());
    let json: Value = serde_json::from_slice(&fault.stdout).unwrap();
    assert_eq!(json["kind"], "runtime_fault");
    assert_eq!(json["diagnostic"]["code"], "runtime.contract.precondition");
    std::fs::remove_file(artifact).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
