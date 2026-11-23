# Roadmap

The current user-selected priority is [general-purpose language expressiveness](language-completeness.md),
before ecosystem and workspace expansion. Prefer a small composable core; add constructs only when demonstrated
patterns justify them. Native compilation, TDD, and editor support remain required. See
[decision 0014](../decisions/0014-language-first-development.md).

Active bounded release, 2026-11-04: the user authorized exactly four families through local language 1.0.
[Decision 0033](../decisions/0033-bounded-language-1-release.md) and the
[fixed acceptance contract](../spec/language-v1-acceptance.md) govern completion. Core 0.12 development
implements generic data, definition-time checking, pure function values and F64. All three programs/changes
pass focused acceptance. Language 1.0 freezes these rules; final package/check status is in the
[release receipt](../benchmarks/results/2026-11-04-language-v1-release.json). No ecosystem/workspace expansion follows automatically.

Preserved checkpoint, 2026-11-04: Core 0.10 adds List-guarded recursive values, Core 0.11 adds explicit
generic functions, and compiler 0.11.1 adds optional direct LLVM lowering. C remains default after the
recorded backend comparison. **This is the stopping point for proactive language feature growth.**
[Decision 0032](../decisions/0032-stop-feature-growth-at-the-evaluation-checkpoint.md) fixes the evaluation
checkpoint and scope rule. The [generic-data proposal](next-language-milestone.md) is deferred, not a
prerequisite. Continue with existing-specification fixes, review and evaluation design; record missing
capabilities without automatically implementing them. Earlier milestones retain their historical scope.

The user-requested [future additions and stopping points](future-language-scope.md) first defined the closed
candidate, then the user explicitly selected it. The earlier checkpoint remains available for evaluation;
neither later implementation nor release completion turns new features into prerequisites for that study.

## Language-first sequence and checkpoint

1. Structured values and their memory/lifetime semantics, beginning with a bounded record-value slice.
2. Alternative values and explicit failure handling.
3. Growable data and text/bytes, with specified allocation and exhaustion behavior.
4. Reuse and program organization driven by representative programs.
5. Language-only acceptance workloads, then revisit ecosystem and workspace milestones.

This sequence now has implemented language slices and a local acceptance audit, not production or ecosystem
readiness. The acceptance gate and deferred areas are defined in the language-completeness goal. Selecting
the next concrete target is separate; no ecosystem expansion starts automatically. The numbered milestones below preserve the earlier
long-term roadmap and existing implementation history; their numbers are not the current execution order.
The broader agent-workflow hypothesis remains a later evaluation target. This sequence is no longer an
open-ended instruction to add another construct: the current checkpoint above controls continuation.

