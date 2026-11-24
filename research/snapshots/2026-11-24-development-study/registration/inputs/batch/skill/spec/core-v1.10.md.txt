# Lobo Core v1.10

Status: implemented by compiler/extension 1.10.0. Inherit [Core 1.9](core-v1.9.md).
This adds two modern-source conveniences. Checked format remains lobo-checked/1.0, native ABI 1 and
optional SQLite extension 1. No runtime type, dependency, resource authority or admission limit is added.

## Immutable record updates

```lobo
Search { ...state, duplicate: true }
```

A constructor can contain one `...base` first, followed by zero or more explicit field assignments.
The record owner remains explicit; its generic arguments can be inferred under existing rules, including
from the base. Explicit generic arguments remain available. The base must have the same nominal record
and exactly matching type arguments. Structural field similarity does not permit a different record.

Evaluate the base exactly once, even when every field is overridden. Then evaluate override expressions
once each in their written order. Preserve all unmentioned fields and construct a new immutable value.
Reject unknown or repeated explicit fields, a later/multiple base, two-dot spelling and incompatible values.
The spread does not merge arbitrary objects, spread lists, introduce mutation or accept variadic arguments.
Existing constructor forms and field requirements are unchanged. Trailing commas are not added.
An update without overrides, `Search { ...state }`, preserves every field. Empty Text, empty Lists and
None variant values remain present values, and an explicitly empty replacement still replaces its field.
An untyped empty object is not a base. Records still require 1..16 fields; no empty object/partial-record
merge or automatic skipping of missing/optional values is added.

Source elaboration infers/checks the target and expands the update into one ordinary fold iteration.
Its initial accumulator is the base. Its body binds override values in source order and constructs the
complete record from those bindings and accumulator fields. Compiler-generated names use deterministic
per-function ordinals, remain unavailable to source authors and do not depend on whitespace or byte offsets.
This expansion defines fuel/allocation behavior; it need not consume the same fuel as a handwritten full
constructor. Ordinary budgets and generic/type/layout/expansion checks still apply. No performance gain is
claimed. Native backends and the evaluator execute the same existing checked operations.

The temporary `base` field on the source AST is removed before checked validation. Checked deserialization
rejects an explicitly supplied record `base` field, including null, under every admitted language version.
No new checked expression kind is introduced. Diagnostics on invalid overrides retain the original value
range rather than an internal generated name.

## Standalone try statements

```lobo
try begin(db);
try execute(db, statement, parameters);
commit(db)
```

A block can interleave let bindings and standalone expressions whose root operation is prefix try.
A semicolon discards the successful payload; Err returns from the current function under the existing
[Core 1.8 rules](core-v1.8.md). The operand is evaluated once. Error payload types still match exactly,
postconditions still run, actions remain checked and no error conversion or automatic cleanup is added.
The entire try expression can be parenthesized. The existing contextual `try(value)` function call remains
a normal call, not propagation.

Blocks still end with a value expression. A semicolon after the last try does not create an implicit return
value. Ordinary discarded calls, compound expressions such as `try f()+1;` and discarded plain values remain
unsupported. Legacy source keeps its earlier grammar and diagnostic behavior. Direct try remains forbidden
in contracts, including when it appears inside a nested block.

Elaboration represents each statement as an unused compiler-generated ordinary binding containing the
existing Unary try node. Source checks cannot allocate or reference those reserved names. The success value
is computed normally and remains subject to the existing memory/fuel accounting.

## Compatibility and tooling

Current source reports core/1.10. Core 1.0 through 1.9 checked artifacts retain their prior admission rules,
including Core 1.9 SQLite support. A lowered program can use an older tag if all its checked operations
already satisfy that version; new source syntax itself is not admitted by an old compiler.

The VS Code extension recognizes three dots as one operator, navigates update bases/override fields and
infers generic fields from the base. It handles interleaved statements and original-range diagnostics.
The programming skill includes an executable example. A bounded migration covers maintained SQLite flows;
frozen fixtures and the original source-migration checkpoint remain unchanged.
See [decision 0058](../decisions/0058-record-updates-and-standalone-try.md).
