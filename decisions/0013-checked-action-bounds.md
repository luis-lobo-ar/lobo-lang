# 0013: Implement checked action bounds before host resource access

- Status: implemented in compiler 0.3.0 / extension 0.4.0 (static layer only)
- Date: 2026-10-17
- Risk mode: standard for static language checking; no new resource authority is introduced.

## Outcome and alternatives

Implement real compiler support for `actions` rather than merely renaming documentation. Use explicit
conservative sets on functions, checked at every call edge, including recursive cycles. Inferred whole-program
sets are an alternative but cannot check a declared boundary without an explicit bound, and introduce more
inference machinery. Declaration-only parsing without call checks would provide misleading protection.

This first slice intentionally does not implement HTTP/storage APIs or native FFI; their type, ownership,
authorization, isolation, and failure contracts remain prerequisites. An action label is not a capability.
No ambient resource access is added. Success is checked source, revalidated artifacts, native/reference
agreement, agent-visible declared sets, and matching editor support—not runtime security enforcement.

## Decisions

Follow [Core v0.3](../spec/core-v0.3.md) and [checked artifacts v0.3](../spec/checked-artifact-v0.3.md).
Omitted/empty declarations are pure bounds; contracts always require pure callees. Check all branches and
fold parts. Nonempty declarations may overapproximate scalar bodies; labels have no executable meaning yet.
Reject duplicate/noncanonical/oversized artifacts rather than trusting or silently repairing them.

Use compiler 0.3.0 and extension 0.4.0; Core/artifact versions advance to 0.3 because of new mandatory metadata
and checks. LTP 0.2 and native adapter 0.1 remain independent. Supersedes decision 0012's unsupported-clause
boundary only; its terminology, protocol migration, historical evidence, and ledger policy remain valid.

## Verification

On 2026-10-17, the local, uncommitted working tree passed:

Environment: Linux x86_64, Rust/Cargo 1.92.0, Clang 18.1.3, Node.js 22.19.0; the real host used the installed
VS Code executable through Xvfb. No compiler/editor dependency was added.

- `cargo test --locked` and `cargo test --locked --release`: all 42 integration tests, including eight action
  tests, inherited conformance, native/reference comparisons, and existing ASan/UBSan suites.
- `npm --prefix editors/vscode test`: all 51 tests, including the new executable example, contextual action scopes, navigation through
  comment-separated declarations, and source-check JSON for action diagnostics.
- The real VS Code host test with a fresh temporary profile: definition navigation, exact action-error
  squiggles on unsaved source, then diagnostic removal after adding the caller's declaration. The harness
  observed its completion marker and exit 0. VS Code also logged an incidental bootstrap SIGPIPE warning;
  it did not prevent completion. Normal user settings/installation were not changed.
- Rust formatting, Clippy with warnings denied, JSON/JSONL/local-link checks, and diff whitespace checks.
- Release build, example source check with stable-ID action summaries, and native pipeline execution: 41 -> 42.
- `npm run package`: extension 0.4.0 VSIX, eleven entries, no tests, dev dependencies, or compiler bundled.

TDD first observed all seven initial action cases fail on missing syntax, then pass with parser/checker
implementation. Editor tests initially failed for unsupported action parsing/coloring and the new language
version, then passed after their corresponding implementation. The eighth compiler test checks the executable
example, preserved contract faults, and identical emitted C with and without conservative annotations.
The former unsupported-action regression was explicitly revised against Core v0.3, not silently weakened.

Artifacts from this run:

- `target/release/lobo`: 717,544 bytes; SHA-256
  `1f92c65e33974fbd32c721f007edff92547650e4af31e947820d1c9636955d34`.
- `dist/lobo-syntax-0.4.0.vsix`: 14,795 bytes; SHA-256
  `3cfa5d2b7056cbbe4c6fa6b2d8660ec2e7b24344fe7b0be8c7dbd9400d46ce31`.

No commit, publication, normal-profile installation, benchmark rerun, or benchmark-worktree mutation was
performed. No new host operations, capabilities, quotas, native FFI, or OS sandbox are implemented.
Action IDs are open-vocabulary upper-bound labels, not verified registrations of executable library APIs.
TextMate highlighting remains lexical and has the documented multiline ambiguity; compiler diagnostics
and structural navigation do not derive meaning from highlighting. These tests do not prove arbitrary
program correctness or completion of the wider resource-security model.
