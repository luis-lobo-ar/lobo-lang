# Execution comparisons

## C versus direct LLVM native backend

Compiler 0.11.1 exposes both routes with unchanged source semantics. After a release build, run
`python3 benchmarks/llvm-backend.py --out NEW_DIRECTORY` for public workloads, independent expected
outputs, alternating repeated builds/runs and raw tool/source/artifact identities. See
[the recorded comparison](results/2026-11-04-llvm-backend-verification.md). Runtime includes process startup,
parsing and JSON. This is not an agent construction/comprehension or language-productivity benchmark.

## Agent-first verification prototype

The separate testing-workflow experiment is not a language execution-speed comparison:

```sh
cargo build --locked --release --bin lobo-test
python3 benchmarks/agent-testing.py
```

It compares affected-mode verification after a passing baseline with forced-full native verification on
the same 64 checks and six seeded edit scenarios. It inspects every case's retained outcome, not just
the first eight failures shown in compact summaries. Source/case/runner identities, execution counts,
response bytes, tool metadata and local wall times are retained in
[the prototype report](results/2026-10-18-agent-testing-prototype.json).
This is one scripted run per scenario, not measured LLM token savings, repair success, universal dependency
soundness or a controlled hardware benchmark. Both modes use the same runner/compiler and compact protocol;
output-size comparisons do not establish an advantage over other frameworks. See
[the testing protocol](../spec/agent-testing-v0.1.md) for reuse assumptions and remaining gates.

## Real-agent repair pilot

The next experiment uses actual Codex CLI repair attempts on five seeded bugs, paired across
conventional full-run text and structured affected-mode feedback. Both use the same native backend
and visible checks. Read [the registered protocol](../decisions/0025-real-agent-testing-pilot.md)
before interpreting the results. This is a feedback/selection comparison, not a benchmark against
pytest, Jest or Cargo, and five heterogeneous pairs cannot establish general productivity gains.
See the [pilot analysis](results/2026-11-03-agent-repair-pilot.md) and
[ten recorded trials](results/2026-11-03-agent-repair-pilot.json).

```sh
cargo build --locked --release --bin lobo-test
python3 -m unittest discover -s benchmarks -p test_agent_repair.py -v
python3 benchmarks/agent-repair.py --canary
python3 benchmarks/agent-repair.py --run
```

`--canary` uses one excluded repair to check the transport. `--run` makes exactly ten sequential
model trials using the existing authenticated CLI and quota. It does not log in, install anything,
publish, or copy credentials. Each trial has a 90-second agent deadline and independent native
final grading; raw JSONL, final sources, usage, test results and the frozen harness/protocol are retained
in a private directory under ignored `target/agent-repair-*`. The controller uses fresh temporary
task directories, cleans those directories when each trial finishes, and keeps its fixed criteria
outside the editable workspace. Do not run other benchmark/test builds concurrently with the pilot.

The file mailbox under `.check-io` preserves one foreground test session across shell calls under
workspace-write sandboxing. It is an experiment adapter, not a public persistent testing service.
Snapshot retention and other prototype limits still apply. Final grading forces fresh execution
of visible and additional signed/boundary inputs, with mathematical expectations computed outside
the candidate. Test-client/guide edits, unexpected files and symlinked sources reject acceptance.
The sandbox and integrity audit do not establish containment against a hostile same-user agent.

## Repeated library-repair study

See the [completed findings](results/2026-11-03-library-repair-study.md) and
[all 30 records](results/2026-11-03-library-repair-study.json). Twenty-nine repairs passed;
one controller interruption remains an infrastructure failure without replacement.

[Decision 0026](../decisions/0026-repeated-library-repair-study.md) registers two larger repair
tasks in a five-source project, five repetitions each, and three arms: concise full-run text,
full-run JSON and affected-mode JSON. This separates representation from selection. Each task
seeds three defects across two files; independent native final grading remains mandatory.

```sh
python3 -m unittest discover -s benchmarks -p 'test_agent_*.py' -v
python3 benchmarks/agent-repair-study.py --canary
python3 benchmarks/agent-repair-study.py --run
```

The two canaries are excluded. The registered run makes exactly 30 sequential CLI conversations
using existing authentication/quota, with 120-second agent deadlines and explicit failure records.
All artifacts, frozen inputs, raw JSONL, repaired sources, usage, host timings and exploratory
paired analysis live privately under ignored `target/agent-study-*`. Run other suites/builds after
the study, not concurrently. No model weights, CPU frequency or competing host tasks are pinned.

The experiment adapter uses unique per-request file messages so simultaneous list/detail/replay
commands cannot overwrite each other. One consumer serializes operations on the native session.
Requests are bounded and response tokens correlate each client with its result. This supersedes
the earlier pilot mailbox for new experiments; the historical pilot harness stays unchanged for
attribution. It is not a public daemon, persistent evidence store or hostile-agent sandbox.

All arms offer the same list/detail/replay commands. Actual usage is measured rather than forced.
Multiple seeded defects create an opportunity for multiple repair candidates; agents may still
repair them in one edit. Report that behavior rather than claim unexercised workflow benefits.
Paired bootstrap intervals are exploratory: five repetitions per task and two related workloads
do not establish general agent productivity or superiority over existing test frameworks.

The SIGTERM interruption and specific recovery are recorded in the findings; the historical
recovery script refuses a different prefix or changed inputs. It is not a general resumable
service. Export preserves raw counters and corrects a printed-source timeout false positive;
interrupted elapsed time/usage remain unknown.

## Lobo skill comparison

[Decision 0028](../decisions/0028-lobo-skill-onboarding-comparison.md) compares the repository
skill with equivalent flattened knowledge on two new tasks, five repetitions and two arms.
See [findings](results/2026-11-03-skill-evaluation.md), [all data](results/2026-11-03-skill-evaluation.json)
and [trace audit](results/2026-11-03-skill-evaluation-audit.json). All twenty attempts passed;
no accuracy or consistent speed/token advantage was demonstrated. Three scope probes remain flagged.

```sh
python3 -m unittest discover -s benchmarks -p 'test_*.py' -v
python3 benchmarks/skill-evaluation.py --canary
python3 benchmarks/skill-evaluation.py --init
python3 benchmarks/skill-evaluation.py --directory PRINTED_DIRECTORY --block 0
```

Run blocks 1 through 4 sequentially against that same directory; each block makes four model
attempts using existing authentication/quota. Fixtures and identities are checked, outcomes are
atomic, and started attempts cannot be replaced. Private evidence is outside build cleanup under
ignored `benchmarks/private-evidence/`. Run builds and other suites after measurement.
Export and audit commands are in the findings. Re-exporting existing data makes no model calls.

## Language execution comparisons

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

The first clean, pinned three-way run is saved in
[the baseline report](results/2026-10-10-execution-baseline.json). See
[the interpretation of those measurements](../docs/native-performance.md) for scope and limitations.
