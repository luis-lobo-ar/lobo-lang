# 0021: Add immutable state-driven iteration

- Status: implemented in compiler 0.8.0 / extension 0.9.0; local verification passed
- Date: 2026-10-18
- Risk mode: standard; expression semantics, failure/fuel behavior, IR, native lowering, and editor scope change.

## Need and alternatives

The coverage assessment found stopped-state folds in parser reduction, collection validation, and tree
search. They need predetermined counts and keep spending fuel after reaching a final state. Ordinary
recursion can express termination but grows the current explicit 32-frame budget. Merely raising that limit
does not define stack-safe iteration. Proper tail calls would change recursion's failure policy and need
separate decisions around contracts/call obligations; they are not a transparent optimization here.
Mutable while/break/assignment would require more state/exit semantics than these patterns demonstrate.

Choose one additive immutable state-carrying expression, following [Core v0.8](../spec/core-v0.8.md):
initialize once, test the current state, return on false, otherwise carry the next body value. Local
alternative analysis, not independent prototypes. This supersedes decision 0008's deferral of general
iteration only for immutable carried state; assignment and general mutation remain deferred.

Keep `while` contextual. Charge every condition probe before evaluating it, including the final false
probe, so constant-condition loops cannot evade fuel. Preserve child ordering, all-child static action
checking, pure contracts, heap/lifetime behavior, and finite frontend budgets. Emit native loops without
iteration counters/stack growth. Source/IR versions advance explicitly; CLI encoding/native adapter stay
unchanged. No dependency, runtime service, ambient resource access, or ecosystem expansion is introduced.

## Keeper and falsification plan

Specify first, demonstrate a failing test, then implement parser/checker/resolution/evaluator/native and
bounded editor parsing together. Test exact false-loop fuel, short-circuit/fault ordering, state scopes and
fresh locals, compound types and cleanup, unreachable action calls, contract phases, nontermination,
artifact tampering/depth limits, and a head-match search independent of tail length. Convert concrete
parser/tree patterns without changing frozen expected results. Rebuild compiler/VSIX and run full
debug/release, Clippy/format/docs, editor and actual VS Code host gates. No publication/install/commit.
Cross-file reuse and the full coverage audit remain outstanding; this slice alone cannot complete the goal.

## TDD findings and observed evidence

The first exact-fuel test failed at the unsupported typed-state prefix. After the semantic path was
implemented, a 40-level nesting test aborted the debug compiler's test thread with stack overflow before
its diagnostic. Isolating loop parsing was insufficient: disassembly showed the old seven-level expression
parser retained large debug stack frames per nesting level. Precedence climbing removed that call tower
without increasing thread stacks or lowering MAX_DEPTH. A 24-level loop compiles/round-trips, and the
40-level input now reports `resource.frontend.limit`. Canonical grouping tests and the prior corpus remain.
An adversarial `1 == 2 < 3 < 4` regression then caught a type-error fallback in the initial refactor;
rejecting repeated nonassociative operators before returning to a lower precedence restored syntax rejection.
These were real failed runs, not counted as passing verification.

Final local commands on Linux x86_64, Cargo 1.92.0 / Clang 18.1.3 / Node 22.19.0:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 112 tests pass per profile
  (111 integration tests and one native-launch unit test). Twelve state-driven-loop tests cover exact
  fuel, fault ordering, contracts/phases/subjects, all-child actions, scoping/fresh bindings, nested
  loops/folds, nontermination, cumulative heap including final-condition allocation, compound/native-only
  list layouts, contextual names, source ranges, checked-artifact revalidation/version rejection, and nesting.
- Loop fixtures run through reloaded checked artifacts in the evaluator, optimized native code, and
  ASan/UBSan native code with leak detection. The 1,000-step loop does not exhaust call depth. Head-match
  search succeeds with execution fuel 100 for both one-element and 1,001-element inputs; admission and
  physical input storage still scale with input. This is execution-budget evidence, not a timing benchmark.
- Parser reduction and tree search now use loops rather than stopped-state counts. All frozen
  parser/collection/tree outputs, negative/overflow/deep/variation/exhaustion fixtures still pass; existing
  numerical and structured-value expectations were not changed. Only current-version assertions advanced.
- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, and
  `git diff --check` pass. No dependencies changed.
- `npm --prefix editors/vscode test`: 83 tests pass, including all eleven executable examples. Actual
  VS Code host checks report `LOBO_HOST_TESTS_PASSED` and exit 0: state references in condition/body,
  an unsaved condition error with exact `42` range, and prior parser/tree/action/data navigation checks.
  The final run has no incidental SIGPIPE log seen in earlier slices.
- `npm run package` rebuilt extension 0.9.0 (11 files, no tests/runtime npm dependencies). It was not
  installed into the normal profile. No commit, publication, deployment, or comparison worktree mutation.

The release compiler accepts the new GCD/search example as Core 0.8 with empty action sets. Direct native
smokes return GCD(1071,462)=21, head match index 0, parser `Ok(24)`, and tree keys `[1,2,3]`/count 3/height 3.
New suffixed binaries preserve earlier local native examples. Sizes/hashes describe this working tree,
not a published release:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/lobo` | 1,009,440 | `2d6447deb6809a0cb7a5de8944a4695af99b35a9dc79d84145e56e37c36b068a` |
| `target/native-examples/state-iteration-core08` | 18,776 | `71a6013adf435ccc96fb73eacc693eb8c7c237669799638b5be63214e17e5255` |
| `target/native-examples/parser-core08` | 47,480 | `240cbd5ad35e3dfede4b1bca494d5076fc2bf99c5961ad3e46d2c4effcfd9aba` |
| `target/native-examples/tree-core08` | 31,080 | `5c8eaeff87c293c8049efd0226b122caa1e17ca05c8bc54e32ff1dfc2c9913de` |
| `dist/lobo-syntax-0.9.0.vsix` | 19,487 | `ac0bff2b32d71d648f7afa7e12c104be96421648ba94551fc2e479a4c66ea189` |

Input SHA-256 values: `examples/state_iteration.lobo`
`5deb87b4edead36b4c594e430a5cc3e4c10adc4ac906302afc2769003a22abed`;
`examples/parser.lobo` `a4e4fd24d51eb9cf610b1f0581158962202f048c4c5387873dcce5548adf4ea8`;
`examples/tree.lobo` `a92bcb9cf84d20bc052d2ff9674628f3f5fcac216ca0f7878e27a38f13229a66`;
`tests/while.rs` `a9694d8bfa391a74f71fa3c3954013a37eb651f84129b1c2b90d7ef2ad11e8e9`.

## Remaining boundaries

The state-driven termination gate is met; the [coverage assessment](../docs/language-completeness.md#coverage-assessment-after-the-three-workloads)
keeps cross-file reuse/program composition outstanding. This is finite conformance evidence, not proof
of termination, all compiler behavior, Turing completeness, an OS sandbox, or universal replacement.
Ordinary call-depth policy, cumulative invocation arenas, single-module compilation, and deferred ecosystem
work remain unchanged. The overall goal is active.
