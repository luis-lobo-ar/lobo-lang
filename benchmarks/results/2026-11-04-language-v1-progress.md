# Language 1.0 implementation record

Status: active, not a release. Scope: decision 0033 and the frozen language-v1 acceptance specification.
Baseline receipt: [language-v1-baseline.json](2026-11-04-language-v1-baseline.json).

## Generic data

- Added source/workload tests first. `CARGO_INCREMENTAL=0 cargo test --locked --test generic_data`
  failed both tests at the parser, as expected; raw log `target/language-v1-generic-data-red.log`.
  Corrected the new fixture's while separator from comma to the existing semicolon syntax.
- First implementation run failed: generated nominal names included the identity separator `.` and violated
  identifier rules. Raw log `target/language-v1-generic-data-first.log`. Corrected generated source names;
  stable type identities retain their separators.
- Second run passed both tests, including evaluator, C, LLVM and both sanitized native routes through
  `tests/support`. Raw log `target/language-v1-generic-data-second.log`. This is a focused result only;
  the full regression/editor/skill/release gates remain pending.
- Source `Applied` types lower to concrete nominal declarations. Requests are cached before following
  fields, allowing List-guarded self recursion without host recursion. Function/type requests have separate
  identity domains and admission budgets. No backend runtime or Cargo dependency was added.

These tests are compiler development fixtures, not independent agent productivity/comprehension evidence.
Every failed attempt remains in the raw logs. Later final receipts must preserve these logs outside the
regenerable build cache.

## Definition checking and function values

- Definition fixtures initially had Rust function-declaration typos; that build failure is retained as
  `target/language-v1-definitions-red.log`, not counted as behavioral red evidence. Correcting those typos
  produced the expected failures for unused invalid bodies and nominal declarations in
  `target/language-v1-definitions-red-corrected.log`. The symbolic checker then passed all three tests in
  `target/language-v1-definitions-first.log` (also repeats the generic-data tests).
- The old 64-instance regression failed after the specified limit increase; raw log
  `target/language-v1-data-regression.log`. Updated the historical boundary assertion and replaced the
  previous instance-only body acceptance expectations with the new parametric rejection rules. All seven
  generic-function tests passed in `target/language-v1-generic-regression-updated.log`.
- Added exact 256/257 admission coverage. It exposed an off-by-one caused by processing ordinary bodies in
  the symbolic pass (`target/language-v1-instance-boundary-red.log`). The correction checks ordinary
  signatures there, then checks ordinary bodies in the concrete pass. All four definition tests passed
  in `target/language-v1-instance-boundary-first.log`. Symbolic and concrete requests retain separate budgets.
- Function-value source tests failed before implementation (`target/language-v1-function-values-red.log`).
  The first build exposed an exhaustive Value match in the test-session adapter. The next behavioral run
  exposed the native argument prefix bug. Logs `target/language-v1-function-values-{check,check-second,first,second}.log`
  retain these failures. After corrections, `third.log` passed two tests; `boundaries.log` passed three,
  including direct field invocation, forged artifacts/arguments, invalid IDs/signatures and a stable-ID/local-name
  collision. Successful execution cases include C, LLVM and both sanitized routes plus the evaluator.
- Native references store a closed function index and dispatch to compiled function bodies. No captures,
  interpreter, custom runtime, external authority or dependency was introduced. Fn has a checked empty action
  bound; actionful references are rejected. Source/checked compatibility is now explicitly Core/format 0.12,
  compiler 0.12.0-dev. This is not release 1.0.

## Editor, skill and full regression attempts

- Generic-data navigation test failed before resolver changes (`target/language-v1-editor-data-red.log`);
  the local and bundle navigation suites then passed 52 tests (`editor-data-first.log`).
- Named-reference/callback navigation test failed before changes (`editor-function-red.log`); local/bundle
  navigation then passed 53 (`editor-function-first.log`). Generic-data highlighting already passed the new
  fixture on the existing grammar (`grammar-data-red.log` is a passing baseline, despite its filename).
  An unnecessary trial of separate generic declaration rules was removed; regression coverage remains.
- The first full Rust run found the old metadata-slot limit assertion (`language-v1-first-full.log`), which
  now tests 16,385 slots against the specified 16,384 bound. The module suite subsequently passed 17 tests
  (`language-v1-module-bounds.log`). Subsequent full builds found exhaustive scalar-corpus matches needing
  the new Function variant (`language-v1-second-full.log`, `language-v1-third-full.log`). Those scalar
  generators still intentionally generate only I64/Bool; function values have their own differential tests.
