use lobo_core::{compile_bundle, identities::parse_manifest, SourceFile};
use serde_json::Value as Json;
mod support;
use support::Builds;

#[test]
fn context_callbacks_share_http_framing_with_the_existing_pure_handler() {
    let fixture = r#"module context;
use http {Request,Response,respond_with};
use controller {handle};
fn wrapped(context:I64,request:Request)->Response actions {db.execute,db.read,db.close} {
 if context==42 {handle(request)} else {Response {status:500,content_type:"application/json",body:"context lost"}}
}
fn respond(request:Bytes)->Bytes actions {db.execute,db.read,db.close} {respond_with(42,request,wrapped)}
identities {module:"context",http.Request:"lobo.http.server.Request",http.Response:"example.web.response",http.respond_with:"example.sqlite.rest.dispatch",controller.handle:"example.rest.controller.handle",wrapped:"context.wrapped",respond:"context.respond"}"#;
    let mut sources = vec![SourceFile {
        name: "context".into(),
        text: fixture.into(),
        identities: None,
    }];
    for (name, text, metadata) in [
        (
            "controller",
            include_str!("../examples/native/rest/controller.lobo"),
            Some(include_str!("../examples/native/rest/controller.lobo.ids.json")),
        ),
        (
            "http",
            include_str!("../libraries/http-server/server.lobo"),
            Some(include_str!("../libraries/http-server/server.lobo.ids.json")),
        ),
        ("response", include_str!("../libraries/http/http.lobo"), None),
        ("json", include_str!("../libraries/json/json.lobo"), None),
    ] {
        sources.push(SourceFile {
            name: name.into(),
            text: text.into(),
            identities: metadata.map(|m| parse_manifest(m).unwrap()),
        });
    }
    let builds = Builds::new(&compile_bundle("context", &sources).unwrap());
    let cases: Vec<Json> = serde_json::from_str(include_str!("native/rest_cases.json")).unwrap();
    for mode in ["eval", "native", "sanitized"] {
        for case in &cases {
            let bytes = case["request"]
                .as_str()
                .unwrap()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let arg = format!("Bytes:{bytes}");
            let context = builds.invoke(mode, "context.respond", &[&arg], 10_000_000, 1_000_000);
            let pure = builds.invoke(
                mode,
                "example.rest.controller.respond",
                &[&arg],
                10_000_000,
                1_000_000,
            );
            assert_eq!(context, pure, "{mode}: {}", case["name"]);
        }
    }
}
