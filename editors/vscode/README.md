# Lobo Language Support for VS Code

Compiler/extension 1.7 adds highlighting and built-in signature/type inference for `Native::` operations
and opaque resources. Native built-ins have no source definition; local arguments, fields and result payloads
navigate normally. Diagnostics check source and never execute IO. Legacy Native user variants retain their
old navigation semantics. See repository file `spec/native-interface-v0.1.md` for the native contract.

Syntax highlighting and definition navigation for Lobo language 1.11. Opening a `.lobo` file selects **Lobo**
automatically. Standard TextMate scopes follow your theme's colors.

Extension 1.6.0 supports JSON literals without selecting `object` or `field`. Generated builder aliases retain
original source ranges and stay internal to navigation. Compiler 1.6 resolves these helpers by stable ID from
explicit sources and checks types/actions normally. Compatibility export uses fresh ordinary aliases.
Legacy export does not elaborate generic-call or constructor inference; compile modern source directly.
See `spec/core-v1.6.md`. Existing constructor inference and contextual case navigation remain supported.

Extension 1.4.0 adds navigation through inferred generic results and contextual callback names.
Compiler 1.4 supplies diagnostics for these calls. Explicit calls and function references remain supported.
See `spec/core-v1.4.md` for inference limits. The TextMate grammar needs no additional token.

Extension 1.3.0 introduced tracked adjacent `.lobo.ids.json` metadata for trusted workspace source snapshots.
It overlays unsaved source and metadata documents and invalidates results when either changes. Managed
source requires compiler 1.3 for diagnostics. Inline identity tables and deprecated annotation-based
source retain read support. The editor never assigns IDs or writes metadata. Use the compiler's explicit
`identities init`, `sync` and metadata-only `rename` authoring commands. F2 source refactoring is not included.
Highlighting works without metadata. Managed navigation needs its trusted metadata snapshot.
See `spec/core-v1.3.md` and `docs/compact-syntax-prototype.md` in the checkout for the contract and native commands.

Version 1.1.0 supports typed operator bindings. Binding declarations and overloaded arithmetic symbols
navigate to their exact named target. Explicit project imports enable cross-file navigation through aliases.
The source compiler validates signatures, purity and ownership. Built-in operators expose their canonical
operation in checked output. The compiler source checkout contains the rules in `spec/core-v1.1.md`.

## Features

- Module/function/record declarations and calls, `@id` annotations, and quoted stable identities.
- Record constructors, fields, type references, and definition navigation using declared/structurally inferred
  record types. Unknown/ambiguous field targets are not guessed by spelling; compiler type checks remain authoritative.
- Tagged variant declarations/constructors, payload types, and exhaustive matches; navigation resolves cases
  from their nominal type and match bindings only within the selected arm's lexical scope. Forward/nested
  alternatives compose with record field navigation. Diagnostics for these constructs require compiler 0.5.0+.
- Nested `List<T>` types and `List::empty<T>()`/prepend/head/tail/length built-ins, with structural element-type
  navigation through head/tail and user function returns. Built-ins have no source definition; nominal element
  types still navigate. These features require compiler 0.6.0+ for diagnostics; `List` is now reserved.
- Text/Bytes types, explicit intrinsic operations, and JSON-escaped UTF-8 Text literals (compiler 0.7.0+).
  Escaped quotes do not leak comments/references; string contents and built-ins have no source definition.
  Text/Bytes are reserved consistently with the compiler. JSON escapes and malformed escape tokens get scopes.
- Immutable typed-state `while` loops, including state scope in conditions/bodies (compiler 0.8.0+).
- Selected `import alias @id("declaration.id");` syntax, local alias targets, and explicit-project remote targets.
- `I64`/`Bool`, decimal integers, boolean literals, `let`, contracts, conditionals, and `result`.
- Counted `fold (i in n, a: I64 = 0)` prefixes, without globally reserving `fold`, `in`, or `id`.
- Operators, punctuation, `//` comments, bracket matching, quote/bracket pairing, and line commenting.
- **Ctrl-click / F12** (Go to Definition) and **Alt+F12** (Peek Definition) for function calls,
  parameters, local bindings, and fold index/accumulator bindings in the current file.
- Compiler-produced error squiggles and Problems entries for the current unsaved source in trusted workspaces.

