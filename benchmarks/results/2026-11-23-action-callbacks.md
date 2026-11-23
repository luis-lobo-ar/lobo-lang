# Core 1.11 action-bound callbacks and HTTP extraction

Date: 2026-11-23. Status: verified local milestone. Compiler/extension 1.11.0, Core 1.11,
checked format 1.0, native ABI 1 and optional SQLite extension 1. The user selected this scope.
The [receipt](2026-11-23-action-callbacks.json) records commands, artifacts, failures and limits.

## Result

Callback types declare finite action sets before their return arrow:
`Fn(C, Request) actions { db.close, db.execute, db.read } -> Response`.
The location preserves existing functions returning pure Fn values with their own post-output action clauses.
Omitted/empty bounds stay pure. Parameter/output types and action sets match exactly. Named references capture
no context; C is an ordinary argument. Indirect calls check the enclosing function's bound, including dead
branches, and contracts cannot invoke actionful callbacks. Artifact loads and typed inputs revalidate the
complete target signature. C and LLVM dispatch use the same signature, with normal contracts and quotas.

The SQLite app delegates serving to `serve_with(cap, port, max_requests, db, handle)`. It retains capability
restriction, optional setup, read-only database open and the final database close. Library helpers own HTTP
framing, connection deadlines, partial IO and listener/stream cleanup. Captured outcomes and close-error
precedence remain. The pure API stays available. The selected context API has a concrete DB handler bound;
closures, implicit widening and action polymorphism remain excluded. See [Core 1.11](../../spec/core-v1.11.md),
[decision 0059](../../decisions/0059-action-bound-callbacks-and-http-library.md), the
[HTTP library](../../libraries/http-server/README.md) and [app](../../examples/native/sqlite-rest/app.lobo).

## Verification

The full Rust suite passes 304 tests in 56 result groups, with zero failures or ignored cases.
The final seven callback groups also pass strengthened inference/equality, 64/65-action, forged-signature and
nested old-language assertions. Native comparisons cover the evaluator, C, LLVM and both sanitizer routes.
An actual clock callback succeeds under one native call, defaults to denial without a launch grant, and
returns quota error 11 after a second invocation. Callback contracts, returned/list-stored values, original
diagnostic spans, exact bounds and explicit-context generic inference have executable coverage.

All 178 editor tests and the real isolated VS Code host pass. The host checks callback/result-field
navigation, an unsaved missing-action diagnostic at the invocation, and repair. The 1.11.0 VSIX is packaged
and installed; the app workspace window was confirmed. Skill validation, source-tool tests, format and
Clippy with warnings denied pass. JSON/JSONL/local-link validation passes.

The context HTTP fixture matches the pure handler on all 30 protocol inputs across execution routes.
Actual SQLite servers on Linux C, Linux LLVM and Windows PE through Wine each pass 35 scenarios using 38
connections, including fragmented requests, timeout, truncated input, occupied port and default denial.
The original pure Linux server also passes that socket driver. Both Linux SQLite backends return 500 for
four malformed row types. Linux C and Wine each pass ten SQL operations, two read-only restarts and an
independent Python SQLite file oracle. Actual Windows hardware execution remains unknown.

A saved Core 1.10 compiler loads and executes a representative current pure callback artifact after language
retagging; its checked representation is otherwise identical. New action-bearing callback signatures,
including unused nested types and reference-only cases, cannot be retagged as older languages.

## Corrections retained

The initial Rust, editor and grammar tests fail before implementation. The HTTP import fails before its
helper exists. A first artifact-erasure test incorrectly required rejection of a valid unused pure API;
adding an actual actionful caller made the intended forgery test meaningful. Native harness assumptions
about `--native-clock false` and a zero-call budget were replaced by omission and two calls under one grant.
The initial baseline version probe and a mistyped nearby test target were tooling errors, not compiler bugs.

Extraction exposed required Request/Response visibility and ambiguous authoring aliases; the app keeps the
type imports and their known IDs. An older inference test caught a callback-action diagnostic range moving
to its enclosing call. The implementation now attaches that diagnostic to the original argument, retaining
the old assertion. The first full suite failed the maintained-layout audit on six files. The existing checked
formatter applied layout changes only, and the audit then passes. Linux/Windows emitted C remains byte-identical
after formatting. An incidental test-edit anchor failed before writing the test file and was corrected.
Original logs, failed runs and accepted formatter receipts remain under the raw evidence directory.

## Preservation and limits

All 154 prior sidecar identity values remain. Four app helper IDs move with their definitions; existing pure
API, app entry, controller and model IDs remain. The 64 protected provider/header, model, vendored and frozen
source inputs are byte-identical. All 33 prior paper entries retain their pinned hashes. Before/current source,
compiler, raw evidence and artifacts are preserved in the private bundle recorded by the receipt.

The 107-to-39 app line reduction is a source observation. No agent construction, comprehension, token or
runtime-performance benefit was measured. No automatic cleanup, new native authority, provider/model change,
HTTP expansion or concurrency was added. The Rust suite reruns existing C witnesses and source correspondence;
full TLC was not rerun, and prior finite Linux model claims do not prove callback typing or Windows behavior.
This remains a local milestone in a dirty working tree, preserving prior uncommitted work.
