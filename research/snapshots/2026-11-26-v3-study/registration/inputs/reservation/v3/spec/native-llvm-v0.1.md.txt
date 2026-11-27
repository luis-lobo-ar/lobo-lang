# Direct LLVM native backend v0.1

Status: implemented experimentally in compiler 0.11.1 on Linux x86_64, LLVM/Clang 18.
Core remains `core/0.11-draft`, checked artifacts remain `lobo-checked/0.11`, native adapter remains 0.6.
[Decision 0031](../decisions/0031-direct-llvm-native-backend.md) records the user-authorized change of schedule.

## Invocation and tools

```sh
lobo compile examples/generics.lobo --backend llvm -o generics-llvm
./generics-llvm --entry example.generics.text --arg Text:c3a9
lobo run examples/iteration.lobo --backend llvm --entry example.sum --arg I64:10
lobo compile examples/ast.lobo --emit llvm -o ast.ll
opt-18 -passes=verify -disable-output ast.ll
```

`--backend c|llvm` selects native compilation for `compile` and `run`; default is `c`.
`--emit llvm` produces a complete linked textual LLVM module and invokes host compilation tools.
Do not combine `--backend` with non-native `--emit` modes. Existing explicit source bundles work unchanged.
Unknown/repeated selectors fail with `invocation.cli.invalid`. Native/inspection outputs require new paths.
LLVM has no automatic fallback to C: a missing tool is `invocation.native.tool_unavailable`, and a tool
rejecting input is `invocation.native.backend_failed`. Existing publication/partial-write cleanup applies.

The optional route requires installed `clang-18` and `llvm-link-18` executables on PATH; development
verification also uses `opt-18`. Versions are deliberately explicit. No tool download, Cargo LLVM binding,
JIT, interpreter or runtime LLVM installation is introduced. Executables run with the same system-library
requirements as the C backend. `check`, `--emit checked` and `--emit c` do not gain LLVM tool requirements.

## Lowering and runtime boundary

Checked Lobo functions lower directly to LLVM functions. Signed arithmetic uses LLVM overflow intrinsics
with explicit fault branches; division/remainder guard zero and minimum-I64/-1 before LLVM instructions.
Branches, short circuiting, calls, matches, folds, while loops, contracts and fuel checks form native LLVM
control flow. Every temporary value/context allocation is at function entry, not inside iteration blocks.
No source expression tree is embedded for execution.

Keep the existing C runtime for invocation parsing, output, allocation, persistent lists/buffers, structural
comparison and typed record/variant/layout primitives. Private pointer-based ABI helpers copy/access concrete
C values using their actual types; LLVM reserves aligned slots bounded by the checked inline-width model.
This avoids guessing platform aggregate calling conventions. The C wrappers around entry functions only
adapt typed arguments/results to pointers; they do not contain the Lobo application's bodies.

Clang emits the runtime module with `-O1 -Xclang -disable-llvm-passes -S -emit-llvm`. LLVM's linker composes it
with the directly emitted application module. Final Clang optimization/code generation uses `-O3`, section
garbage collection and stripping, matching normal C native output settings. C-frontend optimization passes
are disabled in the runtime emission step so final optimization sees both modules together.

Value semantics, operand order, action checks, contract subject/outermost phase, fuel, heap, stack, equality
and input/output limits retain the current native rules. Runtime implementation is shared with C, so
backend agreement is not an independent proof of runtime correctness. The evaluator and independent test
oracles remain required. No OS sandbox, capability grant, public FFI or debug-information support is added.

## Inspection and development interfaces

Rust callers can select `NativeBackend::{C,Llvm}` through `compile_native_with_backend` and
`run_native_with_backend`; existing methods retain their C default. `emit_llvm` returns linked LLVM text.
`emit_llvm_sanitized` and `compile_llvm_sanitized` are explicit development interfaces. Production modules
carry no ASan function attributes. Sanitized application functions carry `sanitize_address`; Clang's final
pipeline instruments them and the runtime. UBSan checks originate in the C runtime/layout frontend;
hand-emitted LLVM arithmetic has explicit Lobo guards, not automatically inserted C-frontend UBSan checks.
Active stack-write and misalignment probes must demonstrate instrumentation before claiming coverage.

The agent verification-session prototype continues using its configured C/Clang path. This backend is
available to agents through the explicit compiler CLI/library APIs; no session-cache semantics are changed.
Editor highlighting/navigation is unchanged because source syntax/scopes are unchanged. Existing compiler
diagnostics continue reporting Core 0.11, accepted by extension 0.13.0.

## Acceptance and comparison

Shared native fixtures compare C output with optimized and sanitized LLVM output on every native invocation.
The generated independent-oracle corpus and algorithm boundaries also execute both routes. Include native
CLI/IR verification, operand/fault order, contracts, exact fuel/heap boundaries, recursive/mutual data,
modules, generics, deep loops and resource exhaustion. Finite tests do not establish universal equivalence.

`benchmarks/llvm-backend.py` freezes public sum/parser/AST/generic inputs and independent expected outputs,
alternates repeated optimized builds/runs, records source/tool/artifact identities and rejects failed results.
Runtime measurements include startup, parsing and JSON; tiny tasks may be dominated by these costs.
Keep compile latency, runtime, binary size and maintenance tradeoffs separate when choosing a default.
