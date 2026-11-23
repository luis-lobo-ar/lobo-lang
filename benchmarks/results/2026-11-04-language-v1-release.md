# Lobo language 1.0 local release

Status: passed. Compiler/extension version 1.0.0; source `core/1.0`; checked format `lobo-checked/1.0`.
The [machine-readable receipt](2026-11-04-language-v1-release.json) records hashes, preserved evidence,
exact artifact sizes and the final immutable manifest. This completes the bounded language goal in decision
0033. It does not complete the broader agent workspace/product or authorize another language family.

## What passed

- All four families: generic records/variants, parametric definition-time checking, pure named function
  values with exact signatures/action bounds, and F64 arithmetic/conversions/comparisons/lossless interchange.
- All three frozen programs and planned changes: six configurations, each tested by evaluator, C, direct LLVM,
  C with ASan/UBSan and LLVM with active sanitizer checks. The collection case processes 10,000 values inside
  Lobo under the original 2,000,000 fuel/heap ceilings. The document schema change and fractional numerical
  validation/aggregation produce the fixed expected values/errors. No oracle/budget was relaxed.
- 185 Rust tests; seven optimized float tests repeated under the release profile; 30 Python tests; 122 editor
  tests; real VS Code host assertions; formatting and Clippy with warnings denied; JSON/JSONL/Markdown checks;
  skill validation and six native example executions. Repeated suites are not independent samples.
- The final builder executed 64 commands successfully, including twelve optimized native builds/runs plus
  six reference runs using the packaged release compiler. Native programs were checked against independent
  fixed value oracles and full evaluator output. Tool discovery reports compiler 1.0.0 / language 1.0.
- A separate JSONL session passed a fresh native test with F64 and function-reference inputs through generic
  Box/apply. This verifies the agent-facing test tool integration, not an independent agent experiment.
- All 86 packaged artifact hashes, 215 source hashes and 215 source-archive member hashes were reread and
  verified. Collection C/LLVM executables also ran successfully from `/tmp`, outside the checkout.

## Artifacts

Local directory: `dist/lobo-1.0.0-linux-x86_64`. It contains compiler/test binaries, VSIX, source archive,
six checked programs, twelve standalone ELF executables and complete final command logs/manifest.

| Artifact | Bytes |
| --- | ---: |
| `lobo` | 1,338,456 |
| `lobo-test` | 1,154,872 |
| `lobo-syntax-1.0.0.vsix` | 28,571 |
| Collections, baseline or changed, C / LLVM | 43,464 / 43,472 |
| Documents, baseline or changed, C / LLVM | 39,288 / 39,296 |
| Numerical, baseline or changed, C / LLVM | 22,896 / 22,904 |

Compiler uses the existing optimized size/LTO/strip release profile; programs use optimized Clang native
generation. These are exact local file sizes, not runtime performance measurements. C remains the default;
the earlier LLVM comparison has not been rerun or reinterpreted for 1.0.

Reproduction and installation commands: [local release workflow](../../docs/language-1.0.md).
Source/entry mapping and mathematical expectations: [public acceptance fixtures](../../examples/v1/README.md).
Specification: [language 1.0](../../spec/core-v1.0.md).

## Failures, corrections and scope

The [running record](2026-11-04-language-v1-progress.md) preserves intermediate failures: parser/generated-name
bugs, malformed new fixtures, old admission assertions, missed exhaustive matches, native function input
prefix handling, Function-literal artifact admission, nonfinite checked literals, symbolic/concrete instance
accounting, angle/assignment splitting and source-nominal admission. It also records the stale release compiler
used by the first Python run; rebuilding the matching compiler passed the unchanged Python expectations.
Historical baselines and the first-three-families receipt remain unchanged, including their narrower claims.

All final gates passed on 1.0.0 with fixed source inputs. Intermediate source states were not independently
archived for every failed edit/run; raw logs and original/final checkpoints are retained. Final release source,
compiler, tool identities, commands, outputs and hashes are preserved together. Older regenerable Rust test
executables were removed to free disk space; their identities/sizes/hashes are recorded, and source/evidence/
release files were retained. No Git commit/tag, publication, dependency download or installed-profile change.

Supported release platform is Linux x86-64. Function values are pure/non-capturing, numeric behavior uses the
specified floating execution profile, and admission limits remain explicit. External services/libraries/FFI,
capturing closures, concurrency, macros, self-hosting and the workspace remain separate. These public
development fixtures establish scoped conformance, not universal correctness, production readiness, a speedup,
token efficiency, software construction quality or comprehension gains by independently evaluated agents.
