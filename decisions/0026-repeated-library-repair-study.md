# 0026: Repeat library repairs and separate feedback from selection

- Status: 30 records retained; 29 accepted, one infrastructure interruption; productivity inconclusive
- Date: 2026-11-03
- Risk: standard, bounded local experiment; no production or language changes

The user authorized the next investigation after [0025](0025-real-agent-testing-pilot.md).
Measure whether agent-facing evidence helps repeated, larger repairs. Do not infer productivity
from saved executions alone. Preserve the completed pilot and its frozen harness.

## Contract and alternatives

Use one five-source Lobo project: existing collection filtering, tree construction/traversal,
shared list reversal, a small rule module and an unrelated aggregation module. Two task types
seed three defects each across two files. Collection: inclusive validation boundary, filtered sum,
stable output order. Tree: duplicate comparison, depth increment, traversal direction. Public
criteria are complete from the start; do not inject new faults or change tests during a trial.
Multi-defect work provides an opportunity for multiple candidates, not a forced number of edits.

The ten-trial pilot conflated JSON representation and affected selection. A two-arm expansion
would retain that ambiguity. Choose three arms over a larger all-framework campaign:
`text_full`, `json_full`, `json_affected`. Identical compiler, cases, failure cap (eight), commands,
budgets and source snapshots. Full text versus full JSON measures representation; full JSON
versus affected JSON measures reuse. Text is concise, not intentionally verbose. This is not
a comparison against optimized pytest/Jest/Cargo installations or proof of selector soundness.

## Frozen schedule and budgets

Two task types x five repetitions x three arms = **30 sequential fresh CLI conversations**.
Within each task/repetition block, rotate arms and reverse every other repetition. Interleave
task types by repetition. This balances host/cache drift without pretending the schedule is
randomized. Use the same requested model `gpt-6.1-sol`, high reasoning, Codex CLI settings and
120-second agent deadline. Use existing authentication, no copied secrets, no delegation,
network actions, packages, publication, commits or deployments. Sandbox stays workspace-write.
Canaries are excluded and labeled; freeze the final harness, sources, cases, prompts and this
protocol in private artifacts before the first study trial. No adaptive removals or replacement
attempts. Record failures, timeouts, quota/auth/infrastructure failures, and missing usage.

Host fixes a plan of 64 visible cases: 20 collection, 20 tree, 12 reverse and 12 aggregate.
Each final judge uses a fresh session with those cases plus additional independent signed,
duplicate, boundary and shape inputs. The host computes mathematical expectations, never copies
candidate output. Candidate computation remains native Lobo. Criteria stay outside task dirs.
An experiment-only file mailbox keeps one bounded foreground session alive; no daemon/MCP or
new runtime dependency. Only named source files may change. Hash-check immutable fixtures,
reject symlinks/oversized source, audit file/command events, and save final sources. These checks
are an integrity audit, not protection against a hostile same-user process or a hidden-oracle
OS boundary. Compiler assumptions and the host-declared environment assertion remain.

All arms expose `python3 check.py`, `--list`, `--detail TEST_ID`, `--replay TEST_ID`.
No reference repair or old passing source is exposed. Detail/replay operate on the latest
checked snapshot, identify that snapshot, and report the native observation. Summaries omit
large inputs; list/detail return them on request. The study does not force drill-down or claim
it is logically necessary: agents may infer faults from source. Measure actual detail/replay
usage; zero usage means that benefit was not exercised. Keep eight-snapshot retention explicit;
do not silently reset a full session. Full checking remains a distinct final acceptance gate.

## Outcomes and analysis

Primary: independently accepted repairs out of all scheduled trials per task/arm. Report
timeouts as failures with censored latency, not fast successful repairs. Show every trial and
within-task medians/ranges across five repetitions. Separate correctness, interaction adherence,
and incomplete/infra outcomes. Also record actual terminal input/cached-input/output/reasoning
usage (unknown when absent), command/file-change items, verify/list/detail/replay calls, check
time, feedback bytes, executed/reused cases, candidate counts and final grade. Do not estimate
billing or equate UTF-8 bytes with tokens. Record host load and tool/source/plan/harness hashes.

Compare paired task/repetition outcomes. Give paired deltas and a deterministic paired bootstrap
interval for exploratory median deltas; five repetitions/task and two related workloads remain
too small for broad claims. Do not pick a winner if correctness differs, instrumentation/use is
missing, effects overlap ordinary variability, or the time limiter is unknown. Report check
time as a share of end-to-end repair/grading time. An excluded canary records child CPU versus
wall/check time to distinguish native test cost from the rest of the agent session; it cannot
identify internal model/service latency. Repeated source tasks measure stochastic repeatability,
not workload diversity, cross-model performance, real-repository context search or long projects.

Stop study-specific tuning once the registered run begins. New failure observations belong in
the results and a subsequent iteration. Final repository validation: harness tests, Cargo tests,
JSON/Markdown/whitespace checks. No syntax/compiler/editor changes are planned.

## Pre-run canary correction

The first excluded collection canary passed final grading and used detail/replay, but concurrent
commands overwrote the pilot-style shared request/response filenames and two clients timed out.
The tree canary also passed. Replace shared files with unique per-request filenames, correlated
responses and a single session consumer. Bound pending requests to 32 and total handled requests
to 256 per trial. A four-concurrent-client regression must pass, then rerun both canaries. Never
disable concurrency in the agent merely to hide a transport defect. This is benchmark adapter
repair, not a change to the language or the foreground verifier protocol.

The user also asked about state machines and TLA+. A formal model of frozen plans, snapshots,
reuse admission, pending/failure classification and session lifecycle would complement TDD.
No TLA+ model has been implemented or checked in this study; it would not establish agent
productivity or prove that the implementation refines the model.

Both corrected canaries passed independent grading with zero transport errors. The collection
agent used list, detail and replay; it spent 31.46 seconds overall, 4.32 seconds of local child
CPU, and about one second in host check/detail/replay work. Native verification alone therefore
cannot explain most of the session wall time. These counters do not identify remote model
compute/service waits versus local shell/startup waits. Five study harness tests pass, including
four concurrent detail clients and independently detected single mutants. Freeze now; subsequent
observations belong to the outcome, not study-specific tuning.

## Outcome (appended after the frozen run)

Retained all 30 scheduled attempts. Twenty-nine completed repairs passed 150 fresh native checks
each; the controller received SIGTERM during index 19, which remains an interrupted infrastructure
failure without retry or known latency/usage. A specific recovery controller verified frozen
identities and continued the remaining ten attempts. No trial was replaced.

Affected checking ran 84 rather than 128 recorded verify cases per completed repair, reusing
44 passes. All completed agents used detail, but all repaired in one candidate edit. Timing/token
contrasts differ by workload; two context-read deviations and the interruption remain visible.
No productivity/adoption claim follows. The exporter preserves original timeout counters and
uniformly corrects a printed-source false positive. Missing interrupted latency is never zero.

See [findings](../benchmarks/results/2026-11-03-library-repair-study.md) and
[all 30 records](../benchmarks/results/2026-11-03-library-repair-study.json). Private artifacts
retain the preregistered protocol whose hash appears in the report; this is an appended outcome.
Sixteen Python and 145 Rust tests pass.
