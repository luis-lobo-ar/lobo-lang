# Core v0.10: list-guarded recursive values

Status: implemented experimentally, 2026-11-04; specified before implementation.
Inherits [Core v0.9](core-v0.9.md).
Versions: compiler 0.10.0, `core/0.10-draft`, `lobo-checked/0.10`, native adapter 0.6.

Records and variants may refer recursively to nominal types if every dependency cycle passes
through a List constructor. Existing declaration identities and scope/import rules apply. For example:

```lobo
variant Node @id("example.ast.node") { Num(I64), Sum(List<Node>), Product(List<Node>) }
```

Removing all List edges must leave an acyclic graph. An inline field/payload cycle is still
`compile.record.cycle`, even if another field uses a list. Unknown list element types and malformed
checked-artifact scopes are rejected. Lists break physical layout dependence, not type identity.

Inline width remains one cell for a list handle, two for a buffer, the sum for a record, and a tag
plus the largest payload for a variant. Inline height stops at list indirection. Width <=64 and
inline height <=8 remain admission limits. List element types are checked separately; type-expression
spelling/validation remains bounded to eight nested List constructors. Logical recursive values may
be deeper internally than an inline layout. No source-level cyclic value can be constructed: immutable
construction refers only to already constructed finite values. Sharing remains unobservable.

Prepending a recursive value costs `3 + inline_width(element)` cells. Input charging follows the
actual finite value. Reclamation occurs on invocation termination in native code; the reference
evaluator must reclaim nested list chains without recursive host-stack growth. No new allocator or GC.

Equality retains structural semantics, short circuiting and existing per-list-element fuel charges.
At most 64 compound comparisons may be active. Attempting a 65th produces
`resource.stack.exhausted` in the active execution/contract phase. Scalars and buffers add no compound
comparison level. Flat list length is independent of comparison depth. Fuel can exhaust first.

Input and output logical value depth remains eight, with the root at depth zero. Scalars and empty
collections count as nodes. Excessive recursive CLI input is `resource.input.limit`; it must be detected
during validation before heap spending or execution. Excessive output is `resource.output.limit` before
printing a partial value. Other byte/node limits remain. Native argument wire spelling and JSON shapes
do not change; shapes may now refer to the same nominal type through lists. Old native executables
remain independent; recompile old checked artifacts rather than retagging them.

Required evidence includes recursive/mutual source and artifact checking, native/reference/sanitizer
agreement, malformed arguments, inline-cycle rejection, exact heap thresholds, equality/fuel/depth
faults, deep internal construction/reclamation, module linking, and maintained editor/skill behavior.
Passing these checks is not a mathematical proof of memory safety or an agent-quality comparison.
