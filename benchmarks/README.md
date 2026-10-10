# Execution comparisons

Run from a clean, committed checkout:

```sh
python3 scripts/compare.py
```

The runner builds the Rust/evaluator harness with the speed-optimized Cargo bench profile, emits C from the
same Core example, and compiles the native harness with Clang `-O3`. It compares native Lobo, equivalent Rust,
and the retained reference interpreter. No production implementation changes are needed to select modes.

Inputs vary through `[-1024, 1023]`; black-box barriers and checksums prevent trivial constant evaluation and
dead-code elimination. Every sample checks its checksum against independently calculated integer sums.
The runner alternates implementation order, warms up, and pins itself and its children to one available CPU
on Linux. It does not lock CPU frequency or prevent competing processes from running.

Native and Rust default to 10 million iterations per sample. The interpreter defaults to 100,000 because it
does substantially more work. Measurements are normalized per iteration; differing total checksums are
validated against their own iteration counts. Compilation, process startup, parsing, and output are outside
the timed region. Loop/checksum overhead remains included.

`Artifact.run` revalidates the program and constructs argument/context state on each interpreted invocation.
That is the existing public interpreter API; its result is not a measurement of tree walking alone. Native
Lobo includes host fuel and diagnostic bookkeeping; Rust includes mathematical operations, contracts, and
checked arithmetic, without equivalent host bookkeeping. The compiler may inline and eliminate proven checks.
These are scoped microbenchmarks, not a general speed ranking or matched-host application benchmark.

Reports contain the exact Git revision, dirty status, tool versions, affinity, raw samples, and checksum evidence.
They are generated under `target/comparisons/`, which is ignored by Git. Change sample counts with `--samples`,
`--warmup`, `--native-iterations`, and `--interpreted-iterations`; use `--cpu` to choose an allowed CPU.

See the [worktree workflow](../docs/worktree-comparisons.md) for comparing changes in isolated checkouts.
