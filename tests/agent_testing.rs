use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};
mod support;

fn request(action: Value) -> Value {
    json!({"format":"lobo-test/0.1","action":action})
}

fn run(actions: Vec<Value>) -> Vec<Value> {
    run_with_clang(actions, "/usr/bin/clang")
}

fn run_with_clang(actions: Vec<Value>, clang: &str) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lobo-test"))
        .args(["--session", "--clang", clang])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        for action in actions {
            writeln!(input, "{}", request(action)).unwrap();
        }
    });
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn agent_session_discovers_its_contract_and_closes_explicitly() {
    let results = run(vec![json!({"op":"describe"}), json!({"op":"close"})]);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["format"], "lobo-test/0.1");
    assert_eq!(results[0]["status"], "ok");
    assert!(results[0]["contracts"]["verify"].is_string());
    assert!(results[0]["contracts"]["test"].is_string());
    assert_eq!(results[1]["status"], "closed");
    assert!(results[0]["session"].is_string());
    assert_eq!(results[0]["session"], results[1]["session"]);
    let other = run(vec![json!({"op":"describe"})]);
    assert_ne!(results[0]["session"], other[0]["session"]);
}

const SOURCE: &str = "module app @id(\"app\"); function inc @id(\"app.inc\")(n: I64) -> I64 { helper(n) } function helper @id(\"app.helper\")(n: I64) -> I64 { n + 1 } function constant @id(\"app.constant\")() -> I64 { 7 }";

fn open() -> Value {
    json!({"op":"open","root":"app","sources":[{"name":"app","text":SOURCE}],
    "requirements":[{"id":"increment","description":"Increment by one"},{"id":"constant","description":"Constant is seven"}],
    "tests":[
      {"id":"inc","requirement":"increment","entry":"app.inc","arguments":["I64:41"],"expect":{"kind":"value","value":{"type":"I64","value":"42"}},"fuel":1000,"heap_cells":1000},
      {"id":"const","requirement":"constant","entry":"app.constant","arguments":[],"expect":{"kind":"value","value":{"type":"I64","value":"7"}},"fuel":1000,"heap_cells":1000}
    ]})
}

fn verify(base: &str, source: &str) -> Value {
    json!({"op":"verify","base":base,"root":"app","sources":[{"name":"app","text":source}],
      "environment":"fixture-v1","scope":"affected","max_cases":256,"wall_ms":5000})
}

#[test]
fn managed_metadata_is_explicit_and_part_of_agent_evidence_dependencies() {
    let text =
        "module app; fn inc(n:I64)->I64 {helper(n)} fn helper(n:I64)->I64 {n+1} fn constant()->I64 {7}";
    let ids = json!({"module":"app","inc":"app.inc","helper":"app.helper","constant":"app.constant"});
    let mut plan = open();
    plan["sources"][0] = json!({"name":"app","text":text,"identities":ids});
    let mut candidate = verify("s1", text);
    candidate["sources"][0]["identities"] = ids.clone();
    let mut changed = verify("s2", text);
    changed["sources"][0]["identities"] = ids;
    changed["sources"][0]["identities"]["helper"] = json!("app.renamed_helper_identity");
    let results = run(vec![plan, candidate, changed]);
    assert_eq!(results[1]["status"], "checks_satisfied");
    assert_eq!(results[1]["counts"]["executed"], 2);
    assert_eq!(results[2]["status"], "checks_satisfied");
    assert_eq!(results[2]["counts"]["executed"], 1);
    assert_eq!(results[2]["counts"]["reused"], 1);
}

