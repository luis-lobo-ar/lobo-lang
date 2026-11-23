# Constructor inference and contextual cases

Date 2026-11-22, America/Chicago. Engineering acceptance passed for compiler/language 1.5 and extension 1.5.0.
The user selected constructor inference with TDD, editor maintenance, documentation and the decision ledger.
The [contract](../../spec/core-v1.5.md) and [decision](../../decisions/0046-constructor-inference.md) define scope.

## Verified result

The maintained MVC now returns `Found(User { name: "Luis" })` or `Missing` from a function whose explicit
return type is `Lookup<User>`. The expected nominal identity supplies the case owner and generic arguments.
Qualified variant constructors can infer from payloads; generic record constructors infer from fields and
matching expected types. Typed locals, call arguments, constructor members and loop state provide context.
No global case search, signature inference or generic-call return-context solver is added.

All existing MVC identity bindings are unchanged. Comparing the retained compiler 1.4 with explicit
constructors against release compiler 1.5 with inferred constructors gives exactly equal checked programs
and artifact format, with different language tags. The main programming-skill example also retains its
complete checked program after constructor spelling changes. Native C/LLVM MVC invocations return status
200 with `{"name":"Luis"}` for user 1, and 404 with `Not found` plus newline for user 42.

Final verification passes 226 Rust tests, 153 editor tests, 31 Python benchmark checks, four codemod tests
and six native HTTP checks. Real VS Code host tests cover constructor definitions, duplicate case spellings,
an unsaved incompatible-payload edit, its original source range and repair. Clippy, formatting, skill validation,
native skill examples and documentation links pass. Extension 1.5.0 is packaged locally. It is not installed
into a normal profile or published. The [receipt](2026-11-22-constructor-inference.json) retains exact evidence.

Seven constructor compiler tests cover explicit/inferred equality, evaluator/C/LLVM/sanitizers, generic bodies,
nested data, annotations, sibling/call constraints, imports, unknown owners, conflicts, lexical precedence,
loop initialization, callback purity, loaded-artifact rejection and legacy compatibility. Five new editor
tests cover those navigation contexts and bounded inferred-type growth. Existing tests remain in the totals.
Repeated full-suite runs are verification attempts, not independent research samples.

## Negative attempts and corrections

Before implementation, four positive constructor regressions fail and the existing rejection expectations
already pass. The first compiler implementation passes all five. Loop/callback and artifact/legacy coverage
are added afterward. A misplaced Rust test is briefly inserted into a Lobo fixture string; the resulting
unexpected-byte failure is recorded. Moving that test outside the fixture restores the fixture and runs all
seven cases. The first two complete Rust runs each pass 225 cases; the final suite passes 226.

The initial three editor regressions fail before implementation. The first attempt passes contextual and
imported cases but exposes a test fixture using reserved `result` as a local name. Renaming that fixture
binding preserves its intended assertions. The corrected final five tests are also run against the retained
1.4 editor source; all five fail there and pass with the new implementation.

An additional failing editor regression shows that deeply synthesized constructor types need the same
nesting cutoff as the compiler. Bounded type rendering, a resolution-query budget and snapshot-local successful
query memoization now reject that case without guessing a target. Review also corrects expected-type placement
on the fold initializer rather than its count. The loop-context regression and final host checks pass.
Patch applications with stale contexts made no accepted change; final source and all available run logs are retained.

## Limits and interpretation

Bare cases require an expected variant owner. A local binding or explicit declaration/import alias takes
precedence. Unannotated ambiguous values fail; phantom parameters need explicit context or arguments.
Expected contexts do not solve arbitrary constraints across unannotated bindings or identify an owner from a
case payload alone. Existing generic-callback limits remain. Qualified explicit forms are still available.
The existing parametric/concrete checkers, source budgets, purity and pre-copy type bounds remain in force.
Legacy source remains explicit, and checked-artifact loading never performs source inference.

The generic MVC changes only constructor spelling and the skill example preserves prior checked behavior.
The simpler MVC, frozen legacy comparison, migration fingerprints, old specifications, reports, study fixtures
and the 1.0 release remain unchanged. No agent construction-quality, comprehension, token or runtime-speed
comparison ran. Such effects remain unknown. Checked-code equality does not establish an agent productivity gain.

Local preservation is `benchmarks/private-evidence/constructor-inference-20261122`. It contains before/current
source, compilers, VSIX packages, driver, native observations, logs, paper index and integrity manifests.
The retained smoke driver supports a fresh `--out-dir` and runs from the repository root. This ignored local
archive is not an external backup, Git commit or publication.
