# Complete maintained HTTP refactor

Date: 2026-11-23. Status: verified local milestone. Compiler/extension 1.11.1 retain Core 1.11,
checked format 1.0, native ABI 1 and SQLite extension 1. The user explicitly selected this refactor after
[the scope audit](2026-11-23-http-refactor-scope.json). The
[receipt](2026-11-23-maintained-http-refactor.json) records inputs, commands, failures and artifacts.

## Result and scope

Both native HTTP apps delegate listener lifetime to the library. The stateless entry calls
`serve(cap, port, max_requests, handle)`; SQLite calls `serve_with(cap, port, max_requests, db, handle)`.
Capabilities remain application policy; SQLite bootstrap and database lifetime remain explicit.
`next_connection` owns the only native accept call site and accept/retry loop. `serving` defines request,
poll and error termination. `prepare_request` owns read faults and automatic HTTP rejection. The shared
finish helpers preserve stream/listener close-error precedence. Pure and DB adapters have small request-driving
expressions under their different exact action bounds. This does not claim one arbitrary-action callback loop.

The original listener-based serve is named serve_listener with its old ID and signature. A new owning serve
gets a fresh ID. All 163 prior sidecar ID values remain. An unchanged old stateless app with its old import
IDs compiles against the current library and passes the TCP suite. The final lifetime fixture additionally
uses an alias for the old target and confirms no added net.listen/DB action bound.

The [role manifest](../../scripts/http-refactor-scope.json) covers all 45 maintained Lobo sources using the
existing maintainedPath selection. The audit rejects the actual baseline for app-owned listener lifecycle
and duplicate accept sites; the current inventory has no violations. Raw loopback/OS examples retain their
ABI-testing purpose. The older Rust-hosted Lobo handler already uses shared HTTP helpers; its host has a
different protocol contract and remains an explicit integration example. These reviewed roles are distinct
from frozen v1/comparison/illustrative exclusions. No unrelated language/library feature is added.

## Verification

The complete Rust suite passes 309 tests in 57 result groups, with zero failures or ignored cases.
Four lifecycle groups exercise C, LLVM and both sanitizer routes, with pure boundary observations also using
the evaluator. Actual listener reuse is tested with a root capability plus one listener slot. Quota errors,
stream/listener close precedence, old target action precision and request/poll/error bounds are checked.
The final old-target alias assertion passes after the full suite, with no intervening production change.

An injected C accept function returns timeout 6 without waiting on the kernel. Both adapters stop after
1024 polls. A deliberately valid 1025-poll source mutant compiles and fails the observed-count assertion.
Optimized and ASan/UBSan runs cover both adapters. These eight recorded cases are four baseline cases and
four runs of one deliberate defect, not eight independent experiments or a new formal proof.

All 180 extension tests, the real isolated VS Code host, Clippy with warnings denied, format, skill
validation and 13 source-tool tests pass. The host navigates the owning API/shared helper and diagnoses an
unsaved missing net.listen action on the app's serve call, then repairs it. Extension 1.11.1 is packaged and
installed; the stateless workspace/app window is confirmed. No grammar/navigation runtime change is needed.
The saved 1.11.0 compiler and current 1.11.1 compiler each compile all 45 maintained files with zero source
rewrites and equal checked fingerprints. The language/compiler logic is unchanged.

Stateless and SQLite applications each pass 35 real TCP scenarios and 38 connections on Linux C, Linux LLVM
and Windows PE through Wine. All six runs use the minimum four native slots for the apps' root/restricted
capabilities, listener and active stream. Repeated good/bad requests exercise slot reuse. The preserved old
app adds one compatibility TCP run. Both Linux SQLite routes return 500 for four malformed-row cases.
Linux C and Wine each pass ten SQL operations, two read-only restarts and an independent SQLite file oracle.
Actual Windows hardware execution remains unknown.

## Corrections and tooling metadata finding

The lifecycle tests fail before the owning pure API exists. The audit test initially fails because its tool
is absent, then the implemented guard also rejects the actual old sources. The TCP harness initially rejects
the new handle-budget flag. Library authoring needs explicit RequestResult context for an otherwise unresolved
conditional constructor. No inference feature is added. A first handle-budget test omits the capability slot;
corrected probes use two slots for zero-request lifetime checks and four for complete apps. A boundary probe
omits its typed Record prefix; corrected input now asserts invocation status before reading the value.
The existing source audit catches one layout change, which the checked formatter applies without semantic
rewrite. Original failures and accepted outcomes remain in raw evidence.

A global version replacement accidentally changes transitive editor lock entries. A new installed-version
regression reproduces it. Restoring root-only version edits then exposes older bad metadata in the preserved
baseline: typed-rest-client is recorded as 1.9.01 while the installed tool is 1.8.11, and three dependency
constraints also differ. Five existing fields are repaired from the installed npm/package metadata. No tool
upgrade or dependency installation occurs. The regression now checks installed versions and dependency maps;
the full editor suite and package gate pass. Before snapshots preserve the original erroneous metadata.
This is a reproducibility correction to historical source metadata, not a retrospective change to old runtime
observations or their recorded results. A fresh offline installation was not attempted.

## Preservation and interpretation

All 93 selected compiler/provider, grammar/navigation, model, vendor and frozen source inputs are byte-identical.
All 34 prior paper entries retain their pinned report hashes. Before/current source, compiler, raw outcomes
and artifacts are preserved in the private bundle named by the receipt. Resource grants, SQL/HTTP features,
callback bounds and admission limits remain. No full TLC rerun or extended formal-model claim is made.
The source inventory and shared-policy counts are engineering observations; no agent construction,
comprehension, token or runtime-performance benefit is measured. This remains a local dirty-tree milestone.
See [decision 0060](../../decisions/0060-complete-maintained-http-refactor.md),
[the shared library](../../libraries/http-server/README.md) and
[the stateless app](../../examples/native/rest/app.lobo).
