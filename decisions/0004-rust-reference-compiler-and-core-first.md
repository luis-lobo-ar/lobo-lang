# 0004: Build a Rust reference compiler against a small Core specification

- Status: accepted for prototyping
- Date: 2026-10-10

Backend sequence updated by [decision 0006](0006-native-machine-code-is-the-default.md). Rust, Core-first
specification, and the reference evaluator remain; native C/Clang output now precedes WASM.

## Context

The initial roadmap placed semantic workspace tooling ahead of executable language semantics. Compiler
implementation and a later compiler written in Lobo require precise behavior and a shared conformance oracle.

## Decision

Define Lobo Core first and implement its reference compiler in Rust as a library shared by the CLI and daemon.
Start with a reference evaluator, then add direct WASM lowering. Keep checked language IR independent from the
backend. Specify graph interchange before claiming graph import support.

Use specification-driven TDD for implemented rules and regressions, complemented by differential tests,
properties, fuzzing, reproducibility checks, and eventually bootstrap tests. Full self-hosting remains a later
milestone requiring a larger specified language subset.

LLVM is an optional later backend for native optimization and code generation. It is not a dependency of the
first prototype and no dependency is added by this decision.

## Consequences

- Conformance fixtures have expected behavior before compiler internals are written.
- Agents can distinguish implemented Core features from wider design examples.
- Early executable programs are pure, making semantic comparisons easier to isolate.
- Workspace operations are developed over tested program semantics.
- Rust remains a reference during self-hosting; compiler agreement alone is insufficient evidence.

## Alternatives and review condition

A workspace-first prototype would test agent interactions earlier, but leave interpretation of program behavior
undefined. A LLVM-first compiler would provide native generation earlier, but increase integration effort before
language conformance is established. Revisit the sequence if measured workloads show that these costs obstruct
the agent-workflow experiment.

