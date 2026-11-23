use lobo_core::{compile, compile_bundle, Artifact, SourceFile, Value};
use serde_json::Value as Json;
use std::process::Command;
mod support;
use support::Builds;

const SIMPLE: &str = "module sample;\nfn increment(n: I64) -> I64 { n + 1 }\nidentities { module: \"native.sample\", increment: \"native.increment\" }";

fn bundle(app: &str) -> Artifact {
    compile_bundle(
        "app",
        &[
            SourceFile {
                identities: if app == include_str!("../examples/mvc.compact.lobo") {
                    Some(
                        lobo_core::identities::parse_manifest(include_str!(
                            "../examples/mvc.compact.lobo.ids.json"
                        ))
                        .unwrap(),
                    )
                } else {
                    None
                },
                name: "app".into(),
                text: app.into(),
            },
            SourceFile {
                identities: None,
                name: "http".into(),
                text: include_str!("../libraries/http/http.lobo").into(),
            },
            SourceFile {
                identities: None,
                name: "json".into(),
                text: include_str!("../libraries/json/json.lobo").into(),
            },
        ],
    )
    .unwrap()
}

#[test]
fn stock_compiler_accepts_modern_syntax_without_a_marker_node_or_projection() {
    let modern = compile(SIMPLE).unwrap();
    let legacy = compile("module sample @id(\"native.sample\"); function increment @id(\"native.increment\")(n:I64)->I64 {n+1}").unwrap();
    assert_eq!(modern.to_json().unwrap(), legacy.to_json().unwrap());
    assert_eq!(
        modern.run("native.increment", vec![Value::I64(41)], 100).unwrap(),
        Value::I64(42)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args([
            "check",
            "examples/mvc.compact.lobo",
            "--source",
            "libraries/http/http.lobo",
            "--source",
            "libraries/json/json.lobo",
        ])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let data: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["status"], "ok");
}

#[test]
fn modern_mvc_and_deprecated_source_produce_identical_checked_native_programs() {
    let modern = bundle(include_str!("../examples/mvc.compact.lobo"));
    let legacy = bundle(include_str!("../examples/mvc.lobo"));
    assert_eq!(
        support::resolved_program(&modern),
        support::resolved_program(&legacy)
    );
    for language in [
        "core/1.0",
        "core/1.1",
        "core/1.2",
        "core/1.3",
        "core/1.4",
        "core/1.5",
        "core/1.6",
        "core/1.7",
        lobo_core::LANGUAGE,
    ] {
        let mut saved: Json = serde_json::from_str(&modern.to_json().unwrap()).unwrap();
        saved["language"] = language.into();
        Artifact::from_json(&saved.to_string()).unwrap();
    }
    let builds = Builds::new(&Artifact::from_json(&modern.to_json().unwrap()).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        for (id, status, body) in [(1, "200", "{\"name\":\"Luis\"}"), (42, "404", "Not found\n")] {
            let result = builds.invoke(
                mode,
                "example.mvc.show_user",
                &[&format!("I64:{id}")],
                10_000,
                10_000,
            );
            assert_eq!(result["status"], "ok", "{result}");
            assert_eq!(result["value"]["value"]["fields"]["status"]["value"], status);
            assert_eq!(result["value"]["value"]["fields"]["body"]["value"], body);
        }
        for (fuel, heap) in [(1, 10_000), (10_000, 1)] {
            assert_eq!(
                builds.invoke(mode, "example.mvc.show_user", &["I64:1"], fuel, heap)["kind"],
                "resource_exhausted"
            );
        }
    }
}

