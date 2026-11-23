# 0016: Tagged alternatives with exhaustive match

- Status: implemented in compiler 0.5.0 / extension 0.6.0 (bounded alternatives)
- Date: 2026-10-17
- Risk mode: standard; nominal checking, mixed inline layouts, active union reads, and failure semantics.

## Outcome and alternatives

Implement ordinary typed failure returns and alternative state without scalar error codes or invalid record
field combinations. Follow [Core v0.5](../spec/core-v0.5.md). Keep one underlying mechanism for success/error
results and other alternatives: user-declared tagged variants plus exhaustive expression matching.

Compared options (local design analysis, not independent implementations):

- Status-code records permit contradictory states and require unchecked conventions for field validity.
- Exceptions add a second control/failure path and implicit propagation; they do not model ordinary alternatives.
- Tagged alternatives make valid states explicit and require handling all cases; this is the keeper.

Use one optional typed payload per case; existing records supply multiple fields. No separate Result/Option
primitive, generics, wildcard, guards, nested patterns, or propagation operator is justified yet. Keep immutable
acyclic inline layouts; recursive/growable ownership is a later, separate decision. Match checks every arm for
types/actions/purity but evaluates just one. Internal faults/exhaustion remain diagnostics, not caught values.

## Verification plan and compatibility

Specify inputs before implementation, demonstrate failing tests, then verify source/artifact validation,
known outcomes, optimized/reference differential cases and sanitizer builds. Exercise nullable payloads,
mixed nested layouts, invalid active cases/shape, scopes, evaluation/fuel order, and contracts. Update the
extension in the same slice, run its tests and real host, then package. No dependencies or external authority.
Compiler/Core/artifact advance to 0.5; native adapter to 0.3. Previously built binaries remain independent.
Normal editor installation, publication, commits, and benchmark reruns are not part of this local change.
The contextual `match (...) {` prefix requires parentheses around ordinary calls followed by a block;
the specification records this source-compatibility boundary rather than claiming all old prefixes survive.

## Adversarial findings

The initial three language tests failed on unsupported syntax; implementation made them pass. Additional
checks exposed an unchecked nominal-classification walk over a 3,900-field chain: the library test thread
overflowed its stack. Parsing now bounds field chains, and classification/resolution enforce expression depth
before descending. A permanent regression also covers deeply nested operator trees. No oracle was weakened.

Full-suite reruns twice failed to launch native binaries with Linux `ExecutableFileBusy`, in unrelated existing
scalar/action fixtures. A focused writer-held-executable test reproduced that host error deterministically.
The adapter now retries only failed busy launches, at most 20 times with 5 ms spacing. Temporary writers clear;
persistent busy files and missing binaries still fail. Program results are never retried. Concurrent writer
inheritance is a plausible cause of the observed transient failure, not a proven trace of those specific runs.

## Verification evidence

The final local working tree on 2026-10-17 passed:

- `cargo test --locked` and `cargo test --locked --release`: 61 integration tests plus the native-launch unit
  regression in each profile, including ten variant tests and all inherited scalar/action/fold/record cases.
- Independent division outcomes, nested alternatives/records/fold state, equality, invocation shapes, selected
  arm and exact fuel behavior agree across reference, optimized native, and ASan/UBSan builds. The executable
  [variants example](../examples/variants.lobo) exercises success, zero divisor, and min-I64/-1 overflow.
- Rejections cover missing/duplicate/unknown cases, wrong binding/payload/arm types, mixed inline cycles,
  nominal kind/ID tampering, injected compound literals, scopes, action bounds and contract purity.
  Static helper expansion and layout/depth limits have dedicated regressions.
- All 63 Node/editor tests, plus the real VS Code extension-host completion marker and exit zero. Unsaved
  variant type/case/payload binding navigation and changed-arm compiler diagnostics work. The resolver remains
  bounded structural navigation, not compiler-owned symbol output or a complete type checker. TextMate match
  prefix highlighting is intentionally conservative; see the extension README.
- Formatting, Clippy with warnings denied, JSON/JSONL/local Markdown links, and diff whitespace checks.
- Release compiler and VSIX builds. Release smoke checks report Core 0.5, divide(20,0) returns the ZeroDivisor
  value, and fallback(Ok(5)) returns 5. Earlier native executables and VSIX packages are retained.

Environment: Linux x86_64, Rust/Cargo 1.92.0, Clang 18.1.3, Node.js 22.19.0, installed VS Code under Xvfb
with fresh temporary profiles. No new dependency, normal-profile installation, commit, publication, or
benchmark/worktree change. The transient failure and deterministic red/green evidence are recorded above,
not omitted from the verification history.

Artifacts:

- `target/release/lobo`: 865,984 bytes, SHA-256
  `d772024745ddc0d222263c5bc6ac33cfd17c36be3c2be624444768b3782d4d0f`.
- `dist/lobo-syntax-0.6.0.vsix`: 17,282 bytes, SHA-256
  `c4179caedc9cb610b538fd9b93932587988a0c3a382e51713aaa641868ea30a3`.

The wider language-first goal remains active: growable/recursive data, general text/bytes, representative
acceptance workloads, and broader reuse/composition are still required. Finite tests are not a correctness
proof or a Turing-completeness argument, and native execution is not an OS sandbox.
