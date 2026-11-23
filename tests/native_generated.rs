//! Deterministic native checks with an oracle separate from both compiler backends.
use lobo_core::{compile, Artifact, Type, Value};
use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const SEED: u64 = 0x10b0_c0de_2026_1010;
const FUEL: u64 = 100_000;
type Outcome = Result<Value, &'static str>;

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "lobo-generated-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn sanitized(&self, source: &str) -> PathBuf {
        let c = self.0.join("program.c");
        let binary = self.0.join("sanitized");
        std::fs::write(&c, source).unwrap();
        let output = Command::new("clang")
            .args([
                "-std=c11",
                "-O1",
                "-g",
                "-fsanitize=address,undefined",
                "-fno-sanitize-recover=all",
                "-fno-omit-frame-pointer",
            ])
            .arg(&c)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("Clang and its sanitizer runtimes are required, not optional skips");
        assert!(output.status.success(), "sanitizer build: {output:?}");
        binary
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Retained generated source and binaries at {}", self.0.display());
            return;
        }
        for name in ["program.c", "optimized", "sanitized", "llvm", "llvm-sanitized"] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

fn command(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    // Error paths terminate the invocation and let the OS reclaim CLI arguments.
    // Check invalid memory access and UB; do not claim leak coverage.
    // Assertions need diagnostic kinds, not symbolized stacks. Avoid external
    // symbolizer/debuginfod delays on the deliberately failing probe binaries.
    command.env("ASAN_OPTIONS", "detect_leaks=0:halt_on_error=1:symbolize=0");
    command.env("UBSAN_OPTIONS", "halt_on_error=1:symbolize=0");
    command
}

fn native(binary: &Path, entry: &str, args: &[Value]) -> Json {
    let mut invocation = command(binary);
    invocation.args(["--entry", entry, "--fuel", &FUEL.to_string()]);
    for arg in args {
        let value = match arg {
            Value::I64(n) => format!("I64:{n}"),
            Value::Bool(b) => format!("Bool:{b}"),
            Value::F64(_)
            | Value::Function(_)
            | Value::Record(_)
            | Value::Variant(_)
            | Value::List(_)
            | Value::Bytes(_)
            | Value::Text(_) => {
                panic!("scalar generator does not produce compound values")
            }
        };
        invocation.args(["--arg", &value]);
    }
    let output = invocation.output().unwrap();
    assert!(output.stderr.is_empty(), "{entry}: {output:?}");
    let json: Json =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("{entry}: {error}: {output:?}"));
    let expected_exit = if json["status"] == "ok" { 0 } else { 1 };
    assert_eq!(output.status.code(), Some(expected_exit), "{entry}: {output:?}");
    json
}