#[test]
fn native_metadata_validation_and_original_unicode_ranges_fail_closed() {
    for bad in [
        SIMPLE.replace(", increment: \"native.increment\"", ""),
        SIMPLE.replace("increment: \"native.increment\"", "stale: \"native.increment\""),
        SIMPLE.replace(
            "increment: \"native.increment\"",
            "increment: \"native.increment\", increment: \"duplicate\"",
        ),
        SIMPLE.replace("native.increment", "bad id"),
        SIMPLE.replace("fn increment", "function increment"),
        SIMPLE.replace("native.increment", "native.sample"),
        SIMPLE.to_owned() + " fn extra() -> I64 { 0 }",
    ] {
        let error = compile(&bad).expect_err(&bad);
        assert!(error.code.starts_with("compile."), "{error:?}");
    }
    let renamed = SIMPLE
        .replace("increment(n:", "bump(n:")
        .replace("increment: \"", "bump: \"");
    assert_eq!(compile(&renamed).unwrap().entries(), ["native.increment"]);
    let bad = format!(
        "// 🐺\r\n{}",
        SIMPLE.replace("n + 1", "false").replace('\n', "\r\n")
    );
    let error = serde_json::to_value(compile(&bad).unwrap_err()).unwrap();
    assert_eq!(error["code"], "compile.type.mismatch");
    assert_eq!(error["range"]["start"]["byte"], bad.find("false").unwrap());
    assert_eq!(error["range"]["end"]["byte"], bad.find("false").unwrap() + 5);
    assert_eq!(error["range"]["start"]["line"], 2);
}

#[test]
fn modern_syntax_retains_generics_callbacks_contracts_and_block_match_arms() {
    let source = r#"module general;
record Box<T> { value:T }
variant Choice<T> { None, Some(T) }
fn identity<T>(x:T)->T {x}
fn apply<T,U>(f:Fn(T)->U,x:T)->U {f(x)}
fn main()->I64 ensures { result == 42 } {
  let o = Choice::<Box<I64>>::Some(Box::<I64> {value:41});
  match o { None => 0, Some(b) => { let f = &identity::<I64>; apply::<I64,I64>(f,b.value)+1 } }
}
identities {module:"general", Box:"general.box", Choice:"general.choice", identity:"general.identity", apply:"general.apply", main:"general.main"}
"#;
    let artifact = compile(source).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "general.main", &[], 10_000, 10_000)["value"]["value"],
            "42"
        );
    }
    assert_eq!(
        compile(&source.replace("{x}", "{x+1}")).unwrap_err().code,
        "compile.type.mismatch"
    );
}

#[test]
fn modern_json_objects_check_types_keys_escaping_order_and_empty_values() {
    let source = r#"module document;
use js { Json, object, field, stringify };
fn empty()->Text { js.stringify(json {}) }
fn encode(name:Text)->Text { js.stringify(json {"b":name,"a":"é 🐺"}) }
identities {module:"document",empty:"document.empty",encode:"document.encode",js.Json:"lobo.json.value",js.object:"lobo.json.object",js.field:"lobo.json.field",js.stringify:"lobo.json.stringify"}
"#;
    let artifact = bundle(source);
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "document.empty", &[], 10_000, 10_000)["value"]["value"],
            "{}"
        );
        let result = builds.invoke(mode, "document.encode", &["Text:225c0af09f90ba"], 10_000, 10_000);
        let text = result["value"]["value"].as_str().unwrap();
        assert!(text.starts_with("{\"b\":"), "{text}");
        assert_eq!(
            serde_json::from_str::<Json>(text).unwrap(),
            serde_json::json!({"b":"\"\\\n🐺","a":"é 🐺"})
        );
    }
    for (bad, code) in [
        (source.replace("\"a\":", "\"\\u0062\":"), "compile.syntax.invalid"),
        (
            source.replace("\"b\":name", "\"b\":true"),
            "compile.type.mismatch",
        ),
        (
            source.replace("lobo.json.field", "wrong.field"),
            "compile.module.invalid",
        ),
        (
            source.replace("js.stringify(json {})", "js.unselected(json {})"),
            "compile.syntax.invalid",
        ),
    ] {
        let sources = [
            SourceFile {
                identities: None,
                name: "app".into(),
                text: bad,
            },
            SourceFile {
                identities: None,
                name: "http".into(),
                text: include_str!("../libraries/http/http.lobo").into(),
            },
            SourceFile {
                identities: None,
                name: "json".into(),
                text: include_str!("../libraries/json/json.lobo").into(),
            },
        ];
        assert_eq!(compile_bundle("app", &sources).unwrap_err().code, code);
    }
}
