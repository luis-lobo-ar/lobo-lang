use lobo_core::{compile, Artifact};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn source(body: &str, output: &str) -> String {
    format!("module m @id(\"m\"); function main @id(\"main\")() -> {output} {{ {body} }}")
}
fn builds(source: &str) -> Builds {
    let artifact = compile(source).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    Builds::new(&artifact)
}

#[test]
fn false_condition_returns_initial_without_body_and_has_exact_fuel_cost() {
    let program = builds(&source("while (n: I64 = 41; false) { 1 / 0 }", "I64"));
    for mode in ["eval", "native", "sanitized"] {
        let result = program.invoke(mode, "main", &[], 5, 0);
        assert_eq!(
            result["value"],
            json!({"type":"I64","value":"41"}),
            "{mode}: {result}"
        );
        let result = program.invoke(mode, "main", &[], 4, 0);
        assert_eq!(
            result["diagnostic"]["code"], "resource.fuel.exhausted",
            "{mode}: {result}"
        );
    }
}

#[test]
fn carried_values_nested_loops_and_fresh_body_bindings_compose_without_stack_growth() {
    for (body, output, expected) in [
        (
            "while (n: I64 = 0; n < 1000) { let next = n + 1; next }",
            "I64",
            json!({"type":"I64","value":"1000"}),
        ),
        (
            "while (flag: Bool = false; !flag) { true }",
            "Bool",
            json!({"type":"Bool","value":true}),
        ),
        (
            "while(a: I64 = 0; a < 3) { fold(i in 2, b: I64 = a) { b + 1 } }",
            "I64",
            json!({"type":"I64","value":"4"}),
        ),
        (
            "while(a: I64 = 0; a < 3) { while(b: I64 = a; b <= a) { b + 1 } }",
            "I64",
            json!({"type":"I64","value":"3"}),
        ),
    ] {
        let program = builds(&source(body, output));
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(
                program.invoke(mode, "main", &[], 10000, 0)["value"],
                expected,
                "{mode}: {body}"
            );
        }
    }
}

#[test]
fn all_loop_children_are_type_scope_and_action_checked_even_when_false() {
    for (body, code) in [
        ("while(n: I64 = true; false) { n }", "compile.type.mismatch"),
        ("while(n: I64 = 0; 1) { n }", "compile.type.mismatch"),
        ("while(n: I64 = 0; false) { true }", "compile.type.mismatch"),
        ("while(n: I64 = n; false) { n }", "compile.name.unresolved"),
        (
            "let n = 1; while(n: I64 = 0; false) { n }",
            "compile.name.duplicate",
        ),
        ("while(main: I64 = 0; false) { main }", "compile.name.duplicate"),
        (
            "while(result: I64 = 0; false) { result }",
            "compile.name.reserved",
        ),
        (
            "while(n: I64 = 0; temporary > 0) { let temporary = 1; n }",
            "compile.name.unresolved",
        ),
        (
            "let done = while(n: I64 = 0; false) { n }; n",
            "compile.name.unresolved",
        ),
        (
            "while(n: I64 = 0; false) { let n = 1; n }",
            "compile.name.duplicate",
        ),
    ] {
        assert_eq!(compile(&source(body, "I64")).unwrap_err().code, code, "{body}");
    }
    let leaf = "function leaf @id(\"leaf\")() -> I64 actions { demo.observe } { 1 }";
    for body in [
        "while(n: I64 = leaf(); false) { n }",
        "while(n: I64 = 0; false && leaf() > 0) { n }",
        "while(n: I64 = 0; false) { leaf() }",
    ] {
        let text = format!("{} {leaf}", source(body, "I64"));
        assert_eq!(compile(&text).unwrap_err().code, "compile.action.undeclared");
        let declared = text.replacen("-> I64 {", "-> I64 actions { demo.observe } {", 1);
        compile(&declared).unwrap();
        let contract = format!("module m @id(\"m\"); function main @id(\"main\")() -> I64 requires {{ {body} == 0 }} {{ 0 }} {leaf}");
        assert_eq!(compile(&contract).unwrap_err().code, "compile.action.contract");
    }
}

