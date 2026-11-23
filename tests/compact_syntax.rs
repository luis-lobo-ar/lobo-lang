use lobo_core::{compile_bundle_json, Artifact};
use std::process::Command;
mod support;
use support::Builds;

#[test]
fn compact_mvc_lowers_to_the_same_checked_program_and_native_behavior() {
    let output = Command::new("node")
        .args([
            "scripts/compact-syntax.cjs",
            "tests/fixtures/compact-v0.1/mvc.compact.lobo",
            "--source",
            "tests/fixtures/compact-v0.1/http.lobo",
            "--source",
            "tests/fixtures/compact-v0.1/json.lobo",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let compact = compile_bundle_json(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let mut canonical: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    canonical["sources"][0]["text"] = include_str!("../examples/mvc.lobo").into();
    let canonical = compile_bundle_json(&canonical.to_string()).unwrap();
    // Stable IDs, scopes, expressions, contracts and actions must all survive projection.
    assert_eq!(compact.to_json().unwrap(), canonical.to_json().unwrap());
    let artifact = Artifact::from_json(&compact.to_json().unwrap()).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        for (id, status, body) in [(1, "200", "{\"name\":\"Luis\"}"), (42, "404", "Not found\n")] {
            let result = builds.invoke(
                mode,
                "example.mvc.show_user",
                &[&format!("I64:{id}")],
                10_000,
                10_000,
            );
            assert_eq!(result["status"], "ok", "{mode}: {result}");
            assert_eq!(result["value"]["value"]["fields"]["status"]["value"], status);
            assert_eq!(result["value"]["value"]["fields"]["body"]["value"], body);
        }
        let low_fuel = builds.invoke(mode, "example.mvc.show_user", &["I64:1"], 1, 10_000);
        assert_eq!(low_fuel["kind"], "resource_exhausted", "{mode}: {low_fuel}");
        let low_heap = builds.invoke(mode, "example.mvc.show_user", &["I64:1"], 10_000, 1);
        assert_eq!(low_heap["kind"], "resource_exhausted", "{mode}: {low_heap}");
    }
}
