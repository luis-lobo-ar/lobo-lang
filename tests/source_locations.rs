use lobo_core::{compile, Artifact, Value};
use serde_json::Value as Json;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn unresolved_names_have_exact_source_ranges() {
    let source = "// 🐺\r\nmodule m @id(\"m\");\r\nfunction f @id(\"f\")() -> I64 { missing }";
    let error = compile(source).unwrap_err();
    assert_eq!(error.code, "compile.name.unresolved");
    let json = serde_json::to_value(error).unwrap();
    let start = source.find("missing").unwrap();
    assert_eq!(json["range"]["start"]["byte"], start);
    assert_eq!(json["range"]["end"]["byte"], start + 7);
    assert_eq!(json["range"]["start"]["line"], 2);
    assert_eq!(json["range"]["start"]["character"], 31);
    assert_eq!(json["range"]["end"]["character"], 38);
    assert_eq!(json["subject"], "f");
    assert_eq!(json["source"], Json::Null);
}

fn program(body: &str) -> String {
    format!("module m @id(\"m\"); function f @id(\"f\")() -> I64 {{ {body} }}")
}

#[test]
fn exact_ranges_cover_types_calls_folds_and_repeated_names() {
    for (body, blame, code) in [
        ("let a: I64 = true; 1", "true", "compile.type.mismatch"),
        ("1 + false", "false", "compile.type.mismatch"),
        ("!1", "1", "compile.type.mismatch"),
        ("if 7 { 1 } else { 2 }", "7", "compile.type.mismatch"),
        ("if true { 1 } else { false }", "false", "compile.type.mismatch"),
        (
            "fold(i in false, a: I64 = 0) { a }",
            "false",
            "compile.type.mismatch",
        ),
        (
            "fold(i in 0, a: I64 = true) { a }",
            "true",
            "compile.type.mismatch",
        ),
        (
            "fold(i in 0, a: I64 = 0) { false }",
            "false",
            "compile.type.mismatch",
        ),
        ("missing(1)", "missing", "compile.name.unresolved"),
        ("f(1)", "f(1)", "compile.call.arity"),
        ("let a = 1; let a = 2; a", "a = 2", "compile.name.duplicate"),
        (
            "9223372036854775808",
            "9223372036854775808",
            "compile.literal.range",
        ),
        (
            "-9223372036854775809",
            "-9223372036854775809",
            "compile.literal.range",
        ),
    ] {
        let source = program(body);
        let error = compile(&source).unwrap_err();
        assert_eq!(error.code, code, "{body}");
        let span = error.range.unwrap_or_else(|| panic!("missing range: {body}"));
        let start = source.rfind(blame).unwrap();
        let width = if blame == "a = 2" { 1 } else { blame.len() };
        assert_eq!(span.start.byte, start, "{body}");
        assert_eq!(span.end.byte, start + width, "{body}");
    }
}

#[test]
fn lexer_parser_and_eof_locations_are_scalar_safe() {
    for (source, blame) in [
        (program("🐺"), "🐺".to_owned()),
        (program("1;"), ";".to_owned()),
        ("module m @id(\"bad id\");".into(), "\"bad id\"".into()),
        ("module m @id(\"unfinished".into(), "\"unfinished".into()),
        (program("let if = 0; 1"), "if".into()),
    ] {
        let error = compile(&source).unwrap_err();
        let span = error.range.expect("located syntax error");
        let start = source.rfind(&blame).unwrap();
        assert_eq!(span.start.byte, start, "{source}");
        assert_eq!(span.end.byte, start + blame.len(), "{source}");
        if blame == "🐺" {
            assert_eq!(span.end.character - span.start.character, 2);
        }
    }
    for source in ["", "module", "module m", "module m @id(\"m\"); function"] {
        let error = compile(source).unwrap_err();
        let span = error.range.unwrap();
        assert_eq!(span.start.byte, source.len());
        assert_eq!(span.end.byte, source.len());
    }
}

