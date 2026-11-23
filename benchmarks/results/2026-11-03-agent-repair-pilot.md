# Real-agent repair pilot — 2026-11-03

Ten real Codex CLI repair attempts completed: five tasks, once with each interface. All ten
repairs passed fresh independent native grading. Both interfaces worked; the timing/token
comparison is inconclusive. Keep the prototype experimental.

Inputs and outcomes: [machine-readable report](2026-11-03-agent-repair-pilot.json).
Method: [decision 0025](../../decisions/0025-real-agent-testing-pilot.md).
Rerun: [benchmark instructions](../README.md#real-agent-repair-pilot).

## Observations

| Measurement | Conventional full-run text | Structured affected-mode JSON |
| --- | ---: | ---: |
| Independently accepted repairs | 5/5 | 5/5 |
| Trials using assigned checks | 5/5 | 5/5 |
| Pilot timeouts / infrastructure errors | 0 / 0 | 0 / 0 |
| Repair + final grading, median seconds | 20.24 | 17.93 |
| Repair + final grading, range seconds | 16.61–33.24 | 16.68–24.21 |
| Reported aggregate input tokens, median | 92,277 | 74,167 |
| Reported aggregate input tokens, range | 86,747–92,770 | 73,974–94,190 |
| Cached input tokens, median | 79,104 | 66,944 |
| Cached input tokens, range | 75,392–85,248 | 66,816–85,248 |
| Reported output tokens, median | 420 | 434 |
| Reported output tokens, range | 387–501 | 388–547 |
| Reported reasoning output tokens, median | 9 | 21 |
| Reported reasoning output tokens, range | 0–52 | 8–41 |
| Visible check requests, total | 10 | 10 |
| Fresh visible native case executions | 100 | 80 |
| Reused passing cases | 0 | 20 |
| Host check time, total seconds | 1.558 | 1.520 |
| Delivered check text, total UTF-8 bytes | 2,944 | 5,168 |
| Completed command/file-change items | 25 | 28 |

Token fields come directly from terminal `turn.completed` events, available for all ten trials.
Input includes cached input; these are separate reported fields, not additive token categories.
Reasoning output is retained as reported and is not added to output for a cost estimate. No billing
or price estimate is made. Feedback bytes exclude client framing and are not model-token counts.
Tool items count completed command/file-change events, not provider API requests. All trials
passed, so the elapsed-time summaries contain no excluded or censored failures.

| Task | Conventional seconds | Structured seconds | Final native checks per repair |
| --- | ---: | ---: | ---: |
| Clamp | 16.61 | 21.93 | 38 |
| Inclusive interval overlap | 20.24 | 17.17 | 59 |
| Floor division | 33.24 | 24.21 | 57 |
| Overflow-safe ceiling division | 18.28 | 17.93 | 42 |
| Absolute distance | 23.15 | 16.68 | 49 |

Final grading freshly executed 245 cases per arm (490 total), with no reused outcomes or pending
checks. Every trial ran the visible checks before editing and after one repair; none requested
list/detail. Control cases intentionally represent a small unaffected function: after the edit,
the four control outcomes reused, while all six target cases reran. This confirms the expected
selection behavior on these fixtures, not large-project dependency soundness or mutation coverage.

## Interpretation and limits

The direct benefit here is fewer native executions. Check time was nearly identical, and the
structured interface delivered **more** bytes than concise conventional feedback. Both still
needed two check requests. There is no evidence here that JSON alone streamlines agent testing.
Host checks consumed only about 1.6 seconds across each arm, compared with tens of seconds per
repair. Most elapsed time occurred in agent/session work outside the measured host checks; these
counters do not distinguish model computation, service wait, shell overhead and startup within it.

The structured arm had lower observed median total time and input tokens, but the time difference
is smaller than the within-arm ranges. Five different tasks, each attempted only once per arm,
do not estimate within-task variance or support a causal/general speed or token-saving claim.
One structured clamp agent also performed three extra inspection commands. Prompt caching and
variable model interaction affect totals. The shared 16-CPU host had observed one-minute loads
from about 1.1 to 7.8; it was not isolated. Task order alternated arms but there was no randomized
repeated campaign, confidence interval, or optimized third-party-framework baseline. All tasks are
small scalar fixes, supplied with a language guide; context retrieval, libraries, multiple modules,
larger fault localization and repeated edits remain unmeasured.

Review of all retained commands/file-change events and final sources found task-local reads and
edits to `program.lobo` only, with preserved module/function signatures and controls. Fixture
integrity checks passed. This audit is not hostile-agent containment; workspace-write permits
more reading than the task instructions authorize. Independent finite checks and source review
do not prove arbitrary program correctness, and the final judge shares the native compiler.

Next investigation: repeated multistep repairs where failures require actual drill-down and only
a small dependency closure changes. Compare concise text, compact JSON and affected selection
separately; this pilot bundles representation and selection and cannot attribute their effects.
That larger experiment is a proposal, not a background run or an adoption decision.

## Reproducibility and verification

Requested model `gpt-6.1-sol`, high reasoning; Codex CLI 0.166.0; Clang 18.1.3; Python 3.12.3.
The report retains exact commands, deadlines, runner/harness/protocol hashes, source/plan hashes,
host load, all usage fields, feedback, counts and final grades. The model alias does not identify
an immutable provider weight snapshot. Raw events, final sources, and the preregistered harness/
protocol remain private locally in `target/agent-repair-20261103T195822/`. The current decision
has an appended outcome; the report hash refers to its retained **pre-run** copy.

One pre-pilot socket canary repaired code but received zero checks because the sandbox denied
socket access. It is excluded, as specified. A second file-mailbox canary passed with two delivered
checks and independent grading. No pilot trial was dropped, retried or replaced. Sandbox bypass
was not used. No language/compiler/extension semantics, runtime dependency or public service changed.

Seven harness tests cover schedule fairness, actual usage versus unknown usage, guide/client
generation, symlink rejection, native seeded-defect/repair oracles, real file-client transport,
and reuse versus forced-full execution. TDD first failed with the absent harness; the file-transport
regression also failed against the socket implementation before the correction.

Final repository checks passed: seven Python harness tests; `cargo test --locked --quiet`
(145 Rust tests, including native/reference/sanitizer gates); `./scripts/check.sh` for JSON,
JSONL and Markdown links; and `git diff --check`. No language or extension code changed in this
pilot, so editor-host checks were not rerun. The recorded runner hash still matches the executable.

Follow-up: the proposed repeated library experiment now has
[separate findings](2026-11-03-library-repair-study.md). This pilot analysis remains its historical
interpretation; the follow-up records new evidence, incidents and limits separately.
