# Native boundary models and C witnesses

Two finite TLA+ models check the synchronous native boundary. Compiler 1.7.1 fixes a pending-descriptor
fault-cleanup defect reproduced from the operation model. Language/ABI remain core/1.7 and ABI 1.
The [verification skill](../../.agents/skills/lobo-native-verification/SKILL.md) defines the maintenance loop.
[Coverage fingerprints](coverage.json) identify the dirty working-tree inputs beyond the recorded Git HEAD.
There is no mechanical proof that the C implementation refines the models.

## One-command check

With Java 11+, Clang and the installed TLA tools 1.7.4 jar:

```sh
TLA2TOOLS=/path/to/tla2tools.jar python3 scripts/check-native-models.py --out target/native-model-check-NEW
```

Use a new output directory. This explicitly supplied jar is pinned to SHA-256
`936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88` and TLC 2.19 rev 5a47802.
Its SHA-1 also matches the official release's `bee4a54f3ee3d4afc347c3240ec2d9e93b075104`.
No tools are downloaded. Preflight errors exit 2; failed/inconclusive verification exits nonzero; acceptance
requires the structured results.json status pass. Each command has separate JSON/stdout/stderr evidence.
Baseline models must finish with no queued states. Intended mutations must violate their named invariant;
C mutations must compile and fail the corresponding witness in both builds. Crashes or build failures are
not accepted as the expected mutation outcome. The runner never modifies production source. Acceptance uses explicit checks that remain active under
Python -O. Its negative self-test runs real TLC on a deliberately broken baseline in a scratch copy and
requires the checker to reject it with optimization enabled. The diagnostic --models-only option skips the
C/mutation and checker-self-test stages and does not constitute the full acceptance gate.

Ordinary regression coverage is also available without TLC:

```sh
cargo test --locked --test native_formal --test native_adapter
```

## Bounds and properties

[Authority.tla](Authority.tla) uses three registry slots, generations 1..2, a live-handle limit of three,
five resource-operation calls and four permission atoms. It checks root acquisition only at entry, subset-only
delegation, generation/kind checks, denial before OS access, handle quota, retirement and exit cleanup.
The quota mutant uses a limit of one to expose a quota smaller than registry capacity. Slots are abstract
identities; allocation can choose any free slot, while C chooses the first free slot. Logical slot renaming
in witnesses preserves resource identity. Numeric token encoding and sampled forged tokens are checked by C tests. Neither these tests nor
this model exhaust all 64-bit token patterns. Capabilities remain allocated until invocation teardown; close
and cleanup apply to listener/stream slots. The authority model checks safety, not progress of arbitrary
application calls, which can legitimately continue until source fuel exhaustion.

[Operations.tla](Operations.tla) uses four abstract fuel units, payload length two, six reserved-byte units
and a two-tick timeout. It separates pending socket acquisition, connection outcome, peer validation,
publication, IO reservation, short transfer, interruption, shutdown/EOF and cleanup. A new IO operation gets
a new deadline; retries keep that deadline. Reservations charge the requested length, not the short result.
It checks validated publication, deadline stability, exclusive pending/registered ownership, byte conservation,
EOF after drain and cleanup. The historical untracked-fault variant starts with zero fuel to expose the
shortest fault window. The abstract fuel is a finite progress budget, not an exact count of compiler ticks.

The selected driver admits positive IO requests no larger than its remaining byte quota and models a quota
failure at zero. Requests exceeding a positive remaining quota and other invalid scalar inputs remain in
concrete tests. The model describes a bounded driver that exits on timeout, OS failure, quota failure, fuel fault or drained
EOF. Individual Lobo native calls normally return a result to their caller. The driver assumption is explicit;
the model does not prove eventual termination of every possible Lobo program. Weak fairness of Next excludes
infinite artificial stuttering. Modeled syscalls return, successful short IO respects FIFO byte counts, and
closing an owned descriptor consumes it under the selected Linux contract. Time can remain unchanged or
advance one abstract tick per retry. Large clock jumps, contents, arbitrary addresses, signals/process kill,
thread interleavings, malicious native code, random entropy quality, pointer validity and ABI layout are
outside these state models. Fuel/byte/handle bounds do not establish a cutoff theorem for larger domains.

## Source mapping and atomic steps

