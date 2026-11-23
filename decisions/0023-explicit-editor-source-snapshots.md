# 0023: Use explicit source snapshots for cross-file editor operations

- Status: implemented in extension 0.11.0; local verification passed
- Date: 2026-10-18
- Risk mode: standard; dependency reads, source provenance, cancellation, and nominal navigation.

Complete Core 0.9 editor support without adding an ecosystem resolver. `lobo.sourceFiles` lists a complete
closed source set as workspace-relative `.lobo` files (maximum 16). It applies only to listed documents.
Reject absolute paths, traversal, duplicates, wildcards, unsupported schemes, and out-of-workspace canonical
targets. Imports remain stable declaration IDs, never paths. Dependency reads and compiler execution require
workspace trust; existing compiler-free single-document navigation remains available without trust.

Alternatives (local analysis, not independent prototypes): workspace scanning guesses authority and costs;
requiring a compiler for all navigation loses existing compiler-free behavior; naive spelling matching loses
module-local namespaces and nominal identity. Choose a bounded structural source catalog with stable-ID
ownership, selected aliases, and canonical nominal shapes, paired with the existing authoritative compiler.
The catalog is not a type checker or execution engine. Invalid/ambiguous shapes fail closed.

Read only listed files, bounded before UTF-8 decoding; overlay open unsaved documents. Enforce compiler
aggregate byte/token/source limits and bounded metadata before catalog use. Label every target/error with
its exact source. Capture dependency versions and reject stale results, including changes under an unchanged
root version. Navigation may return function/type/field/case targets in supplied dependencies, never guess
unrelated files. Bundle checks use `check --bundle-stdin`, no shell, Clang, execution, temporary source writes,
downloads, or import resolution. Preserve existing process timeout/output/concurrency/cancellation gates.

Report project/configuration/read/unlocated errors as infrastructure, not guessed squiggles. Attribute a
dependency diagnostic only to its dependency; never transplant its range onto the root. Reschedule and
invalidate when explicit source inputs change, including unsaved edits and external file changes. Configured
paths are user selections, not a promise of OS sandboxing against concurrent hostile filesystem mutation.

TDD gates: cross-file alias/type/field/case targets, inferred hidden nominal values, same-spelling distinct
modules, cycles, malformed/missing closure, budgets, bounded IO and overlays, dependency diagnostic attribution,
staleness/cancellation/trust, actual VS Code definition/squiggle checks, and rebuilt VSIX. Existing local
navigation/source-check behavior and computational workload oracles must remain. No new runtime dependency,
installation, publication, commit, package resolver, daemon, or external authority is introduced.

## Failure matrix and keeper

| Input/event | Required result |
| --- | --- |
| No project / current file not listed / untrusted workspace | Retain local navigation; no dependency IO. Untrusted workspaces never launch compiler checks. |
| Invalid path, missing/special/oversized/non-UTF-8 file | No guessed definition or squiggle; bounded explicit read fails and diagnostics report infrastructure. |
| Unknown/duplicate ID or missing imported shape | Structural bundle navigation fails closed; compiler remains authoritative. |
| Dependency diagnostic | Validate its byte/UTF-16 range against that exact source; publish only at its URI. |
| Dependency edit, close/open, configuration or external explicit-file event | Invalidate old snapshots, cancel/requeue checks, reject late results even if root version is unchanged. |
| Unrelated output/plaintext edit | Do not invalidate source snapshots or create a diagnostics/output feedback loop. |
| Cancellation, compiler timeout/output excess | Existing bounded no-shell process path terminates; no late diagnostic publication. |

Keeper: `project.cjs` owns the explicit file snapshots/watchers and trust/read boundaries. Navigation uses
two structural passes: stable ownership/signatures, then references with canonical nominal keys and source
tokens. Compiler checks share the existing process runner with a separate bundle envelope/flag. Diagnostics
keep per-root contributions so a root success cannot erase another root's dependency error. No compiler
lookup service, workspace index, language server, new dependency, or runtime is needed for this contract.

## TDD findings

Initial tests failed because the bundle navigation/check APIs were absent, then because the live provider
returned local aliases and diagnostics checked roots alone. The workload consolidation test observed two
Cursor records before moving the existing collection implementation to `examples/sequences.lobo`; it now
observes one. Tree/module-example reverse entry IDs remain as ordinary delegating wrappers. The original
collection reverse/cursor IDs survive the owner move; unused internal tree/module cursors were removed.
Wrappers have ordinary call/contract costs; no budget was increased and all existing resource oracles pass.

A FIFO input test reproduced a blocked `open` before regular-file validation. The failed run hit its four-second
outer test timeout; nonblocking open now permits regular-file rejection without waiting for a writer. Size
admission precedes strict decoding. Another regression rejected use of a newer String method unavailable in
older supported extension hosts; Unicode validation now uses explicit surrogate checks. Forged old-version
bundle success originally passed and is now rejected, without removing single-source older-compiler support.
An unrelated-edit regression exposed overbroad invalidation; only Lobo documents or explicit watched inputs
now trigger refresh, avoiding potential output-channel feedback. These failed runs are not passing evidence.

