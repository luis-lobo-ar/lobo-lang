# Changelog

## 1.8.0

- Highlight contextual try propagation and preserve ordinary try calls.
- Infer unwrapped result payloads for field-definition navigation.
- Accept Core 1.8 compiler results and report original-source try diagnostics, including unsaved buffers.

## 1.7.3

- Match the compiler release with the standalone native REST example and explicit Windows TCP build profile.
- Revalidate unchanged syntax and navigation. The workspace includes the REST source closure.

## 1.7.2

- Align with compiler packaging support for the standalone Windows clock/random demo.
- Existing language syntax, native navigation and diagnostics are unchanged and revalidated.

## 1.7.1

- Match the compiler patch release that fixes native pending-descriptor cleanup.
- Revalidate existing native syntax, navigation and diagnostics; no source syntax change.

## 1.7.0

- Highlight typed native operations and nested resource/result types.
- Infer catalog signatures and result payloads for local navigation without fake built-in definitions.
- Retain legacy Native variant navigation; accept core/1.7 compiler diagnostics.
- Add Node regressions and isolated real-host unsaved error/repair checks.

## 1.6.0

Support automatic JSON literal builder dependencies, original-range navigation and native diagnostics.
Keep reserved helper aliases internal and render collision-free ordinary aliases in compatibility exports.
Support current 1.6 checked artifacts and preserve earlier supported versions. No highlighting token changes.

## 1.2.1

Update exact navigation anchors for migrated examples and HTTP/JSON libraries. Add corpus-wide selected-import
navigation checks and run the real host against maintained modern source. No editor runtime or language change.

## 1.2.0

Recognize modern source from its module header without a marker. Highlight and navigate generic `fn`,
record/variant declarations, arrow expressions and block arms. Send original source directly to the 1.2
compiler for diagnostics. Remove the old diagnostic projection and range-mapping path.
Retain deprecated source navigation and explicit mixed-source projects.

## 1.1.1

Added opt-in compact MVC authoring support with one shared projector for source generation, original-range
navigation and mapped diagnostics. Highlighting recognizes compact declarations, selected import groups,
qualified calls, Text-valued JSON objects and arrow match arms after the first-line marker.
The Rust compiler remains language 1.1. The compact source requires projection for direct CLI use.

## 1.1.0

- Highlight typed binary operator bindings and their named function targets.
- Navigate overloaded arithmetic symbols to exact target definitions, including explicit source imports.
- Accept language 1.1 compiler diagnostics and retain earlier supported compiler identities.
- Cover bindings in tokenizer, local/project navigation, real compiler and VS Code host tests.

## 0.13.0

- Recognize Core 0.11 compiler success for single-source and explicit-bundle diagnostics.
- Highlight generic function parameters and explicit calls; navigate function-local type parameters,
  imported templates and concrete result fields after type substitution.
- Add generic grammar/navigation/compiler and real extension-host coverage. Preserve earlier versions.

## 0.12.0

- Recognize Core 0.10 compiler success in source and explicit-bundle diagnostics while retaining the
  existing version allowlist and failure handling.
- Validate recursive List payload highlighting, nominal/nested-case definition navigation, and inline-cycle
  diagnostics. Add the native AST example to grammar coverage and real VS Code definition checks.
- No new syntax tokens or dependencies. Existing trust, snapshot and source-budget rules remain.

## 0.11.0

- Add explicit `lobo.sourceFiles` projects: remote function/type/field/case navigation with stable nominal
  identities, forward/cyclic signatures, distinct local names, and unsaved dependency overlays.
- Add `check --bundle-stdin` diagnostics with exact source attribution, dependency-version invalidation,
  process cancellation/budgets, and external-file watcher refresh. Never transplant a dependency range.
- Require trust for dependency reads; bound explicit regular-file UTF-8 input, reject traversal/globs/outside
  targets/special files, and keep compiler-free local navigation. No scan, download, write, or runtime npm dependency.
- Consolidate collection/tree/module-example reverse helpers and maintain actual workload navigation fixtures.
- TDD caught FIFO blocking, unsupported older-host String methods, and acceptance of old bundle-success versions.
  Added regressions; old source-check compatibility remains. No normal-profile install or publication.

## 0.10.0

