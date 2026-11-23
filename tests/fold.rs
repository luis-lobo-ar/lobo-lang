use lobo_core::{compile, Artifact, Value};
use serde_json::Value as Json;
use std::path::PathBuf;
use std::process::Command;

fn program(body: &str, ty: &str) -> String {
    format!(
        "module iteration @id(\"iteration\"); function main @id(\"iteration.main\")() -> {ty} {{ {body} }}"
    )
}

#[test]
fn zero_iterations_return_initial_without_evaluating_body() {
    let artifact = compile(&program("fold (i in 0, acc: I64 = 41) { 1 / 0 }", "I64")).unwrap();
    assert_eq!(
        artifact.run("iteration.main", vec![], 1000).unwrap(),
        Value::I64(41)
    );
}

struct NativeBuild(PathBuf);

impl NativeBuild {
    fn new(artifact: &Artifact) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("lobo-fold-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let build = Self(directory);
        artifact.compile_native(&build.0.join("optimized")).unwrap();
        let source = build.0.join("program.c");
        std::fs::write(&source, artifact.emit_c().unwrap()).unwrap();
        let output = Command::new("clang")
            .args([
                "-std=c11",
                "-O1",
                "-g",
                "-fsanitize=address,undefined",
                "-fno-sanitize-recover=all",
                "-fno-omit-frame-pointer",
            ])
            .arg(&source)
            .arg("-o")
            .arg(build.0.join("sanitized"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        build
    }

    fn outcomes(&self, entry: &str, args: &[Value], fuel: u64) -> Vec<Json> {
        ["optimized", "sanitized"]
            .into_iter()
            .map(|mode| {
                let mut command = Command::new(self.0.join(mode));
                command.env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1");
                command.env("UBSAN_OPTIONS", "halt_on_error=1");
                command.args(["--entry", entry, "--fuel", &fuel.to_string()]);
                for arg in args {
                    let arg = match arg {
                        Value::I64(n) => format!("I64:{n}"),
                        Value::Bool(b) => format!("Bool:{b}"),
                        Value::Function(_)
                        | Value::F64(_)
                        | Value::Record(_)
                        | Value::Variant(_)
                        | Value::List(_)
                        | Value::Bytes(_)
                        | Value::Text(_) => {
                            panic!("scalar fold corpus does not generate compound values")
                        }
                    };
                    command.args(["--arg", &arg]);
                }
                let output = command.output().unwrap();
                assert!(output.stderr.is_empty(), "{mode}: {output:?}");
                let result: Json = serde_json::from_slice(&output.stdout)
                    .unwrap_or_else(|e| panic!("{mode}: {e}: {output:?}"));
                assert_eq!(
                    output.status.code(),
                    Some(if result["status"] == "ok" { 0 } else { 1 }),
                    "{output:?}"
                );
                result
            })
            .collect()
    }
}

impl Drop for NativeBuild {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Retained fold reproducer at {}", self.0.display());
            return;
        }
        for name in ["program.c", "optimized", "sanitized"] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

#[test]
fn fold_values_and_faults_match_independent_fixtures_in_all_backends() {
    let cases = [
        ("fold (i in 0, a: I64 = 41) { 1 / 0 }", "I64", Ok(Value::I64(41))),
        ("fold (i in 1, a: I64 = 7) { a + i }", "I64", Ok(Value::I64(7))),
        (
            "fold (i in 5, a: I64 = 0) { a + i + 1 }",
            "I64",
            Ok(Value::I64(15)),
        ),
        (
            "fold (i in 3, a: Bool = false) { a || i == 1 }",
            "Bool",
            Ok(Value::Bool(true)),
        ),
        (
            "fold (i in 3, a: I64 = 0) { let fresh = a + 1; fresh }",
            "I64",
            Ok(Value::I64(3)),
        ),
        (
            "fold (i in 3, a: I64 = 0) { fold (j in 2, b: I64 = a) { b + i + j } }",
            "I64",
            Ok(Value::I64(9)),
        ),
        (
            "fold (i in 0, a: I64 = 1 / 0) { a }",
            "I64",
            Err("runtime.integer.zero_divisor"),
        ),
        (
            "fold (i in -1, a: I64 = 1 / 0) { a }",
            "I64",
            Err("runtime.iteration.negative_count"),
        ),
        (
            "fold (i in (1 / 0), a: I64 = 9223372036854775807 + 1) { a }",
            "I64",
            Err("runtime.integer.zero_divisor"),
        ),
        (
            "fold (i in 1, a: I64 = 9223372036854775807) { a + 1 }",
            "I64",
            Err("runtime.integer.overflow"),
        ),
        (
            "fold (i in 1, a: I64 = 0) { (1 / 0) + (9223372036854775807 + 1) }",
            "I64",
            Err("runtime.integer.zero_divisor"),
        ),
        (
            "if false { fold (i in -1, a: I64 = 1 / 0) { a } } else { 8 }",
            "I64",
            Ok(Value::I64(8)),
        ),
    ];
    for (body, ty, expected) in cases {
        let artifact = compile(&program(body, ty)).unwrap();
        let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
        for artifact in [&artifact, &loaded] {
            assert_eq!(
                artifact
                    .run("iteration.main", vec![], 100_000)
                    .map_err(|e| e.code),
                expected.clone().map_err(String::from),
                "{body}"
            );
        }
        let build = NativeBuild::new(&artifact);
        for outcome in build.outcomes("iteration.main", &[], 100_000) {
            match &expected {
                Ok(value) => {
                    assert_eq!(outcome["kind"], "value", "{body}");
                    assert_eq!(outcome["value"], serde_json::to_value(value).unwrap(), "{body}");
                }
                Err(code) => {
                    assert_eq!(outcome["kind"], "runtime_fault", "{body}");
                    assert_eq!(outcome["diagnostic"]["code"], *code, "{body}");
                }
            }
        }
    }
}

#[test]
fn fold_rejects_wrong_types_and_invalid_scope_even_at_zero_count() {
    for (body, code) in [
        ("fold (i in true, a: I64 = 0) { a }", "compile.type.mismatch"),
        ("fold (i in 0, a: I64 = false) { a }", "compile.type.mismatch"),
        ("fold (i in 0, a: I64 = 0) { false }", "compile.type.mismatch"),
        ("fold (i in 0, a: I64 = i) { a }", "compile.name.unresolved"),
        ("fold (i in 0, a: I64 = a) { a }", "compile.name.unresolved"),
        ("fold (i in a, a: I64 = 0) { a }", "compile.name.unresolved"),
        ("fold (i in 0, i: I64 = 0) { i }", "compile.name.duplicate"),
        (
            "let a = 0; fold (i in 0, a: I64 = 0) { a }",
            "compile.name.duplicate",
        ),
        ("fold (main in 0, a: I64 = 0) { a }", "compile.name.duplicate"),
        ("fold (result in 0, a: I64 = 0) { a }", "compile.name.reserved"),
        (
            "fold (i in 0, a: I64 = 0) { let i = 1; a }",
            "compile.name.duplicate",
        ),
        (
            "let value = fold (i in 0, a: I64 = 0) { let fresh = 1; fresh }; fresh",
            "compile.name.unresolved",
        ),
        (
            "if false { fold (i in 0, a: I64 = 0) { missing } } else { 0 }",
            "compile.name.unresolved",
        ),
        ("fold (i in 1, a = 0) { a }", "compile.syntax.invalid"),
    ] {
        assert_eq!(compile(&program(body, "I64")).unwrap_err().code, code, "{body}");
    }
}

#[test]
fn folds_resolve_calls_and_contextual_names_without_shadowing() {
    let source = "module names @id(\"names\"); function fold @id(\"names.fold\")(in: I64) -> I64 { in + 1 } function main @id(\"iteration.main\")() -> I64 { fold (i in fold(2), a: I64 = fold(0)) { fold(a) } }";
    let artifact = compile(source).unwrap();
    assert_eq!(
        artifact.run("iteration.main", vec![], 1000).unwrap(),
        Value::I64(4)
    );
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(loaded.run("iteration.main", vec![], 1000).unwrap(), Value::I64(4));
    for outcome in NativeBuild::new(&artifact).outcomes("iteration.main", &[], 1000) {
        assert_eq!(outcome["value"]["value"], "4");
    }
    let source = "module names @id(\"names\"); function main @id(\"iteration.main\")() -> I64 { let fold = 7; let in = 2; fold + in }";
    assert_eq!(
        compile(source)
            .unwrap()
            .run("iteration.main", vec![], 1000)
            .unwrap(),
        Value::I64(9)
    );
}

#[test]
fn folds_preserve_contracts_and_fault_phases() {
    for (contracts, body, expected_code, phase) in [
        (
            "requires { false }",
            "fold (i in -1, a: I64 = 1 / 0) { a }",
            "runtime.contract.precondition",
            "precondition",
        ),
        (
            "requires { (fold (i in 1, a: I64 = 0) { 1 / 0 }) == 0 }",
            "1",
            "runtime.integer.zero_divisor",
            "precondition",
        ),
        (
            "ensures { (fold (i in -1, a: I64 = 0) { a }) == 0 }",
            "1",
            "runtime.iteration.negative_count",
            "postcondition",
        ),
        (
            "ensures { fold (i in 1, a: Bool = true) { false } }",
            "1",
            "runtime.contract.postcondition",
            "postcondition",
        ),
        (
            "requires { (fold (i in 1, a: I64 = 0) { fail() }) == 0 }",
            "1",
            "runtime.integer.zero_divisor",
            "precondition",
        ),
        (
            "ensures { (fold (i in 1, a: I64 = 0) { fail() }) == 0 }",
            "1",
            "runtime.integer.zero_divisor",
            "postcondition",
        ),
    ] {
        let source = format!("module contracts @id(\"contracts\"); function main @id(\"iteration.main\")() -> I64 {contracts} {{ {body} }} function fail @id(\"contracts.fail\")() -> I64 {{ 1 / 0 }}");
        let artifact = compile(&source).unwrap();
        let error = artifact.run("iteration.main", vec![], 1000).unwrap_err();
        assert_eq!(error.code, expected_code);
        assert_eq!(error.phase, phase);
        for outcome in NativeBuild::new(&artifact).outcomes("iteration.main", &[], 1000) {
            assert_eq!(outcome["diagnostic"]["code"], expected_code);
            assert_eq!(outcome["diagnostic"]["phase"], phase);
            assert_eq!(
                outcome["diagnostic"]["subject"],
                error.subject.as_deref().unwrap()
            );
        }
    }
}

#[test]
fn large_counts_exhaust_fuel_and_iteration_does_not_grow_call_stack() {
    let artifact = compile("module limits @id(\"limits\"); function main @id(\"iteration.main\")(n: I64) -> I64 { fold (i in n, a: I64 = 0) { a + 1 } }").unwrap();
    let build = NativeBuild::new(&artifact);
    assert_eq!(
        artifact
            .run("iteration.main", vec![Value::I64(100)], 1000)
            .unwrap(),
        Value::I64(100)
    );
    for outcome in build.outcomes("iteration.main", &[Value::I64(100)], 1000) {
        assert_eq!(outcome["value"]["value"], "100");
    }
    let constant = compile(&program(
        "fold (i in 9223372036854775807, a: I64 = 0) { a }",
        "I64",
    ))
    .unwrap();
    for (artifact, args) in [(&artifact, vec![Value::I64(i64::MAX)]), (&constant, vec![])] {
        assert_eq!(
            artifact.run("iteration.main", args.clone(), 20).unwrap_err().code,
            "resource.fuel.exhausted"
        );
        for outcome in NativeBuild::new(artifact).outcomes("iteration.main", &args, 20) {
            assert_eq!(outcome["kind"], "resource_exhausted");
            assert_eq!(outcome["diagnostic"]["code"], "resource.fuel.exhausted");
        }
    }
}

#[test]
fn fold_artifacts_are_versioned_deterministic_and_revalidated() {
    let source = program("fold (i in 2, a: I64 = 0) { a + i }", "I64");
    let artifact = compile(&source).unwrap();
    let encoded = artifact.to_json().unwrap();
    assert_eq!(
        encoded,
        compile(&format!("// same\n{source}")).unwrap().to_json().unwrap()
    );
    let mut json: Json = serde_json::from_str(&encoded).unwrap();
    assert_eq!(json["format"], "lobo-checked/1.0");
    assert_eq!(json["language"], lobo_core::LANGUAGE);
    json["format"] = "lobo-checked/0.1".into();
    json["language"] = "core/0.1-draft".into();
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
    json = serde_json::from_str(&encoded).unwrap();
    let node = &mut json["program"]["functions"][0]["body"]["value"];
    node["accumulator"]["name"] = "i".into();
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "compile.name.duplicate"
    );
    json = serde_json::from_str(&encoded).unwrap();
    json["program"]["functions"][0]["body"]["value"]["count"] =
        serde_json::json!({"kind":"Literal","value":{"type":"Bool","value":false}});
    assert_eq!(
        Artifact::from_json(&json.to_string()).unwrap_err().code,
        "compile.type.mismatch"
    );
    let mut old: Json =
        serde_json::from_str(&compile(&program("41", "I64")).unwrap().to_json().unwrap()).unwrap();
    old["format"] = "lobo-checked/0.1".into();
    old["language"] = "core/0.1-draft".into();
    assert_eq!(
        Artifact::from_json(&old.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
}

#[test]
fn each_entered_iteration_charges_fuel_before_its_body() {
    // Current host accounting: function, fold, count, initial, iteration, body = 6.
    let artifact = compile(&program("fold (i in 1, a: I64 = 0) { a }", "I64")).unwrap();
    let build = NativeBuild::new(&artifact);
    assert_eq!(
        artifact.run("iteration.main", vec![], 5).unwrap_err().code,
        "resource.fuel.exhausted"
    );
    assert_eq!(artifact.run("iteration.main", vec![], 6).unwrap(), Value::I64(0));
    for outcome in build.outcomes("iteration.main", &[], 5) {
        assert_eq!(outcome["diagnostic"]["code"], "resource.fuel.exhausted");
    }
    for outcome in build.outcomes("iteration.main", &[], 6) {
        assert_eq!(outcome["value"]["value"], "0");
    }
}

#[test]
fn nested_contract_phases_preserve_underlying_codes_and_restore_body_context() {
    for (outer, helper, body, code, phase, subject) in [
        (
            "requires { (fold (i in 1, a: I64 = 0) { helper() }) == 1 }",
            "ensures { false }",
            "1",
            "runtime.contract.postcondition",
            "precondition",
            "contracts.helper",
        ),
        (
            "ensures { (fold (i in 1, a: I64 = 0) { helper() }) == 1 }",
            "requires { false }",
            "1",
            "runtime.contract.precondition",
            "postcondition",
            "contracts.helper",
        ),
        (
            "requires { (fold (i in 1, a: I64 = 0) { helper() }) == 1 }",
            "requires { true } ensures { result == 1 }",
            "1 / 0",
            "runtime.integer.zero_divisor",
            "execute",
            "iteration.main",
        ),
    ] {
        let source = format!("module contracts @id(\"contracts\"); function main @id(\"iteration.main\")() -> I64 {outer} {{ {body} }} function helper @id(\"contracts.helper\")() -> I64 {helper} {{ 1 }}");
        let artifact = compile(&source).unwrap();
        let error = artifact.run("iteration.main", vec![], 1000).unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.phase, phase);
        assert_eq!(error.subject.as_deref(), Some(subject));
        for outcome in NativeBuild::new(&artifact).outcomes("iteration.main", &[], 1000) {
            assert_eq!(outcome["diagnostic"]["code"], code);
            assert_eq!(outcome["diagnostic"]["phase"], phase);
            assert_eq!(outcome["diagnostic"]["subject"], subject);
        }
    }
}

#[test]
fn parameterized_folds_match_mathematical_sums_and_contextual_edge_cases() {
    let source = "module sums @id(\"sums\"); function main @id(\"iteration.main\")(n: I64, seed: I64) -> I64 { fold (i in n, a: I64 = seed) { a + i + 1 } }";
    let artifact = compile(source).unwrap();
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let build = NativeBuild::new(&artifact);
    for count in -1..=32 {
        for initial in [i64::MIN, 0, i64::MAX] {
            let expected = if count < 0 {
                Err("runtime.iteration.negative_count")
            } else {
                let mathematical = initial as i128 + (count as i128 * (count as i128 + 1)) / 2;
                if mathematical > i64::MAX as i128 {
                    Err("runtime.integer.overflow")
                } else {
                    Ok(Value::I64(mathematical as i64))
                }
            };
            let args = [Value::I64(count), Value::I64(initial)];
            for artifact in [&artifact, &loaded] {
                assert_eq!(
                    artifact
                        .run("iteration.main", args.to_vec(), 10_000)
                        .map_err(|e| e.code),
                    expected.clone().map_err(String::from),
                    "count={count} initial={initial}"
                );
            }
            for outcome in build.outcomes("iteration.main", &args, 10_000) {
                match &expected {
                    Ok(value) => assert_eq!(outcome["value"], serde_json::to_value(value).unwrap()),
                    Err(code) => assert_eq!(outcome["diagnostic"]["code"], *code),
                }
            }
        }
    }
    for (source, expected) in [
        ("module names @id(\"names\"); function fold @id(\"names.fold\")(in: I64) -> I64 { in + 1 } function main @id(\"iteration.main\")() -> I64 { let in = 41; fold(in) }", 42),
        (&program("fold (in in 2, a: I64 = 0) { a + in }", "I64"), 1),
        (&program("fold (i in 2, fold: I64 = 0) { fold + i }", "I64"), 1),
    ] {
        let artifact = compile(source).unwrap();
        assert_eq!(artifact.run("iteration.main", vec![], 1000).unwrap(), Value::I64(expected));
        for outcome in NativeBuild::new(&artifact).outcomes("iteration.main", &[], 1000) {
            assert_eq!(outcome["value"]["value"], expected.to_string());
        }
    }
}
