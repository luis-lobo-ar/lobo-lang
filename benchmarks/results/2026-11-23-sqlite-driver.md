# First native SQLite driver

On 2026-11-23, compiler/extension 1.9.0 and Core 1.9 implement the user-selected SQLite milestone in
[decision 0055](../../decisions/0055-first-sqlite-driver.md). The
[machine-readable receipt](2026-11-23-sqlite-driver.json) records hashes, commands, observations and limits.

## Delivered behavior

SQLite extension 1 adds 22 closed native calls, three opaque resource kinds and five optional result layouts.
Non-SQLite programs retain the previous ABI 1 layouts and omit the engine. Old user-defined identities that
resemble SQLite results keep their previous meaning. Checked artifacts require the extension marker and
matching language version for database operations. Resources cannot be forged as external values.

The pinned SQLite 3.54.1 amalgamation is compiled into C and LLVM runtime output. It needs no SQLite DLL,
Rust server or interpreter at execution. The compiler is still implemented in Rust; embedding the source
increases compiler size and database build time. No installed development SDK was available, so a verified,
self-contained source dependency was selected over system linking. Source and test archive provenance is
retained in [vendor/sqlite](../../vendor/sqlite/README.md) and
[upstream tests](../../tests/upstream/sqlite/README.md). Five Tcl files are retained, and selected binding,
NULL, authorization and transaction behaviors are adapted. The full upstream suite was not executed.

The [SQL library](../../libraries/sql/README.md) provides bound parameters, typed owned rows, structured
errors and explicit transactions. Launch policy selects one file and read/write/schema authority. Restriction
cannot increase rights. Prepared statements require every parameter to be bound; empty text/blob remain
distinct from NULL. Column copies outlive their statements. Explicit finalize/close and fault cleanup stay
available after SQL operation quotas run out. Active transactions roll back on close/fault cleanup.

The [persistent REST example](../../examples/native/sqlite-rest/README.md) bootstraps users, then gives
controllers a read-only database. Lobo owns parsing, routing, database calls and IO loops. Its direct
actionful controller leaves the older pure callback contract intact. The standalone Windows launcher
converts wide arguments to UTF-8. Default launch selects a local database and loopback port 8080, with
128 accepted connections. Explicit arguments inherit no demo grants.

The real eight-source HTTP/JSON/SQL composition exceeded the previous aggregate-token and source-nominal
caps. Measured failures justify 16,384 bundle tokens and 64 source nominal declarations, including synthetic
layouts. Per-file 8,192 tokens, 16 sources, 1 MiB, depth and specialization bounds remain. Negative corpora
still reject bundles and declarations beyond the new caps. No extra syntax or callback feature was added.

## Verification

- 286 Rust tests pass with no failures or ignored tests. SQL programs execute through C, LLVM, C ASan/UBSan
  and LLVM ASan/UBSan; SQL operations deliberately reject reference interpretation.
- The real C adapter passes 21 Linux cases in optimized and sanitized builds, including root authority,
  stale/generation handles, binding, invalid values, policy, busy commit, memory failure, corrupt files,
  side-file symlinks, quota exhaustion and fault cleanup observed before process teardown.
- 168 editor tests, the real isolated VS Code host, navigation/unsaved SQLite diagnostics and VSIX packaging
  pass. The language skill includes an executable SQL example checked through all four native modes.
- The SQLite lifecycle baseline exhausts its queue at 3,973 distinct / 16,450 generated states. Four broken
  models violate their named invariants, and eight isolated C mutants fail their expected logical checks
  under optimized/sanitized builds. The stale-map negative gate also runs under Python `-O`.
- The older native gate passes its two baseline models, eleven broken models, 22 concrete C replays and
  22 isolated C mutant builds. Its 29 covered source fingerprints remain unchanged. SQL model claims are
  separate. [The mapping](../../verification/sqlite/README.md) states their bounds and exclusions.
