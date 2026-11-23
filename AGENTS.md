# Working on Lobo

Lobo is an early research project. It has a Rust Core native compiler and a reference evaluator.
`compile` and `run` use native code by default. Clang is required for compilation and native tests.
The generated native suite additionally requires Clang's ASan/UBSan runtimes; missing tools are failures, not skips.
The optional direct LLVM backend and expanded differential suite require installed `clang-18`, `llvm-link-18`
and (for explicit verifier checks) `opt-18`. No automatic tool installation or fallback is permitted.
The daemon, WASM backend, and self-hosting are not implemented. Core syntax is proposed and experimental.

## Start here

Read, in order:

1. `README.md`
2. `VISION.md`
3. `docs/architecture.md`
4. the document relevant to the task
5. applicable records in `decisions/`

The long-term product is the semantic workspace and agent workflow. The current user-selected priority is
general-purpose language expressiveness before ecosystem/workspace expansion; follow
`docs/language-completeness.md` and decision 0014. Prefer a small composable core over construct parity.
New constructs need demonstrated patterns; keep native compilation, TDD, and editor support throughout.
The 2026-10-18 language-only audit records local milestone acceptance; this does not authorize automatic
ecosystem/workspace expansion. Select the next concrete target with the user and retain all stated limits.
The user-authorized 2026-11-04 evolution adds List-guarded recursive data in Core 0.10 and explicit
generic function specialization in Core 0.11, driven by public AST/reuse workloads. Generic bodies are
checked per requested instance; generic nominal types, direct inline cycles and function values remain absent.
Retain the preserved Core 0.9/0.10 baselines and keep development workloads separate from evaluation holdouts.

Current scope correction: Core 0.11/compiler 0.11.1 is the language evaluation checkpoint. Follow
`decisions/0032-stop-feature-growth-at-the-evaluation-checkpoint.md`. Stop proactive feature additions;
the generic-data continuation proposal is deferred. Work on existing-specification bugs, review, study
design and reproducibility within that checkpoint. Record missing facilities as findings, not automatic
implementation tasks. Only a user-selected bounded milestone reopens feature scope; no experiment or
ecosystem expansion is authorized merely by reaching this checkpoint.
`docs/future-language-scope.md` is a requested backlog with a closed four-family candidate milestone and
completion gates. It is planning only; it does not override decision 0032 or authorize implementation.

Subsequent explicit authorization: the user requested implementing the four-family scope through a local
language version 1.0 and made it the active goal. Follow decision 0033 and spec/language-v1-acceptance.md.
This reopens only that bounded milestone; the earlier Core 0.11 evidence stays frozen. Do not add unrelated
language/ecosystem work or label the release complete before its fixed feature/program/tooling gates pass.

Compiler 1.0.0 / language 1.0 now implements those four feature families. Follow `spec/core-v1.0.md`;
it supersedes Core 0.11's instance-only checking and absent generic-data/function-value rules. The frozen
acceptance programs/changes pass. Final artifact status is in `benchmarks/results/2026-11-04-language-v1-release.json`.
Stop feature growth at this contract; a further milestone requires explicit user selection. The first-three checkpoint receipt is
historical and predates an additional Function-literal admission correction; retain it unchanged.

## Change rules

The user explicitly selected typed operator bindings on 2026-11-08. Language/compiler 1.1 adds only the
bounded arithmetic binding and Text-concatenation scope in decision 0038 and `spec/core-v1.1.md`.
Current source compilation reports core/1.1. Checked format stays 1.0 with revalidated 1.0/1.1 input support.
Preserve the frozen 1.0 release evidence and stop feature growth at this selected milestone.

On 2026-11-14 the user selected the compact MVC authoring prototype in decision 0041. This reopens only
that source-projection and editor scope. Compact 0.1 requires explicit projection before the unchanged Rust
parser. Preserve the canonical MVC comparison and persistent identity values. Supplying both MVC files in
one bundle duplicates IDs. General grammar adoption and further features require separate selection.

The user subsequently selected native adoption and legacy deprecation in decision 0042. Current compiler
and language are 1.2.0/core/1.2. Follow spec/core-v1.2.md. Modern source needs no marker or projection.
The old module-annotation spelling remains deprecated read compatibility with no separate CLI flag.
Keep one resolver/checker/backend pipeline, preserve frozen compilers and stop at this selected syntax scope.