The new host test initially expected an enclosing constructor range; the source diagnostic contract says
incompatible expressions are blamed. It was corrected to the leaf `true`, consistent with the existing
coordinate tests/specification, not a change to compiler semantics. Grammar fixture checks now acknowledge
that the collection contract moved with its helper and that the helper/wrapper have no integer literal;
the shared source is independently tokenized. Frozen computational/numerical expected values are unchanged.

## Observed verification — Linux x86_64

Cargo 1.92.0, Clang 18.1.3, Node 22.19.0; local working tree, not a published release:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 129 tests pass per profile
  (128 integration and one native-launch unit). The 17 module tests include actual collection/tree sharing;
  all frozen parser/collection/tree variations, extremes, parent edges and exhaustion oracles remain.
- Reloaded checked artifacts, evaluator, optimized native and ASan/UBSan/leak checks agree. Active sanitizer
  probes remain in the generated corpus. `cargo build --locked --release --bin lobo`, formatting and
  all-target Clippy with warnings denied pass. No compiler/runtime dependency changed.
- `npm --prefix editors/vscode run package`: 103 tests pass, zero failures/skips on this Linux host;
  VSIX 0.11.0 contains 12 files, including the source-project adapter, not tests/development dependencies.
- Real installed VS Code host tests using a fresh profile and temporary explicit project report
  `LOBO_HOST_TESTS_PASSED` and exit 0. They exercise remote functions/fields/cases with same-spelling types,
  unsaved dependency rename/type errors, unchanged root versions, and external closed-file invalidation,
  alongside earlier local scope/action/value/loop checks. A final run logged incidental bootstrap
  `Unexpected SIGPIPE`, then completed every assertion and reported the marker/exit 0; earlier runs were
  clean. This log is retained rather than described as an error-free host log.
- JSON/JSONL/local-link and whitespace gates pass. No normal editor installation, commit, publication,
  deployment, benchmark rerun, or comparison-worktree mutation occurred. Earlier native examples remain.

Direct native smokes: even selection `[5,2,8]` -> `[2,8]`, count 2, sum 10; tree `[3,1,3,2]` -> `[1,2,3]`,
count 3, height 3. Explicit module-wrapper checking also succeeds with the shared source supplied.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/lobo` | 1,078,512 | `8476c9530b58ec963fb02400ecaa4d8d835c5d880a4204c7526a9002a83d41b3` |
| `target/native-examples/collections-core09` | 18,784 | `15be939bdd244bbabd5256525d340c36c5df2a2f0c0074333bab07d91220b18b` |
| `target/native-examples/tree-core09` | 31,080 | `db656c1e14788cff5659e8dbab5f91d3b626a0a35555f9e8efa1be53ad33bbfc` |
| `dist/lobo-syntax-0.11.0.vsix` | 26,302 | `bac7fa8ba5fac969bd150d6307cf1e2f21969a1f17b1be5e293e8f50e57fdeef` |

Input SHA-256: `examples/sequences.lobo`
`dea6546640035a1a13831851309b7f0ab10c7a36151b76578eb16f7bf0c15384`;
`examples/collections.lobo` `ec09d1a67ff2ec5ff49625186f66009022b61b521e7454af0de3213afc3f5425`;
`examples/tree.lobo` `5b18b333ed29c500d68fd318ca30b422ab12d82c27afddc004343a2e692f4601`;
`tests/modules.rs` `ae27bd26cf85fb574ee685161d007439081b379c87cbec031d09d6885df3de15`;
`navigation.cjs` `d9b407eaa946f4f35c48590f51c1ac8a1f5df22d710ec0349c8631fbd06ebef4`;
`project.cjs` `fbbecb2fa3e375a314c9cc32ce81e15b231ea96bbaa21ca6a6c33952f496e683`;
`compiler.cjs` `c9dfb35c51d3b8d32767abc413a06dbaf99ccd0c5d7749f95155f76bdf9701cc`;
`diagnostics.cjs` `c902637b51dc2d493e56b5095c0baf65a8c600a039a4f69ea564582ad6aabc62`;
`extension.cjs` `fc7c2b1a520122f8206e4856a8d8d74a87531fffeb43470b8ab4879cc296c75e`.
The `.cjs` paths above are under `editors/vscode/src/`.

## Scope conclusion and residual limits

This closes the editor/workload follow-through from decision 0022. The full language goal is separately
audited [requirement by requirement](../docs/language-completeness.md#language-only-requirement-audit--2026-10-18).
It is not agent-workspace/application v1.0 readiness, a Turing-completeness proof, universal replacement,
or permission to start ecosystem integration. The supported platform is locally verified Linux x86_64;
other desktop/remote/older-host variants have structural/API tests, not an advertised full platform matrix.
Filesystem containment is not a race-proof OS sandbox; watchers/host scheduling are not hard-real-time
revision isolation. Native libraries and broader runtime capabilities remain deferred and unimplemented.
