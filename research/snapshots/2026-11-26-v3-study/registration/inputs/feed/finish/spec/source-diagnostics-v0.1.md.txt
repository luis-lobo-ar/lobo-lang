# Source diagnostics v0.1

This tooling extension adds optional source locations to existing compiler diagnostics without changing
Core v0.2 semantics, diagnostic codes/categories, or the then-current `lobo-checked/0.2` format.
Core v0.3 inherits this coordinate/provenance contract and adds source-located action-bound diagnostics;
Core v0.7 additionally maps record/type/field/case/match/list/buffer failures onto source declarations or
expressions, including quoted Text literals. Core v0.8 adds while-state/condition/body diagnostics, retaining
these coordinate rules. Core v0.9 adds optional supplied-source labels for bundle diagnostics; ranges refer
to that source's exact text, never the root by assumption. Unattributable link errors are unlocated. Ordinary
single-source `compile` still omits labels, while the file CLI supplies its path. Original call-name spans
survive stable-ID resolution and are never serialized. The checked artifact is independently versioned as 0.9.
These do not alter coordinate/provenance rules.
This describes compiler/CLI JSON, not the future Lobo Tool Protocol response schema; a protocol adapter
still needs its own explicit mapping.

## Coordinates and provenance

`diagnostic.range` is a half-open range with `start` and `end` positions. Each position contains `byte`
(zero-based UTF-8 byte offset), `line` (zero-based), and `character` (zero-based UTF-16 code units within the
line). CRLF is one line ending; lone CR and LF also end lines. EOF is a zero-width range. Ranges come from the
current source snapshot, not text search through diagnostic messages. A CLI source-file error additionally
has `diagnostic.source` naming the input path. Library compilation and stdin input have no invented filename.

Lexical errors point to the invalid token/scalar or unterminated quoted ID. Parser errors point to the unexpected
token/EOF, and literal-range errors point to the literal. Name/declaration errors point to the offending name/ID;
type errors point to the incompatible expression. Locations may be enclosing expressions where no narrower
blame exists. Limits and failures without a trustworthy source relationship may omit a range.

Source metadata is ephemeral and never serialized in checked artifacts. Reloaded artifact errors and native
runtime faults have no claimed source location. Identical meaning with different whitespace/comments must
still produce identical checked artifact bytes. A saved artifact cannot inject a trusted location.

## Source-only checking

`lobo check --stdin` accepts one UTF-8 source snapshot, applies the existing source/input budgets, and emits
one JSON outcome on stdout with exit status 0 for success, 1 for failure. It does not read a source file, invoke
Clang, execute a Lobo program, create an artifact, or write workspace files. It reports the first diagnostic;
multiple-error recovery, symbols/types queries, and a language server are later work.

## Editor integration

The extension uses this compiler check for unsaved source in trusted workspaces. Executable configuration is
restricted in untrusted workspaces; highlighting/navigation remain available. Invoke the executable directly
with fixed arguments and stdin, never through a shell. Bound process lifetime and output, debounce edits,
cancel superseded work, and never publish a result for an older document version or a closed document.
Use at most two active checks, 250 ms edit debounce, 1 MiB input, 2 MiB combined output, and a five-second
timeout. Request SIGTERM on cancellation/limits, then SIGKILL after 250 ms if the owned child does not close;
this is not a hard real-time or descendant-process containment guarantee.
Compiler-unavailable/invalid-output/time-limit failures are infrastructure messages, not false source errors.

The compiler must be configured/available separately; the VSIX does not bundle an OS-specific compiler.
