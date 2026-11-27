# Lobo Core v0.6: persistent growable lists

Historical version, extended by the current language listed in the [specification index](README.md).
Version-specific resource/interchange rules below describe 0.6, not the current loader.

Status: implemented experimental subset. Inherits [Core v0.5](core-v0.5.md).
Version `core/0.6-draft`, checked artifact `lobo-checked/0.6`, native adapter 0.4.

## Operations and types

`List<T>` is a homogeneous immutable sequence, where T may be I64, Bool, a record, a variant, or another
list. `List` becomes a reserved built-in name. No user-defined type parameters, methods, mutation, implicit
conversions, random-access primitive, or new control construct is added.

```lobo
function range @id("example.range")(n: I64) -> List<I64> {
  fold(i in n, xs: List<I64> = List::empty<I64>()) { List::prepend(i, xs) }
}
```

- `List::empty<T>()`: the empty list with explicit element type; no allocation.
- `List::prepend(value, xs)`: new first element followed by xs, preserving xs unchanged.
- `List::length(xs)`: I64 length, constant-time.
- `List::head(xs)`: first element; empty input faults with `runtime.list.empty`.
- `List::tail(xs)`: all but first element, without allocation; empty input has the same fault.

Arguments evaluate left-to-right exactly once. Operations have exact arity/types; invalid use has
`compile.list.invalid` (or `compile.type.mismatch` for incompatible element types). The zero-argument
`empty<T>()` constructor has fixed grammar; malformed constructor punctuation/arguments are syntax errors.
List values compose with
functions/returns, contracts, matches, fields, bindings, conditionals, and folds. Copies may share storage;
identity is unobservable. Expected empty-list failures may be handled explicitly by testing length and
returning a user-declared variant. Arithmetic/contract/exhaustion faults remain diagnostics, not caught values.

Equality first compares lengths, then elements from first to last. Each visited equal-length element pair
costs one execution-fuel unit before recursively comparing payloads. Nested records compare fields in their
declaration order; variants compare their tag before the active payload. Scalar comparisons keep prior fuel
rules. This bounds otherwise unmetered list traversal and makes equality's first exhausted comparison explicit.

## Storage, lifetime, and bounds

Native values hold an immutable node pointer. Prepend allocates one node containing an inline copied element
and shared tail; head/tail/length and list copying are O(1), excluding element copy width. No deep tail copy.
Native nodes live until invocation termination and are freed on both success and structured failure. No
pointer escapes in JSON. The reference path may reclaim unreachable nodes earlier but uses the same logical
allocation accounting. Host malloc failure is `resource.memory.exhausted`.

`--heap-cells N` is a nonnegative u64 allocation allowance (default 1,000,000 cells) for native and reference
execution. Each node costs `3 + inline_width(T)` cells, including input nodes. Spending is cumulative with
no refund for unreachable values. An I64/Bool/list pointer has inline width 1; records sum fields; variants
cost one tag plus maximum payload width. Node cost bounds native payload storage in eight-byte cells; it is
not a bound on allocator metadata, total process memory, Rust reference overhead, or OS isolation. Exhaustion
is `resource.memory.exhausted`, not a smaller successful result. Prepend charges after evaluating its operands.
Contracts charge allocations/fuel normally but gain no external authority. No external action is added.

Input shape validation precedes heap spending and program execution. After all arguments validate, inputs
are allocated/accounted left-to-right, depth-first in declared field/selected payload/element order. Source
inline widths remain <=64 and per-function compound admission <=2,048 cells. Maximum nominal declarations
remains 32; type nesting/layout height is eight across records, variants, and list constructors. All nominal
dependency cycles, even through lists, are rejected in this slice: recursive named types need a later rule.
List length itself is dynamic (up to I64_MAX), constrained by supplied fuel/allocation allowances rather than
a fixed source array length. Type parsing and classification remain bounded before recursion.

Invocation values have at most 100,000 serialized value nodes across all arguments (`resource.input.limit`);
results have the same bound (`resource.output.limit`). Count one for each typed scalar/record/variant/list
value, recursively including fields, active payload, and elements; list links are not extra JSON value nodes.
Validation/preflight is bounded, follows the above traversal order, and completes before output framing, so
exhaustion never produces half a JSON result. Limits apply independently of heap cells and execution fuel.
Input/output-limit phases are invoke with no subject; program allocation faults retain function/contract phase.
The serialized result Value (excluding the outcome envelope) is additionally limited to 8 MiB of UTF-8 JSON.
The preflight includes declaration field names and element-type metadata, not only data nodes. Resource faults
inside contracts preserve the outer contract phase, including through pure helper calls.

## Interchange and invocation

Type: `{"List":T}`. Empty expression: `{kind:"List",element:T}`. Other operations:
`{kind:"ListOp",op:"prepend"|"length"|"head"|"tail",arguments:[Expr,...]}`. Nominal element references use
stable IDs; artifacts recheck types, arity, cycles, limits, actions, and literal restrictions. Do not inject
lists as Literal nodes. Old artifacts require source recompilation, not envelope retagging.

Runtime JSON: `{"type":"List","value":{"element":T,"elements":[Value,...]}}`. CLI roots use
`List:Count:N[,element tokens...]`; the parameter supplies element types/nominal IDs. Count is canonical
unsigned decimal <=I64_MAX, with no plus sign or leading zeroes except `0`. Nested lists use `Count:N` tokens
inside existing Record/Variant/list streams, followed by exactly N recursively flattened elements in list
order. Existing Record/Variant roots retain their full stable-ID prefixes, and scalar spellings are unchanged.

## Preimplementation acceptance

Empty and 3-element construction/traversal/serialization; list alias preservation; dynamic range and sum;
nested record/variant/list payloads; typed failures from guarded head; function/contract/fold composition;
exact fuel and heap thresholds including inputs/contracts; equality length short circuit and node metering;
malformed inputs/artifacts, unknown element types, nested generic closers, cycles, stale versions, large-list
drop/cleanup, bounded serialization with shared subvalues, native/reference/ASan/UBSan agreement and leak checks.
Editor highlighting/navigation must understand nested List types and built-ins without fabricating a source
definition for intrinsic operations; compiler squiggles must work on unsaved list programs.
