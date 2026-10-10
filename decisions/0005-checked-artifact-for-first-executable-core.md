# 0005: Emit a checked artifact before adding machine code generation

- Status: accepted for the first prototype
- Date: 2026-10-10

Default execution updated by [decision 0006](0006-native-machine-code-is-the-default.md): native executables
are now the primary output; checked artifacts remain available for inspection and reference testing.

## Context

The minimum runnable version needs an actual compiler output, stable call references, and a reloadable program
representation. WASM generation additionally needs a host ABI; the final graph needs its own identity schema.

## Decision

The first compiler emits the experimental `lobo-checked/0.1` JSON expression-tree format and executes it through
a pure reference evaluator. Explicit calls resolve to stable function IDs. This is an intermediate compiler
target before WASM, not a custom bytecode VM or the final semantic workspace graph.

Add Serde and serde_json for strict typed artifact serialization and interoperable JSON CLI responses. These
dependencies address the immediate need to encode, reload, validate, and test checked artifacts; pin resolution
in the committed lockfile. Readers validate versions and recheck program semantics before execution.

## Consequences

- Source checking, compilation, artifact loading, and execution can be demonstrated immediately.
- Conformance cases can compare direct and reloaded execution.
- Local expression nodes do not yet have stable workspace identities.
- No native, WASM, static-proof, or self-hosting support is implied.
- A later graph format must migrate explicitly rather than treat these trees as final canonical storage.

