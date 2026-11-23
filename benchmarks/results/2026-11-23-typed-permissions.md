# Typed permissions verification

Date: 2026-11-23. Compiler/extension 1.11.2, Core 1.11, native ABI 1, SQLite extension 1.
This is a bounded library and tooling change selected by the user. It implements
[decision 0062](../../decisions/0062-typed-permissions.md), following proposal 0061.
The [machine-readable receipt](2026-11-23-typed-permissions.json) records the final gate counts.

## Result

OS and SQL permission domains have separate Permission variants and Permissions records. none(), one(case)
and allow(set, case) construct finite typed sets. Duplicates are idempotent. Empty requests delegate zero rights.
The HTTP library supplies server_permissions() for clock/listen. The snapshot names clock/random rights.
The persistent HTTP example delegates one(Read) after bootstrap. Existing native grants, subset enforcement,
handle quotas and errors are unchanged. No grammar, compiler implementation, provider or model changed.

Old raw OS/SQL helper IDs remain on restrict_mask with their integer signatures. Typed replacements have
new IDs. The checked maintained-source audit covers 46 sources with zero remaining rewrites. Both the saved
1.11.1 compiler and 1.11.2 produce equal checked-program fingerprints on all 46 current sources.
An unchanged pre-migration snapshot source and its sidecar also compile and run against the current libraries
through C and LLVM, exercising the preserved raw helper ID.
All 371 distinct prior maintained declaration/import identity values remain present. This count includes
import bindings, rather than claiming 371 newly independent declarations. Frozen inputs and all 35 prior
paper entries retain their hashes. The preservation check covers 110 compiler/provider/model/upstream,
frozen-fixture and grammar files.

The complete Rust suite passes 312 tests across 59 result groups, with zero failures or ignored tests.
The expanded typed-wrapper targets also pass separately. Formatting, clippy, source-tool, skill and documentation
checks pass.

## Behavior checked

The OS tests exercise all 16 requested sets against all 16 launch-grant sets under C, LLVM, C+ASan/UBSan and
LLVM+ASan/UBSan. A request succeeds exactly when it is a subset of the granted set. All set encodings and
duplicate additions also run through the reference route and native routes. Additional checks cover empty
capability use, child amplification, missing grants, native handle exhaustion, raw negative/unknown bits,
and typed IoError conversion. Integer/case arguments and SQL permission records are rejected by the OS API.
Opaque capabilities are never emitted as successful entry values.

SQL tests cover all eight requested sets against absent/read/write/schema launch profiles, plus duplicate
additions, empty capability use, child amplification and old-ID raw-mask rejection. They execute through
the same four native builds. These delegation tests create no database file. SQL masks remain independent
bits; Schema alone does not imply Read/Write. Existing SQLite engine/provider tests remain in the full suite.

Six real HTTP runs cover pure and SQLite-backed applications through Linux C, Linux LLVM and Windows PE
under Wine. Each passes 35 scenarios across 38 connections with four native handle slots. Malformed-row
HTTP tests pass on both Linux backends. Persistent SQL workflows pass on Linux C and Wine, including their
independent Python SQLite oracle. The snapshot Windows build includes its complete source closure and
input hashes. Wine coverage is distinct from actual Windows hardware, which was not rerun for this patch.

The editor passes 182 tests. The real VS Code extension host verifies cross-file permission/profile navigation,
rejects an unsaved integer mask with a type diagnostic, and clears the diagnostic after repair. Existing
highlighting handles the variants/records/functions without grammar changes. The 1.11.2 VSIX is packaged
and installed. The pure/SQLite workspaces contain seven/nine sources. The root editor configuration replaces
the unrelated hosted-handler source with the new dependency and retains the unchanged 16-source limit.
Its complete configured closure also passes the native compiler check.
The programming skill validates; its native clock example compiles and runs with an explicit clock grant.

## TDD and retained corrections

The initial new OS API test run fails because the typed permission library does not exist. The new raw-call
source-audit regression also fails before its guard is implemented. SQL coverage was added after the shared
API implementation; a replay of the captured pre-change SQL library rejects the missing typed helpers,
while both saved/current compilers accept the implemented library. That replay is retrospective evidence,
not a claim that every SQL assertion preceded implementation.

Retained development failures include authoring-sync alias resolution, a test parameter shadowing a function,
a harness attempting to serialize an opaque capability, and a context-free case name producing an unresolved
name instead of the intended type mismatch. Corrections preserve the error/type oracles: successful entry
probes return scalars and the negative case is explicitly qualified. The source audit initially finds seven
checked-equivalent simplifications in two new/extended files; the established checked formatter applies them.
An intermediate nearby run catches those unformatted files. Clippy catches one useless String conversion in
the new Rust test; it is removed. All original logs remain alongside the final runs.

## Reproduction and limits

Run cargo test --locked, npm --prefix editors/vscode test, the real editor-host command in the editor guide,
node --test scripts/test-modernize-source.mjs scripts/test-http-refactor.mjs, cargo fmt --all --check,
cargo clippy --locked --all-targets -- -D warnings and ./scripts/check.sh. The new integration targets are
tests/typed_permissions.rs and tests/typed_sql_permissions.rs. Existing build/test scripts reproduce the HTTP,
SQL and snapshot executable checks with fresh output paths and explicitly installed toolchains.

Raw command receipts, test logs, source audits and executable artifacts are in target/typed-permissions.
The ignored preservation bundle is benchmarks/private-evidence/typed-permissions-20261123, with before/current
source inventories, the saved baseline compiler, raw results and hashed artifacts. It requires normal backup.
No commit, push or publication is claimed. The receipt pins the retained evidence.

Full TLC is not rerun because no covered provider/runtime/model source changed. Existing formal/source-map
and C witness gates run as part of the Rust suite; their finite-model limits remain. Typed names do not add
a native sandbox. The lexical source audit prevents the known raw-call pattern in maintained sources; it is
not an access-control mechanism. No agent construction, comprehension, token-efficiency or speed effect was
measured. Additional pure record construction may change Lobo fuel/heap use; no runtime optimization is claimed.
