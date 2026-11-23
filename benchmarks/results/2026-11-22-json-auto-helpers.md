# Automatic JSON literal builder dependencies

Date 2026-11-22, America/Chicago. Engineering acceptance passed for compiler/language 1.6 and extension 1.6.0.
The user selected automatic JSON builders with TDD, editor maintenance, documentation and the decision ledger.
The [contract](../../spec/core-v1.6.md) and [decision](../../decisions/0047-json-literal-helper-dependencies.md) define scope.

## Verified behavior

The maintained generic MVC selects `use json { Json, json_response };` and writes
`json { "name": user.name }`. The compiler binds object/field builders by stable ID from explicitly supplied
sources. Dependencies appear in checked module imports under reserved aliases when no correct explicit
builder selection exists. Empty objects add only the object dependency. Ordinary unselected builder names
stay unresolved, and unrelated local/imported functions cannot redirect literals. Normal type, action,
contract and resource checks still apply. Json and directly called functions remain explicit.

The MVC retires exactly two source binding keys, `json.object` and `json.field`. All eleven retained identity
values are unchanged, including every declaration ID. Comparing retained compiler 1.5 against release 1.6
preserves the complete checked program after normalizing only helper alias labels and import ordering.
Artifact format is unchanged; language tags differ. This is not raw artifact byte equality.
Native C/LLVM invocations return 200 and `{"name":"Luis"}` for user 1, and 404 with `Not found` plus newline
for user 42. Four independent skill examples run natively, including JSON escaping and UTF-8.

Final verification passes 229 Rust tests, 155 editor tests, 31 Python benchmark-tool tests, four codemod tests
and six HTTP tests. Rust JSON regressions include evaluator, C, LLVM and both sanitizer routes, artifact reload,
closed missing targets, empty-object minimal dependencies, name collisions, source-name rejection, action
checks and original-range value errors. Real VS Code host checks cover field and imported Json definitions,
an unsaved invalid JSON value, its source range and repair. Formatting, Clippy, skill validation and local
Markdown/JSON checks pass. The extension is packaged locally as `dist/lobo-syntax-1.6.0.vsix`; it is not
installed into a normal profile or published. The [receipt](2026-11-22-json-auto-helpers.json) pins evidence.

## Negative attempts and corrections

All three initial compiler regressions fail before implementation: literals still require selected builders,
and the missing-builder case fails too early as syntax instead of reaching closed module linking.
Both initial editor regressions fail on the same mandatory-import rule. The first implementations pass
these focused suites. Later coverage adds a compatibility-export alias collision without adding a test count.

The first full Rust run stops on one existing expected diagnostic. An explicitly selected `wrong.field`
now fails normal module linking, because literal lowering independently selects the canonical builder.
Update that specific expected code from `compile.syntax.invalid` to `compile.module.invalid`; retain the
rejection and the independent wrong/missing target tests. The final full suite passes. Stale patch contexts
made no accepted mutation; the source archive and final diff identify the actual changes.

The first release smoke run passes MVC equality and native examples, then fails when compiling the optional
legacy export of the inferred generic MVC (`compile.generic.arguments`). That exporter predates generic-call
and constructor inference and does not elaborate those forms. Preserve the failure and document the limit.
The final smoke run checks JSON helper export using the independent nongeneric skill example; it passes.
Editor regressions also compile exported JSON source and a colliding ordinary export alias. Native modern
source needs no export and continues to pass. No expansion of legacy inference support was attempted.

## Preservation and limits

Integrity checks retain 57 selected before-input hashes, 292 surveyed historical files, all 19 prior paper
entries and 86 artifacts in the frozen 1.0 release. Both dependency locks differ only in root package versions.
The explicit compact/legacy MVC comparisons, prior specifications, migration fingerprints, old reports and
research fixtures remain unchanged. Checks/builds do not mutate source or identity metadata.

Local preservation is `benchmarks/private-evidence/json-auto-helpers-20261122`, including before/current
inputs and source, compilers, VSIX packages, native outputs, repeatable smoke/integrity drivers, logs and
hash manifests. The smoke driver accepts a fresh `--out-dir` and runs from the repository root.
This ignored local archive is not an external backup, Git commit or publication.

This change verifies language/tool behavior and simplifies author imports. No agent construction-quality,
comprehension, total-token or runtime-speed comparison ran. Those outcomes remain unknown. Overlapping
regression counts and repeated verification attempts are not independent research observations.
