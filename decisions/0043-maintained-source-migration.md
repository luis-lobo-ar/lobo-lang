# 0043 Maintained libraries and examples use modern source

Date 2026-11-14. Status implemented as a source-only migration.

## Scope and preserved inputs

The user selected migration of maintained libraries and examples to native 1.2 syntax while preserving stable
IDs, TDD, editor support, documentation and frozen research fixtures.

The migration covers the HTTP and JSON libraries and 21 executable example files. It also covers the three
embedded candidate snapshots in `examples/testing/agent-session.jsonl`. Existing modern MVC and skill examples
already use the selected syntax. The [manifest](../scripts/source-migration.json) records exact target paths,
original source hashes, declaration/import identities, dependency closures and checked-program fingerprints.

The frozen `examples/v1` acceptance corpus, legacy `examples/mvc.lobo` comparison, illustrative transfer sketch,
illustrative protocol exchange, versioned specifications, previous reports and release artifacts remain unchanged.
This task does not remove deprecated source parsing or add language features.

## Mechanical migration and TDD

The [codemod](../scripts/migrate-syntax.cjs) moves explicit identities into trailing tables, changes declarations
to `fn`, groups selected imports and adds arrow syntax to existing match blocks. It preserves imported aliases
and block contents. Import groups use an otherwise-unused name, avoiding collisions with bindings and actions.
Original comments and quoted source text remain data.

Before changing maintained files, capture compiled each original dependency closure and pinned the complete
checked `program` hash. Application validates every transformed program before replacing any maintained source.
This checks bodies, identities, scopes, imports, contracts, actions and specialization output together.
The three embedded session programs have separate fingerprints. Original source bytes remain in local evidence.

The unit tests fail before the codemod. The corpus and Rust integration test fail while maintained sources
still use legacy headers. They pass after migration. A scope regression rejects frozen paths and traversal.
Capture refuses to overwrite the baseline. Verification is repeatable without file writes.

```sh
node --test scripts/test-migrate-syntax.mjs
node scripts/migrate-syntax.cjs --verify --compiler target/debug/lobo
cargo test --locked --test source_migration
```

The manifest is a pinned migration checkpoint. A later intended semantic change needs an independently justified
specification/oracle update. Regenerating a baseline to hide a failing behavior comparison is not acceptance.
The tool is a local mechanical rewrite, not an atomic workspace transaction.

## Research driver correction

The repair-study driver constructed historical fixtures by reading live examples and applying literal legacy
mutations. Migration exposed that coupling through three benchmark test errors. The driver now reads
[fixed source copies](../benchmarks/fixtures/repair-study/README.md), whose bytes match the pre-migration inputs.
Only that input path changes. Tasks, mutations, grading, requirements and archived experiment packages remain
unchanged. Checksums for the copies are pinned in the migration manifest and checked by its verifier.

This routing correction changes one driver file. It does not rewrite frozen fixtures. The original driver is
preserved in the local evidence directory. Other surveyed historical source/report/artifact files remain
byte-identical. Python bytecode caches are generated files and excluded from that integrity assertion.

## Tooling and application workflow

Editor fixtures target the new declaration positions. A corpus test checks every migrated selected import
against its exact owning declaration. Legacy fixture tests remain available. Extension maintenance package
1.2.1 changes documentation/fixtures and packaging metadata, with no editor runtime or grammar change.

The maintained web builder selects the current 1.2 compiler rather than the frozen 1.0 compiler. Explicit
`--compiler` and `--out-dir` options support reproducible builds and tests without replacing old output artifacts.
The native Rust HTTP host and its behavior remain unchanged.

The compiler and language remain 1.2.0/core/1.2. Current library/README examples and programming-skill references
use modern source. The [result record](../benchmarks/results/2026-11-14-source-migration.md) retains test outcomes,
negative attempts, checksums and limitations. No agent productivity or performance comparison is claimed.
