# Developing the Lobo compiler

Status: workflow definition in use for the first Rust Core implementation.

## Unit of work

Each change states a rule or defect, scope, acceptance examples, and evidence required. Keep changes small enough
that an agent can explain exactly which program behavior changed and why. Language evolution, implementation
correction, and optimization are distinct intentions, even when they touch the same code.

Update the [change ledger](../CHANGELOG.md) for meaningful changes, linking the governing specification,
decision record, compatibility impact, and verification evidence. Preserve earlier entries; new decisions
supersede older decisions explicitly. Record planned behavior separately from implemented behavior.

## Specification and TDD loop

```text
rule + examples → conformance fixture → observed failure → implementation
                                                         │
accept evidence ← full required checks ← focused checks ──┘
```

1. Define the rule with a stable section and specify boundaries and errors.
2. Add valid, invalid, and boundary examples with expected outcomes independent of implementation internals.
3. Confirm that new tests fail because the behavior is missing or wrong. Record unsupported harness behavior
   separately while the harness is being built.
4. Implement the smallest complete behavior across parsing, resolution, checking, evaluation, and code generation
   as applicable. Unsupported backends must report their support honestly.
5. Run focused conformance and regression cases. Inspect failure evidence, then repair the implementation or
   explicitly revise the specification and expectations when the design itself is wrong.
6. Refactor without changing observable behavior; rerun the checks affected by the refactor.
7. Run required integration checks and record results against the exact revision and environment.

For example, implementing checked addition requires normal addition, both overflow directions, precedence,
operand rejection, and behavior when the expression occurs in an unevaluated branch. One test for `1 + 2` does
not establish the arithmetic rule.

## Editor support is part of language evolution

Every language expansion includes maintenance of the [VS Code extension](../editors/vscode/README.md) in the
same change, not a later cleanup. Review syntax highlighting, declaration/reference navigation, and language
configuration against new syntax, types, scope rules, and executable examples. Add editor regressions alongside
compiler fixtures; cover valid names, ambiguous/contextual tokens, incomplete edits, and out-of-scope references
as applicable. Preserve support for inherited syntax and document any intentional limitations.

Run `npm --prefix editors/vscode test` for language/compiler changes. Extension runtime or navigation changes
also require the documented real VS Code host checks. Rebuild/version the VSIX when extension behavior changes,
and keep installation instructions and advertised support current. Installation into a user's normal profile
and Marketplace publication remain separate actions, not automatic consequences of running tests.

The compiler now supplies [source diagnostic spans](../spec/source-diagnostics-v0.1.md), consumed by editor
squiggles and available to agents. Navigation still uses a lightweight structural resolver. When compiler-owned
symbol/type queries become available, use them for navigation instead of allowing independent parsers to evolve
inconsistent meanings. Neither those queries nor a language server are currently implemented.

## Agent context

Language 1.6 binds JSON literal builders by pinned stable IDs from explicitly supplied sources. Keep their
reserved aliases internal, dependencies inspectable and normal linking/type/action budgets intact. Compare
checked programs after normalizing only helper alias labels and import order. Exercise missing helpers,
collisions, empty objects, native escaping and original-range editor errors. See [the contract](../spec/core-v1.6.md).

Language 1.5 propagates expected nominal types through constructor members and value contexts. Infer a bare
case owner only from that expected identity; preserve local/import precedence. Compare with explicit source,
test ambiguous/absent context and retain source-only artifact admission. See [the constructor contract](../spec/core-v1.5.md).

Language 1.4 elaborates modern inferred calls and contextual named callbacks after module resolution.
Keep that pass bounded and source-only. Compare against explicit spelling before specialization and native
execution. Loaded checked artifacts retain concrete validation. Preserve parametric checks for unused bodies,
purity rejection and exact-source diagnostics. See [the 1.4 contract](../spec/core-v1.4.md).

Managed source uses tracked `.lobo.ids.json` metadata. Include its map in explicit source snapshots and
evidence identities. Checks never repair or allocate IDs. Authoring commands validate before writing.
Test renames, missing/stale state, unsaved metadata and publication failures against
[the 1.3 contract](../spec/core-v1.3.md). Moving source requires moving its sidecar.

An agent needs the relevant specification sections, diagnostic definitions, existing fixtures, affected compiler
stages, implementation entrypoints, and current failure output. It also needs the advertised language/backend
versions so it does not mistake illustrative future syntax for supported behavior.

