# 0009: Declarative VS Code syntax highlighting

- Status: implemented in extension 0.1.0
- Date: 2026-10-13

Extended by [decision 0010](0010-vscode-local-definition-navigation.md): extension 0.2.0 adds read-only
extension-host definition navigation while preserving the declarative syntax grammar.

## Decision

Provide `.lobo` highlighting for the implemented Core v0.2 language through a declarative TextMate grammar
and language configuration in `editors/vscode`. Do not add a language server, compiler invocation, commands,
telemetry, runtime dependencies, or workspace writes. Broader illustrative syntax is not advertised as Core.
Use conventional scopes so installed themes control colors. This is lexical assistance, not validation.

## Development dependencies and evidence

Add three pinned, development-only npm dependencies: Microsoft's `vscode-textmate` and `vscode-oniguruma`
to exercise the same tokenizer/regex dialect used by VS Code, and `@vscode/vsce` to package an installable
VSIX. Plain JavaScript regex tests would not verify TextMate state or Oniguruma behavior. The need is actual
grammar verification and packaging, not a new language runtime; none of these packages ship in the VSIX.
Use Node's built-in test runner and a lockfile, without a bundler or test framework.

Package locally only; marketplace publication, publisher registration, installing into the user's editor,
and repository licensing are separate choices. Keep this private/unlicensed rather than infer a license.
The `lobo-lang` publisher string is a local extension identity, not a registered marketplace claim.

## Verification

On 2026-10-13, all 12 grammar/configuration/example tests passed with Node 22.19.0, TextMate 9.3.2, and
Oniguruma 2.0.1. The VSIX packaged successfully with vsce 4.0.0 and installed/listed as
`lobo-lang.lobo-syntax@0.1.0` in an isolated VS Code 1.105.1 profile. Its seven archive entries contain only
extension metadata, documentation, language configuration, and grammar; no development dependencies or code.
The full 23-test Cargo suite, repository JSON/link checks, and diff whitespace check passed. npm audit reported
zero known vulnerabilities at this run. No visual light/dark-theme review was performed; it remains a manual
check documented in the extension README. No publication or installation into the normal user profile occurred.

TextMate tokenization is line-oriented. Recognize contextual `fold`/`in` only for the unambiguous same-line
prefix; keep conservative identifier/call coloring for split prefixes rather than miscolor ordinary names.
This tested limitation does not change compiler parsing or language semantics.