The user then selected maintained-source migration in decision 0043. Libraries and executable examples now
use 1.2 syntax with unchanged stable IDs and checked programs. Keep examples/v1, the legacy MVC comparison,
illustrative sketches and frozen research fixtures untouched. The repair-study driver reads fixed byte-identical
copies under benchmarks/fixtures/repair-study rather than live examples. scripts/source-migration.json pins the
migration checkpoint. Review intended behavior changes independently instead of recapturing failing goldens.

The user selected managed persistent identities on 2026-11-22. Current compiler/language are 1.3.0/core/1.3.
Follow spec/core-v1.3.md and decision 0044. Ordinary source can use tracked adjacent identity sidecars;
checks stay read-only and explicit authoring commands allocate or retarget IDs. Preserve existing identity
values and frozen fixtures. Keep compiler, editor, skill and evidence synchronized. Stop at this scope.

The user selected local generic call inference and contextual named callbacks on 2026-11-22.
Current compiler/language are 1.4.0/core/1.4. Follow spec/core-v1.4.md and decision 0045. Keep explicit syntax
for unresolved inference, frozen legacy source semantics, checked-code equivalence, bounds and purity.
The generic MVC is a separate maintained example. Stop at this selected inference scope.

The user selected constructor inference and contextual case names on 2026-11-22. Current compiler/language
are 1.5.0/core/1.5. Follow spec/core-v1.5.md and decision 0046. Infer from fields, payloads and explicit
expected nominal types. Preserve lexical precedence, IDs, bounds and frozen fixtures. Do not introduce global
case-name search or a whole-program inference solver. Stop at this selected constructor scope.

The user selected automatic JSON literal builder dependencies on 2026-11-22. Current compiler/language
are 1.6.0/core/1.6. Follow spec/core-v1.6.md and decision 0047. Keep Json and directly called functions
explicit; bind builders by stable ID from supplied sources with inspectable generated imports. Preserve
explicit builder compatibility, type/action/resource checks, declaration IDs and frozen fixtures. Stop here.

The user selected extensive test-first native/OS boundary implementation on 2026-11-22. Current compiler
and language are 1.7.0/core/1.7. Follow decision 0048, spec/core-v1.7.md and spec/native-interface-v0.1.md.
This is a closed 64-bit Linux catalog with explicit grants, opaque handles and bounded loopback networking.
Retain upstream test licenses/provenance and failed attempts. No arbitrary FFI, TLS, HTTP server, OAuth or
concurrency is implied by completing this slice. Preserve old artifacts, namespace semantics and fixtures.
Stop at this contract; expanding OS/network/application scope requires a separately selected milestone.

The user authorized both native TLA+ models and concrete C counterexample tests on 2026-11-22. Current
compiler/extension are 1.7.1; language remains core/1.7 and ABI 1. Follow decision 0049 and
verification/native/README.md. Apply .agents/skills/lobo-native-verification for covered changes. Run
`python3 scripts/check-native-models.py --out NEW_DIRECTORY` with the pinned installed TLA2TOOLS jar.
Do not blindly refresh coverage hashes or treat a timeout/build error as a verified or killed mutation.
Model boundaries, atomicity, source correspondence and deliberate defects must be reviewed together.
This milestone fixes pending-descriptor fault cleanup; it adds no broader ecosystem/language scope.

The user authorized a standalone Windows version of the snapshot demo. Compiler/extension 1.7.2 keep
language core/1.7 and ABI 1. Follow decision 0050 and examples/native/windows/README.md. This is the same
Lobo app with a restricted Windows clock/random provider and console launch policy. It is not a full Windows
compiler/network port. Portable cross-build tools are explicitly supplied; scripts never download/install them.
Keep Wine versus actual Windows coverage separate. Linux model claims do not cover the new Windows provider.

The user selected a standalone REST API after reporting a successful Windows 10 snapshot. Compiler/extension
1.7.3 keep language core/1.7 and ABI 1. Follow decision 0051 and examples/native/rest/README.md. Lobo owns
HTTP parsing, routing and IO loops. The explicit LOBO_WINDOWS_TCP cross-build profile adds Winsock equivalents
of the closed ABI; the snapshot profile remains the default. Keep loopback grants, deadlines, handle generations
and cleanup tests. Preserve Linux formal claims without extending them to Windows. No TLS/auth/storage scope.