The first record slice is implemented in [Core v0.4](../spec/core-v0.4.md): nominal immutable values,
nested acyclic layouts, field access, record-carried folds, and complete native/reference invocation.
Tagged alternatives and exhaustive handling are implemented in [Core v0.5](../spec/core-v0.5.md), including
ordinary typed failure returns; see [decision 0016](../decisions/0016-tagged-alternative-values.md).
Persistent growable lists and allocation accounting are implemented in [Core v0.6](../spec/core-v0.6.md).
All three [frozen workloads](../spec/language-milestone-workloads.md)—parser, collection, and tree—are executable
entirely in Lobo. State-driven termination is implemented in [Core v0.8](../spec/core-v0.8.md), with parser
reduction/tree search converted and an executable early-match search. Closed module linking is implemented
experimentally in [Core v0.9](../spec/core-v0.9.md), including explicit-project editor integration and a
shared collection/tree helper. The language-only audit is recorded in [language completeness](language-completeness.md#language-only-requirement-audit--2026-10-18);
at that Core 0.9 checkpoint recursive named types were not supported. See the [coverage assessment](language-completeness.md#coverage-assessment-after-the-three-workloads).
General byte/UTF-8 text values are implemented
in [Core v0.7](../spec/core-v0.7.md), with explicit encoding/index units and metered immutable buffers. See
[decision 0017](../decisions/0017-persistent-growable-lists.md).

## Milestone 0: design contract

Deliverables:

- project vision and non-goals;
- architecture and trust boundaries;
- initial semantic entities;
- workflow, state, context, and protocol contracts;
- strict protocol envelope schemas;
- one coherent example session;
- recorded foundational decisions.

Exit criteria:

- internal links and JSON examples validate;
- terminology is consistent across documents;
- open questions and deferred decisions are visible;
- the first prototype can be implemented without inventing its lifecycle.

## Milestone 1: executable Core and compiler

Status: first implementation landed locally.

The Rust compiler checks Core source, resolves calls to stable IDs, and produces optimized native executables
through C lowering and Clang/LLVM. Checked artifacts and a reference evaluator support inspection and testing.
Cargo tests execute the 15-case Core corpus, native/reference comparisons, boundary and rejection cases,
resource handling, artifact validation, and CLI round trips.

Exit criteria for this initial slice:

- language grammar, types, arithmetic, evaluation, contracts, and failure categories are defined;
- `check`, `compile`, and `run` work on the Core example;
- source and reloaded artifacts agree on the specified corpus;
- runtime contracts cannot be disabled;
- no unchecked arithmetic wrapping or hidden host actions;
- unsupported wider syntax is rejected;
- limitations and fixture coverage are explicit.

Core v0.2 now adds [counted folds](../spec/core-v0.2.md) with a single immutable carried value, native loops,
runtime faults/fuel checks, and explicit checked-artifact versioning. It does not add general mutation or collections.

Compiler 0.2.1 adds [structured source locations](../spec/source-diagnostics-v0.1.md) and source-only stdin
checking. The VS Code extension consumes these for unsaved-source diagnostics in trusted workspaces while
retaining highlighting/navigation. Compiler-owned symbols/types and migration of navigation remain future slices.

Complete Core graph interchange and node identity remain future workspace work, deferred under the current
language-first goal. The expression-tree artifact is interim, not the canonical semantic workspace.

Core v0.3 / compiler 0.3.0 adds [checked action bounds](../spec/core-v0.3.md), pure contracts, per-call
containment, and agent-visible summaries. Extension 0.4.0 tracks the syntax and diagnostics. This implements
the static foundation of milestone 4 early, not milestone 6's host capabilities, replay, quotas, or isolation.

## Milestone 2: extend native verification and specify optional WASM

Status: the first native-verification slice is implemented. The [native development guide](native-development.md)
documents 100 bounded expression cases (24 fixed probes and 76 generated expressions), independent mathematical
expectations, optimized/reference/reloaded comparisons, ASan/UBSan checks and instrumentation probes, and an
executable GCD/factorial example. This finite corpus is not unrestricted fuzzing or proof of general correctness.
Optional WASM invocation/memory definitions and broader generated coverage remain pending.

Extend the native/reference comparison corpus with generated bounded programs and sanitizer checks.
Define an optional WASM host invocation ABI and memory representation before adding that backend. Native
machine-code execution remains the default. This updates the original WASM-first sequence.

Exit criteria:

- native artifacts run in the documented host environment;
- all supported Core conformance cases agree between native execution and the evaluator;
- generated bounded programs have no unresolved semantic mismatch;
- execution limits produce a classified host result;
- any added WASM host imports are explicit and contain no undeclared ambient resource access.

## Milestone 3: semantic workspace vertical slice

Build a local `lobod` prototype that supports:

1. import of a small JSON graph;
2. content-addressed immutable revisions;
3. symbol and relationship queries;
4. transaction begin/apply/revert;
5. three Core operations: replace a function body, add a function, strengthen a precondition;
6. deterministic textual projection;
7. impact output listing affected subjects and obligations;
8. atomic commit to a local branch pointer.

Exit criteria:

- repeated import and projection are deterministic;
- stale-base commits fail without partial mutation;
- an operation can be replayed idempotently;
- the Core example can be changed without a textual search-and-replace;
- a human can inspect the semantic diff.

Suggested implementation: Rust daemon and CLI, SQLite metadata, content-addressed files, JSONL transport. These are
defaults to validate, not permanent commitments.

## Milestone 4: verification and evidence

Extend Core's existing type checker and executable contracts with actions and invariants as those features
are specified. Add affected-closure selection, structured counterexamples, and evidence manifests.

Exit criteria:

- an invalid transition is rejected with a machine-readable counterexample;
- broadening an action declaration invalidates the relevant evidence;
- verification results cannot attach to a different revision;
- a commit policy can require named evidence kinds.

## Milestone 5: context engine

Implement deterministic context packages over semantic relationships, governing constraints, decisions, and
evidence. Add progressive expansion, explicit omissions, sensitivity redaction, and budget enforcement.

Exit criteria:

- seeded tasks retrieve all governing contracts in the benchmark;
- packages are reproducible for the same inputs and policy;
- protected data is excluded before rendering;
- omission and truncation are machine visible;
- context use is measurably smaller than full-workspace ingestion.

## Milestone 6: capabilities and replay

Extend supported execution environments with specified actions and host capabilities for clock, randomness,
storage, and network access. Capture structured traces and support replay with recorded capability responses.

Exit criteria:

- undeclared actions fail before execution;
- capability scope is enforced by the host;
- a failed run can be replayed without live external access;
- resource budgets terminate execution with a classified result.

## Milestone 7: multi-agent transactions

Add leases, semantic rebase, merge analysis, combined obligations, and coordination views.

Exit criteria:

- independent commuting operations merge automatically;
- shared-invariant conflicts are detected even across different declarations;
- representation-only source conflicts disappear;
- every merged operation retains actor and goal provenance.

## Milestone 8: comparative evaluation and v1.0 readiness

Run the same representative tasks through Lobo and a conventional file/LSP/shell workflow. Measure correctness,
cost, context use, time, unrelated churn, human review effort, and permission-policy violations.

The result determines whether to deepen the language, improve only the tooling model, or stop. A custom runtime is
considered only when a measured workload demonstrates a limitation in existing targets.

The [v1.0 target](v1.0-target.md) defines the intended product and release evidence. Experiments should also run
at earlier milestones, so weak results can change the plan before the entire platform is built.

## Later milestone: self-hosting

Specify the additional language features needed to express the compiler, write the second implementation in
Lobo, and follow the [bootstrap workflow](self-hosting.md). Retain the Rust reference and shared conformance
corpus. Self-hosting is a valuable experiment, not a prerequisite for the agent workspace's v1.0 release.

## Cross-cutting requirements

Every milestone must include:

- schema and format versioning;
- deterministic fixtures;
- threat modeling for new authority;
- bounded outputs and resource use;
- migration and recovery notes;
- evidence for its exit criteria;
- a human-readable projection of all canonical state.
- maintenance of corresponding VS Code highlighting, navigation, diagnostics, and editor regressions.

The [practical solution-readiness gate](v1.0-target.md) additionally tracks essential libraries, foreign-code
interoperability, resource authority, quotas, and logging/auditing. Its first concrete solution is not yet chosen;
do not report native scalar examples as evidence that general applications or these libraries are supported.

## Immediate decisions still needed

- Recursive named types and memory reclamation beyond the current invocation-scoped list arena.
- Evaluate new abstractions only against demonstrated limitations of the implemented module/value core.
- Select the next concrete target separately; ecosystem/application readiness is not implied by the language audit.

## Later workspace decisions

- Exact Core graph schema and expression-node identity for the workspace milestone.
- Hash and canonical serialization format.
- Branch and transaction persistence behavior after daemon restart.
- Contract expressions for later resource-access and persistent-state features; pure Core contracts are defined.
- Whether proof obligations are nodes or typed relationships with evidence attachments.
- Benchmark tasks and baseline agent workflow for evaluation.