fn verify(artifact: &Artifact, binary: &Path, entry: &str, args: &[Value], expected: &Outcome) {
    let interpreted = artifact.run(entry, args.to_vec(), FUEL).map_err(|e| e.code);
    assert_eq!(
        interpreted,
        expected.clone().map_err(String::from),
        "reference {entry} args={args:?}"
    );
    let result = native(binary, entry, args);
    match expected {
        Ok(value) => {
            assert_eq!(result["kind"], "value", "{entry} args={args:?}");
            assert_eq!(result["value"], serde_json::to_value(value).unwrap(), "{entry}");
        }
        Err(code) => {
            assert_eq!(result["kind"], "runtime_fault", "{entry} args={args:?}");
            assert_eq!(result["diagnostic"]["code"], *code, "{entry} args={args:?}");
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Expression {
    text: String,
    ty: Type,
    expected: Outcome,
}

fn literal(value: Value) -> Expression {
    let (text, ty) = match &value {
        Value::I64(n) => (n.to_string(), Type::I64),
        Value::Bool(b) => (b.to_string(), Type::Bool),
        Value::F64(_)
        | Value::Function(_)
        | Value::Record(_)
        | Value::Variant(_)
        | Value::List(_)
        | Value::Bytes(_)
        | Value::Text(_) => {
            panic!("scalar generator does not produce compound values")
        }
    };
    Expression {
        text,
        ty,
        expected: Ok(value),
    }
}

fn wide(value: i128) -> Outcome {
    if value < i64::MIN as i128 || value > i64::MAX as i128 {
        Err("runtime.integer.overflow")
    } else {
        Ok(Value::I64(value as i64))
    }
}

fn binary(op: &str, left: Expression, right: Expression) -> Expression {
    // Mathematical i128 operations followed by explicit I64 range checks, not
    // the evaluator's checked_* implementations. Preserve left-to-right faults.
    let expected = (|| {
        let a = left.expected.clone()?;
        if op == "&&" && a == Value::Bool(false) {
            return Ok(a);
        }
        if op == "||" && a == Value::Bool(true) {
            return Ok(a);
        }
        let b = right.expected.clone()?;
        Ok(match (&a, &b, op) {
            (Value::I64(a), Value::I64(b), "+" | "-" | "*" | "/" | "%") => {
                if matches!(op, "/" | "%") {
                    if *b == 0 {
                        return Err("runtime.integer.zero_divisor");
                    }
                    if *a == i64::MIN && *b == -1 {
                        return Err("runtime.integer.overflow");
                    }
                }
                let (a, b) = (*a as i128, *b as i128);
                return wide(match op {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    "%" => a % b,
                    _ => unreachable!(),
                });
            }
            (Value::I64(a), Value::I64(b), "<") => Value::Bool(a < b),
            (Value::I64(a), Value::I64(b), "<=") => Value::Bool(a <= b),
            (Value::I64(a), Value::I64(b), ">") => Value::Bool(a > b),
            (Value::I64(a), Value::I64(b), ">=") => Value::Bool(a >= b),
            (_, _, "==") => Value::Bool(a == b),
            (_, _, "!=") => Value::Bool(a != b),
            (Value::Bool(a), Value::Bool(b), "&&") => Value::Bool(*a && *b),
            (Value::Bool(a), Value::Bool(b), "||") => Value::Bool(*a || *b),
            _ => unreachable!("ill-typed oracle expression"),
        })
    })();
    Expression {
        text: format!("({} {op} {})", left.text, right.text),
        ty: if matches!(op, "+" | "-" | "*" | "/" | "%") {
            Type::I64
        } else {
            Type::Bool
        },
        expected,
    }
}

struct Generator {
    state: u64,
    input: i64,
    flag: bool,
}

impl Generator {
    fn pick(&mut self, length: usize) -> usize {
        // Fixed xorshift64* stream; wrapping here is generator logic, not Lobo arithmetic.
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        (self.state.wrapping_mul(0x2545_f491_4f6c_dd1d) % length as u64) as usize
    }

    fn leaf(&mut self, ty: Type) -> Expression {
        if self.pick(3) == 0 {
            return Expression {
                text: if ty == Type::I64 { "x" } else { "flag" }.into(),
                ty: ty.clone(),
                expected: Ok(if ty == Type::I64 {
                    Value::I64(self.input)
                } else {
                    Value::Bool(self.flag)
                }),
            };
        }
        if ty == Type::Bool {
            literal(Value::Bool(self.pick(2) == 0))
        } else {
            let values = [i64::MIN, i64::MAX, -31, -7, -2, -1, 0, 1, 2, 7, 31];
            literal(Value::I64(values[self.pick(values.len())]))
        }
    }

    fn expression(&mut self, ty: Type, depth: usize) -> Expression {
        if depth == 0 {
            return self.leaf(ty);
        }
        match self.pick(5) {
            0 => self.leaf(ty),
            1 => {
                let value = self.expression(ty.clone(), depth - 1);
                let expected = value.expected.clone().and_then(|v| match v {
                    Value::I64(n) => wide(-(n as i128)),
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    Value::Function(_)
                    | Value::F64(_)
                    | Value::Record(_)
                    | Value::Variant(_)
                    | Value::List(_)
                    | Value::Bytes(_)
                    | Value::Text(_) => {
                        panic!("scalar generator does not produce compound values")
                    }
                });
                Expression {
                    text: format!("{}({})", if ty == Type::I64 { "-" } else { "!" }, value.text),
                    ty,
                    expected,
                }
            }
            2 => {
                let (ops, operand_type): (&[&str], Type) = if ty == Type::I64 {
                    (&["+", "-", "*", "/", "%"], Type::I64)
                } else if self.pick(2) == 0 {
                    (&["&&", "||", "==", "!="], Type::Bool)
                } else {
                    (&["<", "<=", ">", ">=", "==", "!="], Type::I64)
                };
                let op = ops[self.pick(ops.len())];
                let a = self.expression(operand_type.clone(), depth - 1);
                let b = self.expression(operand_type, depth - 1);
                binary(op, a, b)
            }
            3 => {
                let condition = self.expression(Type::Bool, depth - 1);
                let a = self.expression(ty.clone(), depth - 1);
                let b = self.expression(ty.clone(), depth - 1);
                let expected = match condition.expected {
                    Ok(Value::Bool(true)) => a.expected,
                    Ok(Value::Bool(false)) => b.expected,
                    Err(code) => Err(code),
                    _ => unreachable!(),
                };
                Expression {
                    text: format!(
                        "if {} {{ let branch = {}; branch }} else {{ let branch = {}; branch }}",
                        condition.text, a.text, b.text
                    ),
                    ty,
                    expected,
                }
            }
            _ if ty == Type::I64 => {
                let argument = self.expression(ty.clone(), depth - 1);
                let expected = argument.expected.and_then(|value| {
                    if value == Value::I64(-7) {
                        Err("runtime.contract.precondition")
                    } else {
                        Ok(value)
                    }
                });
                Expression {
                    text: format!("guard({})", argument.text),
                    ty,
                    expected,
                }
            }
            _ => {
                let a = self.expression(Type::Bool, depth - 1);
                let b = self.expression(Type::Bool, depth - 1);
                binary("==", a, b)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Case {
    expression: Expression,
    input: i64,
    flag: bool,
    contracts: &'static str,
}

fn cases() -> Vec<Case> {
    let overflow = Err("runtime.integer.overflow");
    let zero = Err("runtime.integer.zero_divisor");
    let probes = [
        ("x + 1", Type::I64, i64::MAX, "", overflow.clone()),
        ("x - 1", Type::I64, i64::MIN, "", overflow.clone()),
        ("x * 2", Type::I64, i64::MAX, "", overflow.clone()),
        ("-x", Type::I64, i64::MIN, "", overflow.clone()),
        ("x / -1", Type::I64, i64::MIN, "", overflow.clone()),
        ("x % -1", Type::I64, i64::MIN, "", overflow.clone()),
        ("x / 0", Type::I64, 1, "", zero.clone()),
        ("x % 0", Type::I64, 1, "", zero.clone()),
        ("x / -3", Type::I64, 7, "", Ok(Value::I64(-2))),
        ("x / 3", Type::I64, -7, "", Ok(Value::I64(-2))),
        ("x % -3", Type::I64, 7, "", Ok(Value::I64(1))),
        ("x % 3", Type::I64, -7, "", Ok(Value::I64(-1))),
        ("false && (1 / 0 == 0)", Type::Bool, 0, "", Ok(Value::Bool(false))),
        ("true || (1 / 0 == 0)", Type::Bool, 0, "", Ok(Value::Bool(true))),
        (
            "if flag { let chosen = x; chosen } else { 1 / 0 }",
            Type::I64,
            i64::MIN,
            "",
            Ok(Value::I64(i64::MIN)),
        ),
        (
            "(1 / 0) + (9223372036854775807 + 1)",
            Type::I64,
            0,
            "",
            zero.clone(),
        ),
        (
            "(9223372036854775807 + 1) + (1 / 0)",
            Type::I64,
            0,
            "",
            overflow.clone(),
        ),
        (
            "combine(1 / 0, 9223372036854775807 + 1)",
            Type::I64,
            0,
            "",
            zero.clone(),
        ),
        (
            "combine(9223372036854775807 + 1, 1 / 0)",
            Type::I64,
            0,
            "",
            overflow,
        ),
        (
            "guard(-7)",
            Type::I64,
            0,
            "",
            Err("runtime.contract.precondition"),
        ),
        (
            "1 / 0",
            Type::I64,
            0,
            "requires { false }",
            Err("runtime.contract.precondition"),
        ),
        (
            "42",
            Type::I64,
            0,
            "ensures { false }",
            Err("runtime.contract.postcondition"),
        ),
        ("42", Type::I64, 0, "requires { 1 / 0 == 0 }", zero.clone()),
        ("42", Type::I64, 0, "ensures { 1 / 0 == 0 }", zero),
    ];
    let mut cases: Vec<_> = probes
        .into_iter()
        .map(|(text, ty, input, contracts, expected)| Case {
            expression: Expression {
                text: text.into(),
                ty,
                expected,
            },
            input,
            flag: true,
            contracts,
        })
        .collect();
    for index in 0..76 {
        let mut generator = Generator {
            state: SEED + index,
            input: 0,
            flag: index % 2 == 0,
        };
        let values = [i64::MIN, i64::MAX, -7, -1, 0, 1, 31];
        generator.input = values[generator.pick(values.len())];
        let ty = if index % 2 == 0 { Type::I64 } else { Type::Bool };
        cases.push(Case {
            expression: generator.expression(ty, 3),
            input: generator.input,
            flag: generator.flag,
            contracts: "ensures { result == result }",
        });
    }
    cases
}

#[test]
fn deterministic_generated_core_agrees_with_oracle_native_and_sanitizers() {
    let cases = cases();
    assert_eq!(cases.len(), 100);
    assert_eq!(
        cases,
        self::cases(),
        "fixed seeds must reproduce the entire corpus"
    );
    let mut values = 0;
    let mut faults = 0;
    for (batch, chunk) in cases.chunks(10).enumerate() {
        let mut source = "module generated @id(\"generated\");\n".to_owned();
        for (offset, case) in chunk.iter().enumerate() {
            let index = batch * 10 + offset;
            source.push_str(&format!(
                "function case{index} @id(\"generated.case{index}\")(x: I64, flag: Bool) -> {:?} {} {{ {} }}\n",
                case.expression.ty, case.contracts, case.expression.text
            ));
        }
        // Forward calls and nested contracts; helpers are not separate generated cases.
        source.push_str("function guard @id(\"generated.guard\")(a: I64) -> I64 requires { a != -7 } ensures { result == a } { a }\n");
        source.push_str("function combine @id(\"generated.combine\")(a: I64, b: I64) -> I64 { a + b }\n");
        let artifact =
            compile(&source).unwrap_or_else(|e| panic!("seed={SEED:x} batch={batch}: {e:?}\n{source}"));
        let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
        let scratch = Scratch::new();
        let optimized = scratch.0.join("optimized");
        artifact.compile_native(&optimized).unwrap();
        let sanitized = scratch.sanitized(&artifact.emit_c().unwrap());
        let llvm = scratch.0.join("llvm");
        artifact
            .compile_native_with_backend(&llvm, lobo_core::NativeBackend::Llvm)
            .unwrap();
        let llvm_sanitized = scratch.0.join("llvm-sanitized");
        artifact.compile_llvm_sanitized(&llvm_sanitized).unwrap();
        for (offset, case) in chunk.iter().enumerate() {
            let index = batch * 10 + offset;
            let entry = format!("generated.case{index}");
            let args = [Value::I64(case.input), Value::Bool(case.flag)];
            assert_eq!(
                loaded.run(&entry, args.to_vec(), FUEL).map_err(|e| e.code),
                case.expression.expected.clone().map_err(String::from),
                "reloaded seed={SEED:x} case={index} {}",
                case.expression.text
            );
            for binary in [&optimized, &sanitized, &llvm, &llvm_sanitized] {
                verify(&artifact, binary, &entry, &args, &case.expression.expected);
            }
            if case.expression.expected.is_ok() {
                values += 1;
            } else {
                faults += 1;
            }
        }
    }
    assert!(values > 0 && faults > 0);
    println!(
        "seed={SEED:x}: 100 cases ({values} values, {faults} faults), no skips; O3 native + O1 ASan/UBSan"
    );
}

#[test]
fn sanitizer_instrumentation_is_active() {
    let scratch = Scratch::new();
    let binary = scratch.sanitized("#include <stdint.h>\nint main(void) { volatile int64_t maximum = INT64_MAX; return (int)(maximum + 1); }\n");
    let output = command(&binary).output().unwrap();
    assert!(
        !output.status.success(),
        "UBSan failed to detect overflow: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("signed integer overflow"),
        "{output:?}"
    );
    let binary = scratch.sanitized("#include <stdlib.h>\nint main(void) { volatile size_t count = 1; volatile char *memory = malloc(count); if (!memory) return 2; memory[count] = 7; free((void *)memory); return 0; }\n");
    let output = command(&binary).output().unwrap();
    assert!(
        !output.status.success(),
        "ASan failed to detect invalid access: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("AddressSanitizer: heap-buffer-overflow"),
        "{output:?}"
    );
}

#[test]
fn executable_algorithms_match_independent_expected_results() {
    let artifact = compile(include_str!("../examples/algorithms.lobo")).unwrap();
    let scratch = Scratch::new();
    let optimized = scratch.0.join("optimized");
    artifact.compile_native(&optimized).unwrap();
    let sanitized = scratch.sanitized(&artifact.emit_c().unwrap());
    let llvm = scratch.0.join("llvm");
    artifact
        .compile_native_with_backend(&llvm, lobo_core::NativeBackend::Llvm)
        .unwrap();
    let llvm_sanitized = scratch.0.join("llvm-sanitized");
    artifact.compile_llvm_sanitized(&llvm_sanitized).unwrap();
    for binary in [&optimized, &sanitized, &llvm, &llvm_sanitized] {
        for (a, b, expected) in [
            (48, 18, 6),
            (0, 7, 7),
            (7, 0, 7),
            (17, 13, 1),
            (i64::MAX, i64::MAX, i64::MAX),
        ] {
            verify(
                &artifact,
                binary,
                "example.gcd",
                &[Value::I64(a), Value::I64(b)],
                &Ok(Value::I64(expected)),
            );
        }
        for (n, expected) in [
            (0, 1),
            (1, 1),
            (5, 120),
            (10, 3_628_800),
            (20, 2_432_902_008_176_640_000),
        ] {
            verify(
                &artifact,
                binary,
                "example.factorial",
                &[Value::I64(n)],
                &Ok(Value::I64(expected)),
            );
        }
        for args in [[0, 0], [-1, 7], [7, -1]] {
            verify(
                &artifact,
                binary,
                "example.gcd",
                &args.map(Value::I64),
                &Err("runtime.contract.precondition"),
            );
        }
        for n in [-1, 21, i64::MAX] {
            verify(
                &artifact,
                binary,
                "example.factorial",
                &[Value::I64(n)],
                &Err("runtime.contract.precondition"),
            );
        }
    }
}
