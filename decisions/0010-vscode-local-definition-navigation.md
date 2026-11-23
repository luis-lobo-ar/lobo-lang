# 0010: Scoped local definition navigation in VS Code

- Status: implemented in extension 0.2.0
- Date: 2026-10-13

## Decision

Extend the editor package with a small read-only DefinitionProvider for Ctrl-click, F12, and Peek Definition.
Navigate Core function calls, parameters, immutable locals, and fold bindings to their declaration tokens.
Postcondition `result` links to its function declaration. Function lookup is module-wide, but local lookup
obeys binding order, branch/fold scope, and contract visibility. Ignore comments, IDs, keywords, and unresolved
references. Read current document text on every request; do not use stale saved-file state.

Core has one module per file and no imports. Restrict resolution to the current file rather than search the
workspace for matching names. Use a bounded structural lexer/parser with UTF-16 ranges; type checking and
compiler diagnostics remain outside scope. Malformed/incomplete input yields no definition, not a guessed target.

## Alternatives and boundaries

A language server or compiler-backed semantic/span protocol would be the longer-term source of truth, but
neither exists yet and the compiler currently exposes no source ranges. Regex-only matching would misresolve
same-name locals, initializers, and contracts. The lightweight resolver follows the specified Core grammar,
with scope regressions and real extension-host tests to expose drift; revisit when compiler spans or imports land.

This supersedes decision 0009's no-extension-host-code boundary, not its highlighting or safety choices.
Ship two small CommonJS modules compatible with the desktop/remote VS Code extension host, without a bundler,
runtime dependencies, compiler invocation, network access, telemetry, background watchers, or file writes.
Keep syntax highlighting declarative. Standalone browser-host navigation is not implemented.

Bound document size, token count, and nesting; check cancellation at the provider boundary. No caching is
needed for the first slice. Verify with Node scope tests, the existing TextMate suite, real VS Code definition
commands in an isolated profile, VSIX inspection, and repository checks. Do not publish to the Marketplace.

## Evidence

On 2026-10-13, all 31 Node tests passed (12 grammar/configuration, 16 navigation, three provider checks).
The real VS Code 1.105.1 extension-host run completed activation, file association, function/parameter/fold
lookup, UTF-16/CRLF, ignored comments, and unsaved-edit checks through `vscode.executeDefinitionProvider`.
The full 23-test Cargo suite and JSON/link/whitespace checks passed. No dependencies were added or compiler
semantics changed. The VSIX is 9,262 bytes with nine archive entries, including just two runtime modules;
development dependencies/tests are excluded. It was installed as `lobo-lang.lobo-syntax@0.2.0` in the user's
normal profile after the requested navigation update; the prior VSIX remains available for rollback.

Fixture-first review caught unsupported comparison/equality chaining in the initial structural parser;
the resolver now mirrors Core's operator levels and retains a regression. The first host launcher used the
`code` wrapper, which exited before the editor tests completed; it now requires the actual VS Code executable
and an explicit completion marker in addition to exit code zero. The first real host assertion raced language
activation; tests now await extension activation before querying providers. These attempts were not counted
as passing host evidence. Physical Ctrl-click gestures were not automated; the same definition-provider API
used by Go to Definition and Peek Definition was exercised in the editor host.
