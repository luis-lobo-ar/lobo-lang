# Compiler and backends

Status: Rust Core frontend, C/Clang and direct LLVM native backends, and reference evaluator implemented.
WASM is planned. The C native route remains the default.

## Native library and OS boundary

The [2026-11-22 comparison](../research/native-interop-2026-11-22.md) explains native interfaces across ten
languages and the proposal at that checkpoint. Compiler 1.7 now implements a closed, typed Linux OS slice:
[capabilities, clock, randomness and loopback TCP](../spec/native-interface-v0.1.md), linked into C and LLVM
executables. The compiler remains Rust; native support is C over OS services. No Rust host is needed for those
operations. General external bindings, TLS, HTTP serving and the Google-auth/Angular application remain
unimplemented. Native provider code is trusted, not sandboxed.

## Implementation language

The reference compiler is written in Rust, as the eventual daemon will be. A shared compiler library exposes parsing,
resolution, type/action checking, contract handling, and lowering. The `lobo` CLI and eventual `lobod` daemon use that library;
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
At the initial implementation checkpoint, the compiler emitted C and invoked Clang; direct LLVM IR was future work.

Subsequent implementation, 2026-11-04: the user explicitly requested doing the direct LLVM work now.
Compiler 0.11.1 implements `--backend llvm` and `--emit llvm`. Application functions emit LLVM directly;
the existing C runtime/layout helpers compile to a separate module, and llvm-link-18 composes them before
final optimization. The earlier scheduling recommendation below is retained as history and superseded by
[decision 0031](../decisions/0031-direct-llvm-native-backend.md). See the
[implemented backend contract](../spec/native-llvm-v0.1.md). No direct-backend speed advantage is presumed.

Review, 2026-11-04: Core 0.11 already uses LLVM indirectly through Clang, with `-O3` in
`src/native.rs`. Keep this backend while evolving language semantics. Direct emission of LLVM IR
would bypass C parsing and provide more direct control over lowering and debug metadata, but neither
faster compilation nor faster/smaller executables has been measured for that alternative. Prototype it
alongside the existing backend only for a concrete missing capability or a measured bottleneck; require
native/reference/C-backend agreement on contracts, checked arithmetic, ordering and resource faults.
This is a recommendation, not an implemented backend or dependency change. See the
[recorded assessment](../benchmarks/results/2026-11-04-llvm-backend-assessment.md).

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
[checked compound-value interchange](../spec/core-v0.7.md#interchange-and-cli), extended by
[state-driven loop IR](../spec/core-v0.8.md#checked-artifact-extension) and
[module ownership/import scopes](../spec/core-v0.9.md#checked-artifact), are defined. Graph interchange and the WASM host ABI
still need definitions before implementation.

Current language release work: Core 0.12 development implements generic records/variants, parametric
definition checking, pure named function values and F64 on the evaluator and both native routes. Function
values dispatch to compiled bodies; direct LLVM emits double arithmetic/comparison instructions without
fast-math. The fixed collection/document/numerical programs and changes pass focused native/sanitizer
acceptance. See [Core 0.12](../spec/core-v0.12.md) and [the release contract](../spec/language-v1-acceptance.md).
Language 1.0 freezes these rules; final package/check status is in the
[release receipt](../benchmarks/results/2026-11-04-language-v1-release.json). No new backend performance claim is made.


## Try propagation in compiler 1.8

Core 1.8 represents prefix try as a checked Unary operation. Result-shape and exact-error checks run in the
shared checker. The evaluator separates typed early returns from runtime faults. C and LLVM branch to their
function's result/postcondition/frame epilogue, so propagated errors still run postconditions and restore
call metadata. No source projection or interpreter is needed for native programs. See
[the language rule](../spec/core-v1.8.md) and [validation](../benchmarks/results/2026-11-22-try-propagation.md).


## Optional SQLite extension

Core 1.9 adds a separately marked SQLite extension to the closed native catalog. See
[the extension contract](../spec/sqlite-interface-v0.1.md),
[SQL helpers](../libraries/sql/README.md) and the
[persistent REST example](../examples/native/sqlite-rest/README.md).
The C and direct LLVM backends compile the same pinned amalgamation and C adapter into the executable.
No SQLite DLL, Rust service or interpreter is needed to run it. Programs that do not use SQLite retain
the earlier native layouts and omit the engine. Source compilation still uses the Rust compiler.

Run the SQLite adapter, library and source-correspondence Cargo tests, the separate
[SQLite lifecycle gate](../verification/sqlite/README.md), editor tests and extension-host checks.
The test corpus covers binding, NULL/empty values, ownership, transaction failure/rollback, denial,
limits, stale handles, allocation/copy faults and independent on-disk SQLite oracles. Linux and Wine
executable runs are distinct from actual Windows hardware coverage. Retained upstream Tcl tests and
adapted cases do not imply that the entire upstream SQLite suite ran.
