use lobo_core::{compile, Artifact};
mod support;
use support::Builds;

const SOURCE: &str = r#"module m @id("float.module");
function calc @id("calc")(op: I64, x: F64, y: F64) -> F64 {
  if op == 0 { x+y } else { if op == 1 { x-y } else { if op == 2 { x*y } else { x/y } } }
}
function negative @id("negative")(x: F64) -> F64 { -x }
function literal @id("literal")() -> F64 { 1.5 + 2.25e0 }
function equal @id("equal")(x: F64,y: F64) -> Bool { x == y }
function unequal @id("unequal")(x: F64,y: F64) -> Bool { x != y }
function less @id("less")(x: F64,y: F64) -> Bool { x < y }
function less_equal @id("less_equal")(x: F64,y: F64) -> Bool { x <= y }
function greater @id("greater")(x: F64,y: F64) -> Bool { x > y }
function greater_equal @id("greater_equal")(x: F64,y: F64) -> Bool { x >= y }
function finite @id("finite")(x: F64) -> Bool { F64::is_finite(x) }
function nan @id("nan")(x: F64) -> Bool { F64::is_nan(x) }
function to_int @id("to_int")(x: F64) -> I64 { F64::to_i64(x) }
function from_int @id("from_int")(x: I64) -> F64 { F64::from_i64(x) }
function bits @id("bits")(x: F64) -> I64 { F64::to_bits(x) }
function from_bits @id("from_bits")(x: I64) -> F64 { F64::from_bits(x) }
function list_equal @id("list_equal")(x: F64, y: F64) -> Bool {
  List::prepend(x,List::empty<F64>()) == List::prepend(y,List::empty<F64>())
}
"#;

#[test]
fn binary64_arithmetic_comparisons_and_wire_bits_agree_on_all_routes() {
    let a = compile(SOURCE).unwrap();
    let a = Artifact::from_json(&a.to_json().unwrap()).unwrap();
    let builds = Builds::new(&a);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "literal", &[], 1000, 1000)["value"]["value"],
            "0x400e000000000000"
        );
        for (op, bits) in [
            ("I64:0", "0x400e000000000000"),
            ("I64:1", "0xbfe8000000000000"),
            ("I64:2", "0x400b000000000000"),
            ("I64:3", "0x3fe5555555555555"),
        ] {
            assert_eq!(
                builds.invoke(mode, "calc", &[op, "F64:1.5", "F64:2.25"], 1000, 1000)["value"]["value"],
                bits
            );
        }
        for (x, y, eq, ne, lt) in [
            ("F64:0.0", "F64:-0.0", true, false, false),
            ("F64:1.5", "F64:2.25", false, true, true),
            (
                "F64:0x7ff8000000000001",
                "F64:0x7ff8000000000001",
                false,
                true,
                false,
            ),
        ] {
            for (entry, value) in [("equal", eq), ("unequal", ne), ("less", lt), ("list_equal", eq)] {
                assert_eq!(
                    builds.invoke(mode, entry, &[x, y], 1000, 1000)["value"]["value"],
                    value
                );
            }
        }
        assert_eq!(
            builds.invoke(mode, "negative", &["F64:0.0"], 1000, 1000)["value"]["value"],
            "0x8000000000000000"
        );
        assert_eq!(
            builds.invoke(mode, "calc", &["I64:3", "F64:0.0", "F64:0.0"], 1000, 1000)["value"]["value"],
            "0x7ff8000000000000"
        );
        assert_eq!(
            builds.invoke(mode, "calc", &["I64:3", "F64:1.0", "F64:0.0"], 1000, 1000)["value"]["value"],
            "0x7ff0000000000000"
        );
        assert_eq!(
            builds.invoke(mode, "finite", &["F64:0x7ff0000000000000"], 1000, 1000)["value"]["value"],
            false
        );
        assert_eq!(
            builds.invoke(mode, "nan", &["F64:0x7ff8000000000001"], 1000, 1000)["value"]["value"],
            true
        );
        assert_eq!(
            builds.invoke(mode, "from_bits", &["I64:-9223372036854775808"], 1000, 1000)["value"]["value"],
            "0x8000000000000000"
        );
        assert_eq!(
            builds.invoke(mode, "bits", &["F64:-0.0"], 1000, 1000)["value"]["value"],
            "-9223372036854775808"
        );
    }
}

