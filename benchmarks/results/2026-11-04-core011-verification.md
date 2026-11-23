# Core 0.11 generic-function development verification

Date: 2026-11-04. Implemented language/tooling development, not an agent experiment or release publication.
Compiler 0.11.0; `core/0.11-draft`; `lobo-checked/0.11`; native adapter 0.6; extension 0.13.0.

One generic collection implementation now works across I64, Text and caller-owned nominal records,
including imported templates, recursive calls and nested Lists. Concrete functions/scopes feed the
existing checker, evaluator and machine-code backend. The requested specialization's contracts/actions
and all concrete types are checked. Unrequested templates have declaration references resolved, while
local-name and body-type checking is deferred;
this is not universally checked parametric polymorphism.

Design/specification: [decision 0030](../../decisions/0030-explicit-generic-function-specialization.md),
[Core v0.11](../../spec/core-v0.11.md), [public workload](../../spec/generic-reuse-workload.md).
The workload was specified before implementation. Public comprehension probes were added afterward as
development documentation; no agent/human answers or comprehension scores were collected.

## Acceptance evidence

| Gate | Observed result |
| --- | --- |
| `cargo test --locked` | 159 passed, 0 failed/ignored in the final complete run |
| Generic integration tests | 7 passed; reference/native/ASan/UBSan/leak checks, modules, nested/recursive types, actions/contracts, artifact forgery, identity stability and budgets |
| Agent-tool generic invalidation regression | Contract changes execute both dependent instances and reuse the unrelated constant; an unrelated new specialization reuses all three existing checks; full mode executes all |
| Python harness suite | 30 passed |
| Editor/package suite | 114 passed, 0 failed/skipped; VSIX built |
| Real VS Code host | Exit 0 and `LOBO_HOST_TESTS_PASSED`; includes generic parameter, function and nominal argument navigation |
| Strict Clippy and formatting | Passed |
| Release compiler/test runner | Built with the existing optimized release profile |
| Skill | Validator passed; counter, framing, folder and generic Text examples checked/run natively |
| Repository checks | JSON/JSONL/local links and diff whitespace passed |

Generic runtime checks include an eight-cell successful one-element copy versus seven-cell exhaustion,
zero fuel, long recursive calls exhausting the existing stack budget, early membership, structural record
equality, UTF-8 preservation, exact copied order, and failed generic postconditions. Compile admission tests
accept 64 instances and reject a 65th, terminate type-growing recursion, and preserve function IDs when
unrelated instantiations are added. The compiler rejects a function used as a type argument at its call site.

The [native smoke record](2026-11-04-generic-reuse-native.json) contains exact build/invocation commands,
source/compiler/binary hashes and eight native/reference observations. The direct example executable is
22,888 bytes. The preserved Core 0.10 compiler rejects its new syntax. This is no runtime speed comparison.

The [foreground JSONL session](2026-11-04-generic-reuse-test-session.json) freezes four independent expected
values for number/Text/record/membership calls. Full verification passes four. A false identity contract
then fails three dependent tests and reuses membership; adding an unrelated Bool specialization reuses
all four original passes. Final full verification executes all four with no reuse. Every request/response
is retained; no model agent was launched and no expectation was changed to accept the canary.

## Failures and corrections retained

1. Before implementation all three initial generic tests failed at `<`/`::<` with `compile.syntax.invalid`.
2. The first implementation build tried to clone a work item lacking Clone. Replaced indexed cloning with
   an owned VecDeque work list, also avoiding retained duplicate bodies.
3. The first runtime fixture misspelled List CLI arguments and JSON `elements` as `values`. Corrected the
   fixture against the existing List interchange contract; no wire protocol changed.
4. Expanded tests initially used an incorrect postcondition diagnostic/action diagnostic spelling and
   removed imports from a scalar specialization that needed none. Corrected the oracles to existing
   diagnostics and removed actual nominal imports for the visibility forgery instead. Earlier logs remain.
5. Three new editor tests initially failed before generic navigation/grammar support; the extended suite
   then passed. A later bundle-diagnostic test called the single-source adapter with a bundle object;
   corrected it to `checkBundle`. The failed packaging attempt is retained.
6. A new function-as-type test exposed a real lost source range. Validate concrete type argument kinds at
   the requesting call, before generating an instance. The regression now passes with a source range.
7. The first full run containing the new agent-tool regression used the nonexistent expected status
   `all_checks_satisfied`; the actual documented status is `checks_satisfied`. Corrected that fixture;
   the next full run passes all 159 tests. The interrupted-at-test-failure run is retained unchanged.
8. Two patch attempts failed atomically on mismatched contexts; no files changed on those attempts.
   The tool transcript retains those errors; no separate raw filesystem log was created for them.

Exact log identities and a current source manifest are in the [machine-readable verification record](2026-11-04-core011-verification.json).
Raw command logs are copied under ignored `benchmarks/private-evidence/core011-evolution-20261104/`.
The [Core 0.10 baseline](2026-11-04-core010-language-baseline.json) preserves the previous dirty-workspace
compiler/test runner and selected source/spec/test/editor/skill archive. It is not a clean historical release.
The earlier Core 0.9 baseline and Core 0.10 verification records remain unchanged.

## Limits and next boundary

No generic nominal types, function values, inference, traits, universal template checking, new dependency,
runtime dispatch, capability operations or semantic daemon. Generated code can grow with instantiations.
Sixteen source modules, 64 instances, eighty lowered scopes, 1024 metadata slots, 32768 visited expression
nodes and existing type/runtime budgets bound this implementation. Recursive collection helpers retain
the existing call-depth limit; these tests do not establish arbitrary-scale collection performance.

Construction, code comprehension, token cost and productivity advantages over Rust/Python/other languages
remain unknown. Public fixtures are development evidence, not unseen paper evaluation tasks. Keep language
versions and documentation/tooling interventions explicit in later H1/H2/H3 studies.
