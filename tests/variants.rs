use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};

const DECL: &str = "variant Answer @id(\"answer:1\") { Ok(I64), Zero, Overflow }";
const DIVIDE: &str = "function divide @id(\"divide\")(a: I64, b: I64) -> Answer {
  if b == 0 { Answer::Zero } else {
    if a == -9223372036854775808 && b == -1 { Answer::Overflow } else { Answer::Ok(a / b) }
  }
}";
const HANDLE: &str = "function handle @id(\"handle\")(a: Answer) -> I64 {
  match (a) { Zero { 0 }, Ok(n) { n }, Overflow { -1 } }
}";

fn program(body: &str, output: &str) -> String {
    format!("module m @id(\"m\"); {DECL} function main @id(\"main\")() -> {output} {{ {body} }}")
}

#[test]
fn typed_failures_and_exhaustive_handling_return_known_values() {
    let source = format!("module m @id(\"m\"); {DIVIDE} {HANDLE} {DECL}");
    let artifact = compile(&source).unwrap();
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    for (a, b, case, payload, handled) in [
        (20, 4, "Ok", Some(5), 5),
        (1, 0, "Zero", None, 0),
        (i64::MIN, -1, "Overflow", None, -1),
    ] {
        let expected = json!({"type":"Variant","value":{"id":"answer:1","case":case,
            "payload":payload.map(|n| json!({"type":"I64","value":n.to_string()}))}});
        let value = loaded
            .run("divide", vec![Value::I64(a), Value::I64(b)], 1000)
            .unwrap();
        assert_eq!(serde_json::to_value(&value).unwrap(), expected);
        assert_eq!(
            loaded.run("handle", vec![value], 1000).unwrap(),
            Value::I64(handled)
        );
        let output = loaded
            .run_native(&[
                "--entry".into(),
                "divide".into(),
                "--arg".into(),
                format!("I64:{a}"),
                "--arg".into(),
                format!("I64:{b}"),
            ])
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["value"], expected);
    }
}

#[test]
fn matches_reject_missing_duplicate_unknown_cases_and_wrong_bindings() {
    for (body, code) in [
        (
            "match (Answer::Ok(1)) { Ok(n) { n }, Zero { 0 } }",
            "compile.match.invalid",
        ),
        (
            "match (Answer::Zero) { Zero { 0 }, Zero { 1 }, Ok(n) { n }, Overflow { 0 } }",
            "compile.match.invalid",
        ),
        (
            "match (Answer::Zero) { Missing { 0 }, Ok(n) { n }, Overflow { 0 } }",
            "compile.match.invalid",
        ),
        ("match (1) { Zero { 0 } }", "compile.match.invalid"),
        (
            "match (Answer::Zero) { Zero(n) { n }, Ok(n) { n }, Overflow { 0 } }",
            "compile.match.invalid",
        ),
        (
            "match (Answer::Zero) { Zero { 0 }, Ok { 1 }, Overflow { 0 } }",
            "compile.match.invalid",
        ),
        (
            "match (Answer::Zero) { Zero { true }, Ok(n) { n }, Overflow { 0 } }",
            "compile.type.mismatch",
        ),
        ("Answer::Unknown", "compile.variant.case"),
        ("Answer::Ok", "compile.variant.case"),
        ("Answer::Zero(1)", "compile.variant.case"),
        ("Answer::Ok(true)", "compile.type.mismatch"),
    ] {
        assert_eq!(compile(&program(body, "I64")).unwrap_err().code, code, "{body}");
    }
}

#[test]
fn match_evaluates_only_selected_arm_and_bindings_do_not_escape() {
    let source = program(
        "match (Answer::Ok(42)) { Ok(n) { n }, Zero { 1 / 0 }, Overflow { 9223372036854775807 + 1 } }",
        "I64",
    );
    let artifact = compile(&source).unwrap();
    assert_eq!(artifact.run("main", vec![], 100).unwrap(), Value::I64(42));
    let output = artifact.run_native(&["--entry".into(), "main".into()]).unwrap();
    assert!(output.status.success(), "{output:?}");
    let source = program(
        "let x = match (Answer::Ok(1)) { Ok(n) { n }, Zero { 0 }, Overflow { 0 } }; n",
        "I64",
    );
    assert_eq!(compile(&source).unwrap_err().code, "compile.name.unresolved");
}

