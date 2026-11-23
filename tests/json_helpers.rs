use lobo_core::{compile_bundle, Artifact, SourceFile};
use serde_json::{json, Value};
mod support;
use support::Builds;

const APP: &str = r#"module app @id("json.tests");
import Json @id("lobo.json.value"); import JsonObject @id("lobo.json.object_type");
import JsonMember @id("lobo.json.member");
import object @id("lobo.json.object"); import field @id("lobo.json.field");
import array @id("lobo.json.array"); import append @id("lobo.json.append");
import stringify @id("lobo.json.stringify"); import is_number @id("lobo.json.is_number");
import json_response @id("lobo.json.response"); import Response @id("example.web.response");
record Deep @id("json.tests.deep_type") { count:I64, value:Json }
function integer @id("json.tests.integer")(n:I64)->Text {stringify(Json::Integer(n))}
function number @id("json.tests.number")(value:Text)->Text {stringify(Json::Number(value))}
function valid @id("json.tests.valid")(value:Text)->Bool {is_number(value)}
function quote @id("json.tests.quote")(value:Text)->Text {stringify(Json::String(value))}
function boolean @id("json.tests.boolean")(value:Bool)->Text {stringify(Json::Boolean(value))}
function null_value @id("json.tests.null")()->Text {stringify(Json::Null)}
function empty_object @id("json.tests.empty_object")()->Text {stringify(Json::Object(object()))}
function empty_array @id("json.tests.empty_array")()->Text {stringify(Json::Array(array()))}
function nested @id("json.tests.nested")(name:Text)->Text {
  let items = append(append(array(),Json::Integer(1)),Json::String(name));
  let base = field(object(),"items",Json::Array(items));
  let document = field(base,"ok",Json::Boolean(true));
  stringify(Json::Object(document))
}
function named @id("json.tests.named")(name:Text)->Text {
  stringify(Json::Object(field(object(),name,Json::String(name))))
}
function persistent @id("json.tests.persistent")()->Text {
  let base = field(object(),"a",Json::Integer(1));
  let other = field(base,"b",Json::Integer(2));
  let values = append(append(array(),Json::Object(base)),Json::Object(other));
  stringify(Json::Array(values))
}
function deep @id("json.tests.deep")(n:I64)->Text {
  let built = while(state:Deep = Deep {count:0,value:Json::Null}; state.count<n) {
    Deep {count:state.count+1,value:Json::Array(append(array(),state.value))}
  };
  stringify(built.value)
}
function duplicate @id("json.tests.duplicate")()->Text {
  let base = field(object(),"key",Json::Null);
  stringify(Json::Object(field(base,"key",Json::Boolean(true))))
}
function raw_duplicate @id("json.tests.raw_duplicate")()->Text {
  let members = List::prepend(JsonMember {name:"key",value:Json::Null},
    List::prepend(JsonMember {name:"key",value:Json::Boolean(true)},List::empty<JsonMember>()));
  stringify(Json::Object(JsonObject {fields:members}))
}
function response @id("json.tests.response")(name:Text)->Response {
  json_response(200,Json::Object(field(object(),"name",Json::String(name))))
}
function invalid_status @id("json.tests.invalid_status")()->Response {json_response(302,Json::Null)}
"#;

fn builds() -> Builds {
    let artifact = compile_bundle(
        "app",
        &[
            SourceFile {
                identities: None,
                name: "app".into(),
                text: APP.into(),
            },
            SourceFile {
                identities: None,
                name: "json".into(),
                text: std::fs::read_to_string("libraries/json/json.lobo").unwrap(),
            },
            SourceFile {
                identities: None,
                name: "http".into(),
                text: include_str!("../libraries/http/http.lobo").into(),
            },
        ],
    )
    .unwrap();
    Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap())
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

fn encoded(builds: &Builds, mode: &str, entry: &str, arguments: &[&str]) -> String {
    let result = builds.invoke(mode, entry, arguments, 1_000_000, 1_000_000);
    assert_eq!(result["status"], "ok", "{mode}: {entry}: {result}");
    result["value"]["value"].as_str().unwrap().to_string()
}

#[test]
fn json_values_preserve_types_order_escaping_and_immutable_builders() {
    let builds = builds();
    let controls: String = (0u8..32).map(char::from).collect();
    for mode in ["eval", "native", "sanitized"] {
        for (entry, expected) in [
            ("json.tests.null", "null"),
            ("json.tests.empty_object", "{}"),
            ("json.tests.empty_array", "[]"),
        ] {
            assert_eq!(encoded(&builds, mode, entry, &[]), expected);
        }
        for value in [false, true] {
            assert_eq!(
                encoded(&builds, mode, "json.tests.boolean", &[&format!("Bool:{value}")]),
                value.to_string()
            );
        }
        for value in [i64::MIN, -10, -1, 0, 1, 10, i64::MAX] {
            let result = encoded(&builds, mode, "json.tests.integer", &[&format!("I64:{value}")]);
            assert_eq!(result, value.to_string());
            assert_eq!(serde_json::from_str::<Value>(&result).unwrap(), json!(value));
        }
        for name in [
            "",
            "Luis",
            "é 🐺 猫",
            "\"\\\n\r\t",
            &controls,
            r#"x"},"admin":true,"x":""#,
            &"a".repeat(512),
        ] {
            let argument = text(name);
            let quoted = encoded(&builds, mode, "json.tests.quote", &[&argument]);
            assert_eq!(serde_json::from_str::<String>(&quoted).unwrap(), name);
            let named = encoded(&builds, mode, "json.tests.named", &[&argument]);
            let expected: serde_json::Map<String, Value> =
                [(name.to_string(), json!(name))].into_iter().collect();
            assert_eq!(
                serde_json::from_str::<Value>(&named).unwrap(),
                Value::Object(expected)
            );
            let nested = encoded(&builds, mode, "json.tests.nested", &[&argument]);
            assert_eq!(
                serde_json::from_str::<Value>(&nested).unwrap(),
                json!({"items":[1,name],"ok":true})
            );
            assert!(nested.starts_with("{\"items\":[1,"));
        }
        assert_eq!(
            encoded(&builds, mode, "json.tests.persistent", &[]),
            r#"[{"a":1},{"a":1,"b":2}]"#
        );
    }
}

