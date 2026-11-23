# Native tools and test evidence

Compiler 1.8 parses modern source directly through ordinary `check`, `compile`, `run` and bundle commands.
The [native MVC guide](../../../../docs/compact-syntax-prototype.md) gives commands and explicit HTTP/JSON
dependencies. No projection or marker is required. Preserve metadata IDs when renaming source bindings.
Compiler 1.8 also accepts argument-based generic call inference and contextual named callbacks.
`compile.generic.inference` means that inputs did not determine every parameter. Supply explicit type
arguments; do not guess types, relax the expected result or change a frozen requirement.

Source files without an inline identity table use a tracked adjacent `.lobo.ids.json` file. Initialize
new source with `lobo identities init FILE --source DEPENDENCY`; `sync` adds new declarations while
preserving existing IDs. These commands validate before writing metadata. Checks/builds stay read-only.
`identities rename FILE OLD NEW` retargets metadata after a source rename, without editing source text.
Stale keys fail. Move sidecars with source files and restore lost IDs from version control.

Maintained libraries and examples use modern 1.2 syntax. The repository's
[migration record](../../../../decisions/0043-maintained-source-migration.md) identifies historical exceptions
and the repeatable checked-program audit. Frozen study fixtures and acceptance programs retain their exact
source. A newer spelling is not permission to rewrite those research inputs.

For an HTTP application, see the [native web-server sample](../../../../examples/web-server/README.md).
That older Rust host owns HTTP networking and calls compiled Lobo through the typed CLI. Compiler 1.7 also
provides [bounded native loopback operations](../../../../spec/native-interface-v0.1.md) without that host.
Use only catalog operations; action labels do not grant runtime authority.
The [pure HTTP helper module](../../../../libraries/http/README.md) supplies typed responses, prefix matching,
method checks and JSON string quoting. Add its source explicitly; use quoting for dynamic JSON values and
remember that `json_document` accepts an already-serialized document, not a record or JSON parser input.

The [typed JSON helpers](../../../../libraries/json/README.md) provide Json values, immutable object/array
builders and `stringify` / `json_response`. Add both `libraries/json/json.lobo` and `libraries/http/http.lobo`
as explicit sources. Modern `json { "text": value }` accepts Text expressions, requires selected `Json`,
and binds its object/field helpers internally by stable ID. Select `stringify` or `json_response` when called.
Generated dependencies are inspectable in checked imports; ordinary unselected names remain unresolved.
The [JSON example](json-example.lobo) runs with those same two explicit sources.
For explicit construction, wrap strings with `Json::String`, integers with `Json::Integer`, decimal text with
`Json::Number`, and builders with `Json::Object` / `Json::Array`. Field names must be unique. Decimal text
is grammar-validated before emission and preserves precision. The MVC view is a small executable example.

Use the compiler path supplied by the project. In this repository build both native tools with:

```sh
cargo build --locked --release --bin lobo --bin lobo-test
```

For an ordinary program:

```sh
lobo check program.lobo --source helpers.lobo
lobo run program.lobo --source helpers.lobo --entry app.entry --arg I64:7
lobo compile program.lobo --source helpers.lobo -o fresh-output
./fresh-output --entry app.entry --arg I64:7
```

`compile` refuses to overwrite an existing output path. `run` compiles natively through trusted
Clang; generated executables run independently. `eval` selects the reference evaluator explicitly
for correctness comparisons. It is not the application execution path. Do not install a compiler
or Clang without task authorization. No interpreter, daemon, package resolver or FFI is implied.

Compiler 0.11.1 offers an explicit direct LLVM route: `lobo compile program.lobo --backend llvm -o fresh`
or `lobo run program.lobo --backend llvm --entry app.entry`. It requires installed `clang-18` and
`llvm-link-18`. `lobo compile program.lobo --emit llvm -o fresh.ll` invokes those tools and emits linked
LLVM text; `opt-18 -passes=verify -disable-output fresh.ll` verifies it. Keep backend identity with evidence.
The default remains C/Clang; unavailable LLVM tools are errors, not permission to install or silently switch.
The JSONL verification-session prototype retains its existing configured C backend; no LLVM session/cache
selector is implied. Both native routes obey the same source semantics and executable invocation contract.

Arguments are typed atoms: `I64:7`, `Bool:true`, `Text:<UTF-8 hex>`, `Bytes:<hex>` and
`List:Count:3,I64:1,I64:2,I64:3`. Record fields follow declaration order:
`Record:app.pair:I64:1,I64:2`. A variant payload starts with its nominal ID and case,
for example `Variant:app.answer:Case:Found,I64:7`. Results are typed JSON; I64 values are decimal
strings. Sources and budgets must accompany a reproducible result.

Language 1.0 also accepts pure function atoms `Function:stable.id` with the exact expected signature, and
`F64:1.5` / `F64:0x3ff8000000000000`. Function JSON includes ID/signature; F64 JSON carries sixteen hex bit
digits. NaN/infinity require bit atoms. Preserve signed zero and nonfinite bit values when inspecting results.

The no-IO checker accepts a complete source envelope through `lobo check --bundle-stdin`:

```json
{"format":"lobo-source/0.1","root":"main","sources":[{"name":"main","text":"module m; fn f() -> I64 { 7 } identities {module:\"m\",f:\"f\"}"}]}
```

For managed source use `"format":"lobo-source/0.2"` and attach `"identities":{"module":"m","f":"f"}`
to the source object instead of putting the table inside `text`. Metadata counts toward bundle bounds.
The compiler performs no filesystem access for this envelope. A sidecar path inside a label is never loaded.

Labels are diagnostic identities, not filesystem paths. It validates the entire supplied source
set. It does not compile native code, execute the program or resolve imports from disk.

When a host supplies `check.py` or another project test client, use that contract instead of
inventing a runner. If it supports `--list`, `--detail TEST_ID` or `--replay TEST_ID`, use those
to retrieve missing input/observation information. Do not assume every project exposes these flags.

The raw `lobo-test --session --clang /absolute/path/to/clang` protocol is `lobo-test/0.1` JSONL.
Open fixes requirements, tests, expected outcomes and budgets; verification submits complete
candidate sources with an explicit base snapshot. `all` forces execution; `affected` may reuse
eligible prior passes under the same host-asserted environment. It does not prove environment
isolation or selector soundness. Eight snapshots include open; limit exhaustion does not evict.
Compiler errors create no accepted snapshot. Failed/pending checks do not authorize acceptance.
Explain/replay identify retained source/check observations; replay can reproduce a failure.
Exit 0 means clean transport. Inspect `checks_satisfied`, counts, pending/uncovered requirements
and any required `full_execution` gate. EOF/close ends the foreground session and owns cleanup.

Canonical contracts: [native tools](../../../../spec/native-v0.1.md),
[source bundles](../../../../spec/core-v0.9.md),
[agent testing](../../../../spec/agent-testing-v0.1.md).

Compiler 1.8 reports compile.try.operand, compile.try.output, compile.try.error or compile.try.contract
for propagation errors. Read the original-source range. Preserve the function's result and exact error type;
use an explicit match if the program needs to recover, convert an error or close resources before returning.

For SQLite, compile with the SQL source supplied, then invoke with --sqlite-file ABSOLUTE_PATH and
--sqlite-mode read|write|schema. The native run shortcut does not forward these grants. The persistent
REST source list and build command are in [its guide](../../../../examples/native/sqlite-rest/README.md).
Inspect both transport status and typed SqlResult errors. File/role grants and query quotas are explicit.
No database access is performed by compiler checks or editor diagnostics.
