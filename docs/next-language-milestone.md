# Deferred proposal for a later language milestone

Date: 2026-11-04. Status: deferred after the user's scope correction. Not an active requirement,
implemented feature or frozen study protocol. [Decision 0032](../decisions/0032-stop-feature-growth-at-the-evaluation-checkpoint.md)
supersedes the immediate recommendation below. The remaining text preserves that proposal as history.
The later [finite future scope](future-language-scope.md) defined a closed four-family candidate. The user
subsequently authorized that scope through local language 1.0 in decision 0033. This earlier proposal remains
historical; the active implementation follows the fixed language-v1 acceptance contract instead.
Preserved checkpoint: compiler 0.11.1, Core 0.11, explicit generic functions and two native backends.
C remains default. The [paper evidence](../research/local-evidence.md) and its frozen bundle remain unchanged.

## Generic data declarations address the next demonstrated gap

Add type parameters to ordinary records and variants. Existing generic functions cannot define reusable
compound loop state or a reusable typed result. The [sequence helper](../examples/sequences.lobo) still
uses an I64-specific Cursor. The [generic reuse workload](../spec/generic-reuse-workload.md) uses recursive
helpers, which retain the existing 32-call limit. A generic Cursor would let the same iterative algorithm
carry two lists across different element types without duplicating its data declaration.

Illustrative declarations only. The current compiler does not accept this syntax:

```lobo
record Cursor<T> @id("reuse.cursor") { remaining: List<T>, reversed: List<T> }
variant Option<T> @id("reuse.option") { Some(T), None }
variant Result<T, E> @id("reuse.result") { Ok(T), Err(E) }
```

These are ordinary user-defined types, not new special-purpose keywords, host libraries or implicit imports.
[Rust's generic data types](https://doc.rust-lang.org/book/ch10-01-syntax.html) and
[Result](https://doc.rust-lang.org/std/result/) supply established comparison baselines. Their existence
does not establish an agent benefit for Lobo. The justification here is the concrete reuse/iteration gap.

## Use one public workload to set the scope

Proposed acceptance targets, to freeze before implementation:

- One iterative reverse implementation and generic Cursor used with I64, Text and caller-owned records
  across modules. Preserve value order, nominal identity, action bounds and contracts.
- One optional lookup operation returning `Option<T>` and one validation operation returning
  `Result<T, E>`, with explicit exhaustive handling of success and failure.
- Empty/singleton inputs, UTF-8 values, nested instantiated types, distinct same-spelled nominal types
  in different modules, malformed artifacts, rejected inline cycles and bounded instantiation growth.
- An internally generated 10,000-element case using iteration, with budgets fixed before the run.
  It must not depend on 10,000 recursive calls or a host implementation of the algorithm.
- Definition navigation for type parameters, instantiated fields/cases and imported declarations.
  Maintain source diagnostics, the agent skill and ordinary executable examples.

Specify constructor/call spelling, type identity, module visibility, recursive layout rules, artifact
versioning, allocation and compiler admission before production edits. Avoid adding inference, traits,
closures or another runtime in this slice. Keep both native routes and the evaluator as verification paths.

## Keep generic checking as an explicit design decision

Core 0.11 checks local names and body types per requested specialization. Adding generic data types must
not be described as fixing that limitation. Examine declaration-time checking of universally valid generic
operations as a separate next slice. Define what happens to existing type-dependent templates before
changing their acceptance. Do not add a trait system merely to copy another language.

Function references become a later candidate only when a workload needs the same collection algorithm
with several behaviors and named helpers cause demonstrated duplication. Capturing closures and their
lifetime/action rules require separate evidence and design. Extra numerical types or collections likewise
follow actual workload needs.

## End the slice at a measurable checkpoint

Follow the existing TDD workflow: freeze requirements/oracles, observe focused failures, implement the
smallest mechanism, run native/reference/sanitizer/artifact/editor checks, and record every correction.
Preserve the pre-change compiler/source baseline. Finish with one small multi-module application using
the reusable state/result types, such as an in-memory document validator and transformation.

At that checkpoint, reassess breadth before adding more constructs. A later bounded agent pilot should
measure construction acceptance, correct evolution and code comprehension separately. Keep prediction,
dependency understanding, fault localization and change impact distinct from passing tests. Whole-trajectory
tokens/time are costs, not substitutes for those outcomes. Use independent held-out requirements and
language/tool/documentation controls from the [paper plan](../research/paper-plan.md).

This recommendation launches no compiler implementation, model trial, dependency installation, ecosystem
work or backend migration. There is no new performance, construction or comprehension result.
