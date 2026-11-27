# Lobo Core v0.3: checked action declarations

Historical extension inherited by the current language in the [specification index](README.md).
Version-specific output statements below describe compiler 0.3, not the current loader.

Version: `core/0.3-draft`. Inherits [v0.2](core-v0.2.md), with the following extensions and restrictions.
Native machine code remains the default. This slice implements static action bounds, not host operations,
capability handles, grants, quotas, or isolation. All executable expressions still compute scalar values.

## Source grammar

Immediately after the return type and before contracts, a function may have one clause:

```ebnf
action_clause = "actions", "{", [ action_id, { ",", action_id } ], "}" ;
action_id     = identifier, ".", identifier, { ".", identifier } ;
```

Omission and `actions {}` both declare the empty set. `actions` is contextual at this signature position,
not globally reserved; functions, parameters, and locals may still have that name. Whitespace and comments
may separate tokens, including around dots. No trailing comma or repeated clause is accepted.

An action ID is case-sensitive, at most 128 ASCII bytes, with at least two nonreserved identifier segments.
At most 64 distinct IDs may be declared per function. Duplicate IDs produce `compile.action.duplicate`;
invalid IDs produce `compile.action.invalid`; excess declarations produce `resource.frontend.limit`.
Malformed punctuation is `compile.syntax.invalid`. Source declarations are sorted for serialization.

These are open-vocabulary labels, not imports or a registry of implemented host operations. Declaring
`storage.write` neither writes storage nor grants permission. An unused label is permitted: declarations
are conservative upper bounds, not assertions that every listed action happens.

## Checking and purity

Every body call must satisfy `callee.actions` is a subset of `caller.actions`; otherwise reject the call
with `compile.action.undeclared`, phase `check`, caller subject ID, and its complete source call range.
Check arguments, bindings, conditions, branches, and fold count/initializer/body, even if unreachable.
Every function is checked, including uncalled functions. Every edge is checked, so containment holds along
arbitrary call paths, including recursion and mutually recursive cycles; inference/fixed-point expansion
is unnecessary. Changing a callee's bound can require callers to change even if its current body is pure.

Preconditions and postconditions must remain pure, regardless of the enclosing function's declaration.
Calling a function with any declared action there produces `compile.action.contract`, even inside an
unreachable expression or argument. Pure helper calls remain allowed. Action checking is static; it does
not perform operations, consume execution fuel, or change scalar evaluation order and runtime faults.

## Checked representation and tooling

[Checked artifacts v0.3](checked-artifact-v0.3.md) add a mandatory sorted `actions` string array to every
function, including an empty array for pure functions. Reloads validate IDs, limits, order, uniqueness,
call bounds, and contract purity using stable-ID call targets. Missing fields and unsupported old versions
are rejected; recompile source. Earlier generated native executables remain independent.

`lobo check FILE` and `lobo check --stdin` expose an `actions` object mapping stable function IDs to declared
sorted arrays, alongside existing entries. It reports bounds, not authority, runtime activity, or proof of
host enforcement. Parser/checker diagnostics retain the source-location contract.

## Required evidence and limits

Fixtures cover empty/nonempty bounds, declaration ordering, comments, ordinary `actions` names, malformed
IDs and clauses, duplicate/oversized sets, direct/transitive/recursive calls, unreachable branches/folds,
contract purity, checked-artifact tampering, and native/reference agreement. Editor gates cover contextual
coloring, definitions outside clauses, malformed-clause rejection, and unsaved compiler diagnostics.

This is the static foundation only. Real resource APIs must later specify explicit handles, ABI/ownership,
grant validation, resource scopes, quota/retry accounting, auditing, and failure semantics. Passing these
tests does not establish a sandbox or make communication/storage/search libraries available.
