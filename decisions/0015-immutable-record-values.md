# 0015: Add bounded immutable record values

- Status: implemented in compiler 0.4.0 / extension 0.5.0 (bounded record slice)
- Date: 2026-10-17
- Risk mode: standard; value layout/copying/native correctness require adversarial and sanitizer checks.

## Outcome and alternatives

Follow [the language-first goal](../docs/language-completeness.md). Records must compose with existing
functions, conditionals, contracts, equality, and folds, not merely decorate the parser. Multi-value fold
state enables iterative GCD without a new loop construct. Implement nested acyclic records so ordinary
composition is not restricted to a one-off pair type.

Choose immutable by-value semantics and bounded inline native layouts. Reference/heap identity would
introduce ownership, allocation, destruction, and aliasing semantics before the workloads need them;
scalar tuple encoding would preserve the expressiveness gap. Fieldwise equality avoids C padding issues.
Do not claim this bounded slice supplies growable data or completes the general-purpose language goal.

## Rules and compatibility

Follow [Core v0.4](../spec/core-v0.4.md). Constructors preserve written evaluation order; types are nominal;
recursive layouts are rejected. Static layout/copy budgets apply on source and artifact reload. Invocation
validates complete record shapes. No new authority, dependency, garbage collector, or heap runtime is added.
Compiler 0.4.0, Core/artifact 0.4, extension 0.5.0, native adapter 0.2; LTP remains independently proposed 0.2.

## Evidence

On 2026-10-17 (local timezone), the uncommitted working tree passed:

- `cargo test --locked` and `cargo test --locked --release`: all 51 integration tests, including nine record
  tests, inherited Core/action/fold cases, native/reference comparisons, and ASan/UBSan suites.
- Record input/output comparisons through the evaluator, optimized native binaries, and ASan/UBSan builds:
  nested shapes, malformed leaf types/counts, extreme integers, and typed JSON results. The executable GCD
  example also agrees on independently expected results, including a maximum-I64 input.
- Source/artifact rejection for unknown/duplicate/missing fields, nominal mismatches, cycles, layout depth/
  expansion, copy budgets, forged literals, unknown types, and stable-reference tampering. Source metadata
  stays ephemeral; constructor evaluation order and first faults remain observable.
- All 57 editor tests and a real VS Code extension-host check: forward/nested type and field navigation,
  aliases/function returns/fold state, plus compiler squiggles on unsaved `p.missing` code. Navigation remains
  bounded structural type flow, not the authoritative checker or a full language server.
- Rust formatting, Clippy with warnings denied, JSON/JSONL/local-link and diff whitespace checks.
- Release build and VSIX packaging; example swap returns fields 2/1 and native GCD(1071,462) returns 21.

TDD first observed all five initial record cases fail because declarations were unsupported; implementation
made them pass. Adversarial review then produced a failing regression for an unknown return type on a forward
call: body checking indexed an absent layout and panicked. Validating every function signature before any body
fixed it. A grammar regression also caught consumed `->` tokens; explicit punctuation captures restored scopes.
Existing scalar generator changes only adapt exhaustive matches/cloning to the expanded value/type enums;
their expected arithmetic/control-flow outcomes were not changed. Version assertions follow Core v0.4.

Environment: Linux x86_64, Rust/Cargo 1.92.0, Clang 18.1.3, Node.js 22.19.0; real VS Code used a fresh temporary
profile under Xvfb. No dependency, installation into the normal editor profile, commit, publication, benchmark
rerun, or benchmark-worktree change was made. Earlier VSIX files remain available.

Artifacts:

- `target/release/lobo`: 794,928 bytes; SHA-256
  `95ae4daa818c37c5a50abed785260f1aff39350da3054fc3d3cf4ff7cbd8731e`.
- `dist/lobo-syntax-0.5.0.vsix`: 16,004 bytes; SHA-256
  `82cae2e1c72ca88f6be302cc08714729369efc5d3a12be2df6e752beaeca98f1`.

This is not completion of the general-purpose language goal. Alternative/recursive/growable data, text,
structured failure values, and broader program composition remain. The layout/copy limits are explicit
admission rules, not a proof of exact machine stack use, total correctness, or resource isolation.
