# 0014: Prioritize general-purpose language expressiveness before the ecosystem

- Status: accepted current goal; implementation remains incremental
- Date: 2026-10-17
- Risk mode: standard for a consequential priority decision; no runtime behavior is changed here.

## Context and alternatives

The user wants Lobo to replace other languages at the level of expressing programs, without copying every
construct those languages offer. Additional constructs should follow evidence that recurring patterns benefit.
The user explicitly set this as the current goal and deferred the ecosystem.

The earlier roadmap favored a semantic-workspace vertical slice after scalar Core. Another option would
develop communications/storage/search and FFI first. Neither addresses the current inability to express
ordinary structured computation without scalar encodings or foreign application logic.

## Decision

Adopt [the language-completeness goal](../docs/language-completeness.md): a small composable language core,
demonstrated on several independent self-contained computational workloads, with native compilation and
TDD. Specify data representation and memory/lifetime behavior before heap-backed features; begin with a
bounded record-value slice, then grow the model from concrete needs. This selects a direction, not final syntax.

Defer ecosystem integrations and the workspace daemon. Preserve the long-term agent-oriented product vision,
explicit authority boundaries, diagnostics, conformance, and maintained editor support. Do not conflate
computational expressiveness, Turing completeness, application readiness, and mature ecosystem support.
Language expressiveness is not a promise to duplicate all hardware/domain-specific facilities or performance.

This supersedes the immediate workspace-first sequencing in the roadmap and agent guidance, not the canonical
graph decision or existing Core semantics. Compiler 0.3.0, Core/artifact 0.3, extension 0.4.0, native adapter
0.1, and proposed LTP 0.2 are unchanged by this planning change. No new language feature is implemented here.

## Evidence and follow-through

Maintain the goal in the roadmap, README, agent guidance, and change ledger. Workload cases need exact
acceptance fixtures before implementation; the example categories are not a completed corpus. New language
slices retain spec-first/TDD, native/reference checking, failure/limit cases, versioning, and editor gates.

Validation on 2026-10-17: `cargo test --locked` passed all 42 integration tests, including native/reference
and sanitizer suites; `scripts/check.sh` validated JSON, JSONL, and local Markdown links; diff whitespace
checks passed. A fresh-reader review found no conflicting priority guidance or confusion between supported
features, planned slices, and readiness claims. No compiler/extension implementation changed, so extension
host tests, packaging, release builds, and benchmarks were not rerun for this planning-only update.

Follow-up on 2026-10-18: the [language-only audit](../docs/language-completeness.md#language-only-requirement-audit--2026-10-18)
records evidence for structured/growable data, computation, failures, shared modules, native/reference gates
and explicit-project editor tooling. Decision 0023 and the ledger retain exact results and residual limits.
The original planning record above remains historical. This milestone does not authorize ecosystem expansion
or establish universal replacement, Turing completeness, application readiness or the workspace v1.0 target.
