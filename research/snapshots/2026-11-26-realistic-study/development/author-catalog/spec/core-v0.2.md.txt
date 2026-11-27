# Lobo Core v0.2: counted folds

Historical extension, inherited by the current language in the [specification index](README.md).
Version-specific artifact/rollout rules below describe the 0.2 compiler, not the current loader.

Version: `core/0.2-draft`. This extension inherits the semantics of
[Core v0.1](core-v0.1.md), adding one pure counted-fold expression. Native machine code remains the default.
The checked format is [lobo-checked/0.2](checked-artifact-v0.2.md).

## Grammar and contextual words

Add this alternative to `primary`:

```ebnf
fold = "fold", "(", identifier, "in", expression, ",",
       identifier, ":", type, "=", expression, ")", block ;
```

The first identifier is the index and the second is the accumulator. Recognize a fold only when the next four
tokens at a primary-expression position are `fold`, `(`, an identifier token, and `in`. Matching that prefix
commits to fold parsing; a malformed remainder is a syntax error, not a fallback call. Otherwise ordinary
identifier/call parsing applies. `fold` and `in` are contextual words, not newly reserved global names:
functions named `fold`, parameters named `in`, and calls such as `fold(in)` remain legal.

## Static rules

- The count expression must be I64.
- The accumulator annotation is mandatory and may be I64 or Bool. The initializer and body's final value
  must match it exactly. The fold's result has that type.
- Count and initializer are checked in the outer lexical environment. Neither new name is visible there.
- The body additionally sees index:I64 and accumulator:annotated-type. These names must be distinct, valid,
  nonreserved, and cannot shadow visible locals, parameters, or functions. Body bindings cannot shadow them.
- Body locals do not escape; every iteration has a fresh lexical environment. Outer bindings stay immutable.
- Check all expressions and bindings even for a constant zero count or an unreachable fold.
- Nested folds obey the existing frontend and expression-depth limits.
- Folds may occur anywhere their result type is valid, including within contracts. Only the complete
  `requires`/`ensures` expression must be Bool; a nested numeric fold is permitted.

## Sequential evaluation

1. Evaluate the count once. If it faults, stop.
2. If the count is negative, produce `runtime.iteration.negative_count` immediately. Do not evaluate the
   initializer or body. A negative constant count is well typed and remains a runtime fault if reached.
3. Evaluate the initializer once, including for zero iterations. If it faults, stop.
4. For count zero, return the initializer; do not evaluate the body.
5. For positive count n, enter iterations in ascending index order, 0 through n-1. The first accumulator is
   the initializer. Each body's final value becomes the next iteration's accumulator; return the last value.
6. Each entered iteration consumes fuel before its body, in addition to normal evaluated-expression charges.
   A fault or exhaustion stops the invocation; no partial accumulator is returned.

There is no assignment statement, general mutation, `break`, `continue`, custom stride, parallel reduction,
collection allocation, or multiple accumulators in this slice. Bindings within each iteration are immutable;
the next accumulator is a new carried value, not a mutation of an outer variable.

Checked arithmetic, first-fault ordering, short-circuiting, and runtime contracts retain their existing rules.
Runtime faults in folds evaluated during contracts preserve their primary code and enclosing contract phase.
Calling a function in a body uses its ordinary contracts and depth limits.

## Host limits and native lowering

Any nonnegative I64 count is valid; there is no hidden iteration cap or upfront fuel admission test. Finite
count does not guarantee completion under host budgets or termination of recursive body calls. Exhaustion
is `resource_exhausted`, never a language arithmetic fault or a successful partial value. Exact cross-backend
fuel accounting remains outside conformance, but each backend must charge every entered iteration.

Iteration itself adds no function-call frames and no count-dependent stack allocation. Body calls and nested
expressions retain their usual limits. Lower folds to sequential C loops with typed index/accumulator locals,
then Clang machine code, not recursive desugaring or an evaluator. With count <= I64_MAX, an executed index
is at most MAX-1; the loop counter can reach MAX only to terminate. Counter arithmetic must avoid C overflow.
Optimizations must preserve observable failures, contracts, and the native host's resource checks.

## Required examples

| Expression | Expected outcome |
| --- | --- |
| `fold (i in 0, a: I64 = 41) { 1 / 0 }` | 41 |
| `fold (i in 0, a: I64 = 1 / 0) { a }` | `runtime.integer.zero_divisor` |
| `fold (i in -1, a: I64 = 1 / 0) { a }` | `runtime.iteration.negative_count` |
| `fold (i in 5, a: I64 = 0) { a + i + 1 }` | 15 |
| `fold (i in 1, a: I64 = 7) { a + i }` | 7 |
| `fold (i in 3, a: Bool = false) { a || i == 1 }` | true |
| `fold (i in 1, a: I64 = 9223372036854775807) { a + 1 }` | `runtime.integer.overflow` |
| `fold (i in true, a: I64 = 0) { a }` | `compile.type.mismatch` |
| `fold (i in 0, a: I64 = 0) { false }` | `compile.type.mismatch` |

Additional fixtures cover scope, duplicate names, initializer visibility, nesting, calls, contract phases,
first-fault ordering, huge counts with finite fuel, checked-artifact tampering, and contextual-name compatibility.

## Compatibility and rollout

The new compiler emits only `core/0.2-draft` / `lobo-checked/0.2` and rejects old checked-artifact versions
with `invocation.artifact.version`; no implicit retagging, migration, or dual-version loader is provided.
Recompile source to obtain the new artifact. Earlier generated executables and the old compiler remain usable.
Inherited source programs keep their meaning, including ordinary names `fold` and `in`. Existing host-specific
limits remain applicable. Compiler release and native-adapter versions are separate from the language version.

This is an expression-tree extension, not canonical workspace graph interchange. A single scalar accumulator
can express iterative sums and factorial; ordinary iterative GCD needs multiple carried values and is not
enabled by this feature alone. No new general speed ranking or termination proof is claimed.