#[test]
fn generic_template_changes_invalidate_instances_and_unrelated_additions_keep_evidence() {
    let source = r#"module app @id("app");
      function identity<T> @id("identity")(x: T) -> T { x }
      function number @id("number")() -> I64 { identity::<I64>(7) }
      function flag @id("flag")() -> Bool { identity::<Bool>(true) }
      function constant @id("constant")() -> I64 { 9 }"#;
    let changed = source.replace("{ x }", "ensures { false } { x }");
    let unrelated = format!("{source} function text @id(\"text\")() -> Text {{ identity::<Text>(\"hi\") }}");
    let plan = json!({"op":"open","root":"app","sources":[{"name":"app","text":source}],
      "requirements":[{"id":"generic","description":"Generic results and unrelated constant"}],
      "tests":[
        {"id":"number","requirement":"generic","entry":"number","arguments":[],"expect":{"kind":"value","value":{"type":"I64","value":"7"}},"fuel":1000,"heap_cells":1000},
        {"id":"flag","requirement":"generic","entry":"flag","arguments":[],"expect":{"kind":"value","value":{"type":"Bool","value":true}},"fuel":1000,"heap_cells":1000},
        {"id":"constant","requirement":"generic","entry":"constant","arguments":[],"expect":{"kind":"value","value":{"type":"I64","value":"9"}},"fuel":1000,"heap_cells":1000}
    ]});
    let mut full = verify("s2", source);
    full["scope"] = json!("all");
    let results = run(vec![
        plan,
        verify("s1", source),
        verify("s2", &changed),
        verify("s2", &unrelated),
        full,
    ]);
    assert_eq!(results[1]["status"], "checks_satisfied", "{}", results[1]);
    assert_eq!(results[2]["counts"]["executed"], 2);
    assert_eq!(results[2]["counts"]["reused"], 1);
    assert_eq!(results[2]["failures"].as_array().unwrap().len(), 2);
    assert_eq!(results[3]["counts"]["executed"], 0);
    assert_eq!(results[3]["counts"]["reused"], 3);
    assert_eq!(results[4]["counts"]["executed"], 3);
    assert_eq!(results[4]["counts"]["reused"], 0);
}

#[test]
fn native_change_verification_reuses_unaffected_evidence_and_replays_an_exact_failure() {
    let broken = SOURCE.replace("{ 7 }", "{ 8 }");
    let results = run(vec![
        open(),
        verify("s1", SOURCE),
        verify("s2", &broken),
        json!({"op":"explain","snapshot":"s3","test":"const"}),
        json!({"op":"replay","snapshot":"s3","test":"const"}),
    ]);
    assert_eq!(results[0]["status"], "opened", "{}", results[0]);
    assert_eq!(results[1]["status"], "checks_satisfied", "{}", results[1]);
    assert_eq!(results[1]["counts"]["executed"], 2);
    assert_eq!(results[2]["status"], "failed", "{}", results[2]);
    assert_eq!(results[2]["counts"]["executed"], 1);
    assert_eq!(results[2]["counts"]["reused"], 1);
    assert_eq!(results[2]["failures"][0]["test"], "const");
    assert_eq!(results[2]["failures"][0]["difference"]["path"], "$.value");
    assert_eq!(results[2]["failures"][0]["difference"]["actual"], "8");
    assert_eq!(results[3]["bundle"]["sources"][0]["text"], broken);
    assert_eq!(results[3]["test"]["expect"]["value"]["value"], "7");
    assert_eq!(results[4]["status"], "failed");
    assert_eq!(results[4]["reproduced"], true);
}

#[test]
fn transitive_contract_and_layout_changes_invalidate_the_right_checks_and_full_mode_never_reuses() {
    let transitive = SOURCE.replace("n + 1", "n + 2");
    let contract = SOURCE.replace("{ helper(n) }", "requires { guard(n) } { helper(n) }")
        + " function guard @id(\"guard\")(n: I64) -> Bool { true }";
    let layout = format!("{SOURCE} record Unused @id(\"unused\") {{ value: I64 }}");
    let mut full = verify("s2", SOURCE);
    full["scope"] = json!("all");
    let results = run(vec![
        open(),
        verify("s1", SOURCE),
        verify("s2", &transitive),
        verify("s2", &contract),
        verify("s2", &layout),
        full,
    ]);
    assert_eq!(results[2]["status"], "failed");
    assert_eq!(results[2]["failures"][0]["test"], "inc");
    assert_eq!(results[2]["counts"]["reused"], 1);
    assert_eq!(results[3]["counts"]["executed"], 1);
    assert_eq!(results[4]["counts"]["executed"], 2);
    assert_eq!(results[4]["counts"]["reused"], 0);
    assert_eq!(results[5]["full_execution"], true);
    assert_eq!(results[5]["counts"]["executed"], 2);
}

