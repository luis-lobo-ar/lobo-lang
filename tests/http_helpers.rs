use lobo_core::{compile_bundle, Artifact, SourceFile};
mod support;
use support::Builds;

fn builds() -> Builds {
    let app = r#"module app @id("helpers.app");
import Response @id("example.web.response"); import PathMatch @id("lobo.http.path_match");
import text_response @id("lobo.http.text_response"); import json_document @id("lobo.http.json_document");
import not_found @id("lobo.http.not_found"); import method_not_allowed @id("lobo.http.method_not_allowed");
import read_method @id("lobo.http.read_method"); import strip_prefix @id("lobo.http.strip_prefix");
import json_string @id("lobo.http.json_string");
function plain @id("helpers.plain")(value: Text) -> Response { text_response(200,value) }
function json @id("helpers.json")(value: Text) -> Response { json_document(404,value) }
function missing @id("helpers.missing")() -> Response { not_found() }
function method @id("helpers.method")() -> Response { method_not_allowed() }
function invalid @id("helpers.invalid")() -> Response { text_response(302,"redirect") }
function read @id("helpers.read")(value: Text) -> Bool { read_method(value) }
function prefix @id("helpers.prefix")(value: Text, start: Text) -> Text {
  match(strip_prefix(value,start)) { NoMatch { "<no-match>" }, Matched(rest) { rest } }
}
function quote @id("helpers.quote")(value: Text) -> Text { json_string(value) }
"#;
    let artifact = compile_bundle(
        "app",
        &[
            SourceFile {
                identities: None,
                name: "app".into(),
                text: app.into(),
            },
            SourceFile {
                identities: None,
                name: "http".into(),
                text: include_str!("../libraries/http/http.lobo").into(),
            },
        ],
    )
    .unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    Builds::new(&artifact)
}

fn text(value: &str) -> String {
    format!(
        "Text:{}",
        value
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

#[test]
fn typed_responses_preserve_body_content_type_and_supported_status() {
    let builds = builds();
    for mode in ["eval", "native", "sanitized"] {
        for (entry, input, status, kind, body) in [
            (
                "helpers.plain",
                Some("é\n"),
                "200",
                "text/plain; charset=utf-8",
                "é\n",
            ),
            (
                "helpers.json",
                Some("{\"error\":true}"),
                "404",
                "application/json",
                "{\"error\":true}",
            ),
            (
                "helpers.missing",
                None,
                "404",
                "text/plain; charset=utf-8",
                "Not found\n",
            ),
            (
                "helpers.method",
                None,
                "405",
                "text/plain; charset=utf-8",
                "Method not allowed\n",
            ),
        ] {
            let input = input.map(text);
            let args: Vec<_> = input.as_deref().into_iter().collect();
            let result = builds.invoke(mode, entry, &args, 200000, 100000);
            assert_eq!(result["value"]["value"]["id"], "example.web.response");
            let fields = &result["value"]["value"]["fields"];
            assert_eq!(fields["status"]["value"], status, "{mode}: {result}");
            assert_eq!(fields["content_type"]["value"], kind);
            assert_eq!(fields["body"]["value"], body);
        }
        assert_eq!(
            builds.invoke(mode, "helpers.invalid", &[], 200000, 100000)["diagnostic"]["code"],
            "runtime.contract.precondition"
        );
    }
}

#[test]
fn methods_and_prefix_matching_handle_empty_and_unicode_values() {
    let builds = builds();
    for mode in ["eval", "native", "sanitized"] {
        for (method, expected) in [
            ("GET", true),
            ("HEAD", true),
            ("POST", false),
            ("get", false),
            ("", false),
        ] {
            assert_eq!(
                builds.invoke(mode, "helpers.read", &[&text(method)], 200000, 100000)["value"]["value"],
                expected
            );
        }
        for (value, start, expected) in [
            ("/greet/é", "/greet/", "é"),
            ("é/猫", "é/", "猫"),
            ("é", "é", ""),
            ("é", "", "é"),
            ("", "", ""),
            ("a", "ab", "<no-match>"),
            ("é", "e", "<no-match>"),
        ] {
            assert_eq!(
                builds.invoke(
                    mode,
                    "helpers.prefix",
                    &[&text(value), &text(start)],
                    200000,
                    100000
                )["value"]["value"],
                expected
            );
        }
    }
}

#[test]
fn json_strings_round_trip_quotes_controls_and_utf8() {
    let builds = builds();
    let controls: String = (0_u8..32).map(char::from).collect();
    for mode in ["eval", "native", "sanitized"] {
        for value in [
            "",
            "hello",
            "\"\\\n\r\t",
            "é 🐺 猫",
            r#"x"},"admin":true,"x":""#,
            &controls,
            &"a".repeat(512),
        ] {
            let result = builds.invoke(mode, "helpers.quote", &[&text(value)], 200000, 100000);
            assert_eq!(result["status"], "ok", "{mode}: {result}");
            let encoded = result["value"]["value"].as_str().unwrap();
            assert_eq!(serde_json::from_str::<String>(encoded).unwrap(), value);
        }
        let result = builds.invoke(mode, "helpers.quote", &[&text("\"\\")], 200000, 100000);
        assert_eq!(result["value"]["value"], "\"\\\"\\\\\"");
    }
}
