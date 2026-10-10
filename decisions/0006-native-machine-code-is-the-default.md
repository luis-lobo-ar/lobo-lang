# 0006: Generate native machine code by default

- Status: accepted and implemented
- Date: 2026-10-10

## Context

The user explicitly requires a compiled language that produces binary machine code. The checked-artifact
evaluator established semantics but did not satisfy that execution requirement. This decision updates the
sequence in decisions 0003 through 0005.

## Decision

`lobo compile` emits an optimized native executable by default. The Rust frontend resolves and checks Core,
lowers it to C with explicit sequencing and checked operations, and invokes Clang's LLVM optimizer and code
generator. Clang is a build-time tool dependency; generated programs require neither Clang nor Lobo at runtime.

The generated C contains statically compiled functions and a small command-line/runtime-check adapter.
It does not contain expression trees, parsing, a bytecode interpreter, or the Rust evaluator. A later direct
LLVM IR backend can replace the C lowering without changing language semantics or the agent workspace design.

`lobo run` compiles and runs native code in a temporary directory. `lobo eval` explicitly selects reference
evaluation for tests. `--emit checked` preserves the versioned IR format and `--emit c` exposes backend input.
WASM is a later optional backend rather than the first production execution target.

## Consequences

- Executables run directly on the build host and use its system libraries.
- Contract checks, overflow faults, evaluation order, and short-circuiting survive code generation.
- Native/reference differential tests are required for language changes.
- Compilation needs Clang on PATH and native tests do not silently skip when it is absent.
- File-size reports distinguish the compiler executable from the executable generated for a Lobo program.
- Performance comparisons use generated native code and separate compilation/startup from repeated execution.

