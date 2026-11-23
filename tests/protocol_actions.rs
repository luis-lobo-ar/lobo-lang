use lobo_core::{compile, Value};
use serde_json::Value as Json;

#[test]
fn tool_descriptor_requires_actions_without_a_legacy_alias() {
    let schema: Json =
        serde_json::from_str(include_str!("../spec/schemas/tool-descriptor.schema.json")).unwrap();
    let required = schema["required"].as_array().unwrap();
    let properties = schema["properties"].as_object().unwrap();

    assert!(required.iter().any(|field| field == "actions"));
    assert!(!required.iter().any(|field| field == "effects"));
    assert!(properties.contains_key("actions"));
    assert!(!properties.contains_key("effects"));
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(properties["actions"]["type"], "array");
    assert_eq!(properties["actions"]["maxItems"], 64);
    assert_eq!(properties["actions"]["items"]["type"], "string");
    assert!(required.iter().any(|field| field == "required_capabilities"));
}

#[test]
fn protocol_schemas_and_examples_use_ltp_0_2_and_actions_checks() {
    for (name, source) in [
        (
            "request-envelope",
            include_str!("../spec/schemas/request-envelope.schema.json"),
        ),
        (
            "response-envelope",
            include_str!("../spec/schemas/response-envelope.schema.json"),
        ),
        (
            "tool-descriptor",
            include_str!("../spec/schemas/tool-descriptor.schema.json"),
        ),
    ] {
        let schema: Json = serde_json::from_str(source).unwrap();
        assert_eq!(
            schema["$id"],
            format!("https://lobo-lang.invalid/spec/ltp/0.2/{name}.schema.json")
        );
        if name != "tool-descriptor" {
            assert_eq!(schema["properties"]["ltp"]["const"], "0.2");
        }
    }

    let session = include_str!("../examples/agent-session.jsonl");
    let protocol = include_str!("../docs/tool-protocol.md");
    let json_blocks = protocol
        .split("```json\n")
        .skip(1)
        .map(|block| block.split("```").next().unwrap());
    let mut verification_requests = 0;
    let mut descriptors = 0;
    for source in session
        .lines()
        .filter(|line| !line.trim().is_empty())
        .chain(json_blocks)
    {
        let message: Json = serde_json::from_str(source).unwrap();
        if message.get("ltp").is_some() {
            assert_eq!(message["ltp"], "0.2");
        }
        if message["tool"] == "verification.run" {
            let checks = message["arguments"]["checks"].as_array().unwrap();
            assert!(checks.iter().any(|check| check == "actions"));
            assert!(!checks.iter().any(|check| check == "effects"));
            verification_requests += 1;
        }
        if message.get("name").is_some() {
            assert!(message["actions"].is_array());
            assert!(message.get("effects").is_none());
            descriptors += 1;
        }
    }
    assert_eq!(verification_requests, 2);
    assert_eq!(descriptors, 1);
}

#[test]
fn contextual_actions_syntax_does_not_reserve_a_core_identifier() {
    let source = r#"module demo @id("demo");
        function actions @id("actions")(n: I64) -> I64 { n }
        function main @id("main")() -> I64 { actions(42) }"#;
    assert_eq!(
        compile(source).unwrap().run("main", vec![], 100).unwrap(),
        Value::I64(42)
    );

    let declared = r#"module demo @id("demo");
        function main @id("main")() -> I64 actions { http.request } { 42 }"#;
    assert_eq!(
        compile(declared).unwrap().run("main", vec![], 100).unwrap(),
        Value::I64(42)
    );
}
