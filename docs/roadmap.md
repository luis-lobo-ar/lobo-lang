# Roadmap

The roadmap prioritizes testing the agent-workflow hypothesis. It deliberately postpones a custom VM, optimizer,
package ecosystem, and rich syntax.

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
- no unchecked arithmetic wrapping or hidden host effects;
- unsupported wider syntax is rejected;
- limitations and fixture coverage are explicit.

Next, specify complete Core graph interchange and node identity. The expression-tree artifact is an interim
compiler representation, not the canonical semantic workspace.

## Milestone 2: extend native verification and specify optional WASM

Extend the native/reference comparison corpus with generated bounded programs and sanitizer checks.
Define an optional WASM host invocation ABI and memory representation before adding that backend. Native
machine-code execution remains the default. This updates the original WASM-first sequence.

Exit criteria:

- native artifacts run in the documented host environment;
- all supported Core conformance cases agree between native execution and the evaluator;
- generated bounded programs have no unresolved semantic mismatch;
- execution limits produce a classified host result;
- any added WASM host imports are explicit and contain no undeclared ambient effects.

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

Extend Core's existing type checker and executable contracts with effects and invariants as those features
are specified. Add affected-closure selection, structured counterexamples, and evidence manifests.

Exit criteria:

- an invalid transition is rejected with a machine-readable counterexample;
- broadening an effect invalidates the relevant evidence;
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

Extend supported execution environments with specified effects and host capabilities for clock, randomness,
storage, and network access. Capture structured traces and support replay with recorded capability responses.

Exit criteria:

- undeclared effects fail before execution;
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

## Immediate decisions still needed

- Exact Core graph schema and expression-node identity for the workspace milestone.
- Hash and canonical serialization format.
- Branch and transaction persistence behavior after daemon restart.
- Contract expressions for later effectful and persistent-state features; pure Core contracts are defined.
- Whether proof obligations are nodes or typed relationships with evidence attachments.
- Benchmark tasks and baseline agent workflow for evaluation.

