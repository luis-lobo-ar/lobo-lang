# Checked artifact v0.2

Historical format, extended by the current format in the [specification index](README.md). The 0.2 compiler emits
`lobo-checked/0.2` with language `core/0.2-draft`. The structure and validation rules
of [v0.1](checked-artifact-v0.1.md) are inherited, with the version strings changed and one expression variant:

| Kind | Required fields |
| --- | --- |
| `Fold` | `index`: identifier; `accumulator`: parameter-shaped name/type; `count`, `initial`: expressions; `body`: block |

For example, `fold (i in n, total: I64 = 0) { total + i }` has index `i`, accumulator
`{"name":"total","ty":"I64"}`, count Name(n), initial Literal(I64:0), and a block whose final value is
Binary(+, Name(total), Name(i)). All fields are mandatory; unknown fields are rejected. Calls within the count,
initializer, and body resolve to stable IDs, just as other expressions do.

Loaders validate [fold scope and type rules](core-v0.2.md) regardless of count or reachability. A claimed checked
artifact does not bypass these checks. Existing input/depth limits and deterministic serialization rules remain.
The 0.2 reader accepts only the 0.2 format/language pair; older artifacts must be rebuilt from source, not
silently retagged. Older compiler readers reject the new mandatory format/node. Neither version is canonical
workspace graph storage. Native executables do not embed this artifact or the evaluator.
