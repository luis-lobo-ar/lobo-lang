# Lobo Core v0.4: immutable record values

Historical version, extended by the current language in the [specification index](README.md).
Version-specific limits/interchange below describe the 0.4 reader, not the current loader.

Version: `core/0.4-draft`, inheriting [Core v0.3](core-v0.3.md). Records add compound values without new
authority, identity, assignment, heap allocation, or implicit conversions. Compiler/artifact version 0.4.

## Declarations and expressions

```lobo
record Pair @id("example.pair") { left: I64, right: I64 }
function swap @id("example.swap")(p: Pair) -> Pair {
  Pair { right: p.left, left: p.right }
}
```

Records may appear before or after functions. Each has a unique stable declaration ID, a nonreserved name,
and 1–16 distinctly named fields. Fields have I64, Bool, or declared record types; forward type references
are allowed. Declaration/type/constructor names share the module declaration namespace. `record` remains
contextual at top level, as `actions` does in signatures. All existing ordinary names remain legal except
collisions with newly declared record names. References to record types/constructors resolve to stable IDs.

Types are nominal: two declarations with identical fields are distinct types. Record constructors use
`Name { field: expression, ... }`; provide every field exactly once in any order, without a trailing comma.
The parser recognizes a constructor by identifier, `{`, identifier, `:` lookahead; empty constructors are
invalid. This preserves `if flag { expression }` even when conditions use ordinary identifiers. Postfix
`.field` applies to record-valued expressions, including calls/parentheses/conditionals, and binds tighter
than unary and binary operations. Fields are not implicitly module/global names.

Fields evaluate left-to-right in their written order, stopping on the first fault. Their source order must
not be normalized in artifacts. Selection evaluates its base once. A copy, parameter, return, conditional,
contract result, or fold accumulator preserves the complete value. Equality/inequality recursively compare
corresponding fields of the same nominal type; arithmetic/ordering/boolean operators still require their
existing scalar types. No record update shorthand, methods, implicit defaults, or mutation is added.

## Representation and bounds

Reject direct/indirect layout cycles (`compile.record.cycle`), more than 32 record declarations, nesting
deeper than eight records, or layouts exceeding 64 scalar leaves (`resource.frontend.limit`). No references
or user-visible addresses exist. Native lowering uses compiler-named by-value C structs, dependency-ordered
layouts, generated fieldwise equality, and ordered constructor temporaries; not padding-byte comparisons.
No garbage collector or custom heap is introduced. The reference implementation uses owned values.

Record-valued parameters/output and expression temporaries contribute their scalar-leaf width to a per-function
frontend budget of 2,048 record cells, bounding aggregate copy/storage expansion. This is an admission budget,
not a promise about exact C stack size or an OS sandbox. Existing call/fuel/frontend limits still apply.

Unknown types/constructors: `compile.type.unknown`; missing/duplicate/unknown constructor fields:
`compile.record.fields`; invalid selections: `compile.record.field`; nominal/type mismatch retains
`compile.type.mismatch`. All fields/branches/contracts/fold parts are checked even if unreachable. Reloaded
artifacts revalidate the same rules, including bounds and constructor/selection targets.

## Invocation and checked interchange

Format `lobo-checked/0.4` adds mandatory `program.records`, record type references as `{"Record":"stable-id"}`,
and Record/Field expression variants. A Record expression contains `record` (stable ID) and ordered `fields`
objects with `name` and `value`. A Field expression has `base` and `field`. Record declarations have `name`,
`id`, and ordered parameter-shaped `fields`. Scalar literals remain scalar; record literal injection is rejected.

Record runtime values use `{"type":"Record","value":{"id":"stable-id","fields":{"name":typed-value}}}`.
Host inputs must match the complete declared shape recursively, not just the nominal tag. Record CLI arguments
use `Record:stable-id:I64:1,Bool:true,...`, scalar leaves flattened in declaration order (depth-first). The
selected parameter determines the exact full ID prefix, so IDs containing colons are unambiguous. Wrong IDs,
leaf types/counts, or malformed values fail before execution. Native output preserves the typed JSON value
shape, including nested records. Old scalar arguments remain unchanged. The native adapter advances to 0.2.

Recompile old artifacts rather than retagging; the 0.4 loader accepts only the 0.4 format/language pair.
Old native executables remain independent. Public record fields carry names, not separate stable graph IDs;
canonical semantic graph interchange remains deferred. Dynamic/recursive structures are later work.
