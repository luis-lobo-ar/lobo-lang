# Checked artifact v0.1

Status: experimental compiler output format, implemented by the Rust reference.

`lobo compile --emit checked` emits a JSON artifact with extension `.loboc`. It is a resolved, type-checked expression tree for
the reference evaluator. Native executables are the default compiler output. This is an intermediate
representation, not the final canonical workspace graph.
The language, artifact, and tool protocol have independent versions.

## Envelope and declarations

The top-level object contains exactly `format`, `language`, and `program`:

```json
{
  "format": "lobo-checked/0.1",
  "language": "core/0.1-draft",
  "program": {
    "module": "demo",
    "id": "demo",
    "functions": [
      {
        "name": "main",
        "id": "main",
        "parameters": [],
        "output": "I64",
        "requires": [],
        "ensures": [],
        "body": {
          "bindings": [],
          "value": {"kind": "Literal", "value": {"type": "I64", "value": "42"}}
        }
      }
    ]
  }
}
```

Parameters are `{ "name": identifier, "ty": type }`. Types are the strings `I64` and `Bool`.
A binding is `{ "name": identifier, "annotation": type-or-null, "value": expression }`.
A block contains an ordered `bindings` array and one final expression named `value`.
Declarations, parameters, bindings, arguments, and contracts retain source order.

## Expression variants

Every expression has a `kind` and exactly the fields listed for that variant:

| Kind | Fields |
| --- | --- |
| `Literal` | `value`: typed value |
| `Name` | `name`: resolved lexical name |
| `Unary` | `op`: `-` or `!`; `operand`: expression |
| `Binary` | `op`: Core binary operator; `left`, `right`: expressions |
| `Call` | `function`: stable function ID; `arguments`: ordered expressions |
| `If` | `condition`: expression; `then_block`, `else_block`: blocks |

Function calls target IDs, never display names or source positions. Local names remain lexical bindings for
this first format; they will need explicit node identity in the future workspace graph.
Runtime values follow the Core interchange rule: I64 is a canonical decimal string, Bool a JSON boolean.
For example, `{"type":"Bool","value":true}` is valid; integer JSON numbers are invalid I64 values.

## Validation and determinism

Readers reject unsupported format/language versions and unknown fields, deserialize using bounded input,
and recheck identities, scopes, operators, arity, and types before execution. A producer's claim that an artifact
is checked is not authority to skip validation. `.loboc` is data, never executable host instructions.

The Rust producer emits pretty JSON in the declared structure/variant field order and preserves sequence order.
There is no timestamp, absolute path, or whitespace/comment source metadata in the artifact. Recompiling identical
source, or changing only source whitespace/comments, must reproduce identical artifact bytes with the pinned
compiler. Arbitrary readers need not preserve incoming JSON formatting.

The current producer uses Serde and serde_json, pinned in `Cargo.lock`. This format is not a specified
content-hash encoding and does not establish canonical workspace revision identity.

## Reference limits

The reference frontend limits source to 1 MiB, tokens to 8,192, and expression nesting to 32. The loader limits
artifact input to 4 MiB and uses a bounded JSON nesting parser. Calls have a depth limit of 32; evaluator
expression frames have a depth limit of 256. Each function entry and evaluated expression consumes one unit of
fuel. The CLI defaults to 100,000 fuel and permits an explicit `--fuel` value, including zero.

These limits can yield `resource_exhausted` for an otherwise valid program. The first runner is an in-process
pure evaluator; it has no live external capability imports and is not an OS isolation mechanism.
WASM imports, memory layout, ABI, and external capabilities are separately pending definitions.

