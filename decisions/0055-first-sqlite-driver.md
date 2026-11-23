# 0055: First native SQLite driver

Date: 2026-11-23. Status: selected for implementation by the user's "lets start with one".

Implement the SQLite-first recommendation in decision 0054. This reopens a bounded database milestone:
prepared/bound statements, typed owned rows, transactions, explicit grants, independent native limits,
fault cleanup, a persistent-user REST example, Linux and Windows native artifacts, editor/skill/docs and
recorded tests/models. Preserve older REST/demo/research fixtures. No remote driver, NoSQL API, ORM,
concurrency, SQL translation, general FFI or new callback/cleanup syntax is selected.

Use agentic-engineering high-assurance mode for the persistent-data and authority boundary, with test-first
incremental implementation. Compare direct SQLite behavior with the Lobo driver. The existing official C
amalgamation is justified because storage-engine correctness is outside a new language's scope. Pin and
verify source distribution hashes and provenance; no system/tool installation or runtime download.

Chosen alternative: embed the pinned SQLite source only in emitted programs that use the database
extension. This keeps C export self-contained and native programs independent of Rust or installed SQLite.
The compiler grows by the embedded source size, and database build time increases. Existing programs do
not compile/link SQLite. External system-library linking would reduce compiler size but require an
installed development SDK and a portable distribution/linking contract; those are absent here.

Core/compiler/extension 1.9 introduce a separately marked closed SQLite extension version 1. Native ABI 1
and its six preexisting result layouts remain unchanged for programs that do not select SQLite. New
resource/result layouts and operations require the extension marker and Core 1.9. Old checked artifacts
remain admitted by their matching contracts. No resource can be forged as an input or serialized output.

Database grants and handles are separate from the old clock/socket capability mask. Grant exactly one
launcher-selected database file and a read/write/schema role. Source cannot choose another path. SQLite
capability restriction is subset-only; delegated database handles retain their role. Deny extension loading,
ATTACH, unapproved PRAGMA/functions and hidden file access. Document trusted-directory and crash limits.
Connection/statement ownership is tracked before allocation or language faults can strand it. Cleanup
finalizes statements and rolls back/closes connections without remaining user budgets.

Single-statement operations bind values separately. Native execution/call/byte/row/memory budgets are
independent of expression fuel. Commit/rollback states and failure precedence are explicit. Do not claim
hard IO deadlines, filesystem crash proof, native-library sandboxing or remote exactly-once writes.

The pure callback boundary in the old HTTP serve helper cannot perform database actions. Keep it intact.
The persistent example uses a direct actionful controller and reuses parsing/wire/IO helpers. No impure
function-value feature is implied. Extend only what the real application and failure tests require.

Evidence, source/license identity, lifecycle model/mutation results and residual limits will be recorded
before completion. Decision 0054 and its dated paper snapshot remain historical proposals.

The real HTTP/JSON plus SQL source composition failed the previous 8,192-token aggregate bundle cap.
A failing composition regression justifies raising only that aggregate cap to 16,384 for Core 1.9 source
compilation and editor bundle navigation. Individual source lexing remains limited to 8,192 tokens. The
16-source, 1 MiB and depth/type limits remain unchanged. A four-source negative corpus still exceeds the
new aggregate cap. No SQL grammar or function-value feature is added to solve this capacity limit.

After token admission, the same composition reached the previous 32 source nominal-declaration cap.
The database catalog injects five additional nominal result layouts, and SQL/HTTP libraries need their
own typed rows, errors and state. Core 1.9 raises that source cap to 64, including synthetic native layouts.
The 128 specialized nominal-instance cap and eight-level type-depth bound remain unchanged. Retain the
failed composition log and update the negative corpus to 65 declarations. This is workload-driven capacity,
not a new construct or an unbounded frontend. Editor declaration/bundle admission must match.


Implementation accepted on 2026-11-23 after the native/compiler/editor/application gates in
[the final result](../benchmarks/results/2026-11-23-sqlite-driver.md). The initial selection above records
its authorization. Finalize and close bypass SQL operation quotas so teardown cannot hide the original
failure. VACUUM can prepare but its deferred ATTACH is denied at execution; the witness checks denial and
unchanged data. The lifecycle model now matches uncharged finalization. The bounded baseline explores
3,973 distinct states, with four intentional invariant violations and eight matching isolated C mutant
rejections. This is not an engine, OS or unbounded refinement proof.
