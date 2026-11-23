---
name: lobo-native-verification
description: Verify or update Lobo native capability, handle, descriptor, timeout and cleanup behavior with the source-linked TLA+ models and concrete C counterexample tests. Use when changing the native provider, invocation cleanup, these models or their verification harnesses.
---

# Verify the native boundary

Read [coverage and source fingerprints](../../../verification/native/coverage.json) and the
[model contract and replay mapping](../../../verification/native/README.md) first. The reviewed repository
HEAD is `39ec4662869ae992a508a06cf2f3a9cf36240220`. The models and implementation include uncommitted work
beyond that HEAD. Exact covered-file SHA-256 values, not that commit alone, identify the reviewed code.
Do not invent a clean commit or advance the sync revision without checking the mapping.

Coverage includes `src/native_api.c` capability/handle policy, generation retirement, pending descriptor
ownership, connect/wait and IO lifecycle; `src/native_runtime.c` fault/exit cleanup; and the model/C replay
inputs listed in coverage.json. Compiler typing, ABI memory safety, arbitrary kernel behavior, real concurrency
and general 64-slot correctness are outside the formal result. These are finite models, not a refinement proof.

## Run

Use an installed Java 11+ and the pinned TLA tools 1.7.4 jar. Its SHA-256 is
`936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`.
Set TLA2TOOLS to its location; no download or fallback is performed. From the repository root:

```sh
python3 scripts/check-native-models.py --out target/native-model-check-NEW
cargo test --locked --test native_formal --test native_adapter
```

Replace NEW with a fresh output name. The command reads TLA2TOOLS or accepts `--tla2tools PATH`.
It checks both baseline models, eleven intentional model defects, their concrete C witnesses, and eleven
isolated C mutations, each under optimized and ASan/UBSan builds. Its negative checker self-test rejects a
known-broken baseline under Python -O; acceptance checks must never rely on removable Python assertions. It writes `results.json`, command receipts,
raw TLC logs, counterexamples and reproducible mutated source copies. Compilation failure is not a killed
mutation. A timeout, tool error, stale source map or incomplete state queue cannot pass.

Read every model result with the pstack `/statechart` skill. Follow that skill's current documentation to
produce charts and plain-language paths; do not copy its implementation commands into this skill. Use a
scratch directory for visualization. The smaller chart configuration is illustrative and distinct from the
full acceptance bounds. Preserve raw failing traces and do not mistake terminal stuttering for a deadlock.

## If the source map is stale

Do not simply refresh hashes to clear the gate. Review each changed covered function against the transition
mapping. Match actual atomic steps: a connect descriptor exists before publication, close retires/increments
its slot, deadlines change between operations, and reservations charge requested bytes before short IO.
Check explicit error returns separately from language faults and kernel process teardown.

If behavior is intentionally changed, update the model, mapping, assumptions and witness tests together.
If the implementation violates the existing contract, retain the counterexample and reproduce it with the
actual C harness before fixing it. Change only the authorized behavior. Keep a failing-before log and prove
that the relevant isolated C mutant fails the same witness; never accept a mutant compilation error as proof.

Update coverage.json hashes only after that review, recording the real HEAD and dirty-source caveat. Then run
the full verification command from a fresh output directory. If no model or C mutation fails, investigate
vacuity or an ineffective mutation. All intended baseline actions must have nonzero successor coverage.

## Finish

Run the full repository checks for product changes, retain all failed/corrected runs and source/tool identities,
and add the result to the ledger and paper evidence register. Preserve earlier reports and frozen release/study
inputs. Report exact bounds and assumptions, meaningful C findings and exclusions. A bounded model pass does
not establish production security, absence of undefined C behavior or agent productivity.

Use `/maintain-formal-verification-skill` after changes that may have moved the implementation beyond this
mapping. Validation requires reviewing the model against current source, not only rerunning an old model.

Compiler 1.7.2 conditionally appends a separate Windows snapshot provider to C export. The existing Linux
provider prefix was reviewed byte-identical; these model claims remain Linux-only. Changes to the Windows
file require the separate Windows/Wine build and provider tests documented in examples/native/windows/README.md,
not merely a passing Linux model check.

Compiler 1.7.3 adds src/native_windows_tcp.c under the explicit LOBO_WINDOWS_TCP cross-build profile.
Run scripts/test-windows-tcp.py and scripts/test-native-rest.py with an isolated Wine prefix for its real PE
provider and HTTP tests. Existing TLA+ claims remain Linux-only. The Linux covered files are unchanged.

## SQLite extension 1

Compiler 1.9.0 adds the separate closed SQLite extension. Read
[SQLite coverage](../../../verification/sqlite/coverage.json) and its
[lifecycle contract](../../../verification/sqlite/README.md). Run
`python3 scripts/check-sqlite-models.py --out target/sqlite-models-NEW` as well as the older native gate.
The five model configurations include four deliberate defects; eight C mutants replay those defects under
optimized and sanitized builds. Finalize and close remain available after SQL query quotas run out. Review
that distinction when changing operation charging. The model abstracts one connection/statement and two
writes. It does not prove the engine, a general registry cutoff, crash durability or Windows OS behavior.
Run the SQL adapter, C/LLVM program, editor and Linux/Wine executable gates separately. Preserve upstream
source and test provenance; adapted cases are not an execution of the full upstream SQLite test suite.
