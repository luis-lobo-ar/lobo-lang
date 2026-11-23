# LLVM backend assessment

Date: 2026-11-04. Source/documentation review only; no new compiler implementation or benchmark.

Lobo already uses LLVM indirectly. `Artifact::compile_native` in `src/native.rs` calls `emit_c`,
writes a temporary C translation unit and invokes Clang with `-std=c11 -O3 -g0`, function/data
sections, linker section garbage collection and stripping. The resulting artifact is a native
executable. The separate evaluator remains a correctness reference.

Reviewed `src/native.rs` SHA-256:
`1b922ba3b171a12ae9ee48d9f1fabf5cf3cfc81f92df1cdf4ece0d49b59ca094`.
Existing design: [compiler and backends](../../docs/compiler-and-backends.md).

Clang's [official toolchain description](https://clang.llvm.org/docs/Toolchain.html) states that its
IR generation stage produces LLVM IR and describes optimization/code-generation stages. LLVM's
[frontend tutorial](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl03.html) shows
direct frontend emission of LLVM IR and requires matching documentation to the selected LLVM release.
Both primary sources were consulted on the review date. No secondary-source performance claims adopted.

Recommendation: continue using Clang/LLVM now. A future direct route would be checked Lobo IR to
LLVM IR to native object code/executable. Potential engineering benefits are bypassing C parsing and
more direct control over lowering/metadata; these are architectural inferences, not measured gains.
Direct LLVM integration adds a new lowering implementation and toolchain/version obligations. It does
not itself supply Lobo type/action checks, contracts, capability enforcement or agent context.

An optional future prototype should have an explicit missing-capability or bottleneck target. Retain
the C backend as a comparison, require existing semantic tests to agree across both native routes and
the evaluator, then compare end-to-end compilation time, execution time, artifact size, debugging and
maintenance requirements on pinned workloads/toolchains. Preserve the same checks and budgets.
No automatic speed/size advantage, new dependency, implementation commitment or authorization to migrate
is inferred from this review. The current language-first priority remains.
