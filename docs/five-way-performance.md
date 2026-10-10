# Five-way comparison: 100 measured samples

Measured 2026-10-10 from clean source commit `f14fb8382a15eabdc34e17750e091e680b84e2dd`, on branch
`compare/python-node` in `.worktrees/python-node`. Linux x86_64, AMD Ryzen 9 5900HX, pinned to CPU 0.
Rust 1.92.0, Clang 18.1.3, Python 3.12.3, Node.js 22.19.0. CPU frequency and competing system load were not locked.

```sh
python3 scripts/compare.py --suite --samples 100 --summary
```

Each of two workloads has three regimes and five implementations: 30 groups of 100 measured samples,
or 3,000 measured samples total. Each group also has two discarded sample processes. All checksums matched
the independent oracle. The [raw report](../benchmarks/results/2026-10-10-five-way-100-samples.json) contains
all batch timings, counts, checksums, environment versions, and the measured source identity.

## Small batches

Median total time in microseconds, excluding process startup. These are cold execution batches, not isolated
function latency: timing, loop, dispatch, checksum, and first-use costs remain included. There is no in-process
warmup in these regimes. One-call input is -1024; 100-call inputs are -1024 through -925, so both workloads
have only negative inputs in the small regimes.

| Implementation | Increment: 1 call, us | Increment: 100 calls, us | Absolute: 1 call, us | Absolute: 100 calls, us |
| --- | --- | --- | --- | --- |
| Native Lobo | 0.201 | 0.6165 | 0.211 | 0.621 |
| Rust | 0.211 | 0.310 | 0.220 | 0.330 |
| Interpreted Lobo | 4.519 | 119.904 | 4.629 | 118.3415 |
| Python | 2.344 | 28.478 | 1.954 | 20.288 |
| Node.js (BigInt) | 31.4835 | 66.2785 | 31.1785 | 61.936 |

Do not multiply the one-call measurement by 100 to predict the 100-call batch. Timing overhead is paid once
per batch, and cold dispatch/JIT effects can be amortized. Nor do these results predict command-line startup
latency, which is outside the timed region.

## Sustained batches

Median nanoseconds per call, including loop/checksum overhead. Native and Rust use 10 million calls per sample;
the other implementations use 100,000. Python and Node perform an additional 10,000 untimed calls within each
fresh sample process. This fixed warmup does not guarantee that all JIT compilation has finished. Inputs cycle
through -1024 to 1023. Total batch times differ with the iteration budget; normalized times are shown below.

| Implementation | Increment, ns/call | Absolute, ns/call |
| --- | --- | --- |
| Native Lobo | 4.10 | 3.96 |
| Rust | 0.95 | 0.98 |
| Interpreted Lobo | 1235.38 | 1197.59 |
| Python | 261.65 | 149.71 |
| Node.js (BigInt) | 22.34 | 24.90 |

For these tiny workloads, native Lobo takes about 4.1-4.3 times Rust's time per call, but is roughly 302 times
faster than the reference interpreter, 38-64 times faster than Python, and 5.5-6.3 times faster than Node.
These ratios describe this harness, not general language or application performance. For example, native
increment samples ranged from 4.03 to 8.15 ns/call, and Node increment samples from 20.61 to 59.16 ns/call.
The report preserves those distributions rather than hiding variability behind the medians.

Python uses `int` with explicit I64 overflow checks. Node uses `BigInt`, not floating-point `Number`. All paths
retain the Core mathematical operation and contracts; optimizers may remove checks they can prove redundant.
Rust/native barriers inhibit trivial constant folding; Python/Node lack equivalent barriers. Conversion and
unsigned-checksum costs are included in the dynamic-language paths. Native Lobo includes fuel/context
bookkeeping absent from Rust. The interpreter's public `Artifact.run` revalidates the program and allocates
argument/context state on every call; it is not a tree-walking-only benchmark.

Verification: Python/Node boundary and contract self-tests; a five-way smoke run across an input-cycle boundary;
all 10 Cargo integration tests; Cargo formatting and Clippy; JSON/JSONL/local-link checks; and all measured
checksums. No compiler or language semantics were changed. See the [harness documentation](../benchmarks/README.md)
for implementation details and [worktree workflow](worktree-comparisons.md) for source isolation.
