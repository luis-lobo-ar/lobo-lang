# Compact source projection 0.1

Historical prototype. [Language 1.2](core-v1.2.md) supersedes the projection requirement and directly accepts
modern source, including generic declarations and block-bodied arms. The original prototype rules below
remain an experimental record. Use [the current native guide](../docs/compact-syntax-prototype.md).

Status implemented authoring prototype for Lobo language 1.1. The stock Rust parser accepts canonical 1.1
source. Compact source requires the [projection tool](../scripts/compact-syntax.cjs) or extension 1.1.1.
The prototype is separate from the language version and checked artifact format.

## Admission

The exact first line is `// lobo:compact/0.1`, followed by LF or CRLF. Other source passes through unchanged.
Each compact source begins with `module name;` and ends with one `identities { ... }` table.
UTF-8 source is bounded to 1 MiB and 8,192 tokens. Delimiter and rewrite nesting are bounded to 32 levels.
Generated source has the same byte and token bounds. Bundles retain the compiler's aggregate bounds and
16-source limit. Compiler validation still checks complete explicit bundles.

The prototype admits concrete `fn`, `record` and `variant` declarations. Generic declaration parameters
are rejected by the projector. Types at function boundaries remain explicit. Ordinary expression blocks,
contracts, actions, constructors and operators use inherited 1.1 rules after projection.

## Declaration identities

```lobo
// lobo:compact/0.1
module sample;
fn increment(n: I64) -> I64 { n + 1 }
identities {
  module: "sample.module",
  increment: "sample.increment"
}
```

`fn` lowers to `function`. Each declaration obtains its explicit `@id` from its table entry.
The table uses comma-separated keys and quoted stable ASCII IDs. Trailing commas are rejected.
The module key is `module`. Local declaration keys are their source names. Selected-import keys are
`group.member`. Missing, repeated and unused keys are projection errors. Duplicate ID values remain compiler
errors. IDs are persistent inputs. The projector never derives IDs from source names or function bodies.

## Selected imports and qualified references

```lobo
use http { Response, not_found };
```

The table binds `http.Response` and `http.not_found` to explicit stable declaration IDs.
Each member lowers to an ordinary selected import. Selected names are unique across all groups in a source.
Bare `Response` and qualified `http.Response` both refer to the selected alias. `http.not_found()` lowers to
`not_found()`. Group names are reserved throughout compact source and cannot also be declarations, imports,
parameters, locals or field labels. A qualified reference must name a selected member.

Group spelling does not identify a filesystem path or discover a module. The supplied ID determines the
declaration. Source files remain explicit compiler inputs. Importing an operator target retains existing
operator-availability rules.

## Match expressions

```lobo
match find_user(id) {
  Missing => http.not_found(),
  Found(user) => json.json_response(200, render_user(user))
}
```

The subject is a bare name, a named call or a parenthesized expression. More complex subjects require
parentheses. Each arm uses a case name, an optional parenthesized payload binding and `=> expression`.
Commas separate arms and a trailing comma is rejected. The projector adds canonical subject parentheses
and arm blocks. Exhaustiveness, payload scope and branch types follow the compiler's existing rules.
Comma-containing generic applications within an arm require enclosing parentheses. Standalone block
expressions as arrow bodies are outside this prototype.

## Text-valued JSON objects

```lobo
json { "name": user.name, "message": "Hello" }
```

Each key is a quoted JSON string. Decoded keys are unique. Each value has type Text.
Empty objects are admitted. Fields lower in source order to nested immutable builder calls.
The enclosing `Json::Object` and each `Json::String` preserve the exact canonical construction.
Object construction requires selected `Json` and `object` imports with IDs `lobo.json.value` and
`lobo.json.object`. Nonempty objects also require `field` with ID `lobo.json.field`.
These fixed aliases make the limited lowering inspectable. The full JSON library still supports other
types through explicit constructors. No numeric conversion is implicit.

The existing serializer escapes field names and values. Compiler type checks, contracts, actions, fuel and
heap charges apply to every generated operation. The projector adds no host capability or resource grant.

## Tool output and diagnostics

Without `--out-dir`, the CLI prints a canonical `lobo-source/0.1` JSON bundle to stdout.
With `--out-dir`, the tool creates a fresh directory containing `app.lobo`, numbered dependencies,
`bundle.json` and `inputs.json`. Existing directories are rejected. Output is a source projection, not
compiler acceptance. Input files are explicit regular UTF-8 files with a cumulative 1 MiB read limit.
The CLI prints structured errors to stderr and exits nonzero on projection or IO failure.
Compiler diagnostics on projected files or bundle text use generated-source positions. Original-source
diagnostic mapping is supplied by the editor adapter.

VS Code uses the same projector for unsaved-source navigation and compiler diagnostics. UTF-8 diagnostic
bytes map back to original UTF-16 positions, including CRLF and Unicode. Generated helper tokens have no
clickable origin. Actual parameters, fields, cases, calls and selected imports retain original ranges.
Malformed compact source fails closed. Unmappable compiler outcomes appear in compiler output rather than
guessed squiggles. TextMate highlighting is lexical and uses the opt-in marker.

The prototype has no formatter, automatic identity assignment, rename refactoring or semantic workspace.
General language adoption requires a separate decision and construction/comprehension evidence.
