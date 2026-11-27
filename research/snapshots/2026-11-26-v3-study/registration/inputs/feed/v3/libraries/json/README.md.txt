# Typed JSON helpers

Use `libraries/json/json.lobo` with language 1.6 and the existing pure HTTP helper source.
The [MVC example](../../examples/mvc.compact.lobo) builds typed values in its view and serializes them at the controller's
HTTP response boundary. Both sources are explicit compiler inputs.

```sh
target/release/lobo run examples/mvc.compact.lobo \
  --source libraries/json/json.lobo \
  --source libraries/http/http.lobo \
  --entry example.mvc.show_user --arg I64:1
```

The response has status 200, content type application/json and body `{"name":"Luis"}`.
The same sources appear in the workspace's VS Code configuration for definition navigation.

## Values and builders

Select `Json` in a `use` group and bind its metadata key to `lobo.json.value`. Its cases are `Null`, `Boolean(Bool)`, `Integer(I64)`, `Number(Text)`,
`String(Text)`, `Array(JsonArray)` and `Object(JsonObject)`. These are typed variant constructors.

| Function and stable ID | Result |
| --- | --- |
| `object()` / `lobo.json.object` | Empty JsonObject |
| `field(document, name, value)` / `lobo.json.field` | New JsonObject with a unique named Json value appended |
| `array()` / `lobo.json.array` | Empty JsonArray |
| `append(items, value)` / `lobo.json.append` | New JsonArray with a Json value appended |
| `stringify(value)` / `lobo.json.stringify` | Compact JSON Text |
| `is_number(text)` / `lobo.json.is_number` | Whether the complete text follows JSON's decimal number grammar |
| `json_response(status, value)` / `lobo.json.response` | Typed HTTP Response with serialized JSON body |

JsonObject, JsonArray and JsonMember have stable IDs `lobo.json.object_type`, `lobo.json.array_type` and
`lobo.json.member`. Field/array builders return immutable values. Appending to one value preserves its base.
Serialized fields and array elements follow insertion order. Internally, builders prepend to reversed lists.
Direct record construction must follow that representation. Duplicate object names fail a precondition in
both the field builder and object serialization. Key comparison uses exact Text equality.

In language 1.6, literals need only the explicit `Json` type and any functions you call:

```lobo
use json { Json, json_response };
```

The compiler binds `object` and `field` automatically to their stable IDs in the supplied source closure.
These generated dependencies appear in checked module imports. User functions with those names cannot
redirect a literal. Empty literals need no field helper. Missing helpers, incompatible types and undeclared
actions fail normal checks. Explicit builder imports remain available for direct calls and older source.
See [language 1.6](../../spec/core-v1.6.md). The executable view is:

```lobo
fn render_user(user: User) -> Json {
  json { "name": user.name }
}
```

A controller can pass the result to `json_response(200, render_user(user))`. Existing HTTP helpers admit
statuses 200, 404 and 405. Other statuses fail the response adapter's precondition.
The MVC's companion identity file supplies the stable `render_user` identity. Inline tables remain supported.
The shorthand admits Text fields. Other JSON values use the typed constructors and builders above.

## Strings and numbers

String values and object names use the existing HTTP helper's JSON string quoting. It escapes quotes,
backslashes and ASCII controls and preserves UTF-8. `Json::String(text)` always represents a string value.
`Json::Number(text)` represents a decimal token and is checked before emission. Its grammar permits an
optional minus sign, an integer, an optional fraction and an optional signed exponent. It rejects leading
zeros, whitespace, empty fractions/exponents and NaN/Infinity spellings. These rules follow
[RFC 8259 section 6](https://www.rfc-editor.org/rfc/rfc8259.html#section-6).

Number text preserves its supplied precision and exponent. For example, `Json::Number("1.25e-3")` emits
`1.25e-3`. Receiving libraries can impose their own numeric range/precision limits. `Json::Integer(n)` emits
the full I64 range exactly, including -9223372036854775808. Decimal text provides fractional values in this
version. F64-to-decimal conversion and parsing external JSON remain future work.

## Execution and verification

Serialization uses an explicit pending-work list, collects text chunks and flattens them once. Nesting
consumes the language's ordinary fuel/heap budgets. It avoids recursive function calls per nested value.
Object uniqueness checks can perform quadratic comparisons in the number of fields. Language text/output
and external value-admission bounds continue to apply. Large values can exhaust budgets with classified
resource errors. The HTTP host retains its own body limits.

Run `cargo test --locked --test json_helpers`. Tests reload checked artifacts, independently decode emitted
JSON and cover evaluator/C/LLVM plus both native sanitizer routes, integer/decimal boundaries, duplicate
keys, deep arrays, resource faults, typed responses and the migrated MVC example.

[Decision 0039](../../decisions/0039-typed-json-source-helpers.md) records the API, duplicate-key policy and
dependency on the existing quoting helper. [HTTP helper documentation](../http/README.md) describes the
raw serialized-document adapter and its response policy.
