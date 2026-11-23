# Future additions and explicit stopping points

Date: 2026-11-04. Status: the user explicitly authorized the four-family milestone through local language
1.0 in [decision 0033](../decisions/0033-bounded-language-1-release.md), superseding decision 0032 for that
scope only. All four families and the three fixed programs/changes pass focused acceptance in Core 0.12
development, now versioned as language 1.0. Final package/check status is in the
[release receipt](../benchmarks/results/2026-11-04-language-v1-release.json). Core 0.11/compiler 0.11.1 remains a preserved evaluation
checkpoint. Application libraries/workspace services remain outside this language-only completion gate.
It does not replace the broader [product v1.0 target](v1.0-target.md).

## Enough for the current research checkpoint

No new language feature is required. The implemented subset already supports the public parser/evaluator,
collection transformation with typed failures, tree/AST computation, module composition and native tooling
checks. That is enough for a scoped construction/comprehension study, not every possible application.
Study design must freeze tasks, versions, outcomes and budgets before trials. Negative results and missing
capabilities are findings, not instructions to expand the language during the study. A valid study can
report no advantage; outperforming another language is not a prerequisite for reporting it.

## A closed candidate for a later reusable language core

The authorized scope is exactly four feature families. Additional ideas do not silently join this milestone.
The table preserves the reasons identified at the earlier checkpoint; it is not a list of current omissions.
[Acceptance details](../spec/language-v1-acceptance.md) were frozen before production changes. Implementation
and negative results are retained in [the running record](../benchmarks/results/2026-11-04-language-v1-progress.md).

| ID | Recommended addition | Concrete reason | Sufficient implementation for this milestone |
| --- | --- | --- | --- |
| L1 | Generic records and variants | Generic functions cannot yet carry reusable compound loop state or a reusable typed optional/error result | A shared Cursor<T>, Option<T> and Result<T,E> work across I64, Text and caller-owned records in separate modules, including nested types, exhaustive matches, contracts and guarded recursion |
| L2 | Definition-time generic checking | An unused Core 0.11 template can still hide local-name/type errors until instantiated | Check bodies against their declared type requirements before use, issue source-located errors, then recheck concrete instances. Define compatibility with current templates. Limit constraints to operations required by the frozen workloads |
| L3 | Non-capturing function references | A reusable algorithm cannot yet receive a named predicate/transform as a value | One map/filter-style algorithm accepts two different named functions, with checked signatures/action bounds, preserved contracts and explicit resource/failure behavior. Captured environments are outside this item |
| L4 | One floating-point type, F64 | I64-only arithmetic excludes ordinary fractional numerical inputs unless users invent an encoding | Define and implement literals, arithmetic, conversions, comparisons and interchange, including rounding, non-finite values and signed zero. A small numerical aggregation workload has independent expected outcomes and consistent backend behavior |

L1 and L3 extend reusable expression of data/behavior; L2 makes generic diagnostics predictable; L4 is a
proposed expansion of practical numeric scope. L4 is not a claim that floating point is needed for
computational completeness. None is needed to reopen or finish the existing research checkpoint.
Minimal type requirements do not imply an unrestricted trait, overload or metaprogramming system.
Static generic type checking does not prove contracts, termination or arbitrary program correctness.

