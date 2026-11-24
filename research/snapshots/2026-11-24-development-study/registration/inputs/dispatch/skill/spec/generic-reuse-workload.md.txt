# Generic reuse development workload

Frozen before implementation, 2026-11-04. Public development fixture, never a held-out evaluation.

Define one recursive `copy<T>(xs: List<T>, suffix: List<T>) -> List<T>` helper that preserves
the order of xs followed by suffix, and one `contains<T>(xs: List<T>, value: T) -> Bool` helper
with early termination. Instantiate the same definitions for I64, UTF-8 Text, and a caller-owned
record. The library must not import that record. Use explicit type arguments, including inside
recursive calls. All computation must run in Lobo, through reference/native/sanitizer paths.

Acceptance: copy [1,2] onto [3] gives [1,2,3]; empty copy preserves suffix. Text copy preserves
Unicode and order. Record membership uses structural equality, with present/absent/empty cases.
Nested List arguments and mutually recursive generic helpers must work without unbounded compiler
recursion. Repeated identical requests share a specialization. Source order must not change its ID.
Missing/extra type arguments, type arguments on ordinary functions, mismatched values, unknown types,
missing action bounds, impure contracts and type-growing instantiation must fail predictably.
Checked-artifact round trips must recheck concrete bodies and scopes; forged generic syntax must fail.

This demonstrates reusable first-order functions. It does not require generic nominal types, type
inference, closures, traits, or a universal generic-body type theorem. Compile-time and runtime
resource limits remain observable. No productivity, token-cost, or comprehension improvement is assumed.

## Public comprehension probes

Added as development documentation after initial implementation, not frozen unseen evaluation questions.
An assessment should distinguish knowing the output order from understanding dependencies and invalidation:

- Predict `copy([1,2], [3])` and show which expression preserves that order.
- Identify why an imported helper can use a caller's record while its private helper remains library-owned.
- Explain why one passing I64 specialization does not establish that `x + x` supports Bool.
- Identify every instantiated dependent invalidated by changing a generic contract, and why adding an
  unrelated specialization leaves existing function identities unchanged.
- Localize a missing type argument versus a concrete body type error to their respective source locations.

Compiler/tool checks support the corresponding language facts. No model/human answers have been collected
or scored; construction acceptance and comprehension remain separate outcomes.
