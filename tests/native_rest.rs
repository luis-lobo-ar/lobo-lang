use lobo_core::{compile_bundle, SourceFile};
use serde_json::Value as Json;
mod support;
use support::Builds;

fn build() -> Builds {
    let files = [
        (
            "controller",
            include_str!("../examples/native/rest/controller.lobo"),
            Some(include_str!("../examples/native/rest/controller.lobo.ids.json")),
        ),
        (
            "server",
            include_str!("../libraries/http-server/server.lobo"),
            Some(include_str!("../libraries/http-server/server.lobo.ids.json")),
        ),
        ("http", include_str!("../libraries/http/http.lobo"), None),
        ("json", include_str!("../libraries/json/json.lobo"), None),
    ];
    let sources: Vec<_> = files
        .into_iter()
        .map(|(name, text, identities)| SourceFile {
            name: name.into(),
            text: text.into(),
            identities: identities.map(|value| {
                let sidecar: Json = serde_json::from_str(value).unwrap();
                serde_json::from_value(sidecar["identities"].clone()).unwrap()
            }),
        })
        .collect();
    Builds::new(&compile_bundle("controller", &sources).unwrap())
}
fn decode(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn rest_protocol_has_identical_framing_and_json_on_both_native_backends_and_reference() {
    let builds = build();
    let cases: Vec<Json> = serde_json::from_str(include_str!("native/rest_cases.json")).unwrap();
    for mode in ["eval", "native", "llvm", "sanitized", "llvm-sanitized"] {
        for case in &cases {
            let input: String = case["request"]
                .as_str()
                .unwrap()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            let output = builds.invoke(
                mode,
                "example.rest.controller.respond",
                &[&format!("Bytes:{input}")],
                10_000_000,
                1_000_000,
            );
            assert_eq!(output["status"], "ok", "{mode} {}: {output}", case["name"]);
            let wire = String::from_utf8(decode(output["value"]["value"].as_str().unwrap())).unwrap();
            let (header, body) = wire.split_once("\r\n\r\n").unwrap();
            let mut lines = header.split("\r\n");
            let status: i64 = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
            assert_eq!(
                status,
                case["status"].as_i64().unwrap(),
                "{mode} {}",
                case["name"]
            );
            let headers: std::collections::HashMap<_, _> =
                lines.map(|line| line.split_once(": ").unwrap()).collect();
            assert_eq!(headers["Connection"], "close");
            assert_eq!(headers["Content-Type"], "application/json");
            let length: usize = headers["Content-Length"].parse().unwrap();
            if case["name"] == "head" {
                assert!(body.is_empty() && length > 0);
            } else {
                assert_eq!(length, body.len());
            }
            if !case["body"].is_null() {
                assert_eq!(serde_json::from_str::<Json>(body).unwrap(), case["body"]);
            }
            if status == 405 {
                assert_eq!(headers["Allow"], "GET, HEAD");
            }
        }
    }
}
