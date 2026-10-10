# Lobo Tool Protocol

The Lobo Tool Protocol (LTP) is the agent-facing interface to `lobod`. It is transport-independent; the first
implementation should support newline-delimited JSON over standard input/output, with local sockets added only
when needed.

The protocol version is independent from the language and graph-format versions.

## Design rules

- Requests and responses are typed and schema-versioned.
- Every request has an ID and an actor/session identity.
- State-dependent requests name a revision or transaction.
- Mutating requests include an idempotency key.
- Tools disclose effects, required capabilities, cost limits, and retry behavior.
- Responses are bounded; large artifacts are returned by content-addressed reference.
- Diagnostics are structured, with prose as an optional presentation field.
- Operations are cancellable when cancellation can be safe.
- Unsupported mandatory fields or features cause explicit rejection.

## Discovery

An agent should be able to enter an unfamiliar environment using a small universal surface:

```text
protocol.describe
tools.list
tool.describe
workspace.describe
capabilities.current
policies.applicable
```

`tool.describe` returns the request and response schemas, safety tier, effects, required capability patterns,
timeouts, idempotency semantics, cancellation support, and examples.

## Request envelope

```json
{
  "ltp": "0.1",
  "request_id": "req-01JABC",
  "session_id": "session-01JAAA",
  "tool": "verification.run",
  "target": {
    "workspace": "payments",
    "transaction": "txn-01JABD"
  },
  "idempotency_key": "verify-after-transition-v1",
  "budget": {
    "wall_time_ms": 30000,
    "output_bytes": 65536
  },
  "arguments": {
    "scope": { "kind": "affected_closure" },
    "checks": ["types", "effects", "contracts", "tests"]
  }
}
```

Read-only requests omit `idempotency_key`. A request must not identify state through an ambient working directory.

## Response envelope

```json
{
  "ltp": "0.1",
  "request_id": "req-01JABC",
  "status": "rejected",
  "result": null,
  "diagnostics": [
    {
      "code": "contract.precondition.missing",
      "category": "verification_failed",
      "severity": "error",
      "subject": "finance.cancel_transfer",
      "message": "Cancellation must exclude settled transfers.",
      "counterexample": {
        "state": "Settled",
        "event": "Cancel"
      },
      "repairs": [
        {
          "operation": "contract.strengthen_precondition",
          "arguments": {
            "expression": "transfer.state == Pending"
          }
        }
      ]
    }
  ],
  "evidence": [],
  "usage": {
    "wall_time_ms": 281,
    "output_bytes": 912
  }
}
```

Suggested repairs are inert data. The host must treat a follow-up repair as a new operation requiring the usual
validation and authority.

## Tool descriptor

Each tool publishes metadata similar to:

```json
{
  "name": "transaction.commit",
  "version": "0.1.0",
  "safety_tier": "shared_mutation",
  "summary": "Atomically create a revision from a verified transaction.",
  "effects": ["workspace.revision.create", "branch.head.compare_and_set"],
  "required_capabilities": ["workspace.commit:{workspace}:{branch}"],
  "idempotency": "same key and transaction returns the original outcome",
  "cancellable": false,
  "default_timeout_ms": 10000,
  "request_schema": "lobo://schemas/tools/transaction.commit/request/0.1",
  "response_schema": "lobo://schemas/tools/transaction.commit/response/0.1"
}
```

## Initial tool families

| Family | Representative operations |
| --- | --- |
| Discovery | `protocol.describe`, `tools.list`, `tool.describe` |
| Workspace | `workspace.describe`, `revision.get`, `projection.render` |
| Search | `symbol.find`, `references.find`, `graph.query` |
| Context | `context.build`, `context.expand`, `context.explain_omissions` |
| Transactions | `transaction.begin`, `operation.apply`, `operation.revert`, `transaction.commit` |
| Verification | `obligations.list`, `verification.plan`, `verification.run` |
| Execution | `run.start`, `run.cancel`, `trace.explain`, `run.replay` |
| Coordination | `lease.acquire`, `merge.analyze`, `conflicts.explain` |
| Policy | `capabilities.current`, `policies.applicable`, `approval.attach` |

The prototype should implement only the subset required by the first vertical slice.

## Progress and cancellation

Long-running operations emit ordered events tied to the request:

```text
accepted → planned → progress* → completed | failed | cancelled
```

Progress messages report completed units and revised estimates without flooding raw logs. Cancellation is a
request, not proof that work stopped; final status confirms whether cancellation took effect.

## Safety tiers

| Tier | Typical tools | Default handling |
| --- | --- | --- |
| Read-only | search, context, describe | Allow within session scope; redact protected data |
| Local reversible | transaction edits, sandbox tests | Allow with budgets and operation log |
| Shared mutation | branch commit, shared package publish | Require explicit grant and compare-and-set target |
| External write | issue creation, service mutation | Require destination-specific authority and audit |
| High impact | deploy, migration, deletion | Require plan, approval, rollback, and narrow target |

Tool discovery exposes the tier so an agent can reason before requesting an operation.

## Schema files

The initial envelope schemas live in [`spec/schemas`](../spec/schemas). They establish strict outer framing while
allowing tool-specific argument and result schemas to evolve independently.

