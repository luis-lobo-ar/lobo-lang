# 0030: Explicit generic function specialization

Date: 2026-11-04. Status: implemented experimentally in Core 0.11; acceptance evidence linked below.

The [reuse workload](../spec/generic-reuse-workload.md) repeats collection logic for unrelated
types. Existing List operations already support every element type, but user functions cannot
abstract over those elements. Add explicit first-order function templates and concrete calls:
`function copy<T> @id("copy")...` and `copy::<Text>(xs, suffix)`.

Alternatives: keep duplicating functions; implement runtime type erasure/dictionaries; or specialize
at compile time. Duplication fails the workload. Erasure would change the value representation and
runtime without evidence of need. Choose specialization before the existing concrete checker/backend.
No new dependency, interpreter, dynamic dispatch, allocator, or host authority is needed.

Templates resolve names in their defining module; each requested concrete body, contracts and action
bounds undergo the ordinary checker. Unused templates are parsed and their declaration references resolve;
local-name and body-type checks occur only for requested instances.
This is instantiation-based checking, not universally checked parametric polymorphism. Unsupported
operations fail on the requested instantiation. Call type arguments are mandatory; no inference,
overloading, constraints, generic records/variants, or function values in this slice.

Lowered specializations have explicit generated scopes. Imports preserve the definition's resolved
dependencies and admit the concrete nominal types supplied at the call. Original caller scopes gain
only generated function imports. The saved artifact contains concrete functions/scopes only and is
fully rechecked on load. Generated identities depend on template identity and structural type arguments;
hash collisions must be detected and rejected. Generated scope/function count and metadata are bounded.

Require failing-before evidence, malformed/artifact/module tests, recursion and exhaustion tests,
native/reference/sanitizer agreement, editor navigation/highlighting, and maintained skill examples.
Retain the Core 0.10 baseline and report limitations alongside results. A future universally checked
generic type system may supersede this rule; it must not be claimed by this implementation.

Implementation: source-local type variables lower through a bounded work list to ordinary concrete
functions and scopes. Requested type arguments are checked as value types at the call boundary, keeping
invalid function-as-type diagnostics attached to the caller. One reserved generated-name namespace avoids
capturing source bindings. No new runtime representation or dependency was needed.

[Verification and corrections](../benchmarks/results/2026-11-04-core011-verification.md) retain the initial
red tests, failed fixture/harness attempts, the source-range regression, native/sanitizer acceptance and
agent-tool invalidation checks. Agent construction/comprehension effects remain unmeasured.
