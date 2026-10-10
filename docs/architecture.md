# Architecture

Status: initial design, not an implementation commitment.

## Overview

Lobo is organized around a persistent workspace daemon named `lobod`. The daemon is the authority for canonical
program state. Agents and human-facing clients do not mutate storage directly; they submit typed operations to
transactions and receive typed results.

```text
┌──────────────── clients ────────────────┐
│ agents │ CLI │ editor │ CI │ review UI │
└───────────────────┬─────────────────────┘
                    │ Lobo Tool Protocol
┌───────────────────▼─────────────────────┐
│                  lobod                  │
│ discovery │ transactions │ policy gate │
│ context   │ impact       │ diagnostics │
├───────────────┬──────────────┬──────────┤
│ semantic DB   │ verifier     │ executor │
│ graph/indexes │ type/effect  │ sandbox  │
├───────────────┴──────────────┴──────────┤
│ revisions │ evidence │ traces │ blobs   │
└───────────────────┬─────────────────────┘
                    │ verified lowering
             WASM / native / other IR
```

## Components

### Semantic database

The database stores nodes and typed relationships with stable logical IDs. A node may represent a declaration,
type, contract, effect, test, policy, requirement, decision, or evidence claim. Revisions are immutable and
content-addressed; logical IDs persist across revisions when identity is preserved.

The initial implementation may use ordinary files and SQLite. The abstraction matters more than the storage
engine.

### Transaction engine

A transaction has:

- a unique ID;
- one immutable base revision;
- an ordered operation log;
- the actor and delegated capabilities;
- resource and time budgets;
- current diagnostics and obligations;
- an expiry and cancellation state.

Operations are applied optimistically. Commit succeeds only if the base is current or the operation log can be
semantically rebased, all mandatory policy gates pass, and required evidence is present. A failed commit never
partially mutates the workspace.

### Context engine

The context engine turns a goal and budget into an attributable package. It traverses semantic relationships,
scores relevance, preserves governing constraints, and records omissions. Context generation is read-only and
always names the revision from which it was derived.

### Verifier

The verifier owns several complementary checks:

- structural and schema validation;
- type and effect checking;
- contract and invariant checking;
- policy evaluation;
- property and example tests;
- optional solver-backed proofs;
- compatibility and resource-budget analysis.

A verifier result is evidence, not a boolean detached from its environment. It records inputs, tool versions,
assumptions, result, and reproducibility metadata.

### Sandboxed executor

The executor runs lowered programs and tests with explicit capabilities. Time, randomness, filesystem access,
network access, and environment inputs will be injectable so that runs can be replayed. Core currently generates
native executables through Clang/LLVM. Native execution is not an OS sandbox; isolation and effectful capability
adapters need their own implementation. WASM remains an optional future execution backend.

### Projection engine

The canonical graph can be projected into:

- stable textual source;
- behavioral and semantic diffs;
- public API documentation;
- capability maps;
- architecture views;
- evidence and risk summaries.

Parsing a textual projection should reconstruct an equivalent graph, but formatting is not canonical identity.

## Core artifacts

| Artifact | Identity | Mutable? | Purpose |
| --- | --- | --- | --- |
| Revision | content hash | no | Complete workspace state |
| Semantic node | logical ID + revision | via transaction | Program or intent element |
| Operation | operation ID | no | Requested semantic mutation |
| Evidence | evidence ID + subject revision | no | Result supporting a claim |
| Context package | package ID + source revision | no | Bounded task context |
| Execution trace | trace ID + artifact hash | no | Replayable runtime observation |
| Capability grant | grant ID | revocable | Authority delegated to an actor |

Large blobs are content-addressed. References never silently resolve to “latest”; callers must request a policy
that resolves moving versions.

## Trust boundaries

The model is not a trusted executor. It proposes operations and tool calls. The host validates tool identity,
arguments, revision, scope, capability grants, budgets, and policy before execution.

The following require distinct authority:

1. Reading workspace content.
2. Mutating a local transaction.
3. Executing code in a sandbox.
4. Accessing external systems.
5. Committing to a shared branch.
6. Deploying or migrating persistent data.

Possessing one does not imply another. Tool descriptions must disclose effects and rollback behavior before a
request is made.

## Consistency model

- Revisions and evidence are immutable.
- Transactions provide snapshot isolation against their base revision.
- Reads within a transaction see the base plus accepted local operations.
- Commit is atomic.
- External effects do not occur inside an ordinary workspace transaction.
- Operations intended to affect external systems require a separate, audited execution plan and idempotency key.

## Failure semantics

Every failure has a stable category, an affected subject, retry guidance, and structured details. Expected top-level
categories include `invalid_request`, `permission_denied`, `revision_conflict`, `budget_exhausted`,
`verification_failed`, `tool_unavailable`, and `internal_error`.

Raw logs may be attached as bounded evidence but are never the only error representation.

## Compatibility and evolution

The graph format, protocol, language semantics, and projections have separate version numbers. Readers must reject
unsupported mandatory features and preserve unknown optional metadata. Migrations create a new revision and retain
the source revision and migration evidence.

