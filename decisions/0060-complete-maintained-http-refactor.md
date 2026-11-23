# 0060: Complete the maintained HTTP refactor

Date: 2026-11-23. Status: implemented and verified locally. Compiler/extension 1.11.1;
language remains Core 1.11, native ABI 1 and SQLite extension 1.

Finish the remaining maintained HTTP application scaffolding identified by the scope audit. Both native
REST apps delegate listener lifetime to the HTTP library. Centralize accept/retry/termination and request
preparation policy; keep thin pure and DB-bound callback adapters with precise existing action sets.
Preserve callback signatures, native grants, timeout/framing/error behavior, captured results before close,
close-error precedence, request/poll bounds and existing stable declaration IDs. Run native/reference/editor
and real socket/persistence gates, including repeated connections with the minimum live native handle budget.

Use a common accept state machine and common prepared-request representation under the existing language.
The two callback adapters still need their distinct action bounds. Their small request-driving expressions
are type adapters, not separate implementations of accept, framing, retry or cleanup policy. A single
arbitrary-action higher-order driver would require action polymorphism, which is outside this refactor.
Giving pure handlers database action bounds would weaken action precision. Generating source from templates
would add tooling and duplicated generated programs for a small library; neither alternative is selected.

Retarget the original listener-based serve declaration to serve_listener with its existing ID and signature.
Add a new capability/port-based serve lifecycle entry and migrate all maintained callers to it. Keep the
context lifecycle entry and moved SQLite IDs. Explicit library helper IDs remain allocated once. Keep the
raw loopback/OS demonstrations as direct ABI exercises. The web-server Lobo handler already uses shared
helpers; its separate Rust-host integration has different protocol limits and remains an explicit host demo.
These are reviewed roles, not claims that all excluded code is a frozen research fixture.

The acceptance inventory covers every maintained Lobo file using the existing maintainedPath predicate,
records each role and rejects unreviewed additions or application-owned HTTP listener loops. Frozen v1,
legacy comparison and illustrative fixtures, prior paper evidence, native providers and model inputs stay
byte-identical. The compiler grammar/type system and checked representation do not change. Replace the
obsolete application-source cleanup assertion with actual listener reuse and close-precedence tests.
Use the agentic-engineering standard workflow, TDD, failure-oriented native checks and the existing ledger.
No new language feature, native authority, dependency, HTTP feature or production deployment is selected.

The [report](../benchmarks/results/2026-11-23-maintained-http-refactor.md) and [receipt](../benchmarks/results/2026-11-23-maintained-http-refactor.json) retain all gates and corrections, including a tooling lockfile metadata repair discovered during version verification. Both old and current compilers accept the maintained source set. Stop at this refactor; small exact-bound callback adapters remain intentional.
