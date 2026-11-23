# 0008: Add counted folds without general mutation

- Status: implemented in compiler 0.2.0
- Date: 2026-10-12
- Risk mode: standard; language scope, failure ordering, native lowering, and artifact compatibility change.

## Outcome and alternatives

Support counted native computation without recursive stack growth, retaining pure Core, immutable bindings,
checked arithmetic, contracts, bounded host execution, and independently testable outcomes.

Choose one counted `fold` expression with a single I64 or Bool accumulator. Mutable `for`/`while` loops would
also require assignment, mutable scoping, and loop-exit definitions; defer those separate semantics. Tail-call
optimization would retain recursive source and interact with the current explicit call-depth policy; it is
not a substitute for specifying iteration. Multiple carried values need a separate structured-value design.

## Decisions

Follow [Core v0.2](../spec/core-v0.2.md): count once, reject negatives before initialization, initialize even
at zero, iterate indices 0..n-1 with fresh immutable bindings, charge every entered iteration, and stop on the
first fault. Type-check unused bodies. Contract expressions may contain numeric folds if the complete contract
is Bool. Preserve contextual ordinary names `fold`/`in` and commit to fold parsing only after the four-token prefix.

Lower to typed native C loops; no evaluator, recursive desugaring, new dependency, or custom runtime. Version
the checked artifact explicitly at 0.2. Reject old artifacts instead of adding implicit migration; retain old
executables/compiler and require source recompilation. Native runtime adapter version remains separate.

## Evidence and rollout

Use fixture-first implementation, then source/reloaded/reference/O3-native/ASan-UBSan comparisons for values,
faults, scope/type rejection, nested folds, contract phases, and finite fuel. Fresh-reader review resolved
contract-result typing and compatibility ambiguities. Keep benchmark baseline worktrees untouched; no
publication or branch merge is part of this change. Performance of new workloads remains unmeasured.

On 2026-10-12, all 23 integration tests passed in both debug and release profiles, including ten fold tests,
the inherited Core fixtures, and generated optimized/sanitized native checks. Formatting, Clippy with warnings
denied, and repository JSON/link checks passed. A nested-contract native diagnostic mismatch discovered by the
fold tests was fixed with dedicated phase/code/subject regressions. These checks establish finite evidence,
not general compiler correctness or platform certification.
