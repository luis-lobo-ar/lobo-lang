# 0003: Target an existing runtime before building a VM

- Status: accepted for prototyping
- Date: 2026-10-10

Execution target sequence updated by [decision 0006](0006-native-machine-code-is-the-default.md): native
machine code is the default and WASM is an optional later backend. The decision to reuse existing tooling remains.

## Context

A custom bytecode and runtime could eventually improve compactness, verification, or execution. Building them now
would consume effort before validating whether Lobo's semantic workspace and tool protocol improve agent work.

## Decision

The first executable Lobo subset will lower to WebAssembly or another established sandboxable target. The team will
consider a custom runtime only after a representative workload exposes a measured limitation that cannot be
addressed reasonably in the existing target.

The first runnable Core slice uses a checked-artifact evaluator before WASM lowering, as recorded in
[decision 0005](0005-checked-artifact-for-first-executable-core.md). WASM remains the first planned backend
for executable code generation; the evaluator is a semantic reference rather than a custom bytecode VM.

## Consequences

- Early work concentrates on agent interaction, state, context, and evidence.
- The runtime boundary must preserve Lobo effects and capabilities rather than inherit ambient host authority.
- Backend-specific behavior cannot silently become language semantics.
- A future custom runtime remains possible, but requires evidence and a migration plan.

