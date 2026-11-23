# Local generic call inference

Date 2026-11-22, America/Chicago. Engineering acceptance passed for compiler/language 1.4 and extension 1.4.0.
The user selected the simpler MVC call `respond_json(find_user(id), render_user)` with TDD and synchronized tooling.
The [decision](../../decisions/0045-local-call-inference.md) and [contract](../../spec/core-v1.4.md) define scope and limits.

## Implemented and verified behavior

A bounded source pass infers omitted generic function arguments from known argument types and monomorphic
callback signatures. It converts named functions to references where an exact Fn signature is expected.
The existing parametric checker, specialization and concrete validation run afterward. Checked artifacts
remain concrete; native backends and the runtime require no new construct.

The [generic MVC](../../examples/mvc.generic.lobo) uses the requested controller spelling. Its tracked IDs
are assigned once by native authoring tools. The simpler MVC and frozen legacy comparison remain unchanged.
Under the current compiler, inferred and explicit source produce exactly equal complete checked artifacts.
Using the retained 1.3 compiler for the explicit spelling and release 1.4 for the inferred spelling also
produces identical checked programs and artifact format; only the language tag differs.

Seven compiler regression tests cover checked-code equivalence, native/sanitizer results, imported aliases,
source-order stability, generic bodies, expected callback specialization, conflicts, unresolved types,
actionful references, exact source ranges, nesting and synthesized-signature growth. C/LLVM release MVC
binaries return the expected 200 and 404 responses. Native skill examples preserve their expected outputs.
Builds and execution leave the example source and sidecar hashes unchanged.

Final verification passes 219 Rust tests, 148 editor tests, 31 Python benchmark checks, four codemod tests
and six native HTTP checks. Real VS Code host checks cover inferred calls, contextual callback navigation
and an unsaved incompatible-argument edit followed by repair. Clippy with warnings denied, formatting,
skill validation and documentation links pass. The 1.4.0 VSIX is built locally, without normal-profile
installation or publication. The [receipt](2026-11-22-call-inference.json) retains commands, inputs and logs.

## Negative outcomes and corrections

The first four compiler regressions fail before implementation. The first source pass passes those cases.
A new failing nesting regression exposes repeated analysis of already elaborated nested arguments. The pass
now revisits only arguments needing contextual elaboration. That regression then passes. Self-review adds
pre-copy type bounds, with shallow and excessive function-signature growth probes. The MVC and growth probes
are additional acceptance coverage, not claims of failing-first implementation evidence.

The two initial editor regressions fail before navigation understands inferred calls. The next attempt
exposes two mistakes in the new test fixture: a field assertion targets the base variable's offset, and
the library repeats the app's entry ID. Correcting those offsets/IDs preserves the intended field and owner
assertions. Both cases pass, followed by a file-based generic MVC navigation case and the real host checks.

The first full Rust suite fails because two test fixtures choose an already-existing timestamp-based
scratch-directory name. A process-local atomic counter now supplements the timestamp and process ID.
The next full run passes before the final type-bound regression; the final run passes all 219 tests.
No frozen test oracle is recaptured. Interrupted patch applications that matched no current source lines
produce no accepted behavior claim. The retained logs separate red, intermediate and final attempts.

## Scope and limitations

This is argument-based inference for modern callers. Deprecated legacy source retains its explicit rules.
Explicit type applications and references remain useful. Constructor/signature inference, return-context
inference, captures and a global constraint solver are outside scope. A generic callback needs a complete
expected signature; the solver does not jointly determine outer unknowns from a partially known generic callback.
These boundaries and stable rejection diagnostics are documented rather than silently guessed.

Expression visits, nesting and pre-copy type growth are bounded. Purity, rigid parameters and artifact
validation retain their prior checks. No runtime performance, agent construction-quality, comprehension or
token-efficiency experiment ran. Such effects remain unknown. Exact lowered-code equality does not establish
an agent productivity or runtime-speed advantage.

The local ignored preservation directory is `benchmarks/private-evidence/call-inference-20261122`.
It contains the before source/compiler/VSIX, current inputs, exact observations, native binaries, logs,
paper index and integrity manifest. Historical reports, versioned specifications, frozen fixtures and the
1.0 release remain intact. Local preservation is not an off-device backup, Git commit or publication.
