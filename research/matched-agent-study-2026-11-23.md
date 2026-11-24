# Frozen Lobo 1.11.2 matched agent study

Date: 2026-11-23. Status: completed exploratory pilot; all original failures and unknowns retained.

This study asks whether an agent can construct correct software, understand a supplied program and
preserve correctness through a requirement change when given Lobo, Rust or Python. The primary outcome
is satisfying the declared checks within a common working budget. Actual tokens and elapsed time are
cost outcomes. Program execution performance is measured separately.

## Frozen inputs and registered design

Compiler/extension 1.11.2, Core 1.11, are frozen at
`75a551fc0aec1fbc94f1fddb2a0f6a59ddb3f122`, tag `research/lobo-1.11.2`. The source archive and compiler
hashes are in [the preflight receipt](../benchmarks/results/2026-11-23-matched-study-preflight.json).
This work changes no language, compiler, maintained library, native provider or editor behavior.

[Decision 0063](../decisions/0063-frozen-matched-agent-study.md) and
[the protocol](../benchmarks/matched-study/README.md) define the comparison. The
[original registration](../benchmarks/results/2026-11-23-matched-study-registration.json) and
[amended registration](../benchmarks/results/2026-11-23-matched-study-registration-amendment-1.json)
retain exact task, guide, source and harness hashes. All task/guide/source and schedule hashes agree
between registrations. Only the controller's infrastructure audit and accounting changed.

The amended cohort schedules 54 fresh conversations: three task families, three languages, three phases
and two repetitions. Model alias `gpt-6.1-sol`, high reasoning, CLI 0.171.0; one shared host and account.
Every attempt has 120 seconds, sixteen command-execution tool items and six visible check submissions.
Build/check work counts in time. The CLI cannot enforce a hard token cap; actual usage is recorded.
Provider-managed cache affects usage accounting. These are equal time/tool/check opportunities, not equal
consumed tokens or a causal comparison holding prior language familiarity constant.

| Family | Construction | Requirement change |
| --- | --- | --- |
| Ledger | Credits/debits, locking, rejection counts, peak and per-event audit | Undo successful balance changes, retaining historical peak and correct lock/history rules |
| Scheduler | Dependency readiness, affordable jobs, deterministic ID ordering, cycles and zero costs | Priority among eligible affordable jobs, with deterministic tie-breaking |
| Byte frames | Length/checksum parsing, truncation and error precedence/offsets | Padding and two-byte little-endian extended lengths |

Construction receives compile-valid incorrect stubs. Change trials start independently from a common
correct base and updated DTO, never from construction survivors. The pure algorithm and Lobo identity
sidecar are editable; model, adapter and utility sources stay fixed. A host mailbox snapshots and compiles
each visible submission, and independently grades the final candidate against held-out observations.

Comprehension receives the full base algorithm/model/utility and contract, with execution and tools
disabled. Six output predictions and five witness/call/impact/predicate/counterexample items are graded
with strict JSON types. Some predictions can be solved from the contract alone, and the predicate-location
item is cued in the question. These are not blinded fault localization or repository-scale comprehension
measurements. Supplied references necessarily differ in idioms and representation across languages.

Guides contain 3443 UTF-8 bytes for Lobo, 1986 for Rust and 1922 for Python. They are language-specific,
not padded or tokenizer-matched. Lobo's guide includes state loops, persistent lists and identity authoring.
Guide token counts are unknown; whole-turn reported usage is observed. This evaluates compact onboarding,
not an ablation of the complete Lobo programming skill or a model trained on Lobo.

## Preflight, failures and infrastructure amendment

All eighteen base/change canonical implementations compiled and agreed with independent deterministic
oracles on 1719 observations. Eighteen deliberately incorrect inclusive-boundary predicates compiled and
were killed by behavioral tests. All 54 larger performance-input/mode reference observations passed.
These checks validate the selected observations, not every possible input or the absence of grader bugs.

Setup failures are retained: incorrect assumptions about native list serialization, a missing complex-match
parenthesis, a constructor needing an expected owner, a scheduler generator field mismatch, nested
conditional syntax and a template-placeholder substitution. The list decoder has a regression against the
actual wire format. No compiler modification was made to accommodate a study reference.

The first cohort completed five attempts, all finally accepted, and began a sixth. Its Lobo construction
attempt received false visible scope errors because the CLI transiently creates empty protected
`.codex`, `.agents`, `.aws` and `.git` directories inside sandboxed commands. The final candidate passed,
but the feedback opportunity was impaired. The controller interrupted the sixth attempt and left 48
assignments unattempted. The original cohort is retained separately, including unknown interrupted usage.