Cross-file support uses an explicit `lobo.sourceFiles` list (see below), never workspace scanning or import
path guessing. It resolves supplied functions/types/fields/cases by stable nominal identity, including
inferred hidden signature types and unsaved dependency text. Missing/ambiguous structural shapes fail closed.

Highlighting uses a declarative grammar; navigation uses a small read-only extension-host provider.
Those features need no compiler. Diagnostics require a separately configured Lobo compiler 0.2.1+.
The extension uses no network access, telemetry, runtime npm dependencies, or workspace file writes.
Source checking does not compile native executables or execute programs. No formatter, language server, autocomplete,
package resolver, or syntax support for the wider illustrative design is included.
Core does not support block comments. Quoted expressions are Text literals with JSON escapes; `@id` values
retain the narrower unescaped ASCII identity syntax.
Core v0.3 `actions { ... }` clauses have contextual highlighting, action-ID scopes, body navigation, and
compiler-produced bound/purity errors (compiler 0.3.0 required for these checks). Resource types/APIs remain
future work. `actions` remains a legal ordinary identifier outside the signature clause.

Navigation reads the current editor text, including unsaved edits. It respects declaration order, branch scope,
fold scope, and match-arm scope. Parameters are visible in contracts, but body locals are not; postcondition `result` navigates
to its function declaration. Names in comments or quoted IDs do not navigate. Types such as `I64` and `Bool`
are built-ins with no source definition. Desktop and remote extension-host navigation is supported;
standalone browser-host navigation is not implemented.

## Install locally

From the Lobo repository root, after building the package:

```sh
code --install-extension dist/lobo-syntax-1.11.2.vsix
```

Or open VS Code's Extensions view, choose **Install from VSIX...** from the menu, and select that file.
Open any `.lobo` example; select **Lobo** in the status bar if a previous file association overrides it.
Reload the window if necessary. In a remote VS Code session, install the VSIX in the window where you edit
Lobo files. Installing or updating this extension does not change the compiler or existing files.

## Explicit source projects

In a trusted workspace, list the complete source set in `.vscode/settings.json`, for example:

```json
{
  "lobo.sourceFiles": ["examples/collections.lobo", "examples/tree.lobo", "examples/sequences.lobo"]
}
```

Only listed documents participate. Paths are relative to the containing workspace folder; maximum 16 files,
1 MiB UTF-8 source bytes and 8,192 tokens in aggregate. An empty list selects only the current workspace file and its adjacent metadata. Reject absolute paths,
traversal, duplicates, globs, unsupported URI schemes, and canonical disk targets outside the workspace.
Regular-file reads are byte-bounded before strict UTF-8 decoding; special files are rejected without waiting
for a FIFO writer. Open documents overlay disk text, including unsaved edits. Native filesystem-backed
desktop/remote extension hosts support this mode; virtual documents keep local-only support.

Imports remain declaration IDs, never paths. The structural catalog supports forward/cyclic function imports,
selected aliases, distinct same-spelling declarations, and exact dependency definition targets without a
compiler. It does not prove type correctness. Bundle diagnostics require compiler 0.9+ and report a dependency
error on that dependency, not on the unchanged root. Missing/configuration/unlocated errors go to Output.
Explicit file watchers and document/configuration events invalidate snapshots and reschedule checks; old
dependency versions cannot publish late results. No package downloads or hidden dependency expansion occur.
This is not an OS sandbox against hostile concurrent filesystem mutation. Limit dependencies by clearing
the list; local highlighting/navigation and single-source diagnostics remain available.

## Compiler diagnostics

Compiler 0.11.1's optional direct LLVM backend changes no source syntax or check envelopes; extension
0.13.0 remains compatible. Editor checks use the Lobo compiler's check commands and do not compile through
either native backend. Backend selection is available through the compiler CLI, not a new editor setting.

Build the compiler with `cargo build --locked --release --bin lobo` from the repository root. This repository's
workspace setting uses `${workspaceFolder}/target/release/lobo`. Elsewhere set **Lobo: Compiler Path** to an
executable path, or place `lobo` on PATH. The path is an executable only, not a shell command with arguments.
Opening this repository as a workspace folder lets the supplied setting resolve; a loose file cannot resolve
`${workspaceFolder}`. `lobo.diagnostics.enabled` can disable compiler checks without disabling navigation.

