use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};

const PAIR: &str = "record Pair @id(\"example.pair\") { left: I64, right: I64 }";

#[test]
fn forward_function_signatures_are_validated_before_record_use() {
    let source = "module m @id(\"m\"); function main @id(\"main\")() -> I64 { bad().x } function bad @id(\"bad\")() -> Missing { 0 }";
    assert_eq!(compile(source).unwrap_err().code, "compile.type.unknown");
    let text = program("Pair { left: true, right: 2 }", "Pair");
    let range = compile(&text).unwrap_err().range.unwrap();
    assert_eq!(&text[range.start.byte..range.end.byte], "true");
    let text =
        "module m @id(\"m\"); record R @id(\"r\") { x: Missing } function main @id(\"main\")() -> I64 { 0 }";
    assert!(compile(text).unwrap_err().range.is_some());
}

#[test]
fn artifact_reload_rechecks_records_types_fields_and_nominal_identity() {
    let text = program("Pair { left: 1, right: 2 }", "Pair");
    let artifact = compile(&text).unwrap();
    let original: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(original["format"], "lobo-checked/1.0");
    assert_eq!(
        original["program"]["functions"][0]["output"],
        json!({"Record":"example.pair"})
    );
    assert_eq!(
        artifact.to_json().unwrap(),
        compile(&format!("// comment\n{text}"))
            .unwrap()
            .to_json()
            .unwrap()
    );
    for (pointer, value, code) in [
        (
            "/program/records/0/fields/0/ty",
            json!({"Record":"missing"}),
            "compile.type.unknown",
        ),
        (
            "/program/records/0/fields/0/ty",
            json!({"Record":"example.pair"}),
            "compile.record.cycle",
        ),
        (
            "/program/functions/0/body/value/record",
            json!("Pair"),
            "compile.type.unknown",
        ),
        (
            "/program/functions/0/body/value/fields/1/name",
            json!("left"),
            "compile.record.fields",
        ),
        (
            "/program/functions/0/body/value",
            json!({"kind":"Literal","value":{"type":"Record","value":{"id":"example.pair","fields":{}}}}),
            "compile.syntax.invalid",
        ),
    ] {
        let mut data = original.clone();
        *data.pointer_mut(pointer).unwrap() = value;
        let error = Artifact::from_json(&data.to_string()).unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.range.is_none());
    }
    let text = format!("module m @id(\"m\"); {PAIR} record Other @id(\"other\") {{ left: I64, right: I64 }} function main @id(\"main\")() -> Pair {{ Other {{ left: 1, right: 2 }} }}");
    assert_eq!(compile(&text).unwrap_err().code, "compile.type.mismatch");
}

