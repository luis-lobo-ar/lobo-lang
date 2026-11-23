# Typed operator bindings implementation evidence

Recorded 2026-11-08. This public development workload implements the user's selected operator milestone.
[Decision 0038](../../decisions/0038-explicit-typed-operator-bindings.md) and
[language 1.1](../../spec/core-v1.1.md) define the accepted behavior. The
[machine-readable receipt](2026-11-08-operators.json) records final command outcomes, hashes and preservation.

## Implemented behavior

Language/compiler 1.1.0 supports `operator +(Vec2, Vec2) -> Vec2 = add;` and equivalent bindings for binary
`-`, `*`, `/` and `%`. Targets are local non-generic pure functions with exact signatures. At least one operand
is a non-generic nominal type owned by the binding module. Explicitly importing the target enables its binding.
Each symbol and ordered operand pair has one binding in the closed bundle. The admission bound is 64 bindings.
Text addition lowers to the existing concat intrinsic. Checked artifacts contain ordinary calls and buffer
operations. Definition-time generic checking still rejects arithmetic on rigid unconstrained parameters.

Editor support includes declaration highlighting, named target and overloaded symbol navigation, explicit
source imports and 1.1 compiler response handling. The programming skill documents the rules and includes
executable binding/Text examples. The packaged extension is `dist/lobo-syntax-1.1.0.vsix`.

## Verification and independent expectations

The focused Rust suite tests canonical equivalence to explicit calls, reload/tampering, Text sugar, signature
and purity rejection, module ownership/imports, coherence, admission limits and 1.0 artifact compatibility.
It covers all five symbols, distinct output types, variant operands, concrete operations in generic bodies,
List/Fn signatures, contracts, match/if/while/fold contexts and first-fault ordering. Canonical body equality
establishes identical operation trees for the paired examples. Sampled native/evaluator executions compare
values and faults across fuel/heap budgets. Differing caller IDs are checked before normalization. Callee IDs
must match exactly. Pure operation counts follow the canonical tree.

The existing `Builds` harness executes the evaluator, C, application LLVM, C ASan/UBSan and LLVM ASan/UBSan.
Its C invocation also compares both LLVM results. The final receipt records full Rust/editor/Python counts,
Clippy with warnings denied, formatting, skill validation, local documentation links and real VS Code host
completion. `tests/operators.rs` retains the focused behavior checks. The real editor-host test invokes the
definition provider on an overloaded symbol and verifies its target location.

The optimized CLI compiles `examples/operators.lobo` into checked output and two ELF executables. Both native
routes return Vec2 with x=7 and y=10, and `Hello, é!` for the UTF-8 argument. Each example binary is 22,904 bytes.
The optimized compiler is 1,368,360 bytes. These sizes describe these artifacts on this host. Execution speed
was unmeasured. Native skill examples also return Pair(4,6) and `<🐺>` through both backends.

All 86 artifact hashes in the frozen 1.0 manifest were reverified. Its manifest SHA-256 remains
`f89af5f98e4d79e62cb3d5cfd37801b7539671d0826cca25ba63ed9fa039cf7a`.
The current compiler also accepts a checked file from that release through its ordinary CLI entry point.

## Failed attempts and corrections

- All four initial operator tests failed at `operator` parsing before implementation. Raw output is
  `target/operators-red.log`.
- The first implementation passed three tests. Its differential test compared different caller IDs as equal.
  The assertion now verifies the appropriate caller identity and normalizes that intentional difference.
  `target/operators-green-attempt1.log` and `operators-green-attempt2.log` retain both outcomes.
- Initial editor tests failed declaration highlighting and local/project symbol navigation. The corresponding
  implementation made them pass. `target/operators-editor-red.log` retains the initial failures.
- Full Rust attempts first caught version-label assertions, then a match diagnostic regression. The version
  assertions now expect core/1.1. Lowering preserves `compile.match.invalid`; the existing ten-test variant
  suite passed after correction. Both full attempts and the focused regression log are retained.
- The initial full editor run rejected core/1.1 success responses. The adapter's supported-version list now
  includes 1.1. A subsequent generic local-binding projection test initially ran against a pre-fix debug
  compiler and reported unavailable. Rebuilt compiler diagnostics retain their defining source and range.
- The first Python run used the older release compiler on the newly updated skill source and failed one
  example check. Building compiler 1.1 before dependent checks resolved the failure.
- A new multiline-target grammar assertion failed before the target region gained comment/newline support.
  `target/operators-editor-multiline-red.log` and `operators-editor-multiline-green.log` retain both outcomes.
- The first VSIX packaging command rejected a repository-relative README link. The packaged README now names
  the specification path in the compiler checkout. Final packaging succeeded.
- Three patch-validation attempts rejected unmatched or incomplete hunks before mutation. The edits were
  reapplied against inspected file content. The interaction transcript retains these tool failures.

## Reproduction and limits

Run `cargo test --locked --test operators`, `cargo test --locked`, and `npm --prefix editors/vscode test` after
building the debug compiler. Run `cargo build --locked --release --bin lobo --bin lobo-test` before the Python
suite because its skill examples invoke the release compiler. The receipt supplies exact remaining commands,
tool versions, preserved raw logs and source/artifact hashes. Recompile the executable example with
`target/release/lobo compile examples/operators.lobo --backend c -o NEW_BINARY`, then invoke
`NEW_BINARY --entry example.operators.combined`.

The scope selects five binary arithmetic symbols, exact concrete bindings and fixed precedence. Generic
nominal binding signatures and operation constraints remain future work. Navigation is a structural projection;
the compiler validates programs. The wider release script in `scripts/release.py` retains its fixed 1.0
acceptance contract. This milestone records the 1.1 compiler build, tests and extension package separately.

Agent software-construction quality, comprehension, repair efficiency, token usage and runtime speed remain
unmeasured for this feature. These public examples are development evidence for a future study. The source
snapshot, native programs and logs are retained locally under `benchmarks/private-evidence/operators-20261108`.
