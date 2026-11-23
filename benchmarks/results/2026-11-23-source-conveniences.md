# Record updates and standalone try, Core 1.10

Compiler/extension 1.10.0 implement both user-selected source conveniences. The
[Core 1.10 contract](../../spec/core-v1.10.md) and
[decision 0058](../../decisions/0058-record-updates-and-standalone-try.md) define the bounded behavior.
The [receipt](2026-11-23-source-conveniences.json) records checks, artifacts and source preservation.

## Implemented

`State { ...state, changed: value }` evaluates one same-type base once, evaluates overrides in written
order and preserves other fields in a new immutable value. Generic arguments can be inferred from the base.
Unknown/repeated fields, mismatched nominal types/arguments and misplaced/multiple/two-dot bases fail.
With no overrides it preserves all fields. Empty Text, empty Lists and None remain values; an explicit empty
override still replaces its field. An untyped empty object is rejected. Partial/dynamic merging is not added.

`try operation();` discards its successful payload and propagates Err under the existing type/action and
postcondition rules. Let bindings and statements can interleave. Blocks still require a final value.
Ordinary calls/values and compound expressions cannot silently become discarded statements. Contextual
try(value) calls and legacy source rejection diagnostics remain compatible. Cleanup remains explicit.

Both forms lower to existing checked nodes. Updates use one ordinary fold iteration with ordered override
bindings; statements use an unused ordinary binding holding Unary try. Generated names are deterministic
per function and independent of whitespace. Checked format 1.0, native ABI 1, SQLite extension 1 and all
admission limits remain unchanged. Temporary source bases cannot be loaded from checked JSON, including null.
Expansion costs ordinary fuel/allocations and is not a runtime optimization claim.

Five maintained SQLite-family sources now use applicable forms, with unchanged sidecar bytes and resource
cleanup order. The new independent programming-skill example executes both conveniences. The older source
migration checkpoint, frozen examples/research, native providers and model inputs remain unchanged.
The VS Code extension highlights one three-dot operator, resolves bases/override fields and inferred generic
members, handles statements, and displays the original bad-override expression in unsaved diagnostics.

## Verification

The final full Rust run passes 296 tests with zero failures or ignored tests. All 173 editor tests pass.
The real isolated VS Code host checks update navigation, standalone try, invalid override diagnostics and
repair. VSIX 1.10.0 is rebuilt. Clippy with warnings denied, formatting, source tools, skill validation and
JSON/local links pass. The checked modernization audit covers 44 files, accepts zero rewrites and makes
143 compiler calls.

Nine convenience conformance groups cover the evaluator, C, LLVM and both ASan/UBSan routes; generic and
cross-module updates; field preservation; order and skipped later faults; propagated errors/postconditions;
reserved names; legacy/loaded-artifact boundaries; whitespace identity; and original override ranges.
A native one-call quota independently checks that a base containing clock IO executes once. Nested updates
at depth 24 work, while excessive nesting returns resource.frontend.limit without a compiler stack overflow.
The preserved actual 1.9 compiler executes a retagged, lowered skill artifact identically to 1.10, demonstrating
that no new checked execution node is needed. It does not establish old source-parser support for new syntax.

The migrated SQLite REST application passes 35 real HTTP scenarios on Linux C, Linux LLVM and Windows PE
under Wine. Both Linux backends also pass four malformed-row HTTP cases. Linux C and Wine each pass ten
SQL operations, two read-only server restarts, Unicode/injection-like values and an independent on-disk
SQLite oracle. Actual Windows hardware coverage remains unknown. Prior API and raw-provider suites run
through the full Cargo gate. The finite models are not rerun for this source-only change: reviewed C/model
fingerprints are unchanged and the ordinary concrete replay/source-map gates pass.

## Failures and corrections

The first new positive tests failed on unsupported update syntax before implementation. The editor tests
also fail on the original resolver. A new test first used the wrong definition-result property; its corrected
form still failed before the resolver change. A grammar expectation used the wrong preexisting try scope;
it was corrected to keyword.control.propagation.lobo. These are test corrections, not language defects.

A precise-range test exposed generated field references blaming the whole update. Their stored ranges now
point at the original override value. A whitespace test exposed generated names based on byte offsets;
per-function ordinals fix canonical identity. Two Parser construction sites were updated for that counter;
a missing initializer and an initially unqualified serde trait caused retained compilation failures.

The first broad run caught changed diagnostic codes/ranges for previously rejected legacy statements and a
parser stack overflow in the existing deep-loop rejection case. The parser now only consumes a semicolon
for an admitted standalone try; other rejection locations/codes are unchanged. Let and record parsing are
split out of recursive parser frames, and tail expressions are boxed. The original nesting limits and test
thread stack remain; no increased stack setting, relaxed limit or weakened oracle hides the failure.
An added empty-value fixture initially shadowed the existing copy function; renaming that local fixes it.
All initial/corrected logs are retained, including the intermediate broad runs rather than only the final pass.

## Evidence and limits

Raw logs, before/current source snapshots, source/tool hashes, migration receipts, compiler binaries and
application artifacts are retained locally under `benchmarks/private-evidence/core110-20261123` with a
manifest. Final reports and the paper index are excluded from that manifest to avoid hash cycles.
The pre-task snapshot includes the earlier uncommitted DX review/utilities. Real repository HEAD alone
therefore does not identify the starting tree; exact input hashes do.

The reviewed C/model inputs and 87 protected source/provenance inputs remain unchanged, as do the five
existing migrated sidecars. All 32 prior paper entries and pinned files, and the 86 frozen v1 release
artifacts, are preserved. This work changes source convenience and charged elaboration, not the deferred
cleanup/callback/project-manifest designs. No new agent, token-cost, construction, comprehension or runtime
performance experiment was run. Artifact sizes are observations, not speedup evidence.
