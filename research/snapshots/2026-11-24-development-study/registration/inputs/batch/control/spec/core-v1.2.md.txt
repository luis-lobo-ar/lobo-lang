# Lobo language 1.2

Status implemented. Compiler 1.2.0 directly accepts modern source through `check`, `compile`, `run`,
reference `eval`, source bundles and verification sessions. No marker or projection step is required.

The value, type, action, contract and execution semantics inherit [language 1.1](core-v1.1.md).
Checked format remains `lobo-checked/1.0`. The compiler emits `core/1.2` and revalidates checked artifacts
tagged core/1.0, core/1.1 or core/1.2.

## Source selection and deprecated syntax

`module name;` selects modern source. Each source ends with one `identities` table.
`module name @id("stable.module");` selects deprecated legacy source, with existing declaration annotations,
selected `import` statements, `function` and block-form match arms. Selection is per source.
An explicit bundle may contain both styles. A file cannot mix their declaration grammars.
The historical `// lobo:compact/0.1` marker is an ordinary comment.

Legacy source remains readable for libraries, migration and historical regression fixtures. The old spelling
is frozen. Deprecation is documented without compiler warnings or a removal date.
The [deprecation decision](../decisions/0042-native-compact-syntax-and-legacy-deprecation.md) records removal criteria.

## Declarations and identities

```lobo
module sample;

record Box<T> { value: T }
variant Option<T> { None, Some(T) }
fn identity<T>(value: T) -> T { value }
fn answer() -> I64 { identity::<I64>(42) }

identities {
  module: "sample.module",
  Box: "sample.box",
  Option: "sample.option",
  identity: "sample.identity",
  answer: "sample.answer"
}
```

The table explicitly binds `module`, each local declaration name and each selected `group.member` key to a
stable unescaped ASCII ID. Existing ID syntax and global uniqueness rules apply. Keys are unique and all
entries must be bound. Missing, duplicate or unused keys are compile errors. Trailing commas are rejected.
The table is final. Renames change source bindings and metadata keys while preserving ID values.
The compiler never regenerates an identity from a name, body or path.

Functions use `fn`, explicit boundary types and existing generic parameter syntax. Records and variants
use existing fields, cases and generic parameters. Generic definition checking and specialization limits
remain unchanged. Inline `@id` annotations are outside the modern declaration grammar.

## Import groups and qualification

```lobo
use http { Response, not_found };
```

Groups precede local declarations and operator bindings. The identity table contains `http.Response` and
`http.not_found` with the exact target declaration IDs. Selected members are aliases in the local module.
Names must be unique across groups. Group names are reserved throughout modern source and cannot be local
declarations, parameters, locals, field labels or selected member names.

`Response` and `http.Response` identify the same selected type. `http.not_found()` and `not_found()` call
the same selected function. Qualified types, constructors, generic calls and named references use the same
alias resolution. Unselected members are errors. Qualification does not discover files or download packages.
The existing explicit bundle and operator-target availability rules still apply.

## Match arms

```lobo
match find_user(id) {
  Missing => http.not_found(),
  Found(user) => json.json_response(200, render_user(user))
}
```

A subject may be a bare name, a named call or a parenthesized expression. Complex subjects use parentheses.
Each arm has a case name, an optional payload binding and `=>` followed by either an expression or a block.
Blocks can contain immutable `let` bindings. Commas separate arms. Trailing commas are rejected.
Exhaustiveness, payload scope, selected-arm execution and exact branch types follow existing rules.
Generic calls inside arms and JSON values support explicit comma-separated type arguments directly.

## Text-valued JSON objects

```lobo
json { "name": user.name, "message": "Hello" }
```

Keys are quoted JSON strings and decoded names are unique. Values have type Text. The empty form is `json {}`.
Source order determines nested immutable builder order. The form lowers to existing `Json::Object`, `object`,
`field` and `Json::String` operations. The selected aliases `Json` and `object` require IDs `lobo.json.value`
and `lobo.json.object`. Nonempty objects also require `field` with ID `lobo.json.field`.
Both HTTP and JSON sources are explicit inputs when using the existing helper library.

The ordinary serializer escapes keys and values. Other JSON values retain explicit constructors, including
validated decimal Text. There is no implicit numeric conversion, schema generation or record reflection.
Existing argument evaluation, contracts, actions, fuel and heap charges apply to each lowered operation.

## Bounds and tooling

Existing source, token, type, nesting, scope and specialization bounds remain in force. The identity table
counts toward the source token budget. Modern JSON syntax remains subject to the checked-expression depth
and native output bounds of its expanded helper operations.

Diagnostics carry original source byte and UTF-16 positions. The editor sends original unsaved sources to
the compiler. Extension 1.2.0 supplies marker-free highlighting and original-position structural navigation.
Node.js is not required to check or compile a program with the Rust CLI.
The earlier JavaScript projection command remains an optional compatibility export utility.
