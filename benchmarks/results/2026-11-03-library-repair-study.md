# Repeated library-repair study — 2026-11-03

**Verdict: reuse reduces native executions; agent productivity remains inconclusive.** All 29
completed repairs passed independent native grading. One of 30 scheduled attempts was interrupted
when the controller received SIGTERM. It remains an infrastructure failure with unknown latency/
usage and no replacement. Do not rank interfaces by raw success rate without that qualification.

[Machine-readable evidence](2026-11-03-library-repair-study.json) ·
[Registered protocol](../../decisions/0026-repeated-library-repair-study.md) ·
[Harness](../agent-repair-study.py) · [Exporter](../agent-study-report.py)

## Design and outcomes

Two repair tasks share a five-source project: collection validation/filtering/order and binary-tree
duplicate handling/depth/traversal. Each seeds three defects across two files. Existing Lobo
collection/tree/sequence workloads, a rule module and independent aggregation make up the project.
Each task/arm has five fresh conversations. Concise full-run text versus full-run JSON isolates
representation; full-run JSON versus affected JSON isolates selection. All arms offer the same
list/detail/replay commands and 64 visible checks. Independent final grading runs 150 fresh cases.

| Task / interface | Accepted / scheduled | Known times | Median seconds | Range seconds |
| --- | ---: | ---: | ---: | ---: |
| Collection / text full | 5/5 | 5 | 31.31 | 27.80–56.05 |
| Collection / JSON full | 4/5 | 4 | 33.76 | 31.94–40.27 |
| Collection / JSON affected | 5/5 | 5 | 29.74 | 27.63–31.81 |
| Tree / text full | 5/5 | 5 | 31.41 | 30.46–35.34 |
| Tree / JSON full | 5/5 | 5 | 30.92 | 28.24–55.49 |
| Tree / JSON affected | 5/5 | 5 | 32.19 | 30.58–34.70 |

Time starts at CLI launch and ends after grading. Known times here are successful repairs; the
interrupted collection/JSON-full attempt stays in the denominator without fabricated latency.
Its recovered source was still defective: final grading observed 40 failures and 110 passes.
Final grading executed 4,500 cases total, with zero reuse/pending; 4,350 belong to accepted repairs.
No attempt hit the registered 120-second agent deadline.

| Metric | Text full | JSON full | JSON affected |
| --- | ---: | ---: | ---: |
| Accepted / scheduled | 10/10 | 9/10 | 10/10 |
| Trace-verified directory/workflow adherence / scheduled | 9/10 | 8/10 | 10/10 |
| Completed trials using detail | 10 | 9 | 10 |
| Completed trials using replay | 8 | 9 | 8 |
| Verify calls per completed trial | 2 | 2 | 2 |
| Recorded fresh verify cases per completed trial | 128 | 128 | 84 |
| Reused passes per completed trial | 0 | 0 | 44 |
| Median delivered feedback bytes per completed trial | 89,211 | 39,808 | 43,956 |
| Recorded host tool seconds, total | 11.87 | 10.44 | 10.60 |
| Known repair/grading seconds, total | 348.93 | 315.71 | 310.27 |

Verify counts exclude replay and final grading. Host observation logs were lost for the interrupted
attempt; its partial native work is unknown, not zero. Full mode runs all 64 cases before and after
editing. Affected mode runs 64 initially and 20 after repairing the library, reusing 44 unrelated
passes: 440 saved executions across ten trials. Every accepted final project has identical source
hashes, with the intended collection/tree/rule fixes and unchanged sequence/aggregation code.

All 29 completed agents used detail (45 calls); 25 used replay. Unlike the scalar pilot, these
capabilities were exercised. However, **every completed repair used one candidate edit and two
verifies**. Multiple defects did not produce multiple repair iterations. Longer edit/verify cycles
and whether drill-down was necessary remain unestablished.

## Tokens, pairing and limits

| Task / interface | Median aggregate input tokens | Input range | Median output tokens |
| --- | ---: | ---: | ---: |
| Collection / text full | 156,935 | 135,383–219,240 | 816 |
| Collection / JSON full | 161,913 | 143,753–175,940 | 927 |
| Collection / JSON affected | 143,717 | 125,111–158,630 | 803 |
| Tree / text full | 157,114 | 136,202–186,797 | 876 |
| Tree / JSON full | 144,288 | 137,551–177,138 | 836 |
| Tree / JSON affected | 159,649 | 136,666–193,317 | 892 |

Terminal CLI usage is available for 29 attempts; interrupted usage is unknown. Cached input and
reasoning output remain separate reported fields with medians/ranges in JSON. Input includes
cached input; output/reasoning are not added to estimate a bill. Bytes are not tokens. JSON reduces
delivered bytes here because large list/detail payloads are compact rather than indented; it does
not show a consistent token or time advantage across the two tasks.

