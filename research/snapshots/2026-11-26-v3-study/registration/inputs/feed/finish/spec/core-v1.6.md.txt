# Lobo language 1.6

Status implemented by compiler 1.6.0. This version extends [language 1.5](core-v1.5.md) with automatic
builder dependencies for modern JSON literals. Source compilation emits `core/1.6`; checked format remains
`lobo-checked/1.0`. Admission revalidates core/1.0 through core/1.6. Other rules are inherited.

## Authoring

```lobo
use json { Json, json_response };

fn render_user(user: User) -> Json {
  json { "name": user.name }
}
```

`Json` remains explicitly selected and bound to `lobo.json.value`. Functions called in source remain
explicit imports or declarations. Literal keys must be unique after string decoding, and values must be
Text. The existing `Json::String` and `Json::Object` construction and ordered builder calls are unchanged.
There is no new runtime operation, implicit conversion or JSON parsing facility.

For each source containing a literal, the compiler resolves the object builder ID `lobo.json.object`.
A nonempty literal also resolves `lobo.json.field`. An empty literal does not add the field dependency.
If a selected import already targets the required stable ID, the compiler reuses that alias. Otherwise it
adds a generated module import named `__lobo_json_object` or `__lobo_json_field`. These dependencies are
visible in the checked artifact's module imports. Each generated import is added once per source and
charged eight tokens against the existing bundle frontend token budget. Existing expression, nesting,
source, type and specialization budgets continue to apply.

A source declaration or import named `object` or `field` with another identity cannot redirect a literal.
Generated aliases are reserved and inaccessible in source. Their presence does not make ordinary
`object()`, `field(...)` or an unselected group member callable. Explicit selected builder imports remain
supported for direct calls and existing literals. The old source grammar remains frozen read compatibility.

## Closed dependencies and checks

Supply the JSON and HTTP helper files explicitly with `--source` or in a source bundle. Generated imports
do not discover paths, fetch packages, allocate IDs, or select library versions. Missing target IDs fail
module linking. The normal resolver and type/action checkers validate selected helpers and their calls;
a helper identity alone does not waive its signature, contracts, or action declarations. Value errors
retain their original source ranges. An explicitly imported invalid target still fails module linking,
even if a literal separately binds the canonical helper.

Checked artifacts contain resolved ordinary calls and imports. Loading performs existing closed validation,
with no literal elaboration or dependency discovery. Compiler 1.6 revalidates earlier artifacts.

## Editor and compatibility export

Extension 1.6.0 mirrors the generated dependencies for navigation. Generated tokens retain original anchors
and have no visible source spelling to navigate from. Explicit Json/function imports and value expressions
retain definition targets and native compiler diagnostics. There is no new highlighting token.

The optional JavaScript compatibility export renders generated aliases as fresh ordinary names such as
`lobo_export_json_object`, avoiding all original token spellings and other generated names. Its text remains
valid legacy syntax; its navigation token stream retains compiler aliases and original ranges. This export
does not elaborate 1.4/1.5 generic-call or constructor inference. Programs using those forms need explicit
legacy equivalents to compile exported text. Ordinary
source compilation requires no export step. Export does not discover dependencies or manage metadata.

## Migration and evidence

The maintained generic MVC removes its two unused source imports and corresponding binding keys.
Every remaining identity value, declaration ID, owner and helper target ID is preserved. Checked import
aliases and ordering differ; equality comparisons normalize only those labels/order, not declarations,
operations, IDs or bodies. The explicit-import compact comparison and frozen research inputs are unchanged.

[Decision 0047](../decisions/0047-json-literal-helper-dependencies.md) records alternatives and checks.
[The engineering result](../benchmarks/results/2026-11-22-json-auto-helpers.md) records observed outcomes;
agent construction, comprehension, token efficiency and runtime-speed effects remain unmeasured.
