# 0019: Express the parser workload with ordinary Lobo stacks

- Status: implemented in existing Core 0.7; local verification passed
- Date: 2026-10-17
- Risk mode: lean; local workload and tests, no compiler/runtime or authority change.

## Contract and choice

Implement frozen workload A in [the workload specification](../spec/language-milestone-workloads.md),
including typed failure offsets, precedence, checked nonnegative I64 arithmetic, and resource exhaustion.
No host-language parser or workload-specific intrinsic may do its work. Keep the native/reference/editor
gates and leave tree/composition work visible rather than treating this workload as the whole goal.

Recursive descent is a straightforward alternative, but directly couples expression nesting to the
existing call-depth allowance. Choose a streaming scanner with explicit immutable value/operator stacks,
structured scanner state, and ordinary fold/function composition. An operator carries its source offset;
an opening parenthesis is a stack delimiter. Check integer and arithmetic bounds before executing checked
I64 operations. The result/error alternatives and stack records are application types, not new syntax.

This is a local alternative analysis, not independent implementations. The finite frozen examples provide
independent expectations; further generated expressions should carry expectations from their construction,
not from a host parser that replaces the workload. Deep nesting, malformed bytes, boundary arithmetic,
fuel/heap faults, and maintained editor navigation are adversarial gates. Runtime budgets still apply;
this choice is not a performance, unbounded-computation, or Turing-completeness claim.

## Observed evidence

`examples/parser.lobo` implements the computation with 12 composed Lobo functions and application-defined
stack/scanner/error types. Compiler source, language semantics, artifact versions, and extension runtime code
are unchanged by this slice. All declared action sets are empty; no external resource grant is introduced.

The first acceptance test failed because the example did not exist. After implementation, expanded testing
exposed a real error-precedence defect: `9223372036854775807+1)` reported arithmetic overflow instead of the
specified unmatched-parenthesis failure. A failing regression preserved the required `Unbalanced(21)` result;
checking for an opening delimiter before reduction fixed it without changing the compiler or numeric oracle.

Final commands on the local working tree, Linux x86_64, Cargo 1.92.0 / Clang 18.1.3 / Node 22.19.0:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 94 tests pass in each profile
  (93 integration tests plus one native-launch unit test). Five workload tests include the frozen cases,
  64 seeded expression variations, every single octet, checked arithmetic/left-association boundaries,
  sticky errors, 128-level nesting, and fuel/heap exhaustion without a partial result.
- Each parser case runs through a reloaded checked artifact in the reference evaluator, an optimized native
  binary, and an ASan/UBSan native binary with leak detection. The workload tests use fuel 1,000,000 and heap
  100,000 except explicit exhaustion cases; deep nesting is not a promise about the smaller default fuel budget.
- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, and
  `git diff --check` pass. Existing tests and mathematical expectations are preserved.
- `npm --prefix editors/vscode test`: 76 tests pass, including grammar/navigation on the full example.
  The first-parameter fixture now accepts nominal types and starts its reference search at that function's
  first contract/body brace rather than accidentally using a later function's contract.
- Real VS Code host checks report `LOBO_HOST_TESTS_PASSED` and exit 0, including function/field navigation
  in the full parser source and prior unsaved diagnostics tests. The host also logs `Unexpected SIGPIPE`;
  this is retained as an environmental observation, not described as a clean log or a test failure.
- `npm run package` rebuilt the local VSIX (11 files, no tests/runtime npm dependencies). No normal-profile
  install, publication, commit, deployment, or benchmark claim is part of this work.

Release-compiler `check` accepts the example. Native `run` returns `Ok(24)` for `12 + 3 * 4` and
`Failed(ArithmeticOverflow(19))` for `9223372036854775807+1`. The separately compiled, stripped, dynamically
linked x86_64 ELF at `target/native-examples/parser` runs directly without the Lobo compiler or Clang.
It is 43,384 bytes. These are working-tree artifacts, not a published release.

| Input/artifact | SHA-256 |
| --- | --- |
| `examples/parser.lobo` | `7351cf62254c6be3b4806900c5fb90e7d6baeb44dd34faaf16e53e1229658acf` |
| `tests/workloads.rs` | `bcd847d1fdcf069250ad5f535836eb47dbf7b655dbe8643bfaae705d81dd8ab2` |
| `target/native-examples/parser` | `65d2d9b47c15b3f8bb4a7b35d5f99024e7ae189d007cd3bc6b025f408213bbbe` |
| `target/release/lobo` (unchanged compiler) | `84794ea7b8b22047fe4d11cc07b0f8c2224c242fad048a9e0975e8042bdff923` |
| `dist/lobo-syntax-0.8.0.vsix` (19,218 bytes) | `dc2028f2ca27e86a81c676d7603e4016d4edf443c777bb27eb2460b4b0b5b844` |

The rebuilt VSIX updates README fixture coverage, not runtime behavior. Its current local hash supersedes
the earlier build at the same path recorded in decision 0018; that historical evidence remains intact.

## Remaining language pressure

The parser avoids recursive call-depth growth, but counted folds still spend bookkeeping fuel after a
terminal state. Delimiter searches and reductions over explicit stacks can cost quadratic work on nested
inputs. This is evidence to consider when evaluating data-dependent iteration patterns, not a claim that
a new control construct is already needed or implemented. Allocation remains cumulative per invocation.
Tree workload C and broader reusable program organization remain pending. Workloads A and B do not by
themselves prove the full language milestone, universal replacement, or Turing completeness.