#[test]
fn initializer_condition_and_body_faults_keep_order_and_contract_phases() {
    for (body, code) in [
        (
            "while(n: I64 = 1 / 0; true) { 9223372036854775807 + 1 }",
            "runtime.integer.zero_divisor",
        ),
        (
            "while(n: I64 = 0; 1 / 0 == 0) { 9223372036854775807 + 1 }",
            "runtime.integer.zero_divisor",
        ),
        (
            "while(n: I64 = 0; true) { 9223372036854775807 + 1 }",
            "runtime.integer.overflow",
        ),
        (
            "while(n: I64 = 0; 1 / (1 - n) > 0) { n + 1 }",
            "runtime.integer.zero_divisor",
        ),
    ] {
        let program = builds(&source(body, "I64"));
        for mode in ["eval", "native", "sanitized"] {
            let result = program.invoke(mode, "main", &[], 1000, 0);
            assert_eq!(result["diagnostic"]["code"], code, "{mode}: {result}");
            assert_eq!(result["diagnostic"]["phase"], "execute");
            assert!(result.get("value").is_none());
        }
    }
    for clause in ["requires", "ensures"] {
        let text=format!("module m @id(\"m\"); function main @id(\"main\")() -> I64 {clause} {{ while(n: I64 = 0; n < 1) {{ helper() }} == 1 }} {{ 1 }} function helper @id(\"helper\")() -> I64 {{ 1 / 0 }}");
        let program = builds(&text);
        for mode in ["eval", "native", "sanitized"] {
            let result = program.invoke(mode, "main", &[], 1000, 0);
            assert_eq!(result["diagnostic"]["code"], "runtime.integer.zero_divisor");
            assert_eq!(
                result["diagnostic"]["phase"],
                if clause == "requires" {
                    "precondition"
                } else {
                    "postcondition"
                }
            );
            assert_eq!(result["diagnostic"]["subject"], "helper");
        }
    }
}

#[test]
fn nontermination_and_allocation_spend_shared_budgets_without_partial_states() {
    let forever = builds(&source("while(n: I64 = 0; true) { n }", "I64"));
    let list = builds(&source(
        "while(xs: List<I64> = List::empty<I64>(); List::length(xs) < 10) { List::prepend(1,xs) }",
        "List<I64>",
    ));
    let once = builds(&source(
        "while(xs: List<I64> = List::prepend(1,List::empty<I64>()); false) { List::prepend(2,xs) }",
        "List<I64>",
    ));
    let condition=builds("module m @id(\"m\"); function main @id(\"main\")() -> I64 { while(n: I64 = 0; probe(n)) { n + 1 } } function probe @id(\"probe\")(n: I64) -> Bool { List::length(List::prepend(1,List::empty<I64>())) > n }");
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            forever.invoke(mode, "main", &[], 1000, 0)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            list.invoke(mode, "main", &[], 1000, 40)["value"]["value"]["elements"]
                .as_array()
                .unwrap()
                .len(),
            10
        );
        assert_eq!(
            list.invoke(mode, "main", &[], 1000, 39)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
        assert_eq!(
            once.invoke(mode, "main", &[], 1000, 4)["value"]["value"]["elements"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            condition.invoke(mode, "main", &[], 1000, 8)["value"]["value"],
            "1"
        );
        assert_eq!(
            condition.invoke(mode, "main", &[], 1000, 7)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
    }
}

#[test]
fn contextual_names_and_precise_ranges_match_the_committed_prefix() {
    let text="module m @id(\"m\"); function main @id(\"main\")() -> I64 { while(41) } function while @id(\"while\")(value: I64) -> I64 { value + 1 }";
    let program = builds(text);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(program.invoke(mode, "main", &[], 1000, 0)["value"]["value"], "42");
    }
    for body in [
        "while(n: I64 = 0, false) { n }",
        "while(n: I64 = 0; false { n }",
        "while(n: I64 = 0; false)",
    ] {
        assert_eq!(
            compile(&source(body, "I64")).unwrap_err().code,
            "compile.syntax.invalid"
        );
    }
    for (body, blame) in [
        ("while(n: I64 = 0; 42) { n }", "42"),
        ("while(n: I64 = 0; false) { true }", "true"),
    ] {
        let text = format!("// 🐺\r\n{}", source(body, "I64"));
        let error = compile(&text).unwrap_err();
        let range = error.range.unwrap();
        assert_eq!(&text[range.start.byte..range.end.byte], blame);
        assert_eq!(range.start.line, 1);
        assert_eq!(
            range.start.character,
            text["// 🐺\r\n".len()..range.start.byte].encode_utf16().count()
        );
    }
}

#[test]
fn checked_loops_are_deterministic_and_reject_forged_children() {
    let text = source("while(n: I64 = 0; n < 3) { n + 1 }", "I64");
    let encoded = compile(&text).unwrap().to_json().unwrap();
    assert_eq!(
        encoded,
        compile(&format!("// shift\n{text}")).unwrap().to_json().unwrap()
    );
    let original: Json = serde_json::from_str(&encoded).unwrap();
    for (path, value, code) in [
        (
            "condition",
            json!({"kind":"Literal","value":{"type":"I64","value":"1"}}),
            "compile.type.mismatch",
        ),
        (
            "initial",
            json!({"kind":"Name","name":"n"}),
            "compile.name.unresolved",
        ),
    ] {
        let mut data = original.clone();
        data["program"]["functions"][0]["body"]["value"][path] = value;
        assert_eq!(Artifact::from_json(&data.to_string()).unwrap_err().code, code);
    }
    let mut data = original.clone();
    data["program"]["functions"][0]["body"]["value"]["state"]["name"] = "main".into();
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "compile.name.duplicate"
    );
    let mut data = original.clone();
    data["program"]["functions"][0]["body"]["value"]["extra"] = true.into();
    assert_eq!(
        Artifact::from_json(&data.to_string()).unwrap_err().code,
        "invocation.artifact.invalid"
    );
}