| Model transitions | Actual provider code / boundary |
| --- | --- |
| Root / Derive | lobo_native_capability_v1 / lobo_native_restrict_capability_v1; entry depth, cached root and mask subset checks |
| Allocate / Use | lobo_native_insert_v1 and lobo_native_lookup_v1/begin_v1; abstract successful OS acquisition, kind/generation and live quota |
| Release | lobo_native_close_v1 and lobo_native_release_v1; consume descriptor, increment generation or retire slot |
| Exit / Finished | lobo_fail/atexit and lobo_cleanup/lobo_native_cleanup; cleanup bypasses exhausted budgets |
| Acquire / Connect / Validate | connect_v1 socket acquisition/pending ownership, syscall outcome, SO_ERROR/getpeername validation and socket_result_v1 transfer into registry |
| RejectPeer / Timeout / OsFailure | explicit native error-return paths close unpublished descriptors; published resources remain registered until driver exit |
| BeginIO / Write / Read | read_v1/write_v1 reservation before IO, nondeterministic short return; pure source driver loops between calls |
| Retry / Fault | wait_v1 absolute deadline and lobo_spend; EINTR/spurious readiness can consume fuel without advancing time |
| Shutdown / Eof | shutdown_write_v1 and positive-count empty read after peer drain; zero-length reads are outside this model |
| Cleanup | pending descriptor plus registry cleanup before arena release; kernel process teardown is distinct |

The synchronous acquisition-to-publication path owns at most one pending descriptor. Acquisition and its
assignment to the pending owner are one model step because no language fault/fuel check occurs between
those two C statements. Publication and pending-owner clearing likewise contain no intervening fault point.
No concurrency is invented. A future callback/thread/signal policy requires reviewing these atomicity claims.

## Counterexample-to-C mapping

[mutations.json](mutations.json) pairs each broken model with a named invariant, real-source mutation and
concrete witness in [formal_trace_harness.c](../../tests/native/formal_trace_harness.c). Model counterexamples
are preserved as action sequences plus raw variable states in counterexample.json. The C translations are
handwritten representative witnesses of those same transitions, not a generic interpreter of arbitrary TLC
traces. The explicit value abstractions below prevent claiming automatic trace refinement.

| Broken rule | Concrete witness / translation |
| --- | --- |
| EntryAuthority | Acquire root; request cached root at depth two; expect authority fault. Fresh-root and cached-root failures share this guard. |
| NoAmplification | Acquire root, derive empty parent, request clock permission in its child; expect Denied. The chosen permission atom represents a requested missing bit. |
| ValidGeneration | Insert real socket in registry, close it, reuse slot, read through old token; expect InvalidHandle. Root/listener setup and logical slot numbers are translated. |
| ValidKind | Obtain capability and use it as a stream; expect WrongKind before IO. Stream is a representative mismatched requested kind. |
| NoDeniedSyscall | Derive empty capability and call listen under a global listen grant; expect Denied with no socket syscall. |
| WithinQuota | Root fills handle limit one; attempt listener allocation; expect Limit and no leaked fd. |
| NoGenerationWrap | Seed slot at the real 56-bit maximum, close, then allocate; retired slot cannot be reused. Model generation two maps to the real maximum. |
| ExitCloses (authority) | Open listener, exhaust calls/fuel, cleanup twice; fd count returns to baseline. |
| ExitCloses (operations) | Acquire pending connect socket, fail first wait with zero fuel and zero remaining native calls, then cleanup; fd count must return to baseline before process teardown. |
| ValidatedPublication | Inject EINTR without a connection; writable/SO_ERROR is insufficient; expect rejection and no published stream/fd leak. |
| FixedDeadline | Inject repeated EINTR with advancing clock; expect Timeout under the original deadline, not a reset deadline or fuel exhaustion. |

Each isolated C mutation removes the corresponding real guard/ownership operation in a copied source tree.
Both optimized and ASan/UBSan witnesses must reject it. This demonstrates sensitivity of the selected checks;
it does not prove every model property for every C execution. Structural TypeOK, ownership, byte accounting,
EOF and liveness checks have no separate one-to-one mutation claim. Existing native IO/ABI suites supplement
the selected witnesses. Inspect each trace and source mapping after changes; a model-only pass can drift.

## Charts, provenance and results

The view modules render reader-facing labels with the `/statechart` skill. The authority chart configuration
uses only two slots, one generation, two calls and a listen-only root; it illustrates control flow and does not
replace the larger acceptance run. IO charts can similarly use explicitly reduced bounds. Charts hide steps
that remain within one reader-facing label; raw TLC output retains the underlying states.

Tool provenance: [official TLA tools 1.7.4 release](https://github.com/tlaplus/tlaplus/releases/tag/v1.7.4),
[official TLC model configuration](https://dl.tlapl.us/tlatoolbox/doc/model/overview-page.html), and
[coverage/result guidance](https://dl.tlapl.us/tlatoolbox/doc/model/results-page.html). TLC runs with one worker,
fixed seed 1 and fingerprint polynomial 0. Fingerprint collision estimates remain in raw output. These are
finite model checks, not TLAPS proofs. The [result report](../../benchmarks/results/2026-11-22-native-formal-verification.md)
retains actual counts, red/green evidence, corrections and unknown research outcomes.