- The first full editor run failed because the new example lacked a contract required by the fixture
  (`language-v1-editor-first-full.log`). Added a meaningful output-length contract. The full suite then passed
  119 tests (`language-v1-editor-second-full.log`).
- The fourth full Rust run passed before the subsequent exact-boundary correction; log
  `target/language-v1-fourth-full.log`. A fresh fifth full run is pending as of this entry.
- Real VS Code host passed using the installed executable and a fresh test profile, including new generic
  constructor, generic field and function-reference navigation assertions (`language-v1-editor-host-first.log`).
  `./scripts/check.sh` passed (`language-v1-doc-check-first.log`); the skill validator reported `Skill is valid!`.
  VSIX packaging and native execution of the expanded skill examples are still pending as of this entry.
- A few manual patch attempts failed their context/format validation and made no filesystem changes;
  corrected patches were applied. These were editing failures, not passing or failing program behavior.

Remaining goal: F64, the three frozen acceptance programs and changes, further adversarial/admission coverage,
final full checks and a local optimized language 1.0 release with reproducible evidence. No agent study or
performance result is inferred from these conformance checks.

### First three families checkpoint

The fifth complete Rust run passed 172 tests (`language-v1-fifth-full.log`), including native differential
and active sanitizer probes. It includes the corrected 256/257 boundary. The extension package was produced
as `dist/lobo-syntax-0.14.0.vsix` (12 packaged files, command reports 27.7 KB); raw packaging output is
`language-v1-editor-package-first.log`. The successful editor suite contains 119 tests. Native skill/example
executions are recorded separately in `language-v1-skill-native.json`; their status must be read from that
receipt rather than assumed from this text. Intermediate edit states were not independently archived for
every failed run; original/final source checkpoints and raw logs are retained, so those intermediate source
identities are unknown. This limitation does not apply to the final release evidence gate.

### Additional adversarial correction

Review while preparing F64 exposed a missing artifact boundary: Function values had not been added to the
existing rejection of constructor-only literal kinds. A forged Function literal could pass source-IR checking
with an unvalidated identity. A new artifact test demonstrated the failure (`language-v1-function-literal-red.log`),
then the literal admission rule was corrected. This supersedes any interpretation of the earlier checkpoint
as exhaustive artifact safety evidence. The retained first-three receipt remains unchanged as historical
evidence; the final release must include this correction and repeat the full gates.

F64's new behavioral fixtures fail at the parser/type boundary before implementation
(`target/language-v1-floats-red.log`: two failures, one existing rejection test passes).

## F64 and fixed program acceptance

- Corrected Function-literal admission then passed all three function-value tests (`language-v1-function-literal-fixed.log`).
- F64's first full focused run passed three tests across evaluator/C/LLVM and both sanitized native routes
  (`language-v1-floats-first.log`). Fixed bit oracles cover arithmetic, nonfinite results, IEEE equality inside
  lists, signed zero, classification and checked integer conversion. An additional artifact literal test failed
  before rejecting nonfinite checked literals (`language-v1-float-artifact-red.log`) and passed after the correction
  (`language-v1-float-artifact-fixed.log`). Five focused tests then passed in `language-v1-floats-boundaries.log`.
- Both native routes and the reference evaluator reject altered up/down/toward-zero rounding and flush/denormal
  modes. Two adversarial profile tests passed (`language-v1-float-profile.log`); only test shims link libm or change
  a test thread's control register, and restore the register. Production adds no dependency or ambient mode change.
- Float/editor fixtures initially failed (`language-v1-editor-float-red.log`). The new navigation fixture also
  lacked its function @id; after correcting that fixture, 84 local navigation/grammar tests passed
  (`language-v1-editor-float-second.log`). Grammar's initial failure is valid before-code evidence; the malformed
  navigation fixture's failure alone does not establish a behavioral regression. Full editor suite passed 121
  (`language-v1-editor-float-full.log`); real F64 editor-host assertions were added for the next final run.
