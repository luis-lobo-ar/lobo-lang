# Initial native measurements

Date: 2026-10-10. Compiler: Lobo 0.1.1. Platform: Linux x86_64, AMD Ryzen 9 5900HX.
Toolchains: Rust 1.92.0 and Ubuntu Clang 18.1.3.

## Executable sizes

| Executable | Size in bytes | Purpose |
| --- | --- | --- |
| `lobo` release compiler | 654,888 | Parse, check, generate native code, and expose reference tools |
| Compiled `examples/core.lobo` | 14,600 | Native increment/absolute functions and invocation/check adapter |

Both are stripped ELF executables using system libraries. The application contains compiled code rather than
the compiler or evaluator. The earlier 544,376-byte figure described the previous compiler executable, not an
executable generated from a Lobo program.

## Native execution microbenchmark

The comparison invokes the generated increment and absolute functions in-process, excluding compilation,
file loading, argument parsing, JSON output, and process startup. Both paths use changing inputs, black-box
barriers, and a checksum to prevent dead-code elimination. Clang and Rust optimize at level 3; functions may
inline. Each measured sample has 10 million invocations. Two warm-up pairs precede nine measured pairs, with
the implementation order alternating. Input is `(iteration & 2047) - 1024`.

| Workload | Lobo median ns/iteration | Rust median ns/iteration | Lobo/Rust |
| --- | --- | --- | --- |
| Increment | 4.62 | 0.81 | 5.7x |
| Absolute value | 4.53 | 1.01 | 4.5x |

Each iteration includes loop and checksum overhead. The Lobo path also resets the host fuel and call-depth
state and includes native fuel/diagnostic bookkeeping. Rust implements the same mathematical operation,
precondition, postcondition, and checked arithmetic, without Lobo's fuel and diagnostic bookkeeping. Some
checks can be removed when the optimizer proves them redundant.

Checksums agree: increment `4680512`, absolute `5119877312`. Lobo's observed ranges were 4.07–5.66 and
4.11–6.36 ns/iteration; Rust's were 0.70–1.00 and 0.96–1.06. CPU frequency and system load were not locked.
These measurements demonstrate that the generated code runs natively and reveal current overhead on two tiny
functions. They do not establish a general language-speed ratio, standalone process latency, or application
throughput. Larger workloads and matched host policies are needed for those claims.

The scratch harnesses for this run are at `/tmp/lobo-native-bench.hRt5aE` (`core.c`, `bench.c`, `bench.rs`, and
`measure.py`). That temporary path is execution evidence, not a permanent benchmark suite. Native/reference
correctness remains covered by the repository tests.