- Linux and Wine executables each pass ten database operations, two read-only REST restarts and an independent
  Python SQLite file oracle. Unicode paths, Unicode/NUL/injection-like bound values, idempotent setup,
  permission denial and rollback are checked. The Windows C provider passes 20 cases under Wine; Linux-only
  symlink handling is excluded from that count.
- Linux and Wine each pass all 35 HTTP scenarios (38 accepted connections). Windows default launch passes
  those scenarios over 128 accepted connections. The older REST binary still passes on both platforms.
- All 31 benchmark-tool Python tests and 12 source-tool Node tests pass. Clippy, formatting, skill validation,
  pinned upstream checks, source modernization (43 files, zero rewrites), JSON and local links pass.
- All 29 prior paper entries and their pinned files, all 86 frozen v1 release artifacts, 22 protected current
  inputs and the original 23-bundle/three-embedded migration checkpoint are preserved.

The optimized Linux executable is **1,432,560 bytes**; Windows x64 is **1,397,248 bytes**. DLL inspection
finds only Windows system runtime/Winsock imports. These are artifact facts, not speedup measurements.

## Failures and corrections retained

Initial unknown-operation, granted-open and library-stub tests failed before implementation. Incidental
fixture mistakes included sidecar format, reserved names, match field grouping and generic token spacing;
these are not counted as meaningful TDD evidence. A floating-point oracle omitted the required `0x` wire
prefix and was corrected against the existing specification.

Tests exposed wrong permission-error classification for denied SQL functions, schema-index authorization,
quota charging that could prevent teardown, and old result-looking user IDs being reinterpreted. Each
behavior has a failing-before and passing-after record. A first compatibility test used a private test API;
the corrected test then reproduced the actual scanner bug. A header-only model harness exposed a missing
explicit math include. The first lifecycle replay assumed VACUUM must fail at prepare; its ATTACH is actually
authorized at step. The corrected witness checks denied execution and unchanged data, without introducing
a fragile SQL-text parser. The model was updated to reflect uncharged finalization.

The first editor host attempted compiler diagnostics before configuring its compiler path. The corrected
host passes. VSIX packaging needed the new repository URL for README links. Other recorded command errors
include unsupported multi-filter Cargo syntax, a nonexistent old-binary path, an unsupported old-checker
`--verify-map` flag and an unsupported compiler `--version` query. Correct commands and direct pinned-hash
checks pass. Unittest discovery under scripts found zero tests; it is not counted as test coverage. Expected
nonzero statechart exits identify broken models, not tool failures. Earlier attempts remain in raw evidence.

## Evidence and limits

The ignored local archive `benchmarks/private-evidence/sqlite-driver-20261123` preserves before/current
sources, input hashes, compilers, source distributions, raw failing/passing logs, mutation receipts,
statecharts and executables with an integrity manifest. Final report/index files are excluded from that
manifest to avoid hash cycles; the paper register pins this report and receipt separately. This local evidence
is distinct from the source code uploaded to the new private GitHub repository.

The finite lifecycle model abstracts eight charged operations, one connection, one statement, two writes and
three rights. It is a reviewed source mapping with representative C witnesses, not automatic refinement,
engine verification, a general registry cutoff proof or a liveness proof. The native engine remains trusted
code. Parent directories must resist hostile replacement; path checks do not constitute a secure VFS.
The adapter does not promise filesystem/power-loss durability, hard OS IO deadlines, process sandboxing,
TLS, authentication, concurrency, remote SQL, NoSQL or an ORM. Wine coverage is not actual Windows hardware
coverage. No new agent trial, token-efficiency, code-comprehension or performance experiment was run, and no
claim about those outcomes follows from this implementation.


The initial GitHub snapshot includes earlier uncommitted milestones. Staged whitespace checking reports
preserved blank EOF lines in historical examples/fixtures and the result register, plus exact upstream
license/test/amalgamation whitespace. These bytes were reviewed and retained to preserve provenance and
frozen inputs. No SQLite implementation or new executable-test source has a whitespace finding.