- Full compiler suite with all four families passed 179 Rust tests (`language-v1-float-full.log`), including
  both native/sanitized backends and the seven float tests. The preceding no-run build succeeded
  (`language-v1-floats-full-build.log`; its shell's final rg exited 1 because no errors matched, not a build failure).
  Documentation/link checks passed (`language-v1-float-doc-check.log`).
- Public acceptance programs were introduced with valid typed placeholder implementations and frozen output
  assertions. Collection and planned-change tests failed behaviorally (`language-v1-collections-red.log`), then
  passed all five execution routes after implementation (`language-v1-collections-first.log`), including the
  10,000-element case under the original budgets. Documents/schema-change tests likewise failed before their
  implementation (`language-v1-documents-red.log`) and passed afterward (`language-v1-documents-first.log`).
  Fractional aggregation/negative-rule tests failed before implementation (`language-v1-numerical-red.log`).
  All three program tests, covering six baseline/change configurations, passed in `language-v1-acceptance-first.log`.
  Actual fixtures and source/entry mapping: `examples/v1/README.md`; independent fixed assertions: `tests/v1_acceptance.rs`.
- Exact source/instance admission tests exposed two issues (`language-v1-data-admission-red.log`): generic nominal
  annotations did not split adjacent `>=` as List annotations do, and expansion had relaxed the previous 32-source
  nominal limit. Unified angle closing and restored source admission. All four generic-data tests pass, including
  the exact 128/129 nominal instance boundary (`language-v1-data-admission-fixed.log`). Source/checked limits remain
  distinct, allowing 32 source nominals plus 128 concrete instances. Editor parsing follows the same angle rule.
- `cargo clippy --locked --all-targets -- -D warnings` passed (`language-v1-clippy-first.log`). Its trailing rg found
  no error lines and exited 1; the Cargo log records successful completion. Final checks must include the later
  acceptance files and version changes.

All implementation families and focused public acceptance are now implemented and passing. Final whole-repository
checks, expanded skill native examples, real F64 editor-host checks, optimized local packaging, final versioning
and immutable release receipts remain pending. The goal is still active; no release or agent-outcome claim yet.

## Prerelease gates

- Expanded guarded generic recursion to a real parent/child value and added six artifact forgery checks.
  The full prerelease Rust run passed 185 tests (`language-v1-prerelease-rust.log`). The editor suite passed
  122 (`language-v1-prerelease-editor.log`), and the actual VS Code host passed the new F64 and previous checks
  (`language-v1-prerelease-host.log`). Documentation/link checking and skill validation passed
  (`language-v1-prerelease-docs.log`); six native example executions passed, with output JSON in
  `language-v1-prerelease-skill.json`.
- The 30-test Python run had one error (`language-v1-prerelease-python.log`): its skill check invokes
  `target/release/lobo`, which was still the preserved 0.11.1 build and could not parse the expanded skill.
  Debug/native fixtures already used the matching compiler. The stale compiler's machine-readable help
  is retained as `language-v1-stale-release-identification.json`; a matching optimized development build
  is required before retrying the unchanged Python expectations. This is a tool-version prerequisite,
  not evidence that the Python suite passed. Release assembly will build its compiler before these gates.
- Removed a misleading reflexive-equality postcondition from the generic identity skill example: F64 NaN
  deliberately violates reflexive language equality. The reference now explains that a typed generic contract
  is not a proof for all values. Benchmark oracles and preserved skill packages remain unchanged.

## Final language 1.0 release

The matching optimized development compiler passed the unchanged 30 Python tests on retry
(`language-v1-prerelease-python-retry.log`), and all seven optimized float tests passed
(`language-v1-prerelease-release-floats.log`). Prerelease Clippy passed again. Version identities then changed
to compiler/extension 1.0.0, `core/1.0` and `lobo-checked/1.0`; old artifacts require recompilation.

The [release builder](../../scripts/release.py) completed all 64 final commands successfully in
`dist/lobo-1.0.0-linux-x86_64`, including fresh 185-test Rust / 30-test Python / 122-test editor runs,
seven optimized float tests, real host, lint/format/docs/skill gates, six native examples, both tool-discovery
contracts, six checked/reference programs and twelve optimized native binaries/runs. Its manifest reports
`passed`, with unchanged input hashes throughout. Final driver output is `target/language-v1-final-release-run.log`.

The separate native JSONL F64/function-input integration passed (`language-v1-final-tool-session.json`).
Post-build audit reread all 86 artifact and 215 source/archive-member hashes, then ran the collection binaries
from `/tmp` (`language-v1-final-artifact-audit.json`). No size/speed thresholds were fitted to results.

Disk space briefly dropped below 1 GiB during the version rebuild. Removed only 28 older regenerable Rust
test-cache executables (730,357,696 bytes), identified from the pre-release run and older than the final build.
The removal catalog preserves each path/size/hash (`language-v1-cache-cleanup.json`). Source, release binaries,
logs and preserved compiler baselines remain intact.

The [final report](2026-11-04-language-v1-release.md) and [receipt](2026-11-04-language-v1-release.json) close
this bounded language goal. No additional feature work, ecosystem expansion, paid study or publication follows.
