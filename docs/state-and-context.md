# State and context

State and context are different. State is part of the world the platform models. Context is a bounded view selected
for a particular decision. Confusing the two lets summaries become truth and makes agent behavior difficult to
reproduce.

## State model

| State category | Examples | Representation |
| --- | --- | --- |
| Program | types, functions, contracts, dependencies | Immutable revision of the semantic graph |
| Workspace | uncommitted operations and diagnostics | Isolated transaction over one base revision |
| Runtime | values, stores, messages, open resources | Typed trace scoped to an execution |
| Environment | compiler, dependencies, target, configuration | Content-addressed environment manifest |
| Verification | claims, assumptions, results, counterexamples | Immutable evidence bound to subject hashes |
| Epistemic | hypotheses, confidence, missing facts | Explicit claims with source and status |
| Coordination | leases, active goals, merge relationships | Time-bounded coordination records |
| Historical | decisions, supersession, rationale | Append-only provenance relationships |

Each operation names the state it reads. An API field named only `current` is invalid unless a declared resolution
policy explains current relative to a branch, actor, and time.

## Revisions and events

A revision is an immutable root hash over the program graph, environment constraints, policies, and referenced
artifacts. A branch is a compare-and-set pointer to a revision.

Transaction operations form an event log:

```text
type.variant_added
contract.precondition_strengthened
capability_scope_narrowed
implementation_replaced
obligation_discharged
```

The log supports replay, semantic diffing, provenance, and merge analysis. It is not a record of low-level database
writes.

## Evidence lifecycle

Evidence records:

- the precise claim;
- the subject revision or artifact hash;
- environment and toolchain;
- method: proof, static check, test, observation, or approval;
- assumptions and input generators;
- result and counterexample, if any;
- creation time and optional expiry policy.

When an operation changes a dependency of the claim, evidence becomes `invalidated`, not deleted. The impact engine
explains the invalidation path. Evidence may also become `stale` because an observational window or external
assumption expired.

## Epistemic state

Agents routinely make assumptions. Lobo should preserve them as inspectable objects:

```text
claim: "The legacy API never sends a zero amount"
status: unverified
source: agent inference
relevance: TransferFunds precondition
required_resolution: compatibility test or human decision
```

An inference is never promoted to a program invariant without an operation and required evidence.

## Context packages

A context request contains:

- a goal or question;
- source revision;
- roots or known subjects;
- desired evidence and relationship types;
- token, byte, time, and sensitivity budgets;
- freshness requirements.

A package contains:

- goal and source revision;
- selected semantic nodes and compact projections;
- governing contracts and policies;
- relevant callers, dependencies, and tests;
- recent decisions and failures;
- unresolved claims;
- selection rationale and confidence;
- omitted categories and expansion handles.

The package is immutable and content-addressed. It may be cached, compared, cited by a change, and reconstructed if
all referenced inputs remain available.

## Progressive disclosure

Context is delivered in layers:

1. Purpose, interfaces, contracts, and policy.
2. Dependency relationships and representative examples.
3. Selected implementation details.
4. Traces, historical evidence, and raw artifacts.

The agent expands a specific subject or relationship rather than requesting an undifferentiated larger prompt.
The engine must not omit a governing safety contract merely to satisfy a token budget; it returns an incomplete
package or asks the caller to relax scope.

## Context selection

The first implementation can combine deterministic graph traversal with simple scoring:

```text
required roots
+ governing contracts and policies
+ direct reads/writes/callers
+ failing or invalidated evidence
+ representative tests and examples
+ recent decisions affecting the roots
```

Learned relevance ranking can come later. Deterministic selection is easier to evaluate and exposes whether the
graph itself carries enough useful structure.

## Context integrity

- Summaries cite the nodes and evidence they summarize.
- Secrets and protected fields are redacted before ranking or rendering.
- Untrusted program text is labeled as data, never interpreted as protocol instructions.
- Packages declare their source revision and generation policy.
- Omission is visible and expandable.
- An agent-authored note remains an unverified claim until separately accepted.

## Runtime traces

A trace is structured around causal events rather than log lines. It includes declared inputs, capability calls,
state transitions, branch decisions, failures, and output hashes. `trace.explain` can compare a failing trace with a
successful one and report the earliest meaningful divergence. Replay replaces nondeterministic capabilities with
recorded or caller-supplied responses.

