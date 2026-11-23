# SQLite adapter lifecycle verification

This finite model checks adapter ownership, permission restriction, transaction guards and cleanup. It is
separate from the older [native networking models](../native/README.md). The
[reviewed source correspondence](coverage.json) pins the actual dirty source, not just repository HEAD.
Review the named functions and mapping before updating any fingerprint.

With an explicitly installed TLA tools 1.7.4 jar whose SHA-256 is
`936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`:

```sh
export TLA2TOOLS=/absolute/path/to/tla2tools-1.7.4.jar
python3 scripts/check-sqlite-models.py --out target/sqlite-models-NEW
cargo test --locked --test sqlite_formal --test sqlite_adapter --test sqlite_programs
```

Use a fresh output directory. `--verify-map` checks fingerprints without Java or compilation. Cargo also
checks that a deliberately stale map fails under Python `-O`. The full checker requires an exhausted TLC
queue, nonzero successor coverage for every intended baseline action, the named invariant violation for
each broken model, successful baseline C witnesses and the expected behavioral rejection for each isolated
C mutant. Build failures, sanitizer crashes, wrong assertions and timeouts do not count as kills.
`--models-only` is diagnostic and does not satisfy acceptance.

The model bounds are eight charged operations, one connection, one statement, at most two pending plus
committed writes and three permission atoms. SQLite engine operations are atomic abstract transitions;
unsuccessful engine calls may stutter. Finalization and exit cleanup do not require remaining operation quota.
Normal database close shares the cleanup mechanism; it is not a separate model transition.

| Broken model | Violated invariant | Concrete C witness and isolated defect |
| --- | --- | --- |
| cleanup | Ownership | Fault with a live statement; omit statement cleanup; observer detects a live child after database teardown. |
| persist | OnlyCommitPersists | Fault with uncommitted insert; replace cleanup rollback with commit; independent SQLite query detects the write. |
| commit | LegalCommit | Commit without an active transaction; remove the state guard; State error assertion fails. |
| amplify | NoAmplification | Restrict read capability to schema rights; remove the subset guard; Denied assertion fails. |

The checker retains TLC traces, command receipts, mutated C copies and baseline/mutant logs in both optimized
and ASan/UBSan modes. These are manually reviewed witness translations, not an automatic refinement proof.
[Lifecycle.tla](Lifecycle.tla) and the [statechart view](lifecycle.view.mjs) support plain-language trace review.
The shortest cleanup counterexample opens a connection, prepares a statement, then exits while leaving its
child alive. The other defects expose unauthorized permission growth, invalid commit acceptance and
uncommitted data becoming persistent.

The model does not prove SQLite itself, filesystem or power-loss durability, C memory safety, arbitrary
32-slot/generation behavior, the compiler ABI, Windows kernel behavior, concurrency or a secure filesystem
sandbox. Path validation assumes trusted parent directories that hostile actors cannot replace. Separate
C/LLVM/sanitizer tests, real file oracles, Linux TCP and Wine PE tests supply additional bounded evidence;
Wine is distinct from a run on actual Windows hardware.
