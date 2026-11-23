# Direct LLVM implementation and comparison

Date: 2026-11-04. Compiler 0.11.1; unchanged Core/artifact 0.11, native adapter 0.6 and extension 0.13.0.
User-authorized implementation, not publication or an agent trial. See [decision 0031](../../decisions/0031-direct-llvm-native-backend.md)
and [the backend contract](../../spec/native-llvm-v0.1.md).

Application arithmetic/control flow, calls, matching, iteration, contracts and fuel checks now emit LLVM
directly. The C runtime and typed value/layout primitives compile to another module. LLVM links both before
native optimization/code generation. There is no C translation of application bodies or runtime expression
interpreter in this route. C remains default; LLVM is explicitly selectable and never an implicit fallback.

## Verification

| Gate | Observed result |
| --- | --- |
| Complete Rust suite | 163 passed, zero failed/ignored |
| Shared native fixtures | Every `Builds` native invocation compares C with optimized and ASan/UBSan LLVM, including values/faults/resources |
| Independent generated oracles | 100 fixed-seed cases and algorithm boundaries exercise evaluator, reloaded artifacts, C/sanitized C and LLVM/sanitized LLVM |
| LLVM-specific tests | Four pass: native ELF/CLI/IR verification, arithmetic/order/phase/fuel, active sanitizer probes, tool/selector failures and no fallback/publication |
| Python suite | 30 passed |
| Editor / real VS Code host | 114 passed; host exit 0 with `LOBO_HOST_TESTS_PASSED` |
| Strict Clippy / formatting | Passed |
| Skill | Validator passes; counter/framing/folder/generic Text examples run correctly on both backends |
| Release / actual artifacts | Optimized compiler built; delivered LLVM executable runs and linked `.ll` passes `opt-18` |

No source syntax or check-envelope change requires extension runtime changes or a version bump. No user-profile
installation occurred. The JSONL verification-session prototype retains its C backend/cache contract; LLVM is
available through compiler CLI/library APIs. LLVM verification checks IR validity, not semantic equivalence.

The delivered sample is `target/native-examples/generics-llvm-core0111`, **22,896 bytes**; its LLVM source is
that path plus `.ll`. The [baseline](2026-11-04-core011-before-llvm-baseline.json) preserves the prior dirty-workspace
compiler/test runner/source archive. C emission is byte-identical to that compiler for all four measured sources.

## Performance comparison

`python3 benchmarks/llvm-backend.py --out NEW_DIRECTORY` pins itself/children to one allowed CPU, alternates
pair order and performs six native builds and eleven timed invocations per backend/workload. Each binary
warms once. All outputs must match independent fixed expectations. Exact inputs, commands, tool/source/compiler/
binary hashes, ranges, CPU usage and observations are in [the final comparison](2026-11-04-llvm-backend-comparison.json).

| Public workload | C compile median | LLVM compile median | C invocation median | LLVM invocation median | C / LLVM bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Five-million-iteration sum | 142.765 ms | 181.966 ms | 9.019 ms | 9.070 ms | 14,664 / 14,664 |
| Parse/evaluate 2048 ones joined by `+` | 467.939 ms | 553.412 ms | 1.213 ms | 1.298 ms | 47,480 / 47,480 |
| Recursive AST rewrite/evaluation | 219.735 ms | 286.845 ms | 0.643 ms | 0.635 ms | 22,888 / 22,896 |
| Generic UTF-8 identity | 237.118 ms | 299.975 ms | 0.628 ms | 0.661 ms | 22,888 / 22,896 |

Two retained rounds show the same pattern, with zero failed benchmark observations. Each round contains
144 valid observations: 48 compile timings, 88 native timings, and eight excluded runtime warmups. The final
round uses the delivered compiler with completed CLI help. Both frozen compilers/harnesses are retained.

No runtime improvement is demonstrated. Ranges overlap, three cases approach the startup floor, and the host
was busy. Medians are not confidence intervals or equivalence proofs. The implemented LLVM route compiled
18–31% slower here with no size advantage. Keep C as default while retaining LLVM for further backend work.
This does not show an intrinsic LLVM limitation or measure agent construction/comprehension/token cost.

## Measurement limits and profiling

Excluded profiling runs check a fifty-million-element sum against its independent expected value. `time -v`
reports one-core user CPU work with negligible system time. LLVM syscall tracing shows 55 calls rather than
an I/O operation per iteration. Assembly/text-section hashes are retained. The inferred runtime limiter is
the checked arithmetic/fuel loop. Tiny AST/generic invocations mainly measure startup, parsing and JSON.
Hardware `perf` counters were denied on both sides; cycles/instruction counts remain unknown. No host
permissions or system settings changed.

Process traces confirm C uses compiler/Clang/cc1/linker; LLVM adds runtime-IR emission and module linking before
final Clang/cc1/linker. Their individual timing shares were not measured. Both routes use the same Clang 18
build, final `-O3`, stripping/section collection, source, budgets and expected outputs. Production LLVM has
no sanitizer attributes. No source, criterion or optimization changes occur between sides within a round.

Compile timing covers the full CLI source-to-published-executable path. Native timing includes startup,
argument parsing and JSON output. Tool/filesystem caches were warm; no cold-cache claim. Other host jobs,
CPU frequency and scheduling remained uncontrolled despite affinity. Small runtime differences are
inconclusive. These public fixtures are not unseen paper evaluation tasks or H1/H2/H3 agent experiments.

## Retained failures and corrections

1. TDD red: `--backend llvm` initially failed with `invocation.cli.invalid`.
2. A test compared evaluator diagnostic prose with native output, which omits it. Compare specified fields;
   remove only `diagnostic.message` from the reference oracle. C/LLVM outputs are compared directly.
3. A constant out-of-bounds ASan probe was optimized away. A one-cell array with runtime offset also collapsed
   to its sole valid cell. Retained original/optimized IR shows this. A two-cell array with runtime offset
   preserves the invalid access; the active probe detects stack-buffer-overflow. A misalignment probe confirms
   C-runtime frontend UBSan. These are corrected probes, not ignored failures.
4. One run hit `No space left on device`, followed by host Rust linker write/crash failures. Removed only
   `target/debug/incremental` (about 3.9 GiB of regenerable cache); sources, releases, baselines and evidence
   remain. Subsequent focused/full runs pass. This was a host-storage failure, not an LLVM application fault.
5. Two atomic patch attempts failed on stale contexts, changing no files. The tool transcript retains them;
   no separate filesystem raw log was created.
6. Hardware profiling permission failures remain unavailable results. An initial process-trace summarizer
   missed interleaved `execve` completions; its correction follows resumed calls without changing original
   traces. No timed result depended on that summary.

Raw command logs and ineffective probe/optimized IR are under ignored
`benchmarks/private-evidence/llvm-evolution-20261104/`. Both comparison directories retain full results,
native binaries, frozen compilers/harnesses and profiling evidence. The [verification manifest](2026-11-04-llvm-backend-verification.json)
pins their identities and exact commands. Private local evidence still needs backup; it is not publication
or independent archiving. No new dependency installation, model trial, commit, publication or profile install.
