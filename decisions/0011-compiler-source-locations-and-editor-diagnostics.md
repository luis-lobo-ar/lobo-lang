# 0011: Compiler source locations and editor diagnostics

- Status: implemented in compiler 0.2.1 / extension 0.3.0
- Date: 2026-10-13
- Risk mode: standard for the compiler location model; explicit trust/cancellation checks for editor processes.

## Outcome and alternatives

Provide precise, structured errors to agents and editor users from the Rust compiler's existing parser/checker.
Use optional, ephemeral node/token spans, converting to byte/UTF-16 coordinates only for source diagnostics.
Do not guess positions by message text: repeated names and identical expressions would misidentify blame.
Do not serialize locations into checked artifacts: that would break comment/whitespace-independent artifact
identity and allow untrusted artifact producers to forge source attribution. A full language server or parallel
editor type checker is unnecessary for the first-error diagnostic slice and would duplicate language semantics.

## Guarantees and scope

Follow [source diagnostics v0.1](../spec/source-diagnostics-v0.1.md). Preserve compiler semantics, stable codes,
artifact bytes/version, native behavior, and existing navigation/highlighting. Locations are additive to JSON.
Source-only stdin checking enables current unsaved-text diagnostics without creating temporary source files.
Use compiler 0.2.1 and extension 0.3.0; language/artifact versions remain unchanged.

The editor executes a configured compiler only in trusted workspaces. No shell, compilation backend, or program
execution is involved. Debounce and cancel checks; reject stale/closed results. Report missing executables,
timeouts, malformed output, and unsupported compiler output explicitly, without fake code squiggles. No runtime
npm dependency or bundled platform-specific compiler is added. Symbols/type queries and migration of definition
navigation to compiler ownership are separate subsequent slices.

## Validation and rollout

Fixture-first exact ranges for parse/name/type/declaration failures, UTF-8/UTF-16 and CRLF, EOF, artifact
determinism and metadata rejection; CLI stdin/file checks; editor trust, output, cancellation, and stale-version
tests; real VS Code host checks. Build/install locally only, preserving earlier VSIX artifacts for rollback.

## Evidence and remaining limits

On 2026-10-13, all 31 compiler integration tests passed in debug and release, including eight new source/CLI
tests. All 45 extension tests passed, including real compiler stdin, malformed/legacy/null responses,
cancellation, graceful/forced termination, trust gates, stale results, and a two-check concurrency limit.
The real VS Code 1.105.1 host verified definition navigation plus exact compiler squiggles for unsaved edits.
Rust formatting, Clippy with warnings denied, JSON/link checks, and diff whitespace checks passed.

TDD observed the missing ranges and unsupported stdin command before implementation, then isolated regressions
for a truncated UTF-8 read boundary, null JSON envelope, ignored SIGTERM, and misleading resource-limit messages.
The first real-editor diagnostics check failed until the tested controller was registered in activation.
These red attempts were not counted as successful verification. The primitive compiler-input budget is not an
application permission/rate-limiting system.

The release compiler is 694,432 bytes; the locally built VSIX is 14,135 bytes with eleven entries and no bundled
compiler, tests, or npm development dependencies. Extension 0.3.0 was installed and the repository opened as a
workspace with its configured release-compiler path. No publication, commit, benchmark-worktree mutation, or
application/library security runtime was performed.

Compiler checks report the first error only. Runtime-source mapping, compiler symbol/type queries, and replacing
the editor's structural navigation resolver remain unfinished. Actual Restricted Mode execution was covered
by unit gates and manifest checks, not an automated native UI trust-prompt interaction. Process termination is
best effort against the owned child, not hard real-time or descendant containment. Native Core remains unsandboxed.
The broader library/FFI/capability/audit readiness requirements are recorded in [the v1.0 target](../docs/v1.0-target.md).
