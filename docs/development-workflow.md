# Development workflow

This document describes the eventual agent workspace. For building the compiler itself, see the
[compiler development workflow](compiler-development-workflow.md), [testing and TDD](testing-and-tdd.md), and
[self-hosting stages](self-hosting.md).

## Goal-bound sessions

An agent session begins with a goal, not an unrestricted shell. The session records:

- the requested outcome and acceptance examples;
- the workspace and base revision;
- granted read, mutation, execution, and external capabilities;
- time, compute, context, and operation budgets;
- applicable policies and human gates;
- the evidence required before commit.

The session may discover additional work, but it cannot silently expand its own authority or acceptance criteria.

## The change loop

```text
bind goal → acquire context → begin transaction → propose operations
    ↑                                                   │
    └──── repair ← inspect counterexample ← verify ─────┘
                                         │
                              review evidence → commit
```

### 1. Bind the goal

The host turns a request into a task contract. Straightforward details may be inferred from repository evidence,
but consequential ambiguity becomes a decision request. Requirements and non-goals receive stable IDs so changes
can cite them.

### 2. Acquire context

The agent asks the context engine a specific question with a budget. The returned package contains relevant
symbols, governing contracts, effect boundaries, callers, tests, decisions, failures, and known uncertainty. It
also declares important material omitted because of the budget.

### 3. Begin a transaction

The transaction names an immutable base revision. Subsequent reads are consistent with that snapshot plus local
operations. No implicit working directory or moving `latest` reference is allowed in protocol calls.

### 4. Propose semantic operations

Preferred operations express intent:

```text
type.add_variant
record.add_field
contract.strengthen_precondition
state_machine.add_transition
function.replace_body
capability.narrow
test.add_example
```

An escape hatch may import a textual patch, but the daemon must parse it into semantic operations before it can be
accepted. Source locations may help presentation; they do not define the mutation.

### 5. Evaluate impact

After each accepted operation, the daemon returns:

- changed semantic subjects;
- new, discharged, and invalidated obligations;
- affected dependents;
- compatibility concerns;
- required policy decisions;
- estimated verification cost.

The agent can revise or revert individual operations without discarding the transaction.

### 6. Verify progressively

Checks run from narrow to broad:

1. operation and graph validity;
2. type and effect checks;
3. changed contracts and examples;
4. affected dependency closure;
5. compatibility and policy gates;
6. broader smoke or integration checks when risk requires them.

Failures return counterexamples and stable diagnostic codes. Passing checks create revision-bound evidence.

### 7. Review and commit

A proposed commit includes a semantic diff, behavioral summary, capability delta, verification manifest,
unresolved assumptions, and recovery notes. Commit is atomic and produces a new immutable revision.

## Multi-agent work

Agents normally use independent transactions. Optional leases communicate intent and prevent waste, but do not
turn file ownership into the conflict model.

Merge analysis has three levels:

1. **Structural:** Can both operation logs apply to the same graph?
2. **Contractual:** Do their combined contracts and effects remain consistent?
3. **Behavioral:** Does combined verification satisfy all affected obligations?

Two operations on the same declaration may commute, while operations in different modules may conflict through a
shared invariant. Lobo reports semantic conflicts and generates joint obligations rather than relying on adjacent
line changes.

## Human gates

A policy may require human review for:

- new or broader capabilities;
- public contract breaks;
- unresolved high-severity obligations;
- external writes, deployments, or data migrations;
- changes to authorization or policy code;
- evidence produced under untrusted conditions.

A gate records the artifact reviewed and exact revision approved. Approval does not transfer to later revisions
unless policy says that the intervening changes are irrelevant.

## Recovery

Workspace operations are replayable and reversible before commit. Committed revisions are immutable; recovery
creates a new revision that semantically reverts earlier operations. External actions require their own declared
compensation or rollback plan because a workspace rollback cannot undo the outside world.

## Example session

For a complete illustrative exchange, see [the JSONL session](../examples/agent-session.jsonl). Its logical flow is:

```text
workspace.describe
context.build
transaction.begin
state_machine.add_transition
verification.run
transaction.propose_commit
transaction.commit
projection.render
```

## Workflow invariants

- Every mutation is attributed to an actor and goal.
- Every mutation names its base revision.
- The agent cannot execute its own proposed tool call without host validation.
- Verification evidence names the exact program and environment checked.
- A context summary never silently becomes canonical program state.
- External effects are separate from local program mutation.
- “Unknown” is represented explicitly rather than converted into success.

