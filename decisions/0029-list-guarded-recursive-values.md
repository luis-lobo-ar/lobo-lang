# 0029: Admit recursive nominal types through existing list storage

- Date: 2026-11-04
- Status: implemented in Core/compiler 0.10 and extension 0.12; verification evidence recorded below
- Risk mode: standard, with native memory and stack-safety gates

The user authorized workload-driven language evolution before broad comparisons. A nested syntax-tree
transformation needs recursive child values. Existing node tables remain valid, but require separate
index/link invariants. Preserve the Core 0.9 baseline and implement a small recursive-value slice.

Alternatives considered locally, not independent agent designs:

1. Automatically box recursive records/variants. This changes existing native copying/layout semantics.
2. Add an explicit Box type. This introduces another allocation/value constructor and memory contract.
3. Permit cycles only across List edges. List storage already provides immutable heap indirection.
4. Retain only node tables. This does not expose recursive typed child values to the development workload.

Choose 3. After removing List edges, the nominal dependency graph must be acyclic. Direct inline cycles
remain errors. Width/inline-height are computed without traversing list elements; list element types
are separately validated. Type spelling stays bounded. Heap allocation retains the current cell model.
No new pointer syntax, collector, dependency, host authority or implicit constructor is introduced.

Native code emission must separate physical type definitions from helper declarations and definitions,
so recursive parse/print/count/equality helpers have prototypes. Input/output logical nesting remains
eight. Structural equality is bounded to 64 active compound values, with a classified stack diagnostic;
long flat lists stay iterative. Reference reclamation must be iterative across nested lists/compounds.

Compatibility: Core/compiler/artifact advance to 0.10; old checked artifacts require recompilation.
Native adapter advances to 0.6 to specify recursive shapes and traversal limits. Existing source and
generated binaries remain meaningful under their original versions. Syntax highlighting needs no new
tokens, but recursive navigation/compiler fixtures, extension compatibility and the Lobo skill move together.

The public AST workload is development evidence, not a held-out agent benchmark. This slice does not
implement arbitrary inline recursion, user generics, function values, formal proofs or the semantic daemon.

Baseline: [preservation record](../benchmarks/results/2026-11-04-core09-language-baseline.json).
Acceptance: [Core v0.10](../spec/core-v0.10.md) and [AST workload](../spec/recursive-ast-workload.md).

Implementation: physical layouts and structural helper declarations are emitted in separate phases.
Reference list reclamation drains nested compound values iteratively; native reclamation retains
the existing invocation-owned allocation list. Depth guards classify input/output/comparison exhaustion.
No new runtime dependency or allocator was added. The editor's old compiler-success allowlist failed
on 0.10 during TDD and now accepts the new declared version while retaining old-version controls.

Evidence: [native smoke](../benchmarks/results/2026-11-04-recursive-ast-native.json),
[verification report](../benchmarks/results/2026-11-04-core010-verification.md).
The original Core 0.9 tests that rejected List cycles were updated for the declared new semantics;
direct inline-cycle and forged-artifact rejection remain tested. Numeric workload expectations were
not changed in response to implementation output. No agent benefit or general safety proof is claimed.