Compiler checks run only in trusted workspaces. In Restricted Mode, no compiler is launched, including one
configured by the user; highlighting and local navigation remain available. Dependency reads are also
disabled, even for user-configured project lists. Executable/source-list workspace settings are
restricted by [VS Code Workspace Trust](https://code.visualstudio.com/api/extension-guides/workspace-trust).
Only trust source and compiler executables you intend to run; this feature is not an OS sandbox.

Checks use `lobo check --stdin` or `lobo check --bundle-stdin`, never a shell, Clang, or program execution. They are debounced
by 250 ms, use at most two active checks, and allow 1 MiB input, 2 MiB combined output, and five seconds per
process. Bundle wire JSON has a separate 4 MiB admission cap. Superseded/closed checks are canceled and
old-version results discarded, including dependency changes under an unchanged root. Termination requests use
SIGTERM with SIGKILL after 250 ms if needed; this is not a hard real-time or descendant-process containment guarantee.
The compiler currently reports its first error, not every error in a broken file. Missing/outdated compilers,
unlocated global limits, invalid output, and timeouts are reported in **Output → Lobo Compiler**, not guessed
source squiggles. Built-in types and runtime faults still have no navigable source error definition.

## Develop and test

Node.js 22+ is required only for extension development. From `editors/vscode`:

```sh
npm ci --ignore-scripts
cargo test --locked --manifest-path ../../Cargo.toml
npm test
npm run package
```

The lockfile pins development tools. Tests use the same Microsoft TextMate tokenizer and Oniguruma engine
as VS Code, not JavaScript regex approximations, and cover fifteen executable source fixtures plus selected-import projects.

Core 0.11 generic function parameters navigate within their function. Explicit `f::<Type>(...)` calls
navigate to the function or imported declaration, and substituted concrete results support field/case
navigation. Highlighting covers parameter declarations and nested call type arguments. Compiler diagnostics
remain the authority for concrete specialization checks; navigation is a structural projection.
The package excludes tests, development dependencies, and editor launch settings.

Navigation/provider tests use Node's built-in runner. Real extension-host tests use an installed VS Code
executable (not its `code` CLI wrapper) and a fresh temporary profile, and require a completed-test marker
as well as a successful process exit. No editor download or new test dependency is needed. On headless Linux:

```sh
LOBO_VSCODE_EXECUTABLE=/usr/share/code/code xvfb-run -a npm run test:host
```

For other platforms, set `LOBO_VSCODE_EXECUTABLE` to the installed VS Code executable and run
`npm run test:host` with a graphical display available. Tests exercise VS Code's definition command,
activation, `.lobo` association, UTF-16/CRLF ranges, and unsaved document edits.
They also check compiler squiggles, changed unsaved errors, remote function/field/case targets, unsaved dependency
errors/overlays, and external explicit-file invalidation. Adapter tests exercise a built debug compiler
and controlled faulty/canceled processes; controller tests cover trust, stale results, and concurrency.

For an Extension Development Host, open this directory in VS Code and press **F5**. Inspect a token with
**Developer: Inspect Editor Tokens and Scopes**. Test against a light and dark theme when changing scopes.
The extension development profile opens the repository examples; it does not install into your normal profile.

## Lexical limitations and publication

Action highlighting recognizes a same-line return type plus clause, or `actions {` at line start. Comment-
separated prefixes may keep ordinary coloring, and split conditional expressions can be lexically ambiguous.
Navigation parses complete declarations across comments/newlines, independent of these lexical heuristics.
List-type and intrinsic prefixes also use same-line lexical recognition; comment-separated prefixes still
navigate but can retain ordinary token colors. Structural type parsing supports eight nested list constructors,
including adjacent `>>` closers and `>=` at a type's close followed by a binding initializer.
Likewise, `match` gets keyword coloring for a same-line simple parenthesized subject followed by `{`;
nested parentheses or split prefixes may keep ordinary call coloring. Case declarations and qualified
constructors get case scopes; arm labels use ordinary call/name coloring. Navigation does parse all these
forms and does not infer a case definition from spelling alone. `variant` and `match` stay ordinary names
outside their contextual forms.

The grammar is not a parser or validity check. Identifier declaration/call classification is lexical and
some same-line patterns cannot span comments/newlines. In particular, only an unambiguous same-line
`fold (identifier in` prefix gets loop-keyword coloring; split prefixes keep ordinary identifier/call
coloring even though the compiler accepts them. This avoids coloring valid ordinary `fold(in)` calls as loops.
Quoted text recovers at end of line while editing; the compiler remains the authority for valid stable IDs.
The navigation parser validates JSON escapes and rejects unpaired UTF-16 surrogates to agree with the
compiler's UTF-8 Text boundary. The TextMate grammar colors escapes but is not a Unicode/JSON validator.

Navigation is a structural Core resolver, not the compiler's validator. Incomplete or structurally invalid
documents yield no definition rather than guessing. It does not itself diagnose type errors or prove contracts;
the compiler diagnostic feature is separate.
It bounds input at 1,048,576 UTF-16 code units, 8,192 tokens, and 32 levels of nesting/operator chains.
Multiline/comment-separated fold prefixes navigate correctly even when their highlighting is conservative.
The bundle catalog additionally enforces aggregate byte/token/source bounds and a 1,024-slot metadata bound.
Core supports explicit imports; only configured sources are loaded, never unrelated workspace files.

This private package is not published to the Marketplace. `lobo-lang` is a local publisher identity, not a
claim of Marketplace registration. Licensing and publication require separate decisions.

Implementation follows the official VS Code documentation:
[syntax highlighting](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide),
[language configuration](https://code.visualstudio.com/api/language-extensions/language-configuration-guide),
and [VSIX packaging](https://code.visualstudio.com/api/working-with-extensions/publishing-extension).
Navigation uses VS Code's [DefinitionProvider API](https://code.visualstudio.com/api/references/vscode-api#DefinitionProvider).
# Language 1.0

Extension 1.0.0 understands generic record/variant parameters, explicit constructors and nested applied types.
Navigation substitutes generic field and match payload types, including through explicit source bundles.
Pure `Fn(T) -> U` signatures, `&named` / `&generic::<Type>` references and calls through function values
retain definition and result-field navigation. Compiler 1.0.0 diagnoses invalid unused generic definitions.
F64 signatures, fractional/exponent literals and conversion/classification operations are supported too.
Final package/check status is recorded in the language 1.0 release receipt. Earlier extension packages remain
preserved as historical checkpoints.


Core 1.8 adds `try producer()` for result propagation. Highlighting and navigation retain the success
payload's type. The compiler reports incompatible error types and forbidden contract use in original source.
Ordinary `try(value)` calls retain their meaning. See `spec/core-v1.8.md` in the Lobo source repository.

Compiler/extension 1.8.1 retain Core 1.8. The maintained-source migration uses inferred constructors and
callbacks in the example corpus. Real extension-host fixtures navigate those forms and reject missing
fixture locations. No additional keyword or navigation runtime change is needed for this source migration.

Core/compiler/extension 1.9 add SQLite resources/results/operations to highlighting, payload/handle
navigation and original-range diagnostics. Bundle navigation admits 16,384 tokens and 64 source nominal
declarations, while individual lexing keeps its existing bound. Open the supplied
[SQLite REST workspace](../../examples/native/sqlite-rest/SQLiteRest.code-workspace) for its complete
eight-source context. Compiler checks never open a database. Runtime access still requires explicit grants.


Core 1.10 adds one `...` record-update operator and interleaved standalone try statements. Navigation
resolves update bases, override fields and generic payloads inferred from the base. Compiler diagnostics
retain the original override expression range. The extension does not execute IO to resolve source.

## Core 1.11 callback actions

The extension highlights `Fn(C, Request) actions { db.read } -> Response`, resolves named and generic
callbacks and infers fields through their results. Pure callback syntax remains supported. Navigation
keeps the declared callback action set in its signature model. Compiler diagnostics check indirect-call
bounds, exact target signatures and contract restrictions against unsaved source. Real host tests remove
an invocation's declared action, assert the call-range squiggle and repair the document. The SQLite app
bundle navigates serve_with into the library and handle into the application controller.

## Maintained HTTP library refactor

Extension 1.11.1 accompanies the unchanged Core 1.11 grammar. The stateless Rest.code-workspace and SQLite
workspace list their complete source closures. Navigation resolves the owning serve/serve_with entries,
controllers, shared accept helper and Connection fields through stable IDs. The real host tests copy the
six-source stateless bundle, navigate into the library, remove net.listen in an unsaved app, assert the
missing-action squiggle on serve(...), then repair it. No new grammar token is introduced.

Compiler/extension 1.11.2 add typed OS and SQLite permission sets without changing Core 1.11.
The maintained HTTP apps use named permission profiles. See [typed permissions](../../decisions/0062-typed-permissions.md).
