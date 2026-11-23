# 0045 Infer generic call arguments and contextual callbacks

Date 2026-11-22. Status implemented. Risk mode standard.

The user selected `respond_json(find_user(id), render_user)` in place of an explicit type application and
function-reference marker. The selected scope is local generic call inference, contextual named functions,
TDD, editor maintenance, documentation and the decision ledger. No framework, new runtime or dependency is added.

## Alternatives and guarantees

Retaining only explicit calls rejects the requested outcome. MVC-specific sugar would embed one web pattern
in the grammar. General bidirectional inference across signatures, constructors and whole programs would
expand the selected scope and introduce further ambiguity and compatibility choices.

Select a bounded source elaboration pass after module resolution and before the existing generic pipeline.
Arguments with known types/signatures determine all outer generic parameters. Complete expected Fn signatures
resolve callback references. Existing definition checking and specialization remain authoritative. This is a
single implementation with self-review and differential tests, not independent candidate implementations.

The [1.4 specification](../spec/core-v1.4.md) defines accepted contexts, generic-callback limitations and errors.
Explicit calls remain supported. Modern callers may infer legacy imports, while frozen legacy syntax retains
its explicit semantics. Loaded checked artifacts do not run source inference. Current native backends and
runtime contracts remain unchanged. Type inference does not allocate or regenerate persistent identities.

## Failure matrix and acceptance

| Input or condition | Required result |
| --- | --- |
| Fully determined ordinary or callback argument types | Same checked program and specialization IDs as explicit spelling |
| Unresolved parameters | `compile.generic.inference`; request explicit type arguments |
| Conflicting nominal, primitive or callback types | `compile.type.mismatch` |
| Actionful callback | Existing `compile.action.reference` rejection |
| Unused malformed generic definition | Parametric rejection |
| Nested analysis or synthesized type growth | Bounded `resource.frontend.limit` |
| Imported alias or changed source order | Same scoped target and deterministic result |
| Editor edits or invalid callback | Original-source navigation and compiler squiggles; stale snapshots invalidated |

Failing-first compiler/editor cases and explicit/inferred checked-code equality drive implementation.
An additional failing nesting regression catches repeated analysis of already elaborated arguments and is
fixed before acceptance. Self-review adds substitution bounds before allocation and a growing-signature probe.
Native C/LLVM and sanitizer checks cover the generic MVC's 200/404 branches.

The maintained generic MVC has its own tracked IDs. The original simple MVC, legacy comparison, migration
manifest, frozen acceptance programs and historical study fixtures remain unchanged. Skill examples retain
existing IDs while demonstrating inference. Explicit source/reference spellings remain useful for cases outside
the bounded solver. This does not claim full Hindley-Milner inference or generic constructor inference.

The first full suite exposes a collision in timestamp-only test scratch directories. A process-local atomic
sequence now supplements their names. This is test infrastructure, separate from compiler semantics.
The [result record](../benchmarks/results/2026-11-22-call-inference.md) records verification and corrections.
No agent construction-quality, comprehension, token-efficiency or runtime-speed effect has been measured.
Stop at this selected language 1.4 scope.