#[test]
fn declaration_and_contract_ranges_are_not_guessed_from_messages() {
    for (source, blame, width) in [
        (
            "module m @id(\"m\"); function f @id(\"f\")(a: I64, a: I64) -> I64 { a }",
            "a: I64",
            1,
        ),
        (
            "module m @id(\"m\"); function f @id(\"m\")() -> I64 { 1 }",
            "\"m\"",
            3,
        ),
        (
            "module m @id(\"m\"); function f @id(\"f\")() -> I64 { 1 } function f @id(\"g\")() -> I64 { 1 }",
            "f @id",
            1,
        ),
        (
            "module m @id(\"m\"); function f @id(\"f\")() -> I64 requires { 1 } { 1 }",
            "1 } {",
            1,
        ),
        (
            "module m @id(\"m\"); function f @id(\"f\")() -> I64 ensures { 3 } { 1 }",
            "3",
            1,
        ),
        (
            "module m @id(\"m\"); function f @id(\"f\")(n: I64) -> I64 { fold(i in 1, n: I64 = 0) { n } }",
            "n: I64 =",
            1,
        ),
    ] {
        let span = compile(source).unwrap_err().range.unwrap();
        let start = source.rfind(blame).unwrap();
        assert_eq!(span.start.byte, start, "{source}");
        assert_eq!(span.end.byte, start + width, "{source}");
    }
    let source = "module m @id(\"m\"); function f @id(\"f\")(n: I64) -> I64 { let x = n; x } function g @id(\"g\")() -> I64 { x }";
    assert_eq!(
        compile(source).unwrap_err().range.unwrap().start.byte,
        source.rfind("x").unwrap()
    );
}

#[test]
fn locations_do_not_change_artifact_bytes_or_trust_loaded_metadata() {
    let source = program("fold(i in 3, a: I64 = 0) { a + i }");
    let encoded = compile(&source).unwrap().to_json().unwrap();
    assert_eq!(
        encoded,
        compile(&format!("// 🐺\r\n{source}\n"))
            .unwrap()
            .to_json()
            .unwrap()
    );
    assert!(!encoded.contains("span"));
    let mut json: Json = serde_json::from_str(&encoded).unwrap();
    json["program"]["functions"][0]["body"]["value"]["span"] = serde_json::json!({"start":0,"end":1});
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "invocation.artifact.invalid"
    );
    let mut json: Json = serde_json::from_str(&encoded).unwrap();
    json["program"]["functions"][0]["body"]["value"]["initial"]["value"] =
        serde_json::json!({"type":"Bool","value":true});
    let error = Artifact::from_json(&json.to_string()).unwrap_err();
    assert_eq!(error.code, "compile.type.mismatch");
    assert!(error.range.is_none());
    let runtime = compile(&program("1 / 0"))
        .unwrap()
        .run("f", Vec::<Value>::new(), 100)
        .unwrap_err();
    assert!(runtime.range.is_none());
}

#[test]
fn stdin_check_is_source_only_structured_and_bounded() {
    for (source, exit, code) in [
        (program("1"), 0, None),
        (program("false"), 1, Some("compile.type.mismatch")),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lobo"))
            .args(["check", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(exit), "{output:?}");
        assert!(output.stderr.is_empty());
        let json: Json = serde_json::from_slice(&output.stdout).unwrap();
        if let Some(code) = code {
            assert_eq!(json["diagnostic"]["code"], code);
            assert!(json["diagnostic"]["range"].is_object());
            assert!(json["diagnostic"]["source"].is_null());
        } else {
            assert_eq!(json["status"], "ok");
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args(["check", "--stdin", "extra"])
        .output()
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Json>(&output.stdout).unwrap()["diagnostic"]["code"],
        "invocation.cli.invalid"
    );
}

#[test]
fn stdin_size_admission_precedes_utf8_decoding_at_the_read_boundary() {
    let source = format!("//{}🐺", " ".repeat(1_048_574));
    let mut child = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args(["check", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(source.as_bytes());
    let output = child.wait_with_output().unwrap();
    let json: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["kind"], "resource_exhausted");
    assert_eq!(json["diagnostic"]["code"], "resource.frontend.limit");
    assert!(json["diagnostic"]["range"].is_null());
}

#[test]
fn file_check_labels_source_but_artifact_check_has_no_claimed_source_location() {
    let scratch = std::env::temp_dir().join(format!("lobo-location-{}", std::process::id()));
    std::fs::create_dir(&scratch).unwrap();
    let source = scratch.join("invalid.lobo");
    std::fs::write(&source, program("missing")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .arg("check")
        .arg(&source)
        .output()
        .unwrap();
    let json: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["diagnostic"]["source"], source.to_str().unwrap());
    assert!(json["diagnostic"]["range"].is_object());
    let artifact = scratch.join("invalid.loboc");
    std::fs::write(&artifact, "{}").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .arg("check")
        .arg(&artifact)
        .output()
        .unwrap();
    let json: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["diagnostic"]["source"].is_null());
    assert!(json["diagnostic"]["range"].is_null());
    for path in [source, artifact] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(scratch).unwrap();
}
