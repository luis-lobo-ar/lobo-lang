# Core 0.10 recursive-value evolution

Date: **2026-11-04**. This is implementation/development evidence, not agent-comparison results.

Core 0.10 admits recursive nominal data when every cycle passes through existing List storage.
Direct inline cycles remain rejected. A public AST evaluates nested sums/products and shifts
literal values while preserving operation and child order. No host routine performs those operations.

## Preserved baseline and native result

The [Core 0.9 record](2026-11-04-core09-language-baseline.json) identifies preserved compiler/test-runner
binaries and a compressed source/spec/test/editor/skill snapshot. These are ignored local artifacts
under `benchmarks/private-evidence/core09-language-baseline-20261104`; they require normal backup and
are not a clean published release. The snapshot includes the preexisting dirty workspace.

The preserved compiler rejects the recursive AST with `compile.record.cycle`. The new native
executable evaluates `Sum([Num(2),Product([Num(3),Num(4)]),Sum([])])` to **14**, and shifting every
literal by one gives **23**. The executable is **22,888 bytes** on this machine. This is artifact
identity, not a size/speed improvement claim. [Native data and hashes](2026-11-04-recursive-ast-native.json).

## Verification

| Check | Result |
| --- | --- |
| `cargo test --locked` | 151 tests pass, none failed or ignored |
| Recursive-value tests included above | Six tests cover source/artifact/module behavior, malformed values, exact budgets, depth/fuel faults, AST cases and deep cleanup |
| `cargo clippy --locked --all-targets -- -D warnings` | Pass |
| `cargo fmt --check` | Pass |
| `cargo build --locked --release --bins` | Pass |
| Python benchmark/harness regression suite | 30 tests pass |
| Extension 0.12 unit/tokenizer/compiler tests | 108 tests pass; VSIX packaged |
| Real VS Code host | Completion marker and exit 0; includes recursive AST definition targets |
| Lobo skill | Frontmatter validator, compiler check and native counter/text/recursive-folder examples pass |
| Foreground JSONL verification tool | Recursive AST executes two cases, reuses two on an unchanged affected run, then executes a fresh full final gate |
| Repository data/docs | JSON/JSONL, local links, hashes and whitespace checked |

Native/reference comparisons include ASan/UBSan and leak checks. Deep internal construction/reclamation
uses 10,000 nested nodes, including success, partial heap exhaustion and rejected deep output. Logical
input/output depth stays eight; structural equality permits 64 active compound values, with classified
failure at the next level. A contract-phase case checks that resource failure is attributed correctly.

## Failures and corrections retained

- The initial new test used a private Rust method and was corrected to observe public serialized output.
  That test-authoring error is not the semantic TDD failure.
- The corrected test failed on the intended Core 0.9 `compile.record.cycle` rejection before implementation.
- A later cross-module test initially used borrowed strings where `SourceFile` requires owned strings.
  The fixture was corrected; no compiler API was changed to accommodate it.
- The editor initially failed four of 107 tests because the old compiler-success allowlist ended at
  Core 0.9. The adapter now explicitly accepts 0.10, retaining previous version/failure controls.
- Adding the AST to grammar fixtures exposed its lack of a contract under that fixture's expectation.
  Reverse now states its existing cardinality requirement as a runtime postcondition; the AST's
  behavioral tests still independently check order and results.
- The missing-Product canary initially removed a branch while leaving a trailing comma, producing
  a syntax error. Removing the separator with the branch tests the intended non-exhaustive-match error.
- The first draft workload table misstated the shifted negative case as -18. Arithmetic review corrected
  it to **-7 before AST implementation/execution**. Expected numbers were not adjusted to program output.

Original test/editor logs are copied to a private evidence directory and hashed in the
[machine verification record](2026-11-04-core010-verification.json). The original Core 0.9 list-cycle
rejection assertions were deliberately updated for the specified 0.10 semantics. Inline-cycle rejection,
forged artifact rejection and all other numeric/fault oracles remain required.

## Compatibility and remaining limits

Compiler/Core/checked artifacts advance to 0.10; native adapter is 0.6; editor is 0.12. Recompile
old checked artifacts instead of retagging them. Existing native binaries remain independent.
No new dependency, collector, Box syntax, host authority, plugin, installation or publication was added.
The skill was maintained; frozen prior agent-study evidence remains historical.

User generics, function values and direct inline recursive layouts remain unsupported. Native/reference
agreement and sanitizer success are finite evidence, not a soundness proof. No new coding agent or human
participant was tested. Construction/comprehension benefits remain unknown. This AST is public
development material, not an unseen confirmatory benchmark.

The [foreground tool data](2026-11-04-recursive-ast-test-session.json) preserves the exact frozen
plan, requests and responses for recursive AST checking. This smoke uses no language-model agent.

Specification and decision: [Core v0.10](../../spec/core-v0.10.md),
[AST workload](../../spec/recursive-ast-workload.md), [decision 0029](../../decisions/0029-list-guarded-recursive-values.md).