#[test]
fn float_to_integer_conversion_is_checked_before_any_native_cast() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        for (input, expected) in [
            ("F64:1.75", "1"),
            ("F64:-1.75", "-1"),
            ("F64:-9223372036854775808.0", "-9223372036854775808"),
        ] {
            assert_eq!(
                builds.invoke(mode, "to_int", &[input], 1000, 1000)["value"]["value"],
                expected
            );
        }
        for input in [
            "F64:9223372036854775808.0",
            "F64:0x7ff0000000000000",
            "F64:0xfff0000000000000",
            "F64:0x7ff8000000000000",
        ] {
            assert_eq!(
                builds.invoke(mode, "to_int", &[input], 1000, 1000)["diagnostic"]["code"],
                "runtime.float.conversion"
            );
        }
        assert_eq!(
            builds.invoke(mode, "from_int", &["I64:9223372036854775807"], 1000, 1000)["value"]["value"],
            "0x43e0000000000000"
        );
    }
}

#[test]
fn numeric_types_are_explicit_and_nonfinite_literals_are_rejected() {
    for expr in ["1.5 + 1", "1.5 % 0.5", "1e999", "F64::to_i64(1)"] {
        let source = format!("module m @id(\"m\"); function f @id(\"f\")() -> F64 {{ {expr} }}");
        assert!(compile(&source).is_err(), "{expr}");
    }
}

#[test]
fn checked_float_literals_and_wire_encoding_are_revalidated() {
    let a = compile("module m @id(\"m\"); function f @id(\"f\")() -> F64 { 1.0 }").unwrap();
    let original: serde_json::Value = serde_json::from_str(&a.to_json().unwrap()).unwrap();
    for bits in [
        "0x7ff0000000000000",
        "0x7ff8000000000000",
        "0x3FF0000000000000",
        "1.0",
    ] {
        let mut data = original.clone();
        data["program"]["functions"][0]["body"]["value"]["value"]["value"] = bits.into();
        assert!(
            Artifact::from_json(&data.to_string()).is_err(),
            "invalid literal admitted: {bits}"
        );
    }
}

#[test]
fn rounding_subnormal_overflow_and_conversion_boundaries_have_fixed_bit_oracles() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        for (op, x, y, bits) in [
            ("I64:0", "F64:1.0", "F64:0x3ca0000000000000", "0x3ff0000000000000"),
            (
                "I64:0",
                "F64:0x3ff0000000000001",
                "F64:0x3ca0000000000000",
                "0x3ff0000000000002",
            ),
            ("I64:3", "F64:0x0010000000000000", "F64:2.0", "0x0008000000000000"),
            ("I64:2", "F64:0x7fefffffffffffff", "F64:2.0", "0x7ff0000000000000"),
        ] {
            assert_eq!(
                builds.invoke(mode, "calc", &[op, x, y], 1000, 1000)["value"]["value"],
                bits
            );
        }
        for (entry, expected) in [("less_equal", true), ("greater", false), ("greater_equal", true)] {
            assert_eq!(
                builds.invoke(mode, entry, &["F64:-0.0", "F64:0.0"], 1000, 1000)["value"]["value"],
                expected
            );
            assert_eq!(
                builds.invoke(mode, entry, &["F64:0x7ff8000000000001", "F64:0.0"], 1000, 1000)["value"]
                    ["value"],
                false
            );
        }
        assert_eq!(
            builds.invoke(mode, "to_int", &["F64:0x43dfffffffffffff"], 1000, 1000)["value"]["value"],
            "9223372036854774784"
        );
        assert_eq!(
            builds.invoke(mode, "to_int", &["F64:0xc3e0000000000001"], 1000, 1000)["diagnostic"]["code"],
            "runtime.float.conversion"
        );
        assert_eq!(
            builds.invoke(mode, "from_int", &["I64:9007199254740993"], 1000, 1000)["value"]["value"],
            "0x4340000000000000"
        );
        assert_eq!(
            builds.invoke(mode, "negative", &["F64:0xfff8000000000001"], 1000, 1000)["value"]["value"],
            "0x7ff8000000000000"
        );
        for invalid in [
            "F64:NaN",
            "F64:inf",
            "F64:1e999",
            "F64:0x3FF0000000000000",
            "F64:1.0garbage",
        ] {
            assert_eq!(
                builds.invoke(mode, "bits", &[invalid], 1000, 1000)["diagnostic"]["code"],
                "invocation.arguments.invalid"
            );
        }
    }
}

