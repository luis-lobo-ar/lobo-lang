# Managed persistent identities

Date 2026-11-22, America/Chicago. Engineering acceptance passed for compiler/language 1.3 and extension 1.3.0.
The user selected removal of visible identity metadata from ordinary source while retaining persistent IDs.
The [decision](../../decisions/0044-managed-persistent-identities.md) and [contract](../../spec/core-v1.3.md)
define this bounded change. This report describes public development fixtures, not a controlled agent study.

## Findings

The maintained MVC now contains only source declarations and expressions. Its twelve bindings remain in a
tracked adjacent `.lobo.ids.json` file, with the exact prior values. The small skill example also retains its
two bindings in a sidecar. Other inline tables remain supported. Frozen legacy sources need no migration.

Compiler checks and native builds read metadata without changing it. Explicit `identities init` and `sync`
allocate missing IDs once using OS randomness. Metadata-only `rename` preserves an ID after the source and
references are renamed. Stale state fails instead of guessing a rename. Moves preserve identity when source
and sidecar move together. Explicit source bundle 0.2 carries metadata to tools without filesystem access.

Managed and frozen legacy MVC inputs produce exactly equal checked artifacts under the same compiler.
Evaluator, C, direct LLVM and both sanitizer routes retain found/missing behavior. Release C/LLVM binaries
return status 200 with `{"name":"Luis"}` for user 1, and status 404 with `Not found` plus newline for user 42.
Both source and sidecar hashes remain unchanged after those release builds and invocations.

Extension 1.3.0 reads bounded adjacent metadata in trusted explicit workspace snapshots and overlays unsaved
metadata. Original-position definition navigation and compiler diagnostics pass. Metadata changes invalidate
old snapshots. Real VS Code host checks include removing/restoring an unsaved binding and observing navigation.
Highlighting requires no metadata. Managed navigation requires a trusted metadata snapshot.

The agent test session accepts explicit metadata. Changing a helper's identity executes its dependent test
again while reusing the unrelated constant result. Metadata participates through the checked-program dependency
graph, rather than being omitted from cache validity.

Final acceptance passes 212 Rust tests, 145 editor tests, 31 Python benchmark checks, four codemod tests and
six native HTTP checks. Real VS Code host, Clippy with warnings denied, formatting, skill validation, native
skill examples and documentation links pass. The updated local VSIX is built. It was not installed into a
normal user profile or published. The [receipt](2026-11-22-managed-identities.json) records commands, hashes,
raw logs, input preservation and exact limits. Repeated suites are verification attempts, not independent samples.

## Negative and interrupted attempts

The first managed-identity regression run fails before implementation. The first partial implementation passes
pure snapshots and MVC behavior but fails the authoring case because the CLI command is not wired yet.
After wiring, all three original cases pass. Import allocation and explicit failure-state coverage are added
and pass. The editor regression initially fails for missing metadata support, then passes after implementation.

The first full Rust run stops at an outdated 1.2 language-tag assertion. Assertions advance to the selected
1.3 contract without changing expected runtime behavior. The next run cannot complete because the host disk
is full. Rust reports output IO failures and linker crashes. The attempted editor-host launch has an empty
log and fails to start during that interval. These are infrastructure failures, not test acceptance.
`cargo clean --package lobo-lang` removes 11.9 GiB of rebuildable artifacts. Research archives stay intact.

The next Rust run catches an oversized-file classification regression in the new loader. It returns an
identity invocation error where the established contract requires `resource.frontend.limit`. The loader is
corrected. A subsequent complete run passes, followed by a fresh final suite with the added session regression.
Clippy initially rejects a contains-key/insert sequence; the allocation loop now uses the map entry API.
The first formatting check fails on the added test and passes after formatting. Premature documentation checks
find the not-yet-written report and then its not-yet-written receipt. The completed link check runs after both exist.

The first historical integrity script mistakenly classifies `spec/README.md`, the current specification index,
as a frozen historical file. The corrected audit separates that intentionally updated index and the previously
recorded repair-driver change. It verifies 292 other surveyed files, the previous sixteen paper entries and
the frozen release. No historical oracle is recaptured. All logs available from these attempts are retained.

## Limits and interpretation

IDs are persistent state. Losing the sidecar requires restoring it from version control. Initialization cannot
recover earlier IDs from a name or body. Source renames and moves remain explicit authoring operations.
There is no F2 source refactoring, automatic file-move hook or automated deletion command. Confirmed obsolete
bindings can be removed in a reviewed metadata edit. Inline sources must extract their valid existing table
before adopting sidecar authoring for new declarations.

Publication is atomic for one sidecar with cooperating writer locks. A multi-file operation can publish only
some sidecars before an IO failure. It is not a semantic transaction, hostile-filesystem sandbox or guaranteed
power-loss recovery mechanism. Interrupted temp/lock files require review. New-ID allocation currently uses
`/dev/urandom`; initial publication requires filesystem hard links. Existing metadata compilation needs no RNG.

No construction-quality, comprehension, token-efficiency or runtime-speed comparison ran. Those effects remain
unknown. Reduced visible boilerplate and passing engineering checks do not establish an agent productivity
gain. The preserved evidence is local and ignored, not an off-device backup, publication or Git commit.
