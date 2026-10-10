# Compiler and backends

Status: Rust Core frontend, native C/Clang backend, and reference evaluator implemented. WASM is planned.

## Implementation language

The reference compiler is written in Rust, as the eventual daemon will be. A shared compiler library exposes parsing,
resolution, type checking, contract handling, and lowering. The `lobo` CLI and `lobod` daemon call that library;
neither duplicates the language rules.

Stable node IDs and centrally owned graph storage avoid making the canonical representation depend on object
addresses or reference lifetimes. Rust implementation types are an implementation detail, separate from the
published Lobo graph format.

## Pipeline

```text
source projection → parse → resolve IDs → check types/contracts → checked Lobo IR
graph import ─────────────→ validate ────────────────────────────┘
                                                               │
                                ┌──────────────────────────────┼─────────────────────┐
                                ▼                              ▼                     ▼
                         reference evaluator             C lowering           future WASM lowering
                                │                              │                     │
                         test/reference behavior        Clang/LLVM → native     portable artifact
```

Graph import begins only once its versioned format is specified. The checked intermediate representation (IR)
contains resolved references and validated types. Its interpreter and backends obey the same language
specification. Differential tests can reveal disagreements, although shared frontend code can hide shared bugs.

The interpreter is a small evaluation engine for semantic tests. It should follow the specification directly
and avoid reusing the backend's arithmetic and control-flow transformations as its expected-result oracle.

## What LLVM provides

LLVM is a collection of compiler tools and libraries. A frontend can translate a language into LLVM IR, use
optimization passes, and generate machine code for supported processors. LLVM also provides facilities for
object files and debug information. It does not define Lobo's types, contracts, capabilities, or agent protocol.
See the official [getting started guide](https://llvm.org/docs/GettingStarted.html) and
[IR reference](https://llvm.org/docs/LangRef.html).

| Name | Role in Lobo |
| --- | --- |
| Rust | Language used to implement the initial compiler |
| Lobo checked IR | Internal representation preserving Lobo semantics |
| LLVM | Optimization and machine-code generation through the Clang backend |
| WebAssembly (WASM) | Planned additional output format |
| WASM runtime | Host that executes WASM with configured imports and limits |

LLVM reduces the work of native code generation and supports optimized application code through Clang. Direct
LLVM library integration would add dependency and toolchain version management. Whether
Rust itself uses LLVM does not require Lobo's compiler to link to LLVM or emit LLVM IR.

WASM and LLVM are not mutually exclusive: LLVM can target WASM. Native execution is now the default, following
the user's requirement recorded in [decision 0006](../decisions/0006-native-machine-code-is-the-default.md).
The current compiler emits C and invokes Clang. A direct LLVM IR backend remains possible later.

## Backend correctness

A backend must preserve specified overflow faults, evaluation order, short-circuiting, and contract checks.
It cannot emit an unchecked host operation and silently inherit behavior that differs from Core.

For example, LLVM optimization flags that assume signed overflow never occurs would be invalid unless the
frontend has established that assumption and otherwise emits the required checks. A contract observed in tests
is not permission to assume it universally. Backend validation establishes artifact validity, not semantic
equivalence to the source program.

WASM execution receives only explicitly configured imports. The output format alone does not establish Lobo's
capability policy; the runner must enforce allowed access and resource limits.

## First deliverable

The compiler implements parsing, resolution, type checking, native lowering, and reference evaluation.
`compile` produces native machine code; `--emit checked` exposes the checked representation whose calls target
stable IDs. Native/reference tests exercise the Core corpus and boundary cases. `run` compiles and executes
native code; `eval` explicitly invokes the reference evaluator.

The [native invocation interface](../spec/native-v0.1.md) and
[checked artifact layout](../spec/checked-artifact-v0.1.md) are defined. Graph interchange and the WASM host ABI
still need definitions before implementation.