Existing-language baselines support these being ordinary mechanisms, not novelty claims. Primary references
consulted on the proposal date are [Rust generic data types](https://doc.rust-lang.org/book/ch10-01-syntax.html),
[function references](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html), and
[floating-point semantics](https://doc.rust-lang.org/std/primitive.f64.html). This proposal makes no measured
agent-benefit claim from those references.

## The later language milestone ends at these gates

Before starting, freeze three public acceptance programs and their exact requirement changes:

1. A generic data transformation across integers, text and records, with optional/typed failure outcomes.
2. A multi-module in-memory document validator/query/transformer, with a schema change and dependent updates.
3. A numerical aggregation/validation program using fractional inputs, explicit conversion and error cases.

The first program includes an internally generated 10,000-element iterative case. Specify realistic sizes
and budgets for all three before implementation. The programs must not outsource their application logic
to Rust, Python, Node or hand-written host wrappers, or require copied type-specific algorithms.
Small pure helper libraries are part of the fixtures, not an unbounded library deliverable.

The milestone is sufficient when all four feature-family acceptance sets and all three program requirements
pass, including the frozen changes, boundary cases, invalid-input/type rejection and specified exhaustion.
Required engineering gates are native/reference agreement, both implemented native backends, active
sanitizer checks, artifact revalidation, stable source diagnostics, and matching editor/agent-skill behavior.
Retain exact versions, source identities, commands, independent oracles, failures and corrections.

Source/type/value/call budgets must be explicit and support the agreed fixtures. This is an implementation
hardening gate, not a requirement to accept unbounded programs. A limitation cannot be silently removed,
or a budget silently raised, merely to change an evaluation outcome. Proposed changes to these gates need
their own recorded scope revision before a measured round.

After these gates pass, the language-only milestone is complete within its advertised scope. Further
features remain separate milestones. Construction and code comprehension are evaluated separately;
passing compiler tests supplies no comprehension score or comparative agent advantage.

## Future application and workspace milestones are separate

| Future area | Examples the user has raised | What would count as enough |
| --- | --- | --- |
| Pure standard libraries | Sequence/map/set operations, searching, text conversion, JSON serialization | Implement the fixed API set required by one selected application, with independent behavior/resource tests. Prefer Lobo libraries over new syntax when they suffice |
| Controlled host access and foreign libraries | Communications, storage, external search, reusable native libraries | One identified native application works end-to-end through a specified ABI/ownership/error boundary and explicit resource grants, cancellation and quotas. Cover denial and failure paths. Select the application first |
| Operational policy | Rate limits, audit, logging and tracing | The selected application's required policies are enforced, with documented charging, revocation, sink failure and recovery behavior. Source action labels alone are not enforcement |
| Agent workspace and context | Typed tools, semantic queries, versioned state, atomic changes, affected checks and evidence | One bounded edit/verify/review workflow works on a pinned revision, with conflict/recovery and stale-evidence checks. Assess its construction/comprehension effects under a separate study protocol |

These rows are areas for later scoping, not a commitment to implement every service in them. Native wrappers
alone do not isolate arbitrary untrusted foreign code. The app-specific trust/resource boundary must be
defined before treating external libraries as safe. None of these rows blocks the current research checkpoint
or the four-family language milestone above.

## Useful possibilities that are not required for enough

Capturing closures, async/concurrency, classes/inheritance, macros/reflection, operator overloading, broad
type inference, higher-kinded types, arbitrary-precision/decimal/SIMD families, a custom GC, further LLVM
optimization and self-hosting remain optional. A later use case may justify one, but another language's
feature list does not. Adding optional work never changes an already frozen acceptance gate retroactively.

User question, 2026-11-05: [symbolic shortcuts proposal](../decisions/0037-symbolic-shortcuts-proposal.md)
identifies Text concatenation in the controller as a candidate for a small post-1.0 prototype. Exact static
resolution and existing operation/resource semantics must remain visible. Arbitrary symbols/precedence and
general user overloads remain deferred. This records a recommendation, not implementation authorization.

On 2026-11-08 the user selected the bounded operator milestone. [Decision 0038](../decisions/0038-explicit-typed-operator-bindings.md)
and [language 1.1](../spec/core-v1.1.md) define exact binary arithmetic bindings and Text concatenation.
The earlier proposal above records the discussion before authorization.

This is a proposed list and stopping contract. No feature, experiment, dependency or ecosystem implementation
starts from it automatically. Current implementation claims remain those in the [paper evidence](../research/local-evidence.md).