#[test]
fn nested_alternatives_records_folds_and_equality_compose() {
    let source = "module m @id(\"m\");
      record Boxed @id(\"box\") { answer: Choice, flag: Bool }
      variant Outer @id(\"outer\") { Box(Boxed), Empty }
      variant Choice @id(\"choice\") { Number(I64), Flag(Bool), Empty }
      function step @id(\"step\")(x: Outer) -> Outer ensures { result == x } { x }
      function main @id(\"main\")() -> I64 {
        let x = Outer::Box(Boxed { flag: true, answer: Choice::Number(7) });
        let y: Outer = fold(i in 3, state: Outer = x) { step(state) };
        if x == y && Choice::Number(7) != Choice::Flag(true) {
          match (y) { Empty { -1 }, Box(b) {
            match (b.answer) { Empty { 0 }, Flag(flag) { if flag { 1 } else { 2 } }, Number(n) { n } }
          } }
        } else { -2 }
      }";
    let artifact = compile(source).unwrap();
    assert_eq!(artifact.run("main", vec![], 1000).unwrap(), Value::I64(7));
    let out = artifact.run_native(&["--entry".into(), "main".into()]).unwrap();
    assert!(out.status.success(), "{out:?}");
    let out: Json = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out["value"], json!({"type":"I64","value":"7"}));
}

#[test]
fn alternate_arms_obey_actions_contract_purity_and_no_shadowing() {
    let source = format!(
        "module m @id(\"m\"); {DECL}
      function marked @id(\"marked\")() -> I64 actions {{ demo.observe }} {{ 1 }}
      function main @id(\"main\")() -> I64 {{
        match (Answer::Zero) {{ Ok(n) {{ marked() }}, Zero {{ 0 }}, Overflow {{ 0 }} }}
      }}"
    );
    assert_eq!(compile(&source).unwrap_err().code, "compile.action.undeclared");
    let permitted = source.replace("main\")() -> I64", "main\")() -> I64 actions { demo.observe }");
    assert!(compile(&permitted).is_ok());
    let contract = permitted.replace("actions { demo.observe } {\n        match", "actions { demo.observe } ensures { match (Answer::Zero) { Ok(n) { marked() == 1 }, Zero { true }, Overflow { true } } } {\n        match");
    assert_eq!(compile(&contract).unwrap_err().code, "compile.action.contract");
    let shadow = program(
        "let n = 1; match (Answer::Ok(2)) { Ok(n) { n }, Zero { 0 }, Overflow { 0 } }",
        "I64",
    );
    assert_eq!(compile(&shadow).unwrap_err().code, "compile.name.duplicate");
}

#[test]
fn artifacts_recheck_alternatives_nominal_kinds_and_patterns() {
    let source = program(
        "match (Answer::Ok(1)) { Ok(n) { n }, Zero { 0 }, Overflow { 0 } }",
        "I64",
    );
    let data: Json = serde_json::from_str(&compile(&source).unwrap().to_json().unwrap()).unwrap();
    for (pointer, value, code) in [
        (
            "/program/variants/0/cases/1/name",
            json!("Ok"),
            "compile.variant.case",
        ),
        (
            "/program/variants/0/cases/0/payload",
            json!({"Variant":"answer:1"}),
            "compile.record.cycle",
        ),
        (
            "/program/variants/0/cases/0/payload",
            json!({"Record":"answer:1"}),
            "compile.type.unknown",
        ),
        (
            "/program/functions/0/body/value/subject/variant",
            json!("Answer"),
            "compile.type.unknown",
        ),
        (
            "/program/functions/0/body/value/arms/0/binding",
            Json::Null,
            "compile.match.invalid",
        ),
        (
            "/program/functions/0/body/value/arms/1/case",
            json!("Ok"),
            "compile.match.invalid",
        ),
        (
            "/program/functions/0/body/value/subject",
            json!({"kind":"Literal","value":{"type":"Variant","value":{"id":"answer:1","case":"Zero","payload":null}}}),
            "compile.syntax.invalid",
        ),
    ] {
        let mut forged = data.clone();
        *forged.pointer_mut(pointer).unwrap() = value;
        let error = Artifact::from_json(&forged.to_string()).unwrap_err();
        assert_eq!(error.code, code, "{pointer}");
        assert!(error.range.is_none());
    }
    let mixed = "module m @id(\"m\"); record R @id(\"r\") { x: V } variant V @id(\"v\") { Next(R), End } function main @id(\"main\")() -> I64 { 0 }";
    assert_eq!(compile(mixed).unwrap_err().code, "compile.record.cycle");
    let distinct = format!("module m @id(\"m\"); {DECL} variant Other @id(\"other\") {{ Ok(I64), Zero, Overflow }} function main @id(\"main\")() -> Bool {{ Answer::Zero == Other::Zero }}");
    assert_eq!(compile(&distinct).unwrap_err().code, "compile.type.mismatch");
}

struct Builds(std::path::PathBuf);
impl Builds {
    fn new(artifact: &Artifact) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lobo-variants-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let build = Self(root);
        std::fs::write(build.0.join("program.c"), artifact.emit_c().unwrap()).unwrap();
        std::fs::write(build.0.join("program.loboc"), artifact.to_json().unwrap()).unwrap();
        artifact.compile_native(&build.0.join("native")).unwrap();
        let output = std::process::Command::new("clang")
            .args([
                "-std=c11",
                "-O1",
                "-g",
                "-fsanitize=address,undefined",
                "-fno-sanitize-recover=all",
            ])
            .arg(build.0.join("program.c"))
            .arg("-o")
            .arg(build.0.join("sanitized"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        build
    }
    fn invoke(&self, mode: &str, entry: &str, argument: Option<&str>, fuel: u64) -> Json {
        let mut command = if mode == "eval" {
            let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_lobo"));
            c.arg("eval").arg(self.0.join("program.loboc"));
            c
        } else {
            std::process::Command::new(self.0.join(mode))
        };
        command
            .env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1:symbolize=0")
            .env("UBSAN_OPTIONS", "halt_on_error=1")
            .args(["--entry", entry, "--fuel", &fuel.to_string()]);
        if let Some(arg) = argument {
            command.args(["--arg", arg]);
        }
        let output = command.output().unwrap();
        assert!(output.stderr.is_empty(), "{mode}: {output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            output.status.success(),
            result["status"] == "ok",
            "{mode}: {result}"
        );
        result
    }
}
impl Drop for Builds {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Variant reproducer: {}", self.0.display());
            return;
        }
        for name in ["program.c", "program.loboc", "native", "sanitized"] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

#[test]
fn variant_inputs_outputs_and_fuel_agree_in_all_three_execution_modes() {
    let text = "module m @id(\"m\");
      variant Inner @id(\"inner\") { None, Number(I64) }
      record Pair @id(\"pair\") { first: Inner, second: Bool }
      variant Outer @id(\"outer:v1\") { None, Item(Pair), Flag(Bool) }
      function echo @id(\"echo\")(o: Outer) -> Outer { o }
      function record_echo @id(\"record.echo\")(p: Pair) -> Pair { p }
      function fuel @id(\"fuel\")() -> I64 { match (Inner::Number(42)) { None { 1 / 0 }, Number(n) { n } } }";
    let artifact = compile(text).unwrap();
    let builds = Builds::new(&artifact);
    let none = json!({"type":"Variant","value":{"id":"inner","case":"None","payload":null}});
    let pair = json!({"type":"Record","value":{"id":"pair","fields":{"first":none,"second":{"type":"Bool","value":true}}}});
    for (entry, argument, expected) in [
        (
            "echo",
            "Variant:outer:v1:Case:None",
            json!({"type":"Variant","value":{"id":"outer:v1","case":"None","payload":null}}),
        ),
        (
            "echo",
            "Variant:outer:v1:Case:Item,Case:None,Bool:true",
            json!({"type":"Variant","value":{"id":"outer:v1","case":"Item","payload":pair}}),
        ),
        (
            "echo",
            "Variant:outer:v1:Case:Flag,Bool:false",
            json!({"type":"Variant","value":{"id":"outer:v1","case":"Flag","payload":{"type":"Bool","value":false}}}),
        ),
        ("record.echo", "Record:pair:Case:None,Bool:true", pair),
    ] {
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(builds.invoke(mode, entry, Some(argument), 100)["value"], expected);
        }
    }
    for argument in [
        "Variant:outer:v1:",
        "Variant:outer:v1:Case:Missing",
        "Variant:outer:v1:Case:None,I64:1",
        "Variant:outer:v1:Case:None,",
        "Variant:outer:v1:Case:Item",
        "Variant:outer:v1:Case:Flag,I64:1",
        "Variant:outer:v1:Case:Item,Case:Number,I64:1",
        "Variant:outer:v1:Case:Item,Case:None,Bool:true,",
        "Variant:outer:v1:Case:Item,Case:Number,I64:9223372036854775808,Bool:true",
        "Variant:outer:Case:None",
        "Record:outer:v1:Case:None",
        "Variant:outer:v1:Case:Item,Case:None,Bool:true,Case:None",
    ] {
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(
                builds.invoke(mode, "echo", Some(argument), 0)["diagnostic"]["code"],
                "invocation.arguments.invalid",
                "{argument}"
            );
        }
    }
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "fuel", None, 5)["value"],
            json!({"type":"I64","value":"42"})
        );
        assert_eq!(
            builds.invoke(mode, "fuel", None, 4)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
    }
    for forged in [
        json!({"type":"Variant","value":{"id":"outer:v1","case":"None","payload":{"type":"I64","value":"1"}}}),
        json!({"type":"Variant","value":{"id":"outer:v1","case":"Item","payload":null}}),
        json!({"type":"Variant","value":{"id":"outer:v1","case":"Flag","payload":{"type":"I64","value":"1"}}}),
        json!({"type":"Variant","value":{"id":"other","case":"None","payload":null}}),
    ] {
        let value: Value = serde_json::from_value(forged).unwrap();
        assert_eq!(
            artifact.run("echo", vec![value], 0).unwrap_err().code,
            "invocation.arguments.invalid"
        );
    }
}

#[test]
fn executable_example_and_contextual_names_work_in_native_and_reference_modes() {
    let artifact = compile(include_str!("../examples/variants.lobo")).unwrap();
    let builds = Builds::new(&artifact);
    for (a, b, case, expected) in [
        (20, 4, "Ok", Some("5")),
        (1, 0, "ZeroDivisor", None),
        (i64::MIN, -1, "Overflow", None),
    ] {
        let result = artifact
            .run(
                "example.variants.divide",
                vec![Value::I64(a), Value::I64(b)],
                1000,
            )
            .unwrap();
        let expected = json!({"type":"Variant","value":{"id":"example.variants.quotient","case":case,"payload":expected.map(|n| json!({"type":"I64","value":n}))}});
        assert_eq!(serde_json::to_value(&result).unwrap(), expected);
        for mode in ["native", "sanitized"] {
            let output = std::process::Command::new(builds.0.join(mode))
                .env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1:symbolize=0")
                .args([
                    "--entry",
                    "example.variants.divide",
                    "--arg",
                    &format!("I64:{a}"),
                    "--arg",
                    &format!("I64:{b}"),
                ])
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let result: Json = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(result["value"], expected);
        }
    }
    let ordinary = "module m @id(\"m\"); function match @id(\"match\")(variant: I64) -> I64 { variant } function main @id(\"main\")() -> I64 { match(42) }";
    let artifact = compile(ordinary).unwrap();
    assert_eq!(artifact.run("main", vec![], 100).unwrap(), Value::I64(42));
    assert!(artifact
        .run_native(&["--entry".into(), "main".into()])
        .unwrap()
        .status
        .success());
    let condition = "module m @id(\"m\"); function match @id(\"match\")(flag: Bool) -> Bool { flag } function main @id(\"main\")() -> I64 { if (match(true)) { 42 } else { 0 } }";
    let artifact = compile(condition).unwrap();
    assert_eq!(artifact.run("main", vec![], 100).unwrap(), Value::I64(42));
}

#[test]
fn nominal_classification_rejects_deep_unchecked_trees_without_stack_overflow() {
    let source = format!(
        "module m @id(\"m\"); function f @id(\"f\")() -> I64 {{ 0{} }}",
        ".x".repeat(3900)
    );
    assert_eq!(compile(&source).unwrap_err().code, "resource.frontend.limit");
    let mut expression = "0".to_owned();
    for _ in 0..20 {
        expression = format!("{}({expression})", "0 + ".repeat(30));
    }
    let source = program(&expression, "I64");
    assert_eq!(compile(&source).unwrap_err().code, "resource.frontend.limit");
}

#[test]
fn alternative_layout_budgets_and_shared_helpers_bound_static_expansion() {
    let finish = "function main @id(\"main\")() -> I64 { 0 }";
    let declarations = (0..9)
        .map(|i| {
            format!(
                "variant V{i} @id(\"v{i}\") {{ {} }}",
                (0..16)
                    .map(|c| format!(
                        "C{c}({})",
                        if i == 0 {
                            "I64".into()
                        } else {
                            format!("V{}", i - 1)
                        }
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>();
    let source = format!("module m @id(\"m\"); {} {finish}", declarations[..8].join(" "));
    let artifact = compile(&source).unwrap();
    let c = artifact.emit_c().unwrap();
    assert!(
        c.len() < 200_000,
        "shared helpers must not recursively duplicate all possible paths: {} bytes",
        c.len()
    );
    assert_eq!(
        compile(&format!(
            "module m @id(\"m\"); {} {finish}",
            declarations.join(" ")
        ))
        .unwrap_err()
        .code,
        "resource.frontend.limit"
    );
    let fields = (0..8).map(|i| format!("f{i}: I64")).collect::<Vec<_>>().join(",");
    let big = (0..8)
        .map(|i| format!("f{i}: Small"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("module m @id(\"m\"); record Small @id(\"small\") {{ {fields} }} record Big @id(\"big\") {{ {big} }} variant V @id(\"v\") {{ Payload(Big), Empty }} {finish}");
    assert_eq!(compile(&source).unwrap_err().code, "resource.frontend.limit");
    let excess = format!(
        "module m @id(\"m\"); variant V @id(\"v\") {{ {} }} {finish}",
        (0..17).map(|i| format!("C{i}")).collect::<Vec<_>>().join(",")
    );
    assert_eq!(compile(&excess).unwrap_err().code, "resource.frontend.limit");
}