The user selected implementing prefix try error propagation. Compiler/extension 1.8.0 and language core/1.8
follow decision 0052 and spec/core-v1.8.md. Exactly two payload-bearing Ok/Err cases are required in operand
and function output, with exact error types. Postconditions run on propagated errors. Keep direct try out of
contracts, preserve contextual try(...) calls and resource cleanup, and retain frozen fixtures/IDs. The snapshot
source is migrated; the existing REST transport is unchanged. Stop at this selected propagation milestone.

The user selected maintained-source modernization after try. Compiler/extension 1.8.1 keep core/1.8 and
ABI 1. Follow decision 0053. Use current inference, named callbacks, Text +, JSON literals and try where
semantically equivalent. Preserve error conversions, recovery and explicit cleanup. Run the checked codemod
audit and retain the older migration checkpoint unchanged. Historical projection inputs are separately frozen.
Do not expand inference or grammar to remove spellings that remain required by the current type checker.

The user selected the first SQLite driver on 2026-11-23. Compiler/extension 1.9.0, Core 1.9 and the separate
SQLite extension marker 1 follow decision 0055 and spec/sqlite-interface-v0.1.md. Native ABI 1 and old layouts
remain unchanged for non-SQLite programs. Pin the vendored engine and upstream test provenance. Keep one
closed static adapter, explicit file/role grants, typed rows, transactional cleanup and native execution limits.
The new persistent REST example uses direct actionful functions; Fn callbacks remain pure. Source capacity
is now 16,384 bundle tokens and 64 source nominals; other bounds remain. Run both native and SQLite model
checks, source correspondence/mutations, real C/LLVM/sanitizer/editor checks, and Linux/Wine executable tests.
Do not add a remote driver, NoSQL, ORM, concurrency, impure callbacks or arbitrary FFI after this milestone.

- Preserve the distinction between canonical state and projections.
- Never introduce ambient authority, implicit moving revisions, or evidence detached from its inputs.
- Protocol operations must declare state, actions, capability needs, budgets, and failure behavior.
- Examples must be labeled illustrative until executable fixtures exist.
- Record durable architectural choices or reversals in `decisions/`.
- Maintain `CHANGELOG.md` for meaningful changes, decisions, compatibility impact, and verification links.
  Keep earlier entries and decision rationale as history; supersede decisions explicitly rather than rewriting them.
- Record every experiment, test outcome, research finding and resulting decision before reporting task completion.
  Include failed, negative, interrupted and inconclusive outcomes. Record unknown values as unknown, never zero/pass.
  Keep commands, input/tool identities, raw-evidence locations, limits and verification with the result.
  Add experiment results to `benchmarks/results/README.md`, decisions to `decisions/`, and a dated ledger entry.
  Preserve original evidence when correcting analysis; label proposals and unimplemented ideas separately.
  Never copy credentials or sensitive logs into reports. Document any evidence that could not be retained.
- Update cross-references when terminology changes.
- Treat the VS Code extension as part of every language change: update its highlighting, definition navigation,
  editor configuration as applicable, and regression fixtures in the same change. Do not let editor support
  silently lag implemented syntax or scope rules; see `docs/compiler-development-workflow.md` for editor gates.
- Do not add a dependency, custom runtime, package system, or concurrency model without a measured need and an
  explicit decision record.
- Maintain `.agents/skills/lobo-programming` with language/tool changes: update compatibility, references
  and executable examples together. Validate the skill and run its native example/link checks.
  Keep benchmark solutions out of its examples and preserve frozen evaluation packages as history.

## Verification

Run:

```sh
cargo test --locked
./scripts/check.sh
```

Cargo runs the Core conformance corpus, behavior/artifact checks, native/reference comparisons, CLI tests,
and generated native/sanitizer checks. See `docs/native-development.md` for fixed seeds, limits, and reproducibility.
The script validates JSON syntax, each JSONL message, and local Markdown links. Review expected results against the specification;
do not edit an oracle just to make an implementation pass.

Language/compiler changes also require `npm --prefix editors/vscode test`. For extension runtime/navigation
changes, run the real extension-host checks documented in `editors/vscode/README.md`, then rebuild the VSIX.
Report unavailable editor checks explicitly; packaging alone is not evidence that navigation works.

