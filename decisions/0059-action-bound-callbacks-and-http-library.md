# 0059: Action-bound callbacks and shared HTTP serving

Date: 2026-11-23. Status: implemented and verified locally, compiler/extension 1.11.0, Core 1.11.

The user authorized callbacks with declared action bounds and moving SQLite REST application scaffolding
into the HTTP library. Use the [Core 1.11 contract](../spec/core-v1.11.md). This supersedes the pure-only
callback restriction for this milestone, not earlier frozen language/research checkpoints.

Use explicit finite callback action sets and explicit context. Keep exact signatures, existing pure callback
meanings, checked module visibility, native grants, postconditions, limits, and cleanup/error precedence.
Place callback actions before the type arrow to avoid changing the meaning of functions returning Fn values
whose own action clause follows their output. Reuse action ID validation and bounds.

Alternatives: action-polymorphic server functions would generalize all handler action sets but add action
variables and union/substitution semantics beyond the concrete demand. Implicit widening requires callback
variance/joins and wire/runtime coercion rules; defer it. Coupling a library directly to this application's
controller hides repetition without creating reuse. The selected API is context-generic with a concrete
DB action bound; existing pure handlers keep their API, and common transport operations are shared.

Use the agentic-engineering standard workflow with TDD and focused adversarial checks. Invariants are exact
callback signatures at source/artifact/invocation boundaries, checked actions at every invocation edge,
unchanged native authorization, and explicit resources closed after captured outcomes. Failure cases include
missing/forged bounds, contract calls, module visibility, failing callback contracts/try, denied native grants,
partial IO/timeouts, quota faults and close failures. Retain raw red/green evidence and prior uncommitted work.
No native provider/model change is planned. Verify source maps without recapturing old model fingerprints.

The [verification report](../benchmarks/results/2026-11-23-action-callbacks.md) and [receipt](../benchmarks/results/2026-11-23-action-callbacks.json) retain red/green outcomes and corrections. The four relocated logical helper IDs keep their values; existing pure APIs remain. No actionable issue was found in the independent source review. Stop feature growth at this selected contract.