Amendment 1 ignores only those four empty regular directories; nonempty entries and symlinks still fail.
A regression and live three-language transport checks verify both transport and scope behavior. One
excluded extension-of-canary attempt itself failed because its miniature fixture had no identity file;
that setup mistake was retained and fixed before the new registration. In total, seven excluded transport
canaries were launched. See [the amendment receipt](../benchmarks/results/2026-11-23-matched-study-amendment-1.json).
The amended cohort is a distinct, fully rerun schedule, not a silent replacement of unsuccessful trials.

## Results

All **54/54** amended-cohort attempts were accepted within the registered working limits. All
3438 final held-out program observations and all 198 comprehension items passed. There were no new
scope violations or missing terminal usage in this cohort. No agent exceeded the registered limits.

| Phase | Lobo | Rust | Python |
| --- | --- | --- | --- |
| Construction | 6/6 | 6/6 | 6/6 |
| Comprehension | 6/6; 66/66 items | 6/6; 66/66 items | 6/6; 66/66 items |
| Requirement changes | 6/6 | 6/6 | 6/6 |

This is a quality ceiling for the selected tasks, not evidence of equal general capability. Lobo can
support these agent tasks with compact onboarding, but this pilot establishes no quality advantage.

[Every attempt and normalized observation](../benchmarks/results/2026-11-23-matched-agent-study.json)
is retained, with 72 public bundles containing all 54 final answers/programs and eighteen canonical
references, plus hashes for private raw evidence and check snapshots. There were 74 visible checks.

| Phase | Lobo median seconds | Rust median seconds | Python median seconds |
| --- | ---: | ---: | ---: |
| construction | 36.60 | 28.95 | 23.42 |
| comprehension | 9.11 | 8.92 | 8.98 |
| change | 27.96 | 26.84 | 21.99 |

Median matched-pair ratios use six same-task/same-phase/same-repetition pairs per row. These are
ratios of reported cumulative **input plus output tokens**, including cached input; not billing ratios.

| Phase | Lobo / comparator | Token ratio median (range) | Wall ratio median (range) |
| --- | --- | ---: | ---: |
| construction | rust | 1.229 (1.032–1.891) | 1.311 (1.018–1.981) |
| construction | python | 1.813 (0.867–2.091) | 1.812 (1.186–2.323) |
| comprehension | rust | 1.048 (1.045–1.058) | 1.046 (0.903–1.219) |
| comprehension | python | 1.055 (1.051–1.064) | 1.005 (0.761–1.089) |
| change | rust | 1.003 (0.839–1.213) | 1.051 (0.912–1.244) |
| change | python | 1.170 (0.840–1.493) | 1.253 (1.193–1.483) |

Lobo's median paired construction token cost was **1.23× Rust and 1.81× Python**, with the same
observed acceptance. Median construction wall time was also higher. Change costs were close to Rust
and higher than Python on these pairs; comprehension quality was ceiling-limited. These observations
do not show a Lobo token-efficiency advantage. They do not isolate syntax from tooling, familiarity,
collection operations, host instructions, guide size or tool choices. The small sample supports no
population ranking or confident effect estimate.

Primary-cohort reported usage totals 4,833,199 input and 36,686 output tokens. Cached input, 4,043,008,
is a subset of input; reported uncached input is 790,191. Do not add cache hits or reasoning output
again. Across the original cohort, amended cohort and excluded canaries there were **67 CLI agent
invocations**, with terminal usage known for 66 and unknown for the interrupted original attempt.
Known input-plus-output usage across those invocations is 5,429,516; the complete total is unknown.
An agent invocation can contain multiple underlying model requests; that request count and actual
billed cost are unknown. Participant usage excludes parent orchestration, harness development and
researcher preparation; total study-development token cost is unknown.

Two Lobo construction attempts exposed a concrete workflow interaction. The authoring command first
rejected the reserved name `result` (`compile.name.reserved`). Because synchronization failed, the
following check reported missing persistent identities for the new helper records. Agents renamed the
identifier, synchronized again and passed. Missing identity messages were a subsequent symptom;
reserved-name rejection was the first failure. This suggests a future diagnostic/onboarding review,
not an automatic language change during a frozen study. Extra steps remain in measured cost.

The test set uses the selected primitive/stdlib environments. Rust and Python have familiar collection
operations; Lobo often uses explicit carried-state traversal. The comparison does not hold abstraction
facilities constant. It also uses the same host check interface for every language, without a separate
ablation of Lobo's affected-verification/cache workflow. These boundaries matter when attributing costs.

## Performance protocol

[The performance registration](../benchmarks/results/2026-11-23-matched-performance-registration.json)
fixes task inputs, optimization, repeat counts and timing boundaries before measurements. All eighteen
canonical implementations form a complete cohort. Every accepted construction/change solution is a
separate success-conditioned cohort; unsuccessful agent work remains in the primary construction counts.