#[test]
fn pending_and_uncovered_checks_are_never_reported_as_acceptance() {
    let mut plan = open();
    plan["requirements"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"untested","description":"No check yet"}));
    let mut limited = verify("s1", SOURCE);
    limited["max_cases"] = json!(1);
    let results = run(vec![plan, limited, verify("s2", SOURCE)]);
    assert_eq!(results[1]["status"], "incomplete");
    assert_eq!(results[1]["counts"]["pending"], 1);
    assert_eq!(results[2]["status"], "incomplete");
    assert_eq!(results[2]["uncovered_requirements"], json!(["untested"]));
    assert_eq!(results[2]["counts"]["passed"], 2);
}

#[test]
fn frozen_criteria_invalid_candidates_and_injected_cache_receipts_cannot_produce_green_reports() {
    let mut injected = verify("s2", SOURCE);
    injected["passed"] = json!(true);
    let results = run(vec![
        open(),
        verify("s1", SOURCE),
        open(),
        injected,
        verify("s2", &SOURCE.replace("n + 1", "false")),
        verify("s2", SOURCE),
    ]);
    assert_eq!(results[2]["code"], "plan.frozen");
    assert_eq!(results[3]["code"], "request.invalid");
    assert_eq!(results[4]["status"], "compile_error");
    assert_eq!(results[4]["diagnostic"]["code"], "compile.type.mismatch");
    assert_eq!(results[5]["status"], "checks_satisfied");
    assert_eq!(results[5]["snapshot"], "s3");
    assert_eq!(results[5]["counts"]["executed"], 0);
}

#[test]
fn environment_changes_unknown_context_and_actions_disable_reuse() {
    let mut changed = verify("s2", SOURCE);
    changed["environment"] = json!("fixture-v2");
    let mut unknown = verify("s2", SOURCE);
    unknown["environment"] = json!("");
    let actionful = SOURCE.replace("I64 { helper(n)", "I64 actions { demo.observe } { helper(n)");
    let mut plan = open();
    plan["sources"][0]["text"] = json!(actionful);
    let results = run(vec![
        plan,
        verify("s1", &actionful),
        verify("s2", &actionful),
        changed,
        unknown,
    ]);
    assert_eq!(results[2]["counts"]["executed"], 1);
    assert_eq!(results[2]["counts"]["reused"], 1);
    for index in [3, 4] {
        assert_eq!(results[index]["counts"]["executed"], 2);
        assert_eq!(results[index]["counts"]["reused"], 0);
    }
}

#[test]
fn expected_runtime_faults_pass_but_resource_exhaustion_and_infrastructure_never_do() {
    let broken = SOURCE.replace("n + 1", "n / 0");
    let mut plan = open();
    plan["sources"][0]["text"] = json!(broken);
    plan["tests"][0]["expect"] = json!({"kind":"fault","code":"runtime.integer.zero_divisor","phase":"execute","subject":"app.helper"});
    let looping = SOURCE.replace("n + 1", "while(state: I64 = 0; true) { state }");
    let results = run(vec![plan, verify("s1", &broken), verify("s2", &looping)]);
    assert_eq!(results[1]["status"], "checks_satisfied");
    assert_eq!(results[2]["status"], "failed");
    assert_eq!(results[2]["failures"][0]["status"], "resource_exhausted");
    let unavailable = run_with_clang(vec![open(), verify("s1", SOURCE)], "/nonexistent/lobo-test-clang");
    assert_eq!(unavailable[1]["status"], "failed");
    assert_eq!(unavailable[1]["failures"][0]["status"], "infrastructure_error");
    assert_eq!(unavailable[1]["counts"]["executed"], 0);
    assert_eq!(unavailable[1]["counts"]["pending"], 1);
}

#[test]
fn contract_only_calls_and_recursive_cycles_are_in_the_dependency_closure() {
    let contract = SOURCE.replace("{ helper(n) }", "requires { guard(n) } { helper(n) }")
        + " function guard @id(\"guard\")(n: I64) -> Bool { true }";
    let mut plan = open();
    plan["sources"][0]["text"] = json!(contract);
    let changed = contract.replace("{ true }", "{ false }");
    let results = run(vec![plan, verify("s1", &contract), verify("s2", &changed)]);
    assert_eq!(results[2]["status"], "failed");
    assert_eq!(results[2]["counts"]["reused"], 1);
    assert_eq!(results[2]["failures"][0]["status"], "runtime_fault");
    let recursive = SOURCE.replace("n + 1", "if n == 0 { 1 } else { helper(n - 1) + 1 }");
    let mut plan = open();
    plan["sources"][0]["text"] = json!(recursive);
    plan["tests"][0]["arguments"] = json!(["I64:3"]);
    plan["tests"][0]["expect"]["value"]["value"] = json!("4");
    let results = run(vec![
        plan,
        verify("s1", &recursive),
        verify("s2", &recursive.replace("{ 1 }", "{ 2 }")),
    ]);
    assert_eq!(results[1]["status"], "checks_satisfied");
    assert_eq!(results[2]["status"], "failed");
    assert_eq!(results[2]["counts"]["reused"], 1);
}

