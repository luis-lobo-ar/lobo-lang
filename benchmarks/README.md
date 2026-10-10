# Execution comparisons

Run from a clean, committed checkout:

```sh
python3 scripts/compare.py
```

The runner builds the Rust/evaluator harness with the speed-optimized Cargo bench profile, emits C from the
same Core example, and compiles the native harness with Clang `-O3`. It compares native Lobo, equivalent Rust,
the retained reference interpreter, Python, and Node.js. Python and Node use only their standard libraries.
No production implementation changes are needed to select modes.

For 100 measured samples of one-call, 100-call, and sustained batches:

```sh
python3 scripts/compare.py --suite --samples 100 --summary
```

Inputs vary through `[-1024, 1023]`; black-box barriers and checksums prevent trivial constant evaluation and
dead-code elimination in native/Rust code. Python/Node consume varying inputs and accumulate a checksum;
their optimizers are not constrained by a native black-box barrier. Every sample checks its checksum against
independently calculated integer sums. Python/Node boundary self-tests also verify contract/overflow failures.
The runner alternates implementation order, warms up, and pins itself and its children to one available CPU
on Linux. It does not lock CPU frequency or prevent competing processes from running.

Native and Rust default to 10 million iterations per sustained sample. The interpreter, Python, and Node
default to 100,000 because they do substantially more work. Measurements are normalized per iteration; differing total checksums are
validated against their own iteration counts. Compilation, process startup, parsing, and output are outside
the timed region. Loop/checksum overhead remains included. Each sample runs in a fresh process. Two discarded
sample processes help prime OS caches, but cannot warm the JIT in later processes. Sustained Python/Node
samples execute 10,000 untimed calls within each process before measurement; this is a fixed warmup budget,
not proof that the JIT has reached a steady state. One-call and 100-call samples have no in-process warmup.
Their medians include timing/dispatch overhead and first-use effects, so use total batch time rather than
treating them as steady-state function latency.

Python uses arbitrary-precision `int` with explicit I64 bounds checks. Node uses `BigInt`, not `Number`, to
preserve the full signed 64-bit domain; this is not a benchmark of JavaScript floating-point arithmetic.
Both preserve pre/postconditions and wrap only the unsigned checksum. Input conversion, checked arithmetic,
and checksum costs are included. Timing uses Python's
[`perf_counter_ns`](https://docs.python.org/3/library/time.html#time.perf_counter_ns) and Node's
[`hrtime.bigint`](https://nodejs.org/api/process.html#processhrtimebigint).

`Artifact.run` revalidates the program and constructs argument/context state on each interpreted invocation.
That is the existing public interpreter API; its result is not a measurement of tree walking alone. Native
Lobo includes host fuel and diagnostic bookkeeping; Rust includes mathematical operations, contracts, and
checked arithmetic, without equivalent host bookkeeping. The compiler may inline and eliminate proven checks.
These are scoped microbenchmarks, not a general speed ranking or matched-host application benchmark.

Reports contain the exact Git revision, dirty status, tool versions, affinity, raw samples, and checksum evidence.
They are generated under `target/comparisons/`, which is ignored by Git. Change sample counts with `--samples`,
`--warmup`, `--native-iterations`, `--interpreted-iterations`, `--dynamic-iterations`, and `--dynamic-warmup`;
use `--cpu` to choose an allowed CPU. `--summary` keeps terminal output compact without dropping raw samples
from the saved JSON. Format 0.2 groups results under `regimes`; historical format 0.1 reports are unchanged.

See the [worktree workflow](../docs/worktree-comparisons.md) for comparing changes in isolated checkouts.

The first clean, pinned three-way run is saved in
[the baseline report](results/2026-10-10-execution-baseline.json). See
[the interpretation of those measurements](../docs/native-performance.md) for scope and limitations.

The five-way, 100-sample suite is saved in
[the raw report](results/2026-10-10-five-way-100-samples.json), with
[small-batch and sustained results](../docs/five-way-performance.md).
