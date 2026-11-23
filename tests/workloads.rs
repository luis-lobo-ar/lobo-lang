use lobo_core::{compile, Artifact};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn parser() -> Builds {
    let source = std::fs::read_to_string("examples/parser.lobo").unwrap();
    let artifact = compile(&source).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    Builds::new(&artifact)
}

fn input(bytes: &[u8]) -> String {
    let mut wire = String::from("Bytes:");
    for byte in bytes {
        wire.push_str(&format!("{byte:02x}"));
    }
    wire
}

fn success(value: i64) -> Json {
    json!({"type":"Variant","value":{"id":"example.parser.answer","case":"Ok",
        "payload":{"type":"I64","value":value.to_string()}}})
}

fn failure(case: &str, offset: i64) -> Json {
    json!({"type":"Variant","value":{"id":"example.parser.answer","case":"Failed",
        "payload":{"type":"Variant","value":{"id":"example.parser.error","case":case,
            "payload":{"type":"I64","value":offset.to_string()}}}}})
}

#[test]
fn parser_obeys_frozen_precedence_and_missing_operand_cases_in_all_modes() {
    let builds = parser();
    for mode in ["eval", "native", "sanitized"] {
        for (source, expected) in [
            ("12 + 3 * 4", success(24)),
            ("(12 + 3) * 4", success(60)),
            ("1 +", failure("MissingOperand", 3)),
        ] {
            let result = builds.invoke(
                mode,
                "example.parser.evaluate",
                &[&input(source.as_bytes())],
                1000000,
                100000,
            );
            assert_eq!(result["value"], expected, "{mode}: {source}: {result}");
        }
    }
}

#[test]
fn parser_reports_frozen_boundaries_as_typed_values_not_runtime_arithmetic_faults() {
    let builds = parser();
    let cases = [
        ("2*3*4+5+6", success(35)),
        ("0", success(0)),
        ("   42  ", success(42)),
        ("", failure("MissingOperand", 0)),
        ("1 2", failure("MissingOperator", 2)),
        ("(1+2", failure("Unbalanced", 4)),
        ("2 & 3", failure("UnexpectedByte", 2)),
        ("9223372036854775808", failure("IntegerOverflow", 18)),
        ("9223372036854775807+1", failure("ArithmeticOverflow", 19)),
        ("9223372036854775807", success(i64::MAX)),
        ("0000042", success(42)),
        ("2*(3+4*(5+6))", success(94)),
        ("2*0+7", success(7)),
        ("3037000500*3037000500", failure("ArithmeticOverflow", 10)),
        ("()", failure("MissingOperand", 1)),
        ("1++2", failure("MissingOperand", 2)),
        ("1(2)", failure("MissingOperator", 1)),
        ("1)", failure("Unbalanced", 1)),
        (")", failure("Unbalanced", 0)),
        ("9223372036854775807+1)", failure("Unbalanced", 21)),
        ("( ", failure("MissingOperand", 2)),
        ("1\t+2", failure("UnexpectedByte", 1)),
        ("\0", failure("UnexpectedByte", 0)),
        ("1+🐺", failure("UnexpectedByte", 2)),
    ];
    for mode in ["eval", "native", "sanitized"] {
        for (source, expected) in &cases {
            let result = builds.invoke(
                mode,
                "example.parser.evaluate",
                &[&input(source.as_bytes())],
                1000000,
                100000,
            );
            assert_eq!(&result["value"], expected, "{mode}: {source:?}: {result}");
        }
        let nested = format!("{}7{}", "(".repeat(128), ")".repeat(128));
        let result = builds.invoke(
            mode,
            "example.parser.evaluate",
            &[&input(nested.as_bytes())],
            1000000,
            100000,
        );
        assert_eq!(result["value"], success(7), "{mode}: {result}");
    }
}

#[test]
fn parser_exhaustion_is_diagnostic_without_a_partial_answer() {
    let builds = parser();
    for mode in ["eval", "native", "sanitized"] {
        for (fuel, heap, code) in [
            (20, 100000, "resource.fuel.exhausted"),
            (1000000, 4, "resource.memory.exhausted"),
        ] {
            let result = builds.invoke(mode, "example.parser.evaluate", &[&input(b"12+3*4")], fuel, heap);
            assert_eq!(result["status"], "error", "{mode}: {result}");
            assert_eq!(result["diagnostic"]["code"], code, "{mode}: {result}");
            assert!(result.get("value").is_none(), "{mode}: {result}");
        }
    }
}

#[test]
fn parser_accepts_new_expression_variations_with_independent_constructed_expectations() {
    let builds = parser();
    // Construct expressions and their mathematical expectations, never parse the input in the harness.
    let mut seed = 0x7a310f29u32;
    let mut cases = Vec::new();
    for _ in 0..16 {
        let mut numbers = [0i64; 4];
        for number in &mut numbers {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *number = i64::from((seed >> 24) % 100);
        }
        let [a, b, c, d] = numbers;
        cases.extend([
            (format!("{a} + {b} * {c} + {d}"), a + b * c + d),
            (format!("({a}+{b})*({c}+{d})"), (a + b) * (c + d)),
            (format!("{a}*({b}+{c}*{d})"), a * (b + c * d)),
            (format!("(({a}))+{b}*{c}*{d}"), a + b * c * d),
        ]);
    }
    for mode in ["eval", "native", "sanitized"] {
        for (source, expected) in &cases {
            let result = builds.invoke(
                mode,
                "example.parser.evaluate",
                &[&input(source.as_bytes())],
                1000000,
                100000,
            );
            assert_eq!(result["value"], success(*expected), "{mode}: {source}: {result}");
        }
    }
}

#[test]
fn every_single_octet_is_classified_without_decoding_or_host_parsing() {
    let builds = parser();
    for mode in ["eval", "native", "sanitized"] {
        for byte in 0u8..=255 {
            let expected = match byte {
                b'0'..=b'9' => success(i64::from(byte - b'0')),
                b' ' | b'(' => failure("MissingOperand", 1),
                b'+' | b'*' => failure("MissingOperand", 0),
                b')' => failure("Unbalanced", 0),
                _ => failure("UnexpectedByte", 0),
            };
            let result = builds.invoke(
                mode,
                "example.parser.evaluate",
                &[&input(&[byte])],
                1000000,
                100000,
            );
            assert_eq!(result["value"], expected, "{mode}: {byte:02x}: {result}");
        }
        for (source, expected) in [
            ("&9223372036854775808", failure("UnexpectedByte", 0)),
            ("9223372036854775807*2*0", failure("ArithmeticOverflow", 19)),
            ("9223372036854775807+1*0", success(i64::MAX)),
            ("0*9223372036854775807*2", success(0)),
            ("9223372036854775807+1+&", failure("ArithmeticOverflow", 19)),
        ] {
            let result = builds.invoke(
                mode,
                "example.parser.evaluate",
                &[&input(source.as_bytes())],
                1000000,
                100000,
            );
            assert_eq!(result["value"], expected, "{mode}: {source}: {result}");
        }
    }
}