Record unresolved design assumptions in the task. Do not resolve ambiguity by copying a host-language default
such as wrapping arithmetic, truthiness, or exception handling into Lobo.

## Oracle review

When implementation and tests disagree, identify the governing rule first. Expected-value snapshots cannot be
updated solely because the current compiler emits a different result. Review changes to assertions and golden
outputs separately from the implementation change, with a rationale tied to the specification.

## Integration evidence

The eventual runner should report structured results with:

- compiler source revision and artifact identity;
- language, fixture, backend, and toolchain versions;
- executed case IDs, seeds, and input artifacts;
- pass, fail, unsupported, skipped, and infrastructure-failure counts;
- mismatch details and reduced reproducers;
- timing and declared resource budgets;
- evidence links and unresolved limitations.

`cargo test --locked` runs the current compiler tests; `scripts/check.sh` validates documentation artifacts.
The future runner should expose focused selection and full required suites through both a CLI and the agent
protocol. A failed test is data for the agent; the host owns execution and acceptance policy.

## First implementation sequence

1. Define the checked IR and graph mapping for Core.
2. Build a fixture loader that distinguishes syntax errors, type errors, values, and runtime faults.
3. Implement source parsing, name resolution, and type checking in Rust.
4. Implement the reference evaluator with required runtime contracts and checked arithmetic.
5. Run the Core corpus end to end, including rejection cases.
6. Define the native invocation adapter and implement C/Clang lowering.
7. Compare evaluator and native outcomes and retain every disagreement as a minimized regression.
8. Build workspace operations around the working semantic representation.

See [testing and TDD](testing-and-tdd.md) for acceptance gates, and
[self-hosting](self-hosting.md) for the later bootstrap extension.


## Native interface 1.7 verification

The [native interface](../spec/native-interface-v0.1.md) has failing-first ABI/artifact/resource/TCP tests,
C and LLVM native routes, ASan/UBSan, a fault-injected C provider harness, a fixed-seed handle model and
exhaustive four-bit delegation masks. [Upstream attribution](../tests/upstream/native/README.md) distinguishes
adapted cases from complete upstream suites. Run `python3 scripts/check-native-upstream.py` and the full
Cargo suite. OS calls intentionally have no reference evaluator fallback. The two native backends share their
C provider, so their agreement does not independently validate that provider.

The [result record](../benchmarks/results/2026-11-22-native-boundary.md) retains red/green attempts, corrections,
commands and scope limits. Editor native-type/signature/payload navigation, unsaved diagnostics and repair
are covered by Node and the real isolated VS Code host. No IO is executed by compiler diagnostics.

## Native state-model gate

Compiler 1.7.1 adds source-linked finite authority/IO TLA+ models and concrete C witnesses. See
[the model contract](../verification/native/README.md) and apply the local lobo-native-verification skill
when changing covered code. The one-command checker requires a pinned installed TLC jar, completed baseline
exploration, enabled-action coverage and expected rejection of isolated model/C mutations. It refuses stale
source mappings. Ordinary Cargo runs the C witnesses without requiring TLC. The model found a real
fuel-fault cleanup gap before connection-handle publication; its failing-before and corrected C runs are
retained in [the result](../benchmarks/results/2026-11-22-native-formal-verification.md).
These checks establish neither C refinement nor unbounded correctness.

## Maintain current source conventions

For a source-only modernization, run the checked audit with a fresh output directory:

```sh
node --test scripts/test-modernize-source.mjs
node scripts/modernize-source.cjs --audit --out target/source-audit
```

The audit reads maintained libraries, executable examples and programming-skill examples. It supplies
closed source bundles by stable IDs and accepts a candidate only when its resolved checked program agrees.
Use --apply with another fresh output directory to write accepted candidates. Review result reports,
including rejected candidates. These flags do not convert matches to try automatically: review error types,
resource cleanup and error precedence, then exercise native behavior. No implicit error conversion exists.
The Rust suite also checks that maintained source has no remaining accepted simplifications.

Keep frozen fixtures and explicit-syntax compatibility tests. Never refresh
`scripts/source-migration.json` to accept a difference. See
[decision 0053](../decisions/0053-maintained-core-1.8-source-modernization.md).