#[test]
fn imported_dependencies_are_checked_before_reuse_and_bad_arguments_do_not_create_snapshots() {
    let root = "module app @id(\"app\"); import helper @id(\"app.helper\"); function inc @id(\"app.inc\")(n: I64) -> I64 { helper(n) } function constant @id(\"app.constant\")() -> I64 { 7 }";
    let library = "module lib @id(\"lib\"); function helper @id(\"app.helper\")(n: I64) -> I64 { n + 1 }";
    let sources = json!([{"name":"app","text":root},{"name":"lib","text":library}]);
    let mut plan = open();
    plan["sources"] = sources.clone();
    let mut first = verify("s1", root);
    first["sources"] = sources.clone();
    let mut changed = verify("s2", root);
    changed["sources"] = sources.clone();
    changed["sources"][1]["text"] = json!(library.replace("n + 1", "n + 2"));
    let mut invalid = verify("s2", root);
    invalid["sources"] = sources.clone();
    invalid["sources"][0]["text"] = json!(root.replace("n: I64", "n: Bool"));
    let mut removed = verify("s2", root);
    removed["sources"] = json!([{"name":"app","text":root}]);
    let results = run(vec![plan, first, changed, invalid, removed]);
    assert_eq!(results[1]["status"], "checks_satisfied");
    assert_eq!(results[2]["status"], "failed");
    assert_eq!(results[2]["counts"]["reused"], 1);
    assert_eq!(results[3]["status"], "compile_error");
    assert_eq!(results[4]["status"], "compile_error");
}

#[test]
fn read_only_explain_pending_replay_zero_budget_and_snapshot_limits_are_explicit() {
    let mut zero = verify("s1", SOURCE);
    zero["max_cases"] = json!(0);
    let mut actions = vec![
        open(),
        zero,
        json!({"op":"replay","snapshot":"s2","test":"inc"}),
        json!({"op":"explain","snapshot":"s2","test":"inc"}),
        json!({"op":"explain","snapshot":"s99","test":"inc"}),
    ];
    for _ in 0..7 {
        actions.push(verify("s1", SOURCE));
    }
    let results = run(actions);
    assert_eq!(results[1]["status"], "incomplete");
    assert_eq!(results[1]["pending_tests"], json!(["inc", "const"]));
    assert_eq!(results[2]["code"], "test.pending");
    assert_eq!(results[3]["result"], Value::Null);
    assert_eq!(results[4]["code"], "snapshot.unknown");
    assert_eq!(results.last().unwrap()["code"], "request.limit");
}

#[test]
fn frozen_checks_reject_duplicate_ids_invalid_expectations_and_zero_test_plans() {
    let mut duplicate = open();
    duplicate["tests"][1]["id"] = json!("inc");
    let mut bad_fault = open();
    bad_fault["tests"][0]["expect"] =
        json!({"kind":"fault","code":"resource.fuel.exhausted","phase":"execute","subject":"app.inc"});
    let mut empty = open();
    empty["tests"] = json!([]);
    let mut bad_args = open();
    bad_args["tests"][0]["arguments"] = json!(["Bool:true"]);
    let results = run(vec![duplicate, bad_fault, empty, bad_args, open()]);
    assert_eq!(results[0]["code"], "plan.test");
    assert_eq!(results[1]["code"], "plan.expectation");
    assert_eq!(results[2]["code"], "plan.limit");
    assert_eq!(results[3]["code"], "test.arguments");
    assert_eq!(results[4]["snapshot"], "s1");
}

