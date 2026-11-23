# Language design

Status: semantic sketch. Example syntax is illustrative and non-normative.

The [language 1.7 specification](../spec/core-v1.7.md), inheriting the earlier rules, defines current experimental
behavior. Language 1.0 added generic data, parametric definition checking and pure function references.
Language 1.1 adds typed operator bindings and Text concatenation. These specifications take precedence over
the sketches here. Language 1.3 accepts modern source with managed identity sidecars. Inline tables and deprecated older source remain readable.
Language 1.7 adds the closed native capability/OS catalog. Richer resource-handling examples below remain future designs. Action clauses and static call bounds
are implemented, as are bounded immutable records/variants, persistent lists, UTF-8 text/bytes, and exhaustive matches. User-declared variants can
carry typed failures; labels do not execute host operations or grant authority.

## Design center

Lobo programs should make these facts cheaply answerable:

- What does this component promise?
- What inputs and states are valid?
- What can it observe or change?
- What consumes its output?
- Which properties have evidence?
- What becomes invalid if it changes?
- Which parts are incomplete or uncertain?

The semantic graph is canonical. Source text exists because it is useful for human review, interchange, and
bootstrapping—not because lines and columns define program identity.

## Semantic entities

The proposed wider language includes:

- primitive, record, enum, collection, and function types;
- components and functions;
- contracts: `requires`, `ensures`, and invariants;
- declared actions and capabilities;
- state machines and legal transitions;
- examples and universally quantified properties;
- resource limits;
- typed holes and obligations;
- policies and approval gates;
- stable logical identities and documentation.

Features such as inheritance, macros, reflection, exceptions, implicit coercion, and unrestricted global mutation
are excluded until they have clear semantics for impact analysis.

## Example

```lobo
component TransferFunds @id("finance.transfer.v1") {
  input {
    from: AccountId
    to: AccountId
    amount: Money where amount > 0
  }

  output TransferReceipt

  requires {
    from != to
    ledger.balance(from) >= amount
  }

  ensures(result) {
    ledger.balance(from)' == ledger.balance(from) - amount
    ledger.balance(to)' == ledger.balance(to) + amount
    ledger.total()' == ledger.total()
    result.amount == amount
  }

  actions {
    read ledger.accounts(from, to)
    write ledger.accounts(from, to)
    emit TransferCompleted
  }

  limits {
    wall_time <= 50.ms
    ledger.writes == 2
  }

  run with capability Ledger.Transfer {
    atomic ledger {
      ledger.debit(from, amount)
      ledger.credit(to, amount)
    }
  }
}
```

## Contracts

Contracts are executable artifacts. The environment may discharge them statically, test them dynamically, enforce
them at runtime, or leave them as explicit unresolved obligations. The selected method and its assumptions are
part of the evidence.

An unproven contract is not silently treated as true:

```lobo
ensures sorted(result)
  evidence required static_proof
```

or:

```lobo
ensures response.latency <= 100.ms
  evidence observed(percentile: 99, window: 7.days)
```

These are different claims and must remain distinguishable.

## Actions, capabilities, and limits

Lobo calls possible resource interactions **actions**. This replaces the earlier term "effects"; see
[decision 0012](../decisions/0012-actions-terminology.md). It changes vocabulary, not authority.

- Actions describe what a function might do, including through functions it calls.
- Capabilities supply authority over particular resources; declaring an action grants nothing.
- Limits constrain execution and resource use, enforced by the host rather than by annotations alone.

In Core v0.3, functions without declared actions have pure bounds. The compiler checks every call edge against
its declaration and keeps contracts pure. In the wider proposal, capability requirements are passed as narrow, opaque resource
handles, not acquired through imports or a global context object. For example, this future syntax is illustrative:

```lobo
function download @id("example.download")(
  client: HttpGet,
  url: Url
) -> Result<Bytes, HttpError>
actions { http.request }
{
  http_get(client, url)
}
```

The host supplies `client` with a destination scope and limits. A program cannot construct authority by creating
an ordinary record or writing `actions { http.request }`. A caller may delegate a narrower grant but cannot
manufacture a broader one; delegation must not reset shared quotas. Time, randomness, environment variables,
and process execution are actions too, not invisible runtime facilities.

Static label/call checking is specified in [Core v0.3](../spec/core-v0.3.md). Host-operation resolution, handle
ownership/lifetimes, scope enforcement, quota accounting, revocation, and failures still need specifications
and fixtures before implementation. Core has no resource handles or host operations; `actions` is contextual
in signatures and remains an ordinary identifier elsewhere.

## Errors

Recoverable failures use explicit result types. Defects such as violated internal invariants terminate the current
sandboxed operation and generate diagnostic evidence. The language does not use unchecked exceptions for ordinary
control flow.

```lobo
function load_account(id: AccountId)
  -> Result<Account, AccountMissing | StorageUnavailable>
```

## State and transitions

Stateful behavior should be representable as legal transitions with preconditions and postconditions:

```lobo
machine TransferLifecycle for Transfer {
  Pending --authorize--> Authorized
  Pending --cancel--> Cancelled
  Authorized --settle--> Settled

  invariant terminal_is_immutable {
    state in {Cancelled, Settled} => no outgoing_transition
  }
}
```

Persistence and transactions are actions of an implementation; the state machine defines permitted behavior.

## Typed holes

Incomplete work is explicit and cannot masquerade as a complete program:

```lobo
run {
  ?implementation(
    must_satisfy: [ledger_conservation, idempotent_by(request.id)],
    allowed_actions: [Ledger.Read, Ledger.Write]
  )
}
```

A hole carries expected type, constraints, permitted actions, and blocking status. Production builds reject holes
unless a policy explicitly permits a declared fallback.

## Identity and references

Public and long-lived entities have stable logical IDs independent of names. Human-readable names may change
without breaking references; changing semantic identity requires an explicit replace or migration operation.

References resolve against a named revision. Ambiguous textual names are a presentation concern, never canonical
links.

## Determinism

Given the same revision, toolchain, declared environment, capability responses, and inputs, verification and
execution should be reproducible. Nondeterminism must be injected and recorded. Resource scheduling may vary, but
observable semantics must follow a declared concurrency model.

## Concurrency position

The first language subset should be sequential. Concurrency will require explicit task ownership, cancellation,
deadlines, communication, and action ordering. It should not be added as syntactic sugar over an unspecified
memory model.