#[test]
fn deeply_nested_source_loops_fail_with_a_diagnostic_not_a_compiler_stack_overflow() {
    let mut body = "0".to_string();
    for i in 0..40 {
        body = format!("while(n{i}: I64 = 0; false) {{ {body} }}");
    }
    assert_eq!(
        compile(&source(&body, "I64")).unwrap_err().code,
        "resource.frontend.limit"
    );
}

#[test]
fn supported_nested_loops_round_trip_and_old_versions_are_rejected() {
    let mut body = "0".to_string();
    for i in 0..24 {
        body = format!("while(n{i}: I64 = 0; false) {{ {body} }}");
    }
    let artifact = compile(&source(&body, "I64")).unwrap();
    let encoded = artifact.to_json().unwrap();
    let loaded = Artifact::from_json(&encoded).unwrap();
    assert_eq!(
        serde_json::to_value(loaded.run("main", vec![], 5).unwrap()).unwrap(),
        json!({"type":"I64","value":"0"})
    );
    let mut old: Json = serde_json::from_str(&encoded).unwrap();
    old["format"] = "lobo-checked/0.7".into();
    old["language"] = "core/0.7-draft".into();
    assert_eq!(
        Artifact::from_json(&old.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
}

#[test]
fn early_match_uses_the_same_execution_fuel_with_a_short_or_long_unused_tail() {
    let text = std::fs::read_to_string("examples/state_iteration.lobo").unwrap();
    let program = builds(&text);
    let short = "List:Count:1,I64:7".to_string();
    let long = format!("List:Count:1001,I64:7{}", ",I64:0".repeat(1000));
    for mode in ["eval", "native", "sanitized"] {
        for input in [&short, &long] {
            let result = program.invoke(
                mode,
                "example.state_iteration.find",
                &[input, "I64:7"],
                100,
                10000,
            );
            assert_eq!(result["value"]["value"], "0", "{mode}: {result}");
        }
        for (input, target, expected) in [
            ("List:Count:3,I64:2,I64:4,I64:7", "I64:7", "2"),
            ("List:Count:0", "I64:7", "-1"),
        ] {
            assert_eq!(
                program.invoke(mode, "example.state_iteration.find", &[input, target], 1000, 1000)["value"]
                    ["value"],
                expected
            );
        }
        assert_eq!(
            program.invoke(
                mode,
                "example.state_iteration.gcd",
                &["I64:1071", "I64:462"],
                1000,
                0
            )["value"]["value"],
            "21"
        );
    }
}

#[test]
fn variant_buffer_and_body_only_list_types_have_native_layouts_and_correct_lifetimes() {
    let text="module m @id(\"m\"); variant Step @id(\"step\") { More(I64), Done(Text) }
      function main @id(\"main\")() -> Step { while(s: Step = Step::More(0); match(s) { More(n) { n < 2 }, Done(t) { false } }) { match(s) { More(n) { if n == 1 { Step::Done(\"🐺\") } else { Step::More(n+1) } }, Done(t) { Step::Done(t) } } } }
      function text @id(\"text\")() -> Text { while(t: Text = \"\"; Text::length(t) < 3) { Text::concat(t,\"🐺\") } }
      function local @id(\"local\")() -> I64 { while(n: I64 = 0; n < 1) { let xs: List<List<I64>> = List::empty<List<I64>>(); n + List::length(xs) + 1 } }";
    let program = builds(text);
    for mode in ["eval", "native", "sanitized"] {
        let value = program.invoke(mode, "main", &[], 1000, 0)["value"].clone();
        assert_eq!(value["value"]["case"], "Done");
        assert_eq!(value["value"]["payload"]["value"], "🐺");
        assert_eq!(
            program.invoke(mode, "text", &[], 1000, 13)["value"]["value"],
            "🐺🐺🐺"
        );
        assert_eq!(
            program.invoke(mode, "text", &[], 1000, 12)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
        assert_eq!(program.invoke(mode, "local", &[], 1000, 0)["value"]["value"], "1");
    }
}

#[test]
fn stack_bounded_expression_parsing_preserves_precedence_associativity_and_rejections() {
    for (a, b, ty) in [
        ("20 - 6 / 2 * 3 + 1", "(20 - ((6 / 2) * 3)) + 1", "I64"),
        (
            "1 < 2 == 3 < 4 && true || false",
            "((1 < 2) == (3 < 4)) && true || false",
            "Bool",
        ),
    ] {
        assert_eq!(
            compile(&source(a, ty)).unwrap().to_json().unwrap(),
            compile(&source(b, ty)).unwrap().to_json().unwrap()
        );
    }
    for body in [
        "1 < 2 < 3",
        "1 == 1 == true",
        "1 < 2 == true == false",
        "1 == 2 < 3 < 4",
    ] {
        assert_eq!(
            compile(&source(body, "Bool")).unwrap_err().code,
            "compile.syntax.invalid"
        );
    }
}
