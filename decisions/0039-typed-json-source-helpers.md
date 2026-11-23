# 0039 Typed JSON source helpers

Date 2026-11-14. The user requested a JSON helper after the MVC example exposed manual JSON concatenation.

Implement a pure Lobo source module with typed Json values, immutable object/array builders, a serializer
and a typed HTTP response adapter. Json supports null, Boolean, I64, validated decimal-number text, Text,
arrays and objects. Object field names are unique. Adding a duplicate key fails a precondition, including
when duplicate members reach serialization through direct record construction. Builders preserve insertion
order through private-convention reversed lists. Public record fields remain inspectable source values.

The number-text validator follows the decimal grammar in [RFC 8259 section 6](https://www.rfc-editor.org/rfc/rfc8259.html#section-6).
Its text representation preserves precision and exponents through serialization. I64 formatting covers
the minimum negative value without negating it. Floating conversion and JSON parsing can be selected in
later work. The [object rules](https://www.rfc-editor.org/rfc/rfc8259.html#section-4) recommend unique names
and describe receiver differences for duplicates. Rejecting duplicates is this library's explicit policy.

Reuse `lobo.http.json_string` for keys and strings, and `lobo.http.json_document` for response construction.
Supplying both helper source modules remains explicit. Existing HTTP source users retain their supplied
source contract. Moving the existing quoting implementation would require an additional source for those
users, so the initial JSON module imports the existing pure helper. A future layering change needs a
compatibility decision.

Serialization uses an explicit work list and text chunks, then flattens the chunks once. It avoids recursive
function calls per nesting level and repeated copying of a growing document. Language fuel, heap and output
bounds govern execution. Object uniqueness checks can scan field pairs and consume quadratic work in the
number of members. These are algorithm choices, with speed and agent outcomes still unmeasured.

Acceptance covers independent JSON decoding, escaped keys/values, controls/UTF-8, full I64 boundaries,
decimal validation, insertion order, immutable builders, duplicate rejection, nested values, resource faults,
typed responses and MVC integration. Check evaluator, C, LLVM and both native sanitizer routes after artifact
reload. Add editor navigation fixtures and explicit workspace sources, update skill/library documentation,
and retain failed attempts and verification evidence. Compiler syntax and checked formats follow language 1.1.
