# 0047 Bind JSON literal builders by stable identity

Date 2026-11-22. Status implemented. Risk mode standard.

The user selected automatic `object`/`field` handling for JSON literals, with TDD, extension maintenance,
documentation and the decision ledger. The current MVC selected builders it never directly called.

## Choice and alternatives

Keep `Json` explicit and resolve builders by stable ID from the supplied source closure. Reuse a correctly
selected builder alias; otherwise add a reserved generated import. Keep dependencies inspectable in checked
module scope. Source checks never allocate identities or discover files. The [1.6 contract](../spec/core-v1.6.md)
defines the full behavior. Stop at this selected scope.

Keeping mandatory builder selections repeats lowering details in author code. Globally importing ordinary
builder names would change lexical lookup and permit accidental collisions. A new JSON runtime intrinsic
would duplicate the library implementation and bypass its contracts. Select generated imports instead,
using the existing closed linker/checker and native backends. This is one implementation with self-review
and differential validation; no independent candidate implementation or agent-performance experiment ran.

The compiler parser records generated imports before module assembly. Each costs eight frontend tokens.
Empty objects need only the object builder. Helpers retain type/action/contract validation. User-defined
same-spelled functions cannot hijack a literal. Source identifiers cannot address reserved compiler aliases.
Correct explicit imports retain their previous lowering. An unrelated invalid explicit helper import now
fails normal module linking rather than the former literal-selection syntax check; it remains rejected.

The editor mirrors this lowering with generated-token provenance. Its optional legacy text export uses
fresh ordinary aliases to remain compatible with the frozen grammar. Navigation uses reserved aliases
internally, preserving original expression ranges. No new grammar highlighting rule is needed.

## Acceptance and preservation

Failing-first compiler/editor regressions cover import omission, checked/native equality, escaping,
empty objects, missing stable targets, local/imported collisions, hidden-name rejection, type/action checks,
source ranges and compatibility export. Real VS Code checks cover field/type definitions and an unsaved
invalid value followed by repair. C/LLVM and sanitizer paths remain part of native verification.

The generic MVC retires only two source binding keys; every remaining ID and all declaration IDs stay fixed.
Comparison normalizes helper alias labels and import order only. Explicit compact/legacy comparison files,
prior specifications, migration fingerprints and frozen research/release fixtures remain unchanged.
The programming skill gains an independent small JSON example. Compiler and extension advance to 1.6.0.

[The result record](../benchmarks/results/2026-11-22-json-auto-helpers.md) retains failures, final checks and
limits. This verifies engineering behavior, not improved agent construction, comprehension or token usage.
