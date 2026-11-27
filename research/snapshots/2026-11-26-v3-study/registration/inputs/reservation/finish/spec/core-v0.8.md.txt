# Lobo Core v0.8: state-driven iteration

Historical extension, inherited by the current language in the [specification index](README.md).
Status: implemented experimental subset; specified before implementation. Inherits [Core v0.7](core-v0.7.md).
Versions: `core/0.8-draft`, `lobo-checked/0.8`, compiler 0.8.0, native adapter 0.5 (CLI unchanged).

## Grammar and scope

```ebnf
while = "while", "(", identifier, ":", type, "=", expression, ";", expression, ")", block ;
```

For example, `while (n: I64 = 0; n < 5) { n + 1 }` returns 5. Recognize this expression only at a
primary-expression position with the token prefix `while`, `(`, identifier, `:`. This commits to parsing;
a malformed remainder is not an ordinary-call fallback. Otherwise `while` remains an ordinary name,
including `function while ...` and `while(value)` calls. Comments/whitespace do not affect recognition.

The initializer is checked/evaluated in the outer scope, without the new binding. The condition and body
see one immutable state binding of the annotated type, plus outer names. The annotation may be any supported
type; initializer/body must match exactly and condition must be Bool. The result has the state type.
No shadowing of visible bindings/functions or reserved names is allowed, just as for fold accumulators.
Body locals are fresh each iteration, do not escape, and are not visible to the condition. Nested loops
have separate state scopes; surrounding values remain unchanged. Check all children even for constant false.

## Sequential semantics and resource charges

1. Charge the ordinary expression-node tick, then evaluate the initializer once. A fault stops immediately.
2. Charge one control-probe tick before evaluating the condition in the current state's scope. Normal
   expression/call/operation charges also apply to the condition, including the final false condition.
3. If false, return the current state without evaluating the body or performing another probe.
4. If true, evaluate the body in a fresh lexical scope. Its result becomes the next carried state; repeat
   step 2. There is no additional body-entry tick besides the control probe and ordinary body charges.

There is no upfront iteration count/admission test or hidden step limit. Fuel exhaustion, heap exhaustion,
checked arithmetic, bounds faults, and callee failures stop the invocation with structured diagnostics;
never return a partial state. A nonterminating loop exhausts finite fuel even with a literal true condition
and trivial body. Initialization/condition/body share the invocation's existing cumulative allocation budget.
Iteration adds no call frames or iteration-dependent native stack storage. Calls retain the 32-frame policy.
Native lowering uses a C loop and typed state local, not recursive desugaring or interpretation. Avoid
iteration counters and C integer-overflow hazards. Optimizers must preserve all observable checks/faults.

Action bounds conservatively include initializer, condition, and body regardless of reachability. Contracts
may use loops, but all children/callees must be pure and any fault retains the enclosing contract phase.
No assignment statement, break/continue, host operation, capability grant, or new authority is added.

## Exact examples and acceptance gates

For a zero-argument function with no contracts:

- `while (n: I64 = 41; false) { 1 / 0 }` returns 41 using exactly five ticks:
  function, loop node, initializer, control probe, false literal. Four ticks exhaust fuel.
- `while (n: I64 = 1 / 0; false) { n }` faults in initialization, before the first probe.
- `while (n: I64 = 0; true) { n }` exhausts finite fuel, never stack depth.
- A false condition skips even an overflowing/allocating body, but that body is still statically checked.
- State-dependent conditions, nested folds/loops, records/variants/lists/buffers, fresh body locals,
  action bounds, pure contracts, source ranges, malformed syntax, and forged artifacts need regressions.
- A long-list search with a match at the head returns with work independent of the unused tail after
  input admission; compare short/long inputs using the same sufficient execution fuel.
- Convert parser reduction and tree search away from stopped-state counted folds, preserve frozen
  application outputs, and run native/reference/ASan/UBSan/leak and real VS Code host gates.

Existing source behavior is retained, including ordinary `while` identifiers and all counted folds. Emit
only the new artifact/language pair and reject older versions with `invocation.artifact.version`; recompile
source rather than retagging checked IR. Previously generated native executables remain independent.

## Checked artifact extension

The expression node is `While` with `state: {name, ty}`, `initial`, `condition`, and `body` (a Block).
State type/function references resolve to stable IDs as elsewhere. Exact source spans are ephemeral and
omitted from artifact JSON. Revalidation checks names/types/scopes/actions, all children, normal expression
depth/compound-temporary limits, and rejects unknown/missing fields. This is compiler IR, not workspace state.

This slice addresses demonstrated state-driven termination, not cross-file composition, general mutable
state, a termination proof, an operating-system sandbox, or a Turing-completeness claim.