#[test]
fn both_native_routes_reject_nonstandard_floating_point_modes() {
    let a = compile("module m @id(\"m\"); function f @id(\"f\")() -> F64 { 1.0 }").unwrap();
    let builds = Builds::new(&a);
    let llvm = a.emit_llvm().unwrap().replace("@main(", "@lobo_generated_main(");
    std::fs::write(builds.0.join("renamed.ll"), llvm).unwrap();
    let driver = r#"
#include <fenv.h>
#include <stdlib.h>
#include <string.h>
#include <xmmintrin.h>
int main(int argc,char **argv) {
 const char *mode=getenv("LOBO_FLOAT_MODE");
 if (strcmp(mode,"up")==0) { if(fesetround(FE_UPWARD)) return 99; }
 if (strcmp(mode,"down")==0) { if(fesetround(FE_DOWNWARD)) return 99; }
 if (strcmp(mode,"zero")==0) { if(fesetround(FE_TOWARDZERO)) return 99; }
 if (strcmp(mode,"ftz")==0) _mm_setcsr(_mm_getcsr()|0x8000);
 if (strcmp(mode,"daz")==0) _mm_setcsr(_mm_getcsr()|0x40);
 return lobo_generated_main(argc,argv);
}
"#;
    for backend in ["c", "llvm"] {
        let prefix = if backend == "c" {
            "#define main lobo_generated_main\n#include \"program.c\"\n#undef main\n"
        } else {
            "int lobo_generated_main(int,char**);\n"
        };
        let source = builds.0.join(format!("profile-{backend}.c"));
        std::fs::write(&source, format!("{prefix}{driver}")).unwrap();
        let binary = builds.0.join(format!("profile-{backend}"));
        let mut cmd = std::process::Command::new("clang-18");
        cmd.args(["-std=c11", "-O3"]).arg(&source);
        if backend == "llvm" {
            cmd.arg(builds.0.join("renamed.ll"));
        }
        let result = cmd.arg("-lm").arg("-o").arg(&binary).output().unwrap();
        assert!(result.status.success(), "{result:?}");
        for mode in ["normal", "up", "down", "zero", "ftz", "daz"] {
            let result = std::process::Command::new(&binary)
                .env("LOBO_FLOAT_MODE", mode)
                .args(["--entry", "f"])
                .output()
                .unwrap();
            assert!(result.stderr.is_empty(), "{result:?}");
            let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            if mode == "normal" {
                assert!(result.status.success(), "{value}");
            } else {
                assert_eq!(
                    value["diagnostic"]["code"], "invocation.float.environment",
                    "{backend}/{mode}: {value}"
                );
            }
        }
    }
}

#[test]
#[cfg(target_arch = "x86_64")]
fn reference_evaluator_rejects_nonstandard_thread_floating_point_modes() {
    struct Mode(u32);
    impl Drop for Mode {
        fn drop(&mut self) {
            unsafe {
                std::arch::asm!("ldmxcsr [{0}]",in(reg)&self.0,options(nostack,preserves_flags));
            }
        }
    }
    let a = compile("module m @id(\"m\"); function f @id(\"f\")() -> F64 { 1.0 }").unwrap();
    for mode in [0x2000_u32, 0x4000, 0x6000, 0x8000, 0x40] {
        let mut old = 0_u32;
        unsafe {
            std::arch::asm!("stmxcsr [{0}]",in(reg)&mut old,options(nostack,preserves_flags));
        }
        let guard = Mode(old);
        let control = (old & !0xe040) | mode;
        unsafe {
            std::arch::asm!("ldmxcsr [{0}]",in(reg)&control,options(nostack,preserves_flags));
        }
        let result = a.run("f", vec![], 100);
        drop(guard);
        assert_eq!(result.unwrap_err().code, "invocation.float.environment");
    }
    assert!(a.run("f", vec![], 100).is_ok());
}
