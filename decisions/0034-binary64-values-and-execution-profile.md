# 0034: Binary64 values and execution profile

Date: 2026-11-04. Status: implemented in the 0.12 development compiler; release gates pending.
This completes the fourth authorized feature family, subject to the fixed numerical workload acceptance.

Use F64 as an explicit binary64 value type with homogeneous arithmetic and comparisons. Preserve all bits in
input/output and explicit bit conversion. Canonicalize arithmetic NaN results to `0x7ff8000000000000`.
Use IEEE equality, including unequal NaNs and equal signed zeros, recursively inside containers. Reject
nonfinite source/checked literals; explicit bit construction/input can represent nonfinite values. Checked
F64-to-I64 conversion rejects nonfinite/out-of-range values before the native cast, then truncates toward zero.

The execution profile requires round-to-nearest ties-even and gradual underflow. The evaluator and native
entry point run non-foldable arithmetic probes for rounding and subnormal input/output. An unsupported mode
returns `invocation.float.environment` rather than silently changing the embedding process's settings.
These probes do not configure or isolate arbitrary host floating-point exception traps; the supported
profile assumes default masked exceptions, as in the standalone Linux executable and ordinary Rust host.
No fast-math, implicit promotions, reassociation, fused operations, decimal type or transcendental library is
introduced. LLVM application arithmetic uses double instructions without fast-math flags; C emits separate
full-expression operations. Concrete supported release platform remains the existing Linux x86-64 toolchain.

Typed JSON stores a `0x` prefix and exactly sixteen lowercase hex digits. CLI input accepts finite decimal
atoms or exact bit atoms; nonfinite decimal spellings are rejected. This preserves negative zero and NaN
payloads without relying on nonstandard JSON numbers. Every F64 uses one logical cell, allocates no heap and
retains the existing value/fuel/input/output rules. Rust's public `Value::F64(u64)` carries bits, not an integer
numeric value. Literal admission and recursive value typing remain checked-artifact boundaries.

Alternatives considered: an f64 JSON number loses nonfinite values and can lose signed-zero/payload evidence;
a decimal-only wire format needs special values and formatting agreement; changing ambient rounding settings
would affect an embedding host and require platform hooks. The chosen bit wire and validated profile avoid
new Cargo dependencies, libm linkage or a custom numeric runtime. Normal and adversarial profile checks, both
native routes, evaluator, sanitizers, editor/skill and final acceptance are required before language 1.0.
