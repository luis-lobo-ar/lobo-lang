# 0017: Persistent lists with explicit allocation accounting

- Status: implemented in compiler 0.6.0 / extension 0.7.0 (persistent-list slice)
- Date: 2026-10-17
- Risk mode: standard with memory/failure-focused adversarial gates; experimental local compiler, no deployment.

## Outcome and alternatives

Bounded inline records/variants cannot express variable-sized input collections or construct one element per
data-dependent iteration. That observed storage gap justifies one growable structure and a small native allocator
adapter, not a package system or external-resource authority. Follow [Core v0.6](../spec/core-v0.6.md).

Local alternative analysis (not independent prototypes):

- Immutable vectors with full copy on prepend/append preserve semantics but copy whole histories during folds.
- Mutable vectors plus unique ownership/borrowing need new aliasing/mutation/scope rules before ordinary traversal.
- Persistent lists share immutable tails and need only head/tail/length/prepend; keep this option for the first slice.

Choose invocation-scoped native nodes, with deterministic cumulative logical cell accounting and no refunds.
Reference nodes may use reference counting, but must avoid recursive destruction of long tails. Do not add a
collector, visible pointers, weak references, finalizers, or recursive named types. Arrays/text/bytes may need
different representations later; this decision does not prescribe a list representation for every collection.

## Failure and validation contract

Source/artifact type validation precedes lowering. Argument shape/node validation precedes heap allocation.
Native input parsing uses a validation pass then an allocation pass. All nodes are freed on success or fault;
only initialized, type-checked payloads are read. Budget zero admits empty lists, but not one node. Fuel meters
comparisons that traverse data. A bounded output preflight prevents shared nested values from expanding into
unbounded or half-written JSON. No host API or action grant is created by allocating local immutable values.

## Delivery gates

Define semantics and expected inputs before production code, observe failing tests, implement native/reference
paths, and adversarially verify bounds, ordering, lifetime, and failure phases under ASan/UBSan/leak detection.
Keep VS Code highlighting/navigation/diagnostics current, use a real extension host and package the extension.
Update the ledger with evidence and limitations. Native remains default. No dependency or publication is planned.
The [three workload fixtures](../spec/language-milestone-workloads.md) are frozen separately from slice tests.

## Implementation and adversarial findings

Native code uses invocation-scoped nodes, copied payload bytes (`memcpy` rather than type-punned storage),
and shared immutable tail pointers. Generated size/alignment assertions check payload admission widths.
Both success and `lobo_fail` exit paths run allocation/argument cleanup. The reference uses standard-library
Arc nodes and iterative tail destruction; a compile-time regression preserves Artifact/Value Send + Sync.
This preserves host-library usability, not a new Lobo concurrency construct.

TDD first observed three unsupported-syntax/type failures. Additional regressions exposed and corrected:

- Eight-level type nesting was rejected one level early; adjacent `>=` generic-close/initializer tokens needed
  contextual splitting. Nested `>>` types and the editor parser now agree.
- Counting output data nodes alone missed long repeated field metadata. The 8 MiB serialized Value byte
  preflight includes names/type metadata. Exactly 8 MiB succeeds; one byte more fails before output framing.
- Resource faults in contracts lost their outer phase in the reference path, and entry-time fuel faults lost
  their subject. Both paths now retain the corresponding contract phase/function attribution.
- Changing payload copying caught a C pointer-declaration regression for nested list elements; independent
  declarations fixed it. No padding-byte equality or payload type punning remains.
- TextMate constructor lookahead confused `::` with a field colon and swallowed contract coloring. Single-
  colon lookahead and the executable collection fixture caught/fixed it without weakening old scope oracles.

The three acceptance workloads were frozen before their implementation. Workload B now lives in
[collections.lobo](../examples/collections.lobo): first-invalid typed returns, ordered filtering/count/sum,
empty/boundary cases, and an odd-predicate variation, all computed in Lobo. Reverse is an ordinary reusable
Lobo function, not a compiler intrinsic. Parser A and tree C remain pending.

## Verification evidence

On 2026-10-17 (local timezone), the final working tree passed:

- `cargo test --locked` and `cargo test --locked --release`: 73 integration tests plus one native-launch unit
  test in each profile, including twelve list tests and all prior scalar/action/fold/record/variant tests.
- Reference, optimized native, and ASan/UBSan comparisons with leak detection enabled for list cases:
  nested inputs/outputs, exact heap and comparison-fuel boundaries, typed failures, contract phases,
  constructor/traversal behavior, serialization expansion, and cleanup after a fault following allocation.
- A 100,000-node reference tail/alias drop regression, host Send/Sync checks, malformed invocation/artifact
  rejection, cycle/type/arity checks, exact output-byte threshold, and collection workload variations.
- All 68 editor tests and a real VS Code extension-host completion marker with exit zero, including unsaved
  List diagnostics and navigation from nested elements through `head` to record fields. One earlier host run
  emitted a VS Code `Unexpected SIGPIPE` warning but completed successfully; a fresh rerun passed cleanly.
- Rust formatting, Clippy with warnings denied, JSON/JSONL/local Markdown links, and diff whitespace checks.
- Release build and VSIX packaging. Release smoke: `[5,2,8]` selects `[2,8]`, count 2, sum 10; a one-even-value
  invocation with eleven heap cells fails in `example.collections.reverse`, not with a partial result.

Environment: Linux x86_64, Rust/Cargo 1.92.0, Clang 18.1.3, Node.js 22.19.0; real VS Code used fresh temporary
profiles under Xvfb. No dependency, ordinary-profile extension installation, publication, commit, benchmark
rerun, or comparison-worktree mutation was made. Prior VSIX packages remain available.

Artifacts:

- `target/release/lobo`: 928,480 bytes, SHA-256
  `dc9761449f1011cfa3e663337e000f913aa445ac6aa12cc4e5651d6ae59c78e5`.
- `dist/lobo-syntax-0.7.0.vsix`: 18,261 bytes, SHA-256
  `e2bbfdeb4088509ccf51c7510cc58b7ff3080ab4fb041e1f966c658aed98cca4`.

## Remaining boundary

This is not completion of the language-first goal. General text/bytes, parser/tree acceptance programs,
recursive named types, and broader composition/reuse still need decisions and evidence. Invocation-scoped
retention may be unsuitable for long-lived applications; cumulative heap cells deliberately do not refund
unreachable values. Limits do not bound allocator overhead/total process memory or stop an OS OOM killer.
Sanitizers/finite fixtures are not a safety proof, and native execution is not an OS sandbox. No network,
storage/search library, FFI, package ecosystem, daemon, or host-resource authority is added.
