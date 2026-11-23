# Compact MVC syntax prototype results

Date 2026-11-14 in the user's America/Chicago timezone. Status engineering acceptance passed.
This is a public development fixture, separate from confirmatory agent evaluation.

## Implemented result

The [compact MVC source](../../examples/mvc.compact.lobo) has executable `fn` declarations, grouped selected
imports, qualified calls, arrow match arms and a Text-valued JSON object. Stable declaration IDs occupy a
trailing table. The [original source](../../examples/mvc.lobo) remains the canonical comparison fixture.
Supplying both representations in one bundle duplicates IDs by design.

The [shared projector](../../editors/vscode/src/compact.cjs) produces canonical source and original-span tokens.
The [CLI](../../scripts/compact-syntax.cjs) reads explicit sources and can preserve original and generated
snapshots in a new output directory. The extension uses the same projector for navigation and mapped diagnostics.
The Rust compiler remains version 1.1.0 and requires generated source. Both execution routes emit native code.

The checked artifact comparison asserts exact serialized equality between both MVC representations.
That assertion includes stable IDs, scopes, expressions, contracts and actions. Artifact reload and independent
expected 200 and 404 responses pass through reference, C, direct LLVM and both native sanitizer routes.
Fuel and heap exhaustion remain classified resource failures under the existing 10,000 invocation budgets.

The standalone CLI demonstration retains inputs, generated source and native executables in
`target/mvc-compact-20261114`. Both native routes return status 200 and `{"name":"Luis"}` for user 1, and
status 404 with `Not found` plus a newline for user 42. The configured editor bundle uses 11 explicit sources
and selects compact MVC rather than its duplicate canonical representation.

## Verification

- `cargo test --locked` passes 201 tests.
- `npm --prefix editors/vscode test` passes 138 tests, including six projection/CLI cases and compact grammar coverage.
- `python3 -m unittest discover -s benchmarks -p 'test_*.py'` passes 31 tests.
- `cargo clippy --locked --all-targets -- -D warnings` passes.
- `cargo fmt --check` passes.
- The real VS Code host passes local and explicit-project compact navigation, original-source squiggles and unsaved repairs.
- Extension 1.1.1 packages with the shared projector included. It is not installed into the user's normal profile.
- Skill validation and executable canonical/compact skill examples pass.
- Explicit CLI source generation, native execution and the configured editor source snapshot pass.

Commands, exit codes, source identities, preserved logs and integrity checks are recorded in the
[machine-readable receipt](2026-11-14-compact-syntax.json).

## Failed attempts and corrections

The first new test could not load the absent projector module. A pass-through scaffold then failed the
expected compact-to-canonical function assertion, establishing a behavioral red result before implementation.
The initial grammar test also failed before compact grammar support.

The first implemented editor run passed three of four cases. Its CRLF test expected line 9 for the `false`
token on zero-based line 8. Counting the fixture lines independently justified correcting that assertion.
The source bytes and diagnostic token remained unchanged.

The first native run passed exact checked-artifact equality but the test read Response fields from the wrong
JSON level. The existing result schema nests record fields under `value.value.fields`. Correcting that test
path retained the independent expected 200 and 404 responses. Its native reproducer is preserved.

The first compact grammar implementation correctly colored an imported `not_found` as an import. The test
incorrectly required every occurrence, including that import, to have function-call coloring. Separate import
and call assertions corrected the oracle.

The first packaging attempt passed all 138 editor tests but failed because new README links pointed outside
the VSIX without a published repository URL. The extension README now names those checkout files as code paths.
The package succeeds without inventing a remote repository or publication URL.

All failed logs remain in the preservation directory alongside passing reruns.

## Interpretation and remaining limits

This prototype demonstrates a source convention with equivalent checked behavior and synchronized tooling.
Moving IDs to a table changes their presentation. It does not remove identity-management work.
Automatic ID assignment, rename refactoring, a formatter and general grammar adoption remain future decisions.
The prototype admits concrete declarations and Text-valued JSON shorthand. Other JSON types retain explicit
constructors. The [prototype reference](../../spec/compact-source-v0.1.md) records the grammar and bounds.

No controlled agent construction, repair, comprehension or token-efficiency study ran for this change.
Those outcomes are unknown. Conformance tests, shorter expressions and preserved behavior do not establish
an agent productivity advantage. No runtime speed or compile-time comparison is claimed.

The local preservation directory is `benchmarks/private-evidence/compact-syntax-20261114`.
It is an ignored local archive, not an off-device backup or published research artifact.
