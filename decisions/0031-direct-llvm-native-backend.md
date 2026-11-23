# 0031: Direct LLVM application lowering alongside C

Date: 2026-11-04. Status: implemented experimentally in compiler 0.11.1 after the user requested doing this now.
Risk mode: standard; compiler semantics and native memory handling require differential validation.
Supersedes the scheduling recommendation in the earlier LLVM assessment, not its unmeasured-gain limits.

Outcome: compile every implemented Core construct from checked Lobo IR directly into LLVM application
functions, retain a standalone native executable, and expose an explicit backend selector and LLVM
inspection output. C remains the comparison/default until evidence supports changing that default.
No language syntax, checked-artifact format or native invocation contract changes.

Alternatives: Rust bindings to LLVM libraries; textual LLVM IR plus installed LLVM tools; or continuing
only C lowering. Choose textual IR with Clang 18 and llvm-link-18: no new Cargo dependency/unsafe FFI,
inspectable artifacts, and LLVM performs module linking rather than a custom textual linker. Tools are
already installed here. Matching tools become an explicit requirement for this optional backend.

Application arithmetic, branches, short circuiting, loops, calls and contracts lower directly to LLVM.
Reuse the existing C native runtime and typed value/layout primitives through a private pointer ABI.
Clang compiles this runtime to LLVM; llvm-link composes it with the directly emitted application module.
This shares runtime implementation, not a C translation of application bodies or an interpreter.
Entry-block allocations bound stack storage across loops. Typed C assignment/access avoids guessing
aggregate ABI layout; checked inline-width bounds and alignment apply. This initial design has no public FFI.

Invariants: preserve operand order, overflow/zero-divisor diagnostics, contract phase/subject, fuel/heap,
stack and traversal limits, persistent value semantics and argument/output validation. Failure matrix:
invalid source/artifact fails before backend work; missing/mismatched tools and rejected IR are host
failures; output publication refuses existing paths and cleans partial writes; runtime language faults
remain structured outcomes. No silent fallback to C on LLVM failure.

Acceptance: failing-before CLI tests; LLVM verifier checks; native/reference/C agreement on fixtures and
generated independent-oracle cases; active sanitizer instrumentation and representative compound/generic/
recursive workloads; exact resource boundaries; maintained editor/skill/docs. Benchmark optimized builds,
alternate repeated runs, validate results, separate compile/runtime/startup, preserve all raw observations
and report inconclusive differences. No automatic speed/size claim or default migration.

Acceptance passed: 163 Rust, 30 Python harness and 114 editor tests, real VS Code host, strict Clippy,
LLVM verification, active ASan/UBSan probes and skill examples on both backends. Two retained comparison
rounds found no demonstrated runtime advantage on four public workloads. Final LLVM compile medians are
18–31% greater, with identical or eight-byte-larger executables. Keep C as default and LLVM explicitly
available. See [verification and comparison](../benchmarks/results/2026-11-04-llvm-backend-verification.md).

Primary references: [LLVM 18 IR](https://releases.llvm.org/18.1.8/docs/LangRef.html),
[module linking](https://releases.llvm.org/18.1.8/docs/CommandGuide/llvm-link.html),
[Clang ASan](https://releases.llvm.org/18.1.8/tools/clang/docs/AddressSanitizer.html).
