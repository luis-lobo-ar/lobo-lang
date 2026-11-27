# Lobo Core v0.7: byte sequences and UTF-8 text

Historical extension, inherited by the current language in the [specification index](README.md).
Status: implemented experimental subset. Inherits [Core v0.6](core-v0.6.md).
Versions: `core/0.7-draft`, `lobo-checked/0.7`, compiler 0.7.0, native adapter 0.5.

## Values and syntax

`Bytes` and `Text` are reserved built-in types. Bytes hold arbitrary octets; Text holds valid UTF-8.
Both are immutable, homogeneous buffer values with no observable identity, implicit conversion, locale,
normalization, or NUL termination. They compose with records, variants, lists, folds, functions, and contracts.
Text equality compares encoded bytes, not a locale-dependent collation or normalization equivalence.
Unicode scalar indexing is not grapheme-cluster indexing. The byte-sequence syntax follows
[RFC 3629 section 4](https://www.rfc-editor.org/rfc/rfc3629#section-4); no Unicode property tables are required.
U+FEFF is preserved as data, not stripped as an implicit BOM.

Quoted expressions are Text literals, using JSON string syntax: raw UTF-8, escaped quotes/backslashes,
`\n`, `\r`, `\t`, `\b`, `\f`, `\/`, and `\uXXXX` including paired surrogates. Reject malformed escapes,
unpaired surrogates, and raw controls/newlines (`compile.syntax.invalid`). Stable `@id` strings still require
unescaped valid ASCII identities; allowing general Text literals does not relax declaration-ID rules.
Each checked Text literal is at most 1 MiB decoded UTF-8 (`resource.frontend.limit`). Literals are static
values with no invocation heap charge. Bytes are constructed through operations, not injected literal nodes.

## Explicit operations

All operands evaluate once left-to-right; arity/receiver errors use `compile.buffer.invalid`, incompatible
argument types use `compile.type.mismatch`. Operations are expressions, not methods or external actions.

| Operation | Result and behavior |
| --- | --- |
| `Bytes::empty()` | Empty Bytes, no allocation |
| `Bytes::length(xs)` | I64 byte length, O(1) |
| `Bytes::at(xs, i)` | I64 octet 0..255; negative/out-of-range index faults `runtime.bytes.bounds` |
| `Bytes::slice(xs, start, count)` | Shared byte view; invalid nonnegative range faults `runtime.bytes.bounds` |
| `Bytes::push(xs, byte)` | New bytes followed by octet; byte outside 0..255 faults `runtime.bytes.range` |
| `Bytes::concat(a, b)` | New concatenation, preserving operands |
| `Bytes::from_list(xs)` | Convert List<I64> in list order; any nonoctet faults `runtime.bytes.range` |
| `Text::bytes(t)` | Shared UTF-8 Bytes view, no validation/allocation |
| `Text::valid(xs)` | Bool: is Bytes a valid UTF-8 sequence? |
| `Text::from_bytes(xs)` | Shared validated Text view; otherwise `runtime.text.invalid_utf8` |
| `Text::length(t)` | I64 Unicode scalar count |
| `Text::at(t, i)` | I64 scalar value; invalid scalar index faults `runtime.text.bounds` |
| `Text::slice(t, start, count)` | Shared view of scalar range; invalid range faults `runtime.text.bounds` |
| `Text::concat(a, b)` | New concatenated Text |

Slices allow start == length only for count zero. Check nonnegative bounds and use subtraction rather than
overflowing start+count. Empty slices remain valid values. Expected failures may be handled by explicit
guards and user-declared variants; faults are not implicitly caught. General regex, formatting, Unicode
normalization/properties, mutation, file/network access, and host libraries remain out of scope.

## Cost, lifetime, and bounds

Native values are pointer/length descriptors; reference values use shared immutable storage/views. Descriptor
inline width is two cells, including inside records/variants/list elements. Heap buffers share the invocation
arena/cleanup introduced in v0.6. Copies, views, and Text-to-bytes conversion do not allocate. A nonempty
new/input buffer of N bytes costs `3 + ceil(N/8)` heap cells; empty buffers cost zero. Input occurrences are
accounted independently even if host reference values share storage. Retention lasts until invocation end
in native mode; no refund, GC, visible pointer, or external authority is introduced.

In addition to ordinary expression ticks:

- Equality checks lengths first; equal-length operands cost one fuel unit per byte pair examined, stopping
  at the first difference. Different-length buffers cost no traversal fuel.
- Concat/push charge total result byte length in fuel before allocation/copy. Validate push range and sum
  length <= I64_MAX first. Exhausted/overflowing allocation is `resource.memory.exhausted`.
- From-list charges its entire list length before validating octets, then allocates/copies once.
- Text valid/from-bytes/length charge full input byte length before scanning (even if invalid data fails early).
- Text at/slice reject negative indices/counts first, then charge full byte length and resolve scalar bounds.
- Byte indexing/slicing and byte length need no traversal fuel beyond ordinary expression/operand ticks.

All input shape/UTF-8 validation precedes heap spending and execution fuel, including nested buffers. Inputs
are limited to 8 MiB total decoded buffer bytes in addition to 100,000 value nodes (`resource.input.limit`).
Returned buffers count as one typed value node; the existing exact 8 MiB serialized Value preflight includes
JSON escapes/hex encoding and runs before result framing. Limits do not constrain total process memory,
allocator overhead, OS OOM behavior, or provide an OS sandbox. Contract phases/subjects remain attributable.

## Interchange and CLI

Types serialize as `"Bytes"`/`"Text"`. Runtime values: `{"type":"Bytes","value":"00ff"}` (lowercase hex)
and `{"type":"Text","value":"🐺\n"}` (JSON text). Checked Text literals use the existing Literal node;
buffer operations use `{kind:"BufferOp",family:"Bytes"|"Text",op:string,arguments:[Expr,...]}`. Artifact
reload rechecks operation/type/arity, literal limits, and stable nominal references. Bytes/other compound
values cannot be injected as Literal nodes. Old artifacts require source recompilation, not retagging.

CLI atoms/root arguments use `Bytes:<hex>` or `Text:<hex>`; Text hex encodes UTF-8. Empty hex is valid; odd,
nonhex, or invalid UTF-8 Text is `invocation.arguments.invalid`. Accept either hex case, emit lowercase Bytes.
The same atomic tokens work in existing record/variant/list streams without ambiguity from commas/NULs.
Source literals remain readable; CLI hex keeps argument transport independent of shell/JSON escaping.

## Preimplementation acceptance

Pin raw/escaped/non-ASCII/NUL literals, surrogate pairs and invalid forms; byte/scalar length and indexing;
boundary/empty/shared slices; immutable concat/push and bulk list conversion; strict UTF-8 boundary/invalid
classes; equality's exact fuel; allocation/copy/validation budgets; typed guarded conversion; nested values;
source/artifact/argument rejection; exact JSON escaping/output limits; sanitizer/leak-checked native/reference
agreement and editor highlighting/navigation/unsaved diagnostics. The arithmetic parser workload remains
ordinary Lobo code, not a new host parsing intrinsic. The full language-first goal remains active.
