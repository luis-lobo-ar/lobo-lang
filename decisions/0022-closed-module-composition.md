# 0022: Compose explicit modules by stable declaration identity

- Status: implemented in compiler 0.9.0; editor/workload follow-through completed by decision 0023 (extension 0.11.0)
- Date: 2026-10-18
- Risk mode: standard; names, nominal identity, IR revalidation, and source provenance change.

Copied reverse/cursor logic in the collection and tree workloads is a demonstrated reuse limitation.
Choose [Core v0.9](../spec/core-v0.9.md): selected imports by stable declaration ID, a closed caller-supplied
source set, module-local names, and one flat checked/native declaration graph with explicit ownership scopes.

Alternatives: textual concatenation fails local-name/identity/scope separation and makes source evidence
ambiguous. Glob/path/registry imports would mix language composition with unrequested resolution authority
and ecosystem policy. Fully separate object files are not needed to share checked Lobo computation yet.
Local alternative analysis, not independent prototypes. Retain native lowering and nominal identity rather
than mangling user bindings or duplicating foreign implementations.

Implement the pure in-memory compiler/linker first, then explicit file/agent/editor adapters. No new IO is
allowed in the compiler API. Revalidation must reject missing/duplicate owners, invalid imports and direct
out-of-scope IR calls/constructors/types; implicit inferred signature types may still flow. Dependency cycles
do not add initialization semantics; recursive data layouts remain rejected. Preserve actions/pure contracts
and aggregate budgets. Attribute errors to exact input text or leave them unlocated, never guess a file.

TDD: missing import support first; then shared nominal values and function aliases, independent native/
reference/ASan/UBSan/leak checks, changed dependency behavior, local-name collisions, malformed/forged closure,
scope/action/contract checks, deterministic ordering, source coordinates, and resource bounds. Maintain import
highlighting/local navigation while cross-file tooling is built; do not claim completion from backend tests.
The user-authorized language-first goal remains active, with no package/network/storage/daemon expansion.

## TDD findings and implementation boundaries

The first shared-module test failed at the unsupported import form. The implemented linker now resolves
each module through its own explicit aliases before validating one flat stable-ID graph. Namespace contexts
are shared, not copied per declaration; a 1,024-slot declaration/import admission bound applies before
building them, including when loading forged artifacts. A regression initially accepted 1,025 functions
and now rejects them with `resource.frontend.limit`. Existing aggregate source/token budgets are unchanged.

Source-range regressions preserved the prior diagnostic oracles rather than changing their expectations.
An adversarial imported record called as a function exposed a compiler panic: the resolved stable ID's length
had been used as the original alias token's length. Calls now retain an ephemeral original name span, and
location conversion rejects invalid byte/UTF-8 boundaries instead of slicing or clamping them. Unknown types
and known-but-invisible types retain distinct errors. Diagnostic source labels serialize as strings but use
boxed storage to keep the Rust error value below Clippy's large-error threshold.

An explicit-file admission test initially returned a UTF-8 read failure for an oversized malformed file.
The bundle CLI now checks bounded raw bytes before decoding, so size admission wins as specified. The pure
compiler API and agent envelope perform no source IO; file reads remain explicit CLI adapter inputs only.
Failed TDD runs above are not counted as successful verification. No expected workload outputs or numerical
oracles changed; only current compiler/artifact version assertions advanced.

## Observed local verification

Linux x86_64, Cargo 1.92.0, Clang 18.1.3, Node 22.19.0:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 128 tests pass per profile
  (127 integration and one native-launch unit test). Sixteen module tests cover aliases/shared nominal
  records/variants, local name collisions, changed dependency behavior, source/IR visibility, forged owners
  and imports, action bounds/pure contracts, mutual function dependencies, recursive-layout rejection,
  deterministic JSON/C, exact dependency source ranges, aggregate source/token/metadata admission, wire
  versions, and explicit CLI/agent paths. The agent check also succeeds with an empty PATH, without Clang.
- Module fixtures compare reloaded checked artifacts in the reference evaluator, optimized native code,
  and ASan/UBSan native code with leak detection. Existing parser/collection/tree and resource oracles pass.
- `cargo build --locked --release --bin lobo`, `cargo fmt --check`,
  `cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, and `git diff --check` pass.
  No dependencies were added.
- `npm --prefix editors/vscode run package`: 87 editor tests pass and extension 0.10.0 is rebuilt.
  Real VS Code host checks using a fresh temporary profile report `LOBO_HOST_TESTS_PASSED` and exit 0,
  including prior navigation/unsaved squiggles and local import alias targets. They do not establish
  remote import navigation or bundle-aware diagnostics. No incidental SIGPIPE was logged in this run.

The release compiler checks and compiles the three explicit module files. Native reverse of `[5,-1,9,0]`
returns `[0,9,-1,5]`, matching the evaluator; double reverse returns the original list. All three functions
have empty action bounds. The example is a stripped native executable, not an interpreter launcher.
Sizes/hashes identify this local working tree, not a published release:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/lobo` | 1,078,512 | `8476c9530b58ec963fb02400ecaa4d8d835c5d880a4204c7526a9002a83d41b3` |
| `target/native-examples/modules-core09` | 18,784 | `58f5e3bcf8da130a2077cf1b2e297c1d1d2d4cbc88080658a0929166826c39bd` |
| `dist/lobo-syntax-0.10.0.vsix` | 20,070 | `5dbfbc1cd53d11cc690c4ffe9d5556808b77cda598b7d3aa795aa4df88bc456b` |

Input SHA-256 values: `src/modules.rs`
`859ae0c379967381bba74aa6b29c62908e78b243ddaab0d53f19c344dbf9e416`;
`tests/modules.rs` `cdae80b18d6756f9bc21ee9385b32cb1abccebd108c1d8ceb52bef6ff6ab9631`;
`examples/modules/sequences.lobo` `c0a7cc64f2cf3ad722c0a5ff417cc02510460251a6ba0c1626c39b4fc1ebe6a5`;
`examples/modules/reverse.lobo` `d42fd9670de9a6133452a020b9294e99e5df68f65103e3b4dacedd9bcad32e0f`;
`examples/modules/mirror.lobo` `0ead2d707083b30c4d4218759be38e2ab44010daacc0e3e81ec3537ce1fb0a49`.

## Gates at the compiler-only checkpoint

The new three-module example shares one reverse function and cursor record. It is not yet the migration of
the existing collection/tree workloads. Their duplicated helpers must be consolidated while preserving
public entry identities, expected results, and resource behavior.

The extension maintains import highlighting and local alias navigation, not cross-file targets or dependency
diagnostics. Those need a bounded explicit source set, unsaved overlays, exact dependency attribution,
cancellation/stale-result checks, and trust enforcement without hidden workspace search or downloads.
These two gates and the final language coverage audit remain open. Local verification cannot establish
universal language replacement, an OS sandbox, or Turing completeness. No installation, publication, commit,
comparison-worktree mutation, or ecosystem expansion is authorized by this slice.

Follow-up on 2026-10-18: [decision 0023](0023-explicit-editor-source-snapshots.md) records the completed
explicit-project editor operations and actual collection/tree helper consolidation, plus the expanded
129-Rust/103-editor test evidence. The earlier incomplete checkpoint above is retained as history.