#[test]
fn ordinary_lobo_laws_test_real_tree_and_shared_sequence_modules_natively() {
    let mut plan = open();
    plan["root"] = json!("checks");
    let sources = json!([
        {"name":"checks","text":include_str!("../examples/testing/checks.lobo")},
        {"name":"tree","text":include_str!("../examples/tree.lobo")},
        {"name":"sequences","text":include_str!("../examples/sequences.lobo")}
    ]);
    plan["sources"] = sources.clone();
    plan["tests"][0]["entry"] = json!("example.testing.reverse_twice");
    plan["tests"][0]["arguments"] = json!(["List:Count:3,I64:9,I64:1,I64:9"]);
    plan["tests"][1]["entry"] = json!("example.testing.duplicate_count");
    plan["tests"][1]["arguments"] = json!(["I64:0"]);
    for case in plan["tests"].as_array_mut().unwrap() {
        case["expect"]["value"] = json!({"type":"Bool","value":true});
    }
    let mut candidate = verify("s1", SOURCE);
    candidate["root"] = json!("checks");
    candidate["sources"] = sources.clone();
    let result = run(vec![plan, candidate]);
    assert_eq!(result[1]["status"], "checks_satisfied", "{}", result[1]);
    assert_eq!(result[1]["counts"]["executed"], 2);
    let files: Vec<lobo_core::SourceFile> = serde_json::from_value(sources).unwrap();
    let artifact = lobo_core::compile_bundle("checks", &files).unwrap();
    let checked = lobo_core::Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let builds = support::Builds::new(&checked);
    for mode in ["eval", "native", "sanitized"] {
        for (entry, arguments) in [
            (
                "example.testing.reverse_twice",
                vec!["List:Count:3,I64:9,I64:1,I64:9"],
            ),
            ("example.testing.duplicate_count", vec!["I64:0"]),
        ] {
            assert_eq!(
                builds.invoke(mode, entry, &arguments, 1000, 1000)["value"],
                json!({"type":"Bool","value":true}),
                "{mode}: {entry}"
            );
        }
    }
}

#[test]
fn transport_versions_unknown_fields_and_oversized_frames_fail_without_running_code() {
    for (input, code) in [
        (
            format!(
                "{{\"format\":\"lobo-test/0.1\",\"action\":{{\"op\":\"describe\",\"{}\":false}}}}\n",
                "x".repeat(65536)
            ),
            "request.invalid",
        ),
        (
            "{\"format\":\"lobo-test/99\",\"action\":{\"op\":\"describe\"}}\n".to_string(),
            "request.version",
        ),
        (
            "{\"format\":\"lobo-test/0.1\",\"action\":{\"op\":\"describe\",\"shell\":\"do not execute\"}}\n"
                .to_string(),
            "request.invalid",
        ),
        (format!("{}\n", " ".repeat(4 * 1024 * 1024 + 1)), "request.limit"),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lobo-test"))
            .args(["--session", "--clang", "/nonexistent/compiler"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let writer = std::thread::spawn(move || {
            let _ = stdin.write_all(input.as_bytes());
        });
        let output = child.wait_with_output().unwrap();
        writer.join().unwrap();
        assert!(output.status.success());
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["code"], code);
        assert!(output.stdout.len() < 8192, "protocol errors must be compact");
    }
}

#[test]
fn retained_observation_budget_stops_without_silently_discarding_evidence() {
    let mut plan = open();
    plan["requirements"] = json!([{"id":"constant","description":"Constant is seven"}]);
    let template = plan["tests"][1].clone();
    plan["tests"] = Value::Array(
        (0..100)
            .map(|n| {
                let mut case = template.clone();
                case["id"] = json!(format!("case.{n}"));
                case
            })
            .collect(),
    );
    let changed = SOURCE.replace(
        "function constant @id(\"app.constant\")() -> I64 { 7 }",
        &format!(
            "function constant @id(\"app.constant\")() -> Text {{ \"{}\" }}",
            "a".repeat(60000)
        ),
    );
    let results = run(vec![
        plan,
        verify("s1", &changed),
        json!({"op":"explain","snapshot":"s2","test":"case.69"}),
    ]);
    assert_eq!(results[1]["status"], "failed");
    assert!(results[1]["counts"]["pending"].as_u64().unwrap() > 0);
    assert_eq!(results[2]["result"]["status"], "infrastructure_error");
    assert_eq!(results[2]["result"]["observed"]["code"], "tool.observation_limit");
}
