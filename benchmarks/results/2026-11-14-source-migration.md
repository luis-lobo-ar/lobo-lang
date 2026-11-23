# Maintained-source migration results

Date 2026-11-14 in America/Chicago. Status engineering acceptance passed.

## Migration and equivalence

Migrated 23 maintained `.lobo` files, comprising HTTP/JSON libraries and 21 executable examples. Three
embedded candidate programs in the testing-session example also use modern syntax. The migration preserves
157 module/declaration identities and 18 selected import bindings. Existing modern MVC and skill examples
already used the selected spelling.

Before editing source, the migration tool captured each original source and compiled dependency closure.
All 23 complete checked-program fingerprints and all three embedded program fingerprints match after
migration. These comparisons cover bodies, identities, imports, scopes, contracts, actions and lowered
specializations. The compiler's 22 input files are unchanged from the native-syntax checkpoint.

The [codemod](../../scripts/migrate-syntax.cjs) is idempotent on modern source. It validates every proposed
checked program before replacing maintained files. Scope checks exclude frozen paths and traversal.
The [manifest](../../scripts/source-migration.json) pins source identities, hashes and explicit closures.
A new Rust integration test repeats the no-write audit. See [decision 0043](../../decisions/0043-maintained-source-migration.md).

## Frozen inputs and the research driver

The frozen `examples/v1` acceptance corpus, old MVC comparison, transfer/protocol sketches, versioned specs,
past result files and frozen release artifacts retain their bytes. An integrity survey verifies 293 existing
historical source/report/artifact files. Fourteen generated Python bytecode-cache paths are excluded.

One repair-study driver had a hidden dependency on live examples. Its fixture builder applied literal old
import mutations to those files, causing three benchmark-test errors after migration. Its only change is
the input directory. Three [fixed source copies](../fixtures/repair-study/README.md) are byte-identical to the
pre-migration examples. Their hashes are checked by the migration verifier. No task, fault, grading rule or
archived experiment package is changed. The original driver remains in the before-source evidence.

## Verification

- Full Rust suite passes 207 tests, including the new corpus fingerprint audit.
- Editor suite passes 142 tests, including exact navigation for all 18 migrated import bindings.
- The real VS Code host passes after fixture positions are updated.
- Python benchmark suite passes 31 tests against fixed historical study inputs.
- Four codemod tests pass, including ID/string/comment preservation, nested matches, scope and malformed input.
- All six web-server checks pass, including C/LLVM handler routes and real loopback HTTP behavior.
- The migrated testing session returns its expected seven responses. Full verification passes two tests.
  The intentionally faulty increment fails one test, reuses the unchanged constant result and reproduces its failure.
- Clippy with warnings denied, formatting, documentation links, skill validation and VSIX packaging pass.

The maintained web builder now selects compiler 1.2 and supports explicit compiler/output paths.
Both default selection and an explicit compiler build succeed into fresh target directories. Existing
`dist/web-server` output is not replaced during verification. The native Rust host code is unchanged.

Extension package `dist/lobo-syntax-1.2.1.vsix` updates maintenance documentation, fixtures and version metadata.
Its runtime implementation and grammar remain unchanged. It is packaged locally, not installed into a normal
user profile. Current library examples, web instructions, README and skill references use modern source.

## Negative checks and corrections

The initial pass-through codemod fails all three unit cases. The corpus verifier and new Rust integration
test fail before source migration because maintained headers still use the old spelling. A later scope test
fails before the explicit write-scope helper is added. All pass after their respective changes.

The first editor suite has six failures because file-based target searches still expect `name @id`.
Navigation already returns the correct new positions. Updating those exact declaration anchors retains the
target and owner assertions. The first real-host run finds additional parser/tree anchors with the same issue.
The repeated host run passes without editor runtime changes.

The first full Rust run stops at the AST negative fixture. Its literal replacement no longer removes the
Product arm after the arrow is inserted. The correction changes only that search literal and adds an assertion
that the mutation happened. The expected `compile.match.invalid` result is unchanged.

The first web-builder command rejects the missing compiler/output options. The maintained workflow is then
updated and verified through native builds and loopback tests. The benchmark suite initially reports three
fixture-drift errors, corrected by the frozen input-directory binding described above.

Logs and commands are retained in the [receipt](2026-11-14-source-migration.json). Original source and the
original driver are under `benchmarks/private-evidence/source-migration-20261114/before`.

## Limits

This is source/workflow migration, not new language semantics. Compiler and language remain 1.2.0/core/1.2.
Deprecated parsing is still required by frozen fixtures and compatibility tests. Removal remains a separate
decision. The codemod validates before writing but is not an atomic semantic-workspace transaction.

No agent construction, comprehension, repair-cost, token-efficiency or runtime-performance comparison ran.
Those effects remain unknown. Exact checked-program preservation and passing tests do not establish an
agent productivity gain. The preservation archive is local and ignored, not an off-device backup or publication.
