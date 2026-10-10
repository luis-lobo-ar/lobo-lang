# Language design

Status: semantic sketch. Example syntax is illustrative and non-normative.

The proposed [Core v0.1 specification](../spec/core-v0.1.md) defines the initial executable subset. It takes
precedence over the sketches here for Core behavior. Richer examples in this document remain future designs.

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

The initial core consists of:

- primitive, record, enum, collection, and function types;
- components and functions;
- contracts: `requires`, `ensures`, and invariants;
- declared effects and capabilities;
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

  effects {
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

## Effects and capabilities

Functions are pure unless they declare effects. Effects describe behavior; capabilities grant authority. Declaring
`network.request` does not itself authorize network access.

```lobo
effects {
  read clock
  request network.host("risk.internal")
}

run with capability RiskService.Read
```

Capability scopes must be attenuable. A caller may pass a narrower grant but cannot manufacture a broader one.
Time, randomness, environment variables, and process execution are effects—not invisible runtime facilities.

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

Persistence and transactions are effects of an implementation; the state machine defines permitted behavior.

## Typed holes

Incomplete work is explicit and cannot masquerade as a complete program:

```lobo
run {
  ?implementation(
    must_satisfy: [ledger_conservation, idempotent_by(request.id)],
    allowed_effects: [Ledger.Read, Ledger.Write]
  )
}
```

A hole carries expected type, constraints, permitted effects, and blocking status. Production builds reject holes
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
deadlines, communication, and effect ordering. It should not be added as syntactic sugar over an unspecified
memory model.