Each configuration has five fresh builds and seven interleaved runtime measurements per mode, after one
warmup. Modes are a normal single-operation CLI invocation, an empty-batch startup control, and one process
performing 100 checksum-consuming operations with varying parameters. Native Lobo uses its frozen C
backend with Clang -O3; Rust uses opt-level3, checked overflow, panic abort and stripped symbols. Python
uses stdlib and warm bytecode caches. All reported results are checked against independent expected values.

Timing covers process launch through exit, adapter parsing and output. Result normalization, correctness
grading and profiling are outside timing. Batch divided by100 is an amortized cost including startup; it
is not a directly timed isolated function call. Source/binary sizes omit shared system runtimes and
libraries. Python has no standalone binary in this comparison. Shared-host load, small workloads,
allocation policy and representation differences limit interpretation. These are not HTTP throughput,
production scaling or language-wide speed rankings.

## Performance observations

[The complete measurements](../benchmarks/results/2026-11-23-matched-performance.json) retain every
build, warmup, timed result, expected value, source/binary hash and profile. All 270 builds, 162 warmups
and 1134 timed process runs across 54 configurations completed without build or correctness failures.
The full canonical cohort has 18 programs; all 36 accepted agent programs were measured separately.
The host has 16 logical CPUs; before/after load averages were approximately 4.9–5.5.

Canonical **100-operation batch** wall times below are milliseconds, median (minimum–maximum), seven
interleaved repeats per cell. Each batch is one process, including startup, parsing and checksum output.

| Workload | Lobo | Rust | Python |
| --- | ---: | ---: | ---: |
| ledger base | 1.274 (1.227–1.414) | 1.166 (0.914–1.410) | 31.629 (29.329–37.831) |
| ledger changed | 1.446 (1.198–1.683) | 1.103 (0.974–1.430) | 31.660 (29.526–32.249) |
| scheduler base | 1.568 (1.347–1.828) | 1.240 (1.001–1.302) | 32.218 (31.726–36.911) |
| scheduler changed | 1.532 (1.463–1.673) | 1.123 (1.011–1.392) | 33.368 (31.596–36.395) |
| frames base | 1.320 (1.227–1.483) | 1.185 (1.029–1.527) | 30.436 (29.196–31.086) |
| frames changed | 1.458 (1.096–1.527) | 1.204 (1.088–1.555) | 30.187 (28.941–31.800) |

Both native configurations have much lower full-process latency than Python for these small programs.
This is largely a startup/import comparison: Python's empty-batch medians were 28.4–29.8ms, versus
approximately 0.86–0.95ms for Lobo and 1.04–1.14ms for Rust. Python cProfile observes imports plus algorithm
iteration; its setup changes preloaded state and cannot allocate the complete CLI time precisely.
Single-operation native timing ranges overlap. Do not subtract noisy medians to estimate nanosecond
compute costs or treat batch/100 as a directly measured function call.

Rust's canonical batch medians are lower in all six cases. Four native timing ranges overlap; both
scheduler variants have separated ranges in this run. This supports the scoped end-to-end observations,
not a general native-language ranking. Data representation/allocation and loop instrumentation are
plausible contributors, but native CPU hot spots were not measured: all nine hardware-counter attempts
were denied by host policy. The denials are retained separately from the zero timed/build failures.
No host security setting was changed. strace, process resource profiles and Python cProfile succeeded
outside timed runs. See the text artifacts linked by the JSON profile manifest.

Canonical stripped Lobo binaries are 31,080–39,280 bytes; Rust binaries are 368,592–371,408 bytes.
Both depend on system shared libraries. Python's sources/bytecode require an external interpreter and
have no comparable standalone binary here. Binary size reflects these programs, adapters and build flags.
Median native compile times across the six canonical cases were 356–422ms for Lobo and 184–212ms for Rust,
each based on five fresh builds. Python's 27–30ms py_compile observations are source/bytecode checks,
not native compilation or an equivalent artifact size. Agent-program timings and sizes remain in their
separate cohort in the full data; they do not improve unsuccessful-task denominators.

## Interpretation and paper boundaries

The original objective remains improved software construction and comprehension at declared cost, not
merely smaller source text or fewer tokens. This pilot measures outcomes across languages but cannot
separate the original H1 checked-representation, H2 abstraction and H3 context/feedback mechanisms.
Such attribution needs feature/guide/feedback ablations and stronger task families.

Two repetitions of three task families are not eighteen independent real-world problems per language.
No immutable model weights, equal prior training exposure, dollar-cost estimate, human-comprehension
outcome, formal proof or general language superiority is established. Keep the earlier research reports,
fixtures and paper evidence pins unchanged; add this study as new dated evidence.

The protocol was locally recorded and hash-pinned; no independent external preregistration was made.
These are current-checkpoint observations, not a controlled longitudinal improvement over Core 0.9.