- Add Core v0.9 selected-import scopes and local alias definition targets; retain ordinary `import(value)` calls.
- Accept compiler 0.9 success envelopes and retain the shared sequence helper as an executable grammar/navigation fixture.
- Cross-file definitions and dependency-aware squiggles remain pending. No dependency loading or workspace
  search is introduced; opaque imported members fail closed. The compiler/CLI already accepts explicit source sets.

## 0.9.0

- Add Core v0.8 typed-state `while` highlighting and definition scope, retaining ordinary `while(value)` calls.
- Accept compiler 0.8 envelopes; test condition/body references, initializer/outside isolation, executable
  state-driven GCD/search, converted parser/tree fixtures, and real-host unsaved condition diagnostics.
- No runtime npm dependency, network service, workspace write, or normal-profile install is added.

## 0.8.0

- Add Core v0.7 Bytes/Text types and intrinsic scopes, JSON escape highlighting, and quoted Text expression
  parsing. Escaped quotes do not leak string contents into comments or guessed definition references.
- Preserve unescaped ASCII `@id` rules; fail navigation closed on malformed escapes/unpaired surrogates.
- Add compiler 0.7 success-envelope compatibility, the eighth executable fixture, and real-host unsaved Text
  diagnostics/navigation through literals containing escaped quotes and non-ASCII characters.

## 0.7.0

- Add Core v0.6 persistent List types and intrinsic operation highlighting/navigation, including nested nominal
  elements, adjacent generic closers, field targets reached through head/tail, and compiler success envelopes.
- Do not fabricate source definitions for built-ins. Reserve List consistently with the compiler.
- Fix record-constructor/field-label lookahead so qualified `::` operations do not consume contract keywords.
- Cover the collection workload, unsaved list diagnostics, and real-host element/field navigation.

## 0.6.0

- Add Core v0.5 variant declarations, payload types, qualified constructors, and contextual match highlighting.
- Navigate variant types/cases and arm-local payload bindings, including forward/nested record payloads;
  structurally incomplete matches fail closed. Add compiler 0.5 success-envelope compatibility.
- Exercise the sixth executable example and real-host navigation/unsaved match diagnostics.
- Runtime faults remain diagnostics, not caught exceptions. No runtime dependencies or normal-profile install.

## 0.5.0

- Add record declaration/constructor/selection highlighting and Core v0.4 compiler compatibility.
- Navigate record types, constructor field labels, and field selections through bounded structural type flow,
  including forward declarations, nested records, function returns, aliases, and fold accumulators.
- Keep unknown field targets unguessed; the compiler remains the authoritative type checker.
- Add record fixture, parser/navigation, native compiler, and real-host unsaved diagnostic regressions.

## 0.4.0

- Support Core v0.3 action clauses in highlighting and structural navigation without globally reserving `actions`.
- Accept Core v0.3 compiler source-check outcomes while retaining Core v0.2 diagnostics compatibility.
- Add action-bound, metadata/navigation, contextual coloring, and unsaved real-host regressions.
- No host resource APIs, runtime npm dependencies, or compiler bundle are added.

## 0.3.0

- Add compiler-owned Problems/squiggles for current unsaved source with exact UTF-16 locations.
- Invoke Lobo 0.2.1+ through `check --stdin`, without a shell or workspace source-file writes.
- Restrict executable configuration and disable compiler checks in untrusted workspaces.
- Debounce edits, cancel superseded checks, discard stale/closed results, and bound input/output/time/concurrency.
- Add source/protocol, lifecycle, trust, and real VS Code diagnostics regression tests.
- Preserve standalone highlighting and local navigation; compiler availability affects diagnostics only.

## 0.2.0

- Add Ctrl-click, Go to Definition, and Peek Definition through a scoped, read-only Core resolver.
- Resolve functions, parameters, immutable locals, fold bindings, and postcondition `result` within the current file.
- Read unsaved editor text; ignore comments/IDs and avoid guessing on incomplete syntax.
- Add bounded parsing, UTF-16 ranges, scope/provider tests, and real VS Code extension-host smoke tests.
- Keep the existing extension identity and syntax grammar; no compiler or runtime dependencies added.

## 0.1.0

- Initial declarative syntax highlighting for Lobo Core v0.2.
- Language association, bracket/quote matching and pairing, and line comments.
- Real TextMate/Oniguruma regression tests and locally installable VSIX.
