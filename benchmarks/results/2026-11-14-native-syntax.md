# Native syntax adoption results

Date 2026-11-14 in America/Chicago. Status engineering acceptance passed.

## Result and compatibility policy

Language and compiler 1.2.0 accept the [modern MVC source](../../examples/mvc.compact.lobo) directly.
Stock `check`, `compile`, `run` and source-bundle commands require no marker or JavaScript projection.
The Rust parser reads declaration identities, selected groups, qualification, `fn`, arrow arms and Text-valued
JSON objects. Generic declarations, callbacks, contracts and block-bodied arms retain existing semantics.

The old spelling is deprecated read compatibility. A source module header selects its grammar automatically.
There is no legacy CLI switch or separate checker, runtime or native backend. Mixed bundles permit existing
libraries and old regression fixtures to participate unchanged. The old spelling is frozen. Removal requires
a maintained-caller migration, replacement coverage and a separate decision. No warnings or removal date are
introduced. See [decision 0042](../../decisions/0042-native-compact-syntax-and-legacy-deprecation.md).

Checked format remains 1.0. Artifacts tagged core/1.0, core/1.1 and core/1.2 are revalidated.
The frozen 1.0 release and earlier source/compiler evidence remain historical artifacts.

## Verification

Five compiler cases failed before implementation. They now pass direct parsing without a Node executable
on PATH, exact checked MVC equality, deprecated-source interoperability, checked reload, original Unicode
and CRLF positions, metadata failures, generic callbacks and JSON escaping/order/empty objects.

Native comparisons retain independent expected status 200 with `{"name":"Luis"}` and status 404 with
`Not found` plus a newline. Reference, C, direct LLVM, C sanitizer and LLVM sanitizer routes agree.
Low fuel and heap remain resource-exhaustion outcomes under the existing 10,000 invocation budgets.

The final full suites pass 206 Rust tests, 141 editor tests and 31 Python tests. Clippy with warnings denied,
formatting, real VS Code host checks and VSIX packaging pass. The editor sends original source directly to
the compiler, without range remapping. Marker-free syntax, generic declaration navigation and block-arm
scopes have explicit regressions. The real host exercises unmarked source, unsaved errors/repairs and mixed
source-project navigation.

The 18 identities in the programming skill's examples survive migration to modern syntax. Exact checked
artifact equality against the retained old example is asserted by the CLI verification. Six native skill
invocations pass. Direct CLI evidence retains 19 observations, including empty-PATH file/bundle checks,
C and LLVM compilation and execution, checked reload and an 11-source editor workspace bundle.

The extension package is `dist/lobo-syntax-1.2.0.vsix`. It is rebuilt, not installed into a normal user profile.
The [native guide](../../docs/compact-syntax-prototype.md) replaces the earlier projection instructions.
The old projection command remains an optional compatibility export utility.

## Negative results and corrections

The first native suite failed all five new cases because modern module headers lacked required legacy
annotations. The new Rust parser made those cases pass without weakening the independent response oracles.

The first editor adoption tests returned unavailable outcomes because the adapter did not yet admit the
new language identity. A second red run also established that the adapter transformed marked text rather
than sending the original snapshot. Native diagnostics now receive original input and core/1.2 is admitted.
The marker-free grammar regression failed before global modern highlighting support.

Clippy rejected a negated optional-state predicate as nonminimal boolean logic. Replacing it with
`is_none_or` preserved the parser condition and passed warnings-denied validation.

A guarded mechanical skill migration expected 20 declarations but discovered 18 and stopped before writing.
The rerun derived the expected count from the original declaration inventory. It preserved all 18 identities
and exact checked behavior. That failed assertion is retained in the tool transcript rather than a separate log.

Intermediate engineering runs retain raw logs. Their exact modified source states were not separately
snapshotted. The final source, compiler binaries, native outputs, extension package and reports are preserved
under `benchmarks/private-evidence/native-syntax-20261114` with hashes.

## Interpretation

This change establishes native source acceptance and toolchain parity. It removes the external authoring
projection requirement from normal compiler use. It does not establish improved agent software construction,
comprehension, repair cost or token efficiency. Those outcomes remain unknown. No runtime or compile-time
performance comparison ran for this change.

JSON shorthand remains Text-valued. Stable IDs are explicit source metadata. Automatic ID assignment, rename
refactoring and formatter development remain separate work. The [1.2 specification](../../spec/core-v1.2.md)
defines source rules and bounds. See [the receipt](2026-11-14-native-syntax.json) for commands and retained evidence.