Paired accepted-trial deltas (after minus before), with exploratory bootstrap intervals:

| Comparison | Accepted pairs | Median delta seconds | Exploratory 95% interval |
| --- | ---: | ---: | ---: |
| Collection JSON full minus text full | 4 | +4.21 | −11.03 to +11.21 |
| Collection JSON affected minus JSON full | 4 | −4.05 | −10.53 to −1.22 |
| Tree JSON full minus text full | 5 | −0.65 | −4.91 to +24.08 |
| Tree JSON affected minus JSON full | 5 | +2.05 | −20.79 to +2.67 |

Collection affected/full favors selection in four observed pairs, but tree reverses direction.
These small, exploratory, success-conditioned intervals are not a general productivity result or
a replacement for failure accounting. Effects overlap ordinary variation; most intervals overlap
zero, one pair is interrupted, and two accepted attempts read extra global context. No adoption
decision follows. Five repetitions sample variability, not broad workload or model diversity.

Host tooling takes roughly 3–3.5% of known total time. Excluded corrected canaries used about one
second of host tooling and 4.1–4.3 seconds of local child CPU in 31–33 seconds overall. Selection
cannot directly explain several seconds of total difference through native savings alone. These
counters do not distinguish remote model computation, service wait, startup and shell waits. The
shared 16-CPU host had recorded one-minute loads from 1.71 to 13.00, without CPU/frequency isolation.

## Incidents and reproducibility

- Excluded canaries exposed a shared-file mailbox race under concurrent commands. Unique request/
  response files and one session consumer fixed it; four simultaneous detail clients pass a
  regression. Two corrected canaries passed before measured trials. The scalar pilot stays unchanged.
- The controller terminated with SIGTERM/exit 143 after 19 completed trials; cause unknown. Index
  19 is retained as interrupted, without retry or invented time/usage. Its orphan processes were
  stopped and source/evidence recovered. A [specific recovery controller](../agent-study-resume.py)
  validated frozen harness/dependency/runner/input identities and continued indices 20–29, without
  replacing the interrupted conversation or resetting its live session. This is not a durable service.
- Indices 14 (JSON full) and 20 (text full), zero-based, read global coding-skill files despite the
  directory-only instruction. Their repairs remain counted and workflow adherence is flagged
  separately. No other external path references, outside file-change events, MCP calls or fixture
  edits were found in the audit. This is not containment: the sandbox allows more reads than the prompt.
- The original timeout counter matched the timeout phrase in successful prints of client source.
  Export corrects four false positives uniformly from raw outcomes, preserving original counters
  and prefix analysis. It also identifies two actual client deadlines after the controller died.
  No task, model prompt, oracle, successful attempt or acceptance criterion was retuned. Counter,
  external-context and unknown-latency regressions pass.

JSON retains all scheduled rows, verify outcomes, grades, usage, audit flags and identities.
Large list/source bundles in detail events have hashes; exact data and raw JSONL remain private
in `target/agent-study-20261103T202419/`, alongside pre-recovery data, frozen pre-run protocol/
harness/dependency, inputs and recovery source. Requested model: `gpt-6.1-sol`, high reasoning;
CLI 0.166.0, Clang 18.1.3, Python 3.12.3. The alias is not an immutable model-weight identity.

Re-export without running any agents:

```sh
python3 benchmarks/agent-study-report.py target/agent-study-20261103T202419/results.json benchmarks/results/2026-11-03-library-repair-study.json
```

## Consequences and verification

Keep evidence reuse, independent finite grading and explicit failures. Do not claim general speed
or token savings. Investigate evidence-only detail payloads: detail currently repeats the whole
source bundle after agents have read it. Longer repair cycles need a subsequent registered task
design, rather than forced edits that manufacture incremental savings.

The user's state-machine/TLA+ suggestion fits session identity, frozen plans, reuse admission,
pending/failure classification, request correlation and recovery. A model could complement TDD by
checking modeled transitions/invariants. **No TLA+ model was implemented or checked here**; checking
one would not prove implementation refinement or productivity.
See [Lamport's overview](https://lamport.azurewebsites.net/tla/high-level-view.html).

All 16 Python harness/report tests and 145 Rust tests pass, including native/reference/sanitizer
gates. TDD first failed with the missing harness; later regressions caught missing concurrent
transport, printed-source timeout classification, context auditing and unknown-latency handling.
Core/compiler/artifact/extension versions and dependencies are unchanged; editor-host checks were
not rerun because language/extension code did not change. No publication, commit, deployment,
public daemon or runtime authority was added.

JSON/JSONL/Markdown-link checks and `git diff --check` also pass. Recorded runner, frozen input,
harness and exporter hashes were verified against their retained artifacts.