#[test]
fn decimal_number_grammar_is_checked_before_output() {
    let builds = builds();
    for mode in ["eval", "native", "sanitized"] {
        for value in [
            "0",
            "-0",
            "12",
            "-12",
            "0.0",
            "-0.125",
            "1e3",
            "1E+3",
            "1e-3",
            "123456789012345678901234567890.123456789",
            "1e400",
        ] {
            let arg = text(value);
            assert_eq!(
                builds.invoke(mode, "json.tests.valid", &[&arg], 100000, 100000)["value"]["value"],
                true,
                "{value}"
            );
            assert_eq!(encoded(&builds, mode, "json.tests.number", &[&arg]), value);
        }
        for value in [
            "", "-", "+1", "01", "-01", "00", ".5", "1.", "1e", "1e+", "1e-", "NaN", "Infinity", " 1", "1 ",
            "1\n", "0x10", "١", "1,2", "null", "1]}", "1e2.3",
        ] {
            let arg = text(value);
            assert_eq!(
                builds.invoke(mode, "json.tests.valid", &[&arg], 100000, 100000)["value"]["value"],
                false,
                "{value}"
            );
            let result = builds.invoke(mode, "json.tests.number", &[&arg], 100000, 100000);
            assert_eq!(
                result["diagnostic"]["code"], "runtime.contract.precondition",
                "{value}: {result}"
            );
        }
    }
}

#[test]
fn nested_values_contracts_and_resource_limits_work_in_native_routes() {
    let builds = builds();
    for mode in ["eval", "native", "sanitized"] {
        for depth in [0, 1, 40, 1000] {
            let actual = encoded(&builds, mode, "json.tests.deep", &[&format!("I64:{depth}")]);
            assert_eq!(actual, format!("{}null{}", "[".repeat(depth), "]".repeat(depth)));
        }
        for entry in [
            "json.tests.duplicate",
            "json.tests.raw_duplicate",
            "json.tests.invalid_status",
        ] {
            let result = builds.invoke(mode, entry, &[], 100000, 100000);
            assert_eq!(
                result["diagnostic"]["code"], "runtime.contract.precondition",
                "{entry}: {result}"
            );
        }
        for (fuel, heap, code) in [
            (0, 100000, "resource.fuel.exhausted"),
            (100000, 0, "resource.memory.exhausted"),
        ] {
            let result = builds.invoke(mode, "json.tests.deep", &["I64:10"], fuel, heap);
            assert_eq!(result["diagnostic"]["code"], code, "{result}");
        }
        let result = builds.invoke(mode, "json.tests.response", &[&text("é\"\n")], 100000, 100000);
        let fields = &result["value"]["value"]["fields"];
        assert_eq!(fields["status"]["value"], "200");
        assert_eq!(fields["content_type"]["value"], "application/json");
        assert_eq!(
            serde_json::from_str::<Value>(fields["body"]["value"].as_str().unwrap()).unwrap(),
            json!({"name":"é\"\n"})
        );
    }
}

#[test]
fn mvc_controller_returns_the_same_typed_http_responses_with_json_values() {
    let artifact = compile_bundle(
        "mvc",
        &[
            SourceFile {
                identities: None,
                name: "mvc".into(),
                text: include_str!("../examples/mvc.lobo").into(),
            },
            SourceFile {
                identities: None,
                name: "json".into(),
                text: std::fs::read_to_string("libraries/json/json.lobo").unwrap(),
            },
            SourceFile {
                identities: None,
                name: "http".into(),
                text: include_str!("../libraries/http/http.lobo").into(),
            },
        ],
    )
    .unwrap();
    let builds = Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "example.mvc.show_user", &["I64:1"], 10000, 10000);
        let fields = &result["value"]["value"]["fields"];
        assert_eq!(fields["status"]["value"], "200");
        assert_eq!(fields["content_type"]["value"], "application/json");
        assert_eq!(fields["body"]["value"], r#"{"name":"Luis"}"#);
        let result = builds.invoke(mode, "example.mvc.show_user", &["I64:42"], 10000, 10000);
        assert_eq!(result["value"]["value"]["fields"]["status"]["value"], "404");
    }
}