#[test]
fn layout_cycles_expansion_and_copy_budgets_are_rejected_before_execution() {
    let finish = "function main @id(\"main\")() -> I64 { 0 }";
    let declarations = (0..9)
        .map(|i| {
            format!(
                "record R{i} @id(\"r{i}\") {{ x: {} }}",
                if i == 0 {
                    "I64".into()
                } else {
                    format!("R{}", i - 1)
                }
            )
        })
        .collect::<Vec<_>>();
    assert!(compile(&format!(
        "module m @id(\"m\"); {} {finish}",
        declarations[..8].join(" ")
    ))
    .is_ok());
    assert_eq!(
        compile(&format!(
            "module m @id(\"m\"); {} {finish}",
            declarations.join(" ")
        ))
        .unwrap_err()
        .code,
        "resource.frontend.limit"
    );
    let eight = (0..8).map(|i| format!("f{i}: I64")).collect::<Vec<_>>().join(",");
    let sixty_four = (0..8)
        .map(|i| format!("f{i}: Small"))
        .collect::<Vec<_>>()
        .join(",");
    let prefix = format!("module m @id(\"m\"); record Small @id(\"small\") {{ {eight} }} record Big @id(\"big\") {{ {sixty_four} }}");
    assert!(compile(&format!("{prefix} {finish}")).is_ok());
    assert_eq!(
        compile(&format!(
            "{prefix} record Huge @id(\"huge\") {{ a: Big, b: Big }} {finish}"
        ))
        .unwrap_err()
        .code,
        "resource.frontend.limit"
    );
    let parameters = (0..33)
        .map(|i| format!("p{i}: Big"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        compile(&format!(
            "{prefix} function main @id(\"main\")({parameters}) -> I64 {{ 0 }}"
        ))
        .unwrap_err()
        .code,
        "resource.frontend.limit"
    );
}

struct RecordBuild(std::path::PathBuf);
impl RecordBuild {
    fn new(artifact: &Artifact) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lobo-records-{}-{stamp}", std::process::id()));
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
}
impl Drop for RecordBuild {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Retained record reproducer at {}", self.0.display());
            return;
        }
        for name in ["program.c", "program.loboc", "native", "sanitized"] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

#[test]
fn record_cli_inputs_agree_in_evaluator_optimized_and_sanitized_native() {
    let text = format!("module m @id(\"m\"); {PAIR} function echo @id(\"echo\")(p: Pair) -> Pair {{ p }}");
    let artifact = compile(&text).unwrap();
    let build = RecordBuild::new(&artifact);
    for argument in [
        "Record:example.pair:I64:-9223372036854775808,I64:9223372036854775807",
        "Record:example.pair:I64:+0001,I64:2",
        "Record:example.pair:I64:1",
        "Record:example.pair:I64:1,I64:2,",
        "Record:example.pair:Bool:true,I64:2",
        "Record:other:I64:1,I64:2",
        "Record:example.pair:I64:1,I64:9223372036854775808",
        "Record:example.pair:I64:1,I64:2,I64:3",
    ] {
        let mut outcomes = Vec::new();
        for mode in ["eval", "native", "sanitized"] {
            let mut command = if mode == "eval" {
                let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lobo"));
                command.arg("eval").arg(build.0.join("program.loboc"));
                command
            } else {
                std::process::Command::new(build.0.join(mode))
            };
            command
                .env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1");
            let output = command
                .args(["--entry", "echo", "--arg", argument])
                .output()
                .unwrap();
            assert!(output.stderr.is_empty(), "{mode}: {output:?}");
            let result: Json = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(output.status.success(), result["status"] == "ok");
            if result["status"] == "ok" {
                outcomes.push(result["value"].clone());
            } else {
                assert_eq!(result["diagnostic"]["code"], "invocation.arguments.invalid");
                outcomes.push(result["diagnostic"]["code"].clone());
            }
        }
        assert!(
            outcomes.windows(2).all(|pair| pair[0] == pair[1]),
            "{argument}: {outcomes:?}"
        );
    }
}

fn program(body: &str, output: &str) -> String {
    format!("module m @id(\"m\"); {PAIR} function main @id(\"main\")() -> {output} {{ {body} }}")
}

fn check_native(artifact: &Artifact, entry: &str, arguments: &[&str], expected: &Json) {
    let mut args = vec!["--entry".into(), entry.into()];
    for arg in arguments {
        args.extend(["--arg".into(), (*arg).into()]);
    }
    let output = artifact.run_native(&args).unwrap();
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(&result["value"], expected);
}

#[test]
fn records_construct_select_copy_return_and_compare_nominal_values() {
    let text = program(
        "let p = Pair { right: 2, left: 1 }; let q: Pair = p; if p == q { q.left + q.right } else { 0 }",
        "I64",
    );
    let artifact = compile(&text).unwrap();
    assert_eq!(artifact.run("main", vec![], 1000).unwrap(), Value::I64(3));
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    check_native(&loaded, "main", &[], &json!({"type":"I64","value":"3"}));
    let record = compile(&program("Pair { left: 1, right: 2 }", "Pair")).unwrap();
    let expected = json!({"type":"Record","value":{"id":"example.pair","fields":{"left":{"type":"I64","value":"1"},"right":{"type":"I64","value":"2"}}}});
    assert_eq!(
        serde_json::to_value(record.run("main", vec![], 100).unwrap()).unwrap(),
        expected
    );
    check_native(&record, "main", &[], &expected);
}

#[test]
fn nested_forward_records_cross_function_and_host_boundaries() {
    let text = "module m @id(\"m\"); record Outer @id(\"outer:1\") { pair: Pair, flag: Bool }
      function swap @id(\"swap\")(p: Outer) -> Outer ensures { result.flag == p.flag } {
        Outer { pair: Pair { left: p.pair.right, right: p.pair.left }, flag: p.flag }
      }
      record Pair @id(\"pair\") { left: I64, right: I64 }";
    let artifact = compile(text).unwrap();
    let value = json!({"type":"Record","value":{"id":"outer:1","fields":{"pair":{"type":"Record","value":{"id":"pair","fields":{"left":{"type":"I64","value":"1"},"right":{"type":"I64","value":"2"}}}},"flag":{"type":"Bool","value":true}}}});
    let input: Value = serde_json::from_value(value.clone()).unwrap();
    let mut expected = value;
    expected["value"]["fields"]["pair"]["value"]["fields"]["left"]["value"] = json!("2");
    expected["value"]["fields"]["pair"]["value"]["fields"]["right"]["value"] = json!("1");
    assert_eq!(
        serde_json::to_value(artifact.run("swap", vec![input], 1000).unwrap()).unwrap(),
        expected
    );
    check_native(
        &artifact,
        "swap",
        &["Record:outer:1:I64:1,I64:2,Bool:true"],
        &expected,
    );
    let mut forged = expected;
    forged["value"]["fields"].as_object_mut().unwrap().remove("flag");
    assert_eq!(
        artifact
            .run("swap", vec![serde_json::from_value(forged).unwrap()], 1000)
            .unwrap_err()
            .code,
        "invocation.arguments.invalid"
    );
}

#[test]
fn record_state_enables_iterative_gcd_without_another_loop_construct() {
    let text = format!("module m @id(\"m\"); {PAIR} function gcd @id(\"gcd\")(a: I64, b: I64) -> I64 requires {{ a > 0 && b > 0 }} {{
      let p = fold(i in 100, state: Pair = Pair {{ left: a, right: b }}) {{
        if state.right == 0 {{ state }} else {{ Pair {{ left: state.right, right: state.left % state.right }} }}
      }}; p.left }}");
    let artifact = compile(&text).unwrap();
    assert_eq!(
        artifact
            .run("gcd", vec![Value::I64(1071), Value::I64(462)], 10000)
            .unwrap(),
        Value::I64(21)
    );
    check_native(
        &artifact,
        "gcd",
        &["I64:1071", "I64:462"],
        &json!({"type":"I64","value":"21"}),
    );
    let example = compile(include_str!("../examples/records.lobo")).unwrap();
    let build = RecordBuild::new(&example);
    for (a, b, expected) in [(1071, 462, 21), (48, 18, 6), (1, 1, 1), (i64::MAX, 2, 1)] {
        assert_eq!(
            example
                .run("example.records.gcd", vec![Value::I64(a), Value::I64(b)], 10000)
                .unwrap(),
            Value::I64(expected)
        );
        for mode in ["native", "sanitized"] {
            let output = std::process::Command::new(build.0.join(mode))
                .env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1:symbolize=0")
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .args([
                    "--entry",
                    "example.records.gcd",
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
            assert_eq!(result["value"]["value"], expected.to_string());
        }
    }
}

#[test]
fn records_reject_invalid_shapes_nominal_mismatches_and_cycles() {
    for (body, output, code) in [
        ("Pair { left: 1 }", "Pair", "compile.record.fields"),
        ("Pair { left: 1, left: 2 }", "Pair", "compile.record.fields"),
        ("Pair { left: 1, extra: 2 }", "Pair", "compile.record.fields"),
        ("Pair { left: true, right: 2 }", "Pair", "compile.type.mismatch"),
        (
            "(Pair { left: 1, right: 2 }).missing",
            "I64",
            "compile.record.field",
        ),
        ("(1).left", "I64", "compile.record.field"),
        ("1", "Missing", "compile.type.unknown"),
    ] {
        assert_eq!(compile(&program(body, output)).unwrap_err().code, code, "{body}");
    }
    let cycle =
        "module m @id(\"m\"); record R @id(\"r\") { next: R } function main @id(\"main\")() -> I64 { 0 }";
    assert_eq!(compile(cycle).unwrap_err().code, "compile.record.cycle");
}

#[test]
fn record_constructors_preserve_first_fault_and_do_not_confuse_conditions() {
    let text = program("Pair { right: 1 / 0, left: 9223372036854775807 + 1 }", "Pair");
    let artifact = compile(&text).unwrap();
    assert_eq!(
        artifact.run("main", vec![], 100).unwrap_err().code,
        "runtime.integer.zero_divisor"
    );
    let output = artifact.run_native(&["--entry".into(), "main".into()]).unwrap();
    let outcome: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(outcome["diagnostic"]["code"], "runtime.integer.zero_divisor");
    let text = program(
        "let flag = true; if flag { (Pair { left: 1, right: 2 }).left } else { 0 }",
        "I64",
    );
    assert_eq!(
        compile(&text).unwrap().run("main", vec![], 100).unwrap(),
        Value::I64(1)
    );
}
