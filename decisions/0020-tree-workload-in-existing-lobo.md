# 0020: Express a growable search tree with ordinary Lobo values

- Status: implemented in existing Core 0.7; local verification passed
- Date: 2026-10-17
- Risk mode: lean; local workload/test change, no compiler/runtime or authority change.

## Contract and choice

Implement frozen workload C in [the workload specification](../spec/language-milestone-workloads.md).
Insertion order determines the binary-search-tree shape, duplicates do not add nodes, negative/extreme I64
keys remain valid, and traversal returns ordered keys/count/height. Test reverse traversal without a new
compiler feature. Native/reference/sanitizer evidence, explicit resource failures, and editor fixtures remain
required. No host-language algorithm or tailored compiler intrinsic may perform the program's computation.

Recursive named values are a plausible alternative, but their layouts/lifetimes and recursive operations
are not yet specified or implemented. The frozen workload explicitly permits a growable node table with
indices. Choose that existing-language representation: immutable nodes carry a contiguous insertion index,
parent index, key, and depth; a root alternative distinguishes an empty tree. Parent index -1 denotes the
root's missing parent and never encodes a key. Child lookup uses parent index and key comparison, not key
subtraction. Insertion searches the existing tree before allocating a node. No sort is substituted for
construction/traversal. Explicit search/traversal state avoids a language call frame per tree level.

Invariants: one root for nonempty trees; each added node has an existing parent; at most one child on each
side; strict lesser/greater child keys; no cycles; depth is parent depth plus one; height is maximum depth.
Duplicate insertion preserves the tree. Allocation/sharing and exhaustion use existing List semantics.
These invariants hold for trees built by the workload, not arbitrary forged helper arguments.

This is a local alternative analysis, not independent prototypes or a performance claim. Full-table child
searches and counted folds may be expensive; record that pressure for the later language coverage assessment.
Completing the three workloads does not by itself complete broader reuse/program organization or establish
Turing completeness. No dependency, external authority, recursive-type syntax, or ecosystem work is added.

## Observed evidence — 2026-10-18

The first acceptance test failed because `examples/tree.lobo` did not exist. The example then passed the
frozen cases using six composed Lobo functions, and the expanded cases passed without any compiler/runtime
change or adjustment to the frozen outputs. `build` constructs the actual node table; `insert` searches
for a missing child or duplicate; `traverse` uses explicit pending nodes; `reverse` restores the collected
list's order. Root/parent indices and depth are inspected independently in the tests, not inferred solely
from sorted output. The summary contract also checks returned count against key-list length.

Final local gates on Linux x86_64 with Cargo 1.92.0, Clang 18.1.3, and Node 22.19.0:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 100 tests pass per profile
  (99 integration tests plus one native-launch unit test), including six tree integration tests.
- Every tree case uses a reloaded checked artifact in the reference evaluator, optimized native code,
  and ASan/UBSan native code with leak detection. Cases cover frozen forward/reverse results, actual
  parent edges/root/depth, empty/single/repeated values, I64 extremes, 40-level chains in both directions,
  32 duplicate-preserving affine variations of a known balanced shape, and fuel/heap exhaustion with
  `execute` diagnostics and no partial summary. Resource allowances are documented in the native guide.
- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, and
  `git diff --check` pass. Existing compiler behavior and test oracles remain unchanged.
- `npm --prefix editors/vscode test`: 79 tests pass, with grammar/parameter navigation for all ten
  executable examples and full-source tree function/parent-field checks.
- Real VS Code host checks report `LOBO_HOST_TESTS_PASSED` and exit 0, including the full tree/parser
  examples and existing unsaved compiler diagnostics. As in the previous parser turn, the host logs
  `Unexpected SIGPIPE`; this environmental observation is not described as a clean log or a failed assertion.
- `npm run package` succeeds with 11 VSIX files and no test/runtime npm dependencies. The runtime and
  extension version remain 0.8.0; the rebuilt README reflects ten executable examples. No normal-profile
  installation, commit, publication, deployment, or benchmark is part of this slice.

Release-compiler `check` accepts all six entries with empty action sets. The compiled
`target/native-examples/tree` is a stripped, dynamically linked x86_64 ELF executable, 31,080 bytes,
and runs without Lobo or Clang. Direct invocations on `[3,1,3,2]` return keys `[1,2,3]` or `[3,2,1]`,
both with count 3 / height 3. These are working-tree artifacts, not a published release.

| Input/artifact | SHA-256 |
| --- | --- |
| `examples/tree.lobo` | `0404891d5d88daddf76c0866187adf543640a35514786dc980a5172e2234198e` |
| `tests/tree.rs` | `3e10d25e11f4b6d1e4b7bcbc8dea97e01341c6f76e6df8bfa6838852a3d425e5` |
| `target/native-examples/tree` | `12a10c71f8123aadfe31270da5d5a4b699c1fa929388cd188017e260cff85c0d` |
| `target/release/lobo` (unchanged compiler) | `84794ea7b8b22047fe4d11cc07b0f8c2224c242fad048a9e0975e8042bdff923` |
| `dist/lobo-syntax-0.8.0.vsix` (19,216 bytes) | `c2caa5244d1ca7a50fec59171f253b9e7e7836b7ca0092150917eee950f8565e` |

The current local VSIX hash supersedes the earlier build at the same path recorded in decision 0019;
that historical evidence remains unchanged.

## Consequences for the active goal

All three frozen workload gates now have execution evidence. They do not prove the full language-first
goal: see the [coverage assessment](../docs/language-completeness.md#coverage-assessment-after-the-three-workloads)
for copied cross-file reversal logic and repeated stopped-state folds. This implementation intentionally
does not hide the cost of full-table child lookup or add a host tree primitive. Tree traversal needs no
per-level language recursion, but the compiler's general 32-frame call-depth limit is unchanged.
Application invariants are not a general proof about arbitrary forged helper inputs or every execution.
Recursive named types remain unsupported; network/storage/search, FFI, packages, and workspace services
remain deferred. The goal stays active for composition, state-driven termination assessment, and the final
requirement-by-requirement coverage audit.
