# Working on Lobo

Lobo is an early research project. It has a Rust Core native compiler and a reference evaluator.
`compile` and `run` use native code by default. Clang is required for compilation and native tests.
The daemon, WASM backend, and self-hosting are not implemented. Core syntax is proposed and experimental.

## Start here

Read, in order:

1. `README.md`
2. `VISION.md`
3. `docs/architecture.md`
4. the document relevant to the task
5. applicable records in `decisions/`

Treat the semantic workspace and agent workflow as the product. Do not optimize syntax, bytecode, or runtime
performance ahead of evidence that the core workflow works.

## Change rules

- Preserve the distinction between canonical state and projections.
- Never introduce ambient authority, implicit moving revisions, or evidence detached from its inputs.
- Protocol operations must declare state, effects, capability needs, budgets, and failure behavior.
- Examples must be labeled illustrative until executable fixtures exist.
- Record durable architectural choices or reversals in `decisions/`.
- Update cross-references when terminology changes.
- Do not add a dependency, custom runtime, package system, or concurrency model without a measured need and an
  explicit decision record.

## Verification

Run:

```sh
cargo test --locked
./scripts/check.sh
```

Cargo runs the Core conformance corpus, behavior/artifact checks, native/reference comparisons, and CLI tests. The script validates
JSON syntax, each JSONL message, and local Markdown links. Review expected results against the specification;
do not edit an oracle just to make an implementation pass.

