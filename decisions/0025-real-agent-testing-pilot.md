# 0025: Measure real repairs before expanding the testing interface

- Status: ten-trial pilot complete; interaction works, productivity comparison inconclusive
- Date: 2026-11-03

The user authorized a real-agent comparison after the scripted selection experiment in
[0024](0024-agent-first-verification-prototype.md). Keep this a local benchmark, not a new daemon,
language feature, or claim that Lobo is faster for agents.

## Frozen pilot protocol

Run five synthetic scalar repair tasks (clamp, inclusive interval overlap, floor division,
overflow-safe nonnegative ceiling division, absolute distance), once per arm: ten sequential
trials, alternating conventional/structured and structured/conventional by task. Use fresh
directories and Codex conversations, model `gpt-6.1-sol`, high reasoning, identical instructions,
90-second agent deadlines, and no cross-trial transcript. No adaptive task removal or replacement.
Record every failure, timeout and infrastructure error. A separate canary may validate wiring;
exclude it explicitly from the pilot. Run only this bounded pilot by default.

Both arms use the same native `lobo-test` binary, frozen visible inputs/expectations and resource
budgets. Conventional feedback renders a full run as concise text with per-case failures.
Structured feedback returns the existing JSON report and permits affected reuse in the same live
session. Both expose `python3 check.py`, `--list`, and `--detail TEST_ID`; details refer only to
the current candidate. Eight failure capsules plus omitted counts bound both defaults. There is
no passing reference source in either workspace. This isolates a feedback/selection bundle;
it does **not** compare against an independently optimized pytest/Jest/Cargo installation.

The host fixes criteria outside the editable task directory. An experiment-only file mailbox
adapter keeps the foreground session alive across agent shell calls. Its lifetime is one trial;
it accepts no arbitrary command or path. Only `program.lobo` may change. Hash-check other fixture
files and reject symlinked sources; `.check-io` contains adapter-generated messages and is exempt
from the source edit rule. This is an integrity audit, not hostile-agent OS isolation.
Limit source size, messages, subprocess time and retained snapshots; do not silently reset a full
session. Final acceptance always uses a fresh, forced-full session with visible and held-out
inputs, independently computed Python mathematical expectations, including signed/boundary cases.
Held-out inputs add no undocumented requirement. Native code performs all candidate computation.

## Measurements and decision

Primary: independently accepted repairs / all five trials per arm. Also report final grading
failures, timeout/infra counts, whether the assigned check interface was used, and elapsed time
from CLI launch through final grading. Timeouts are censored failures, not quick repairs.
Record actual CLI `turn.completed` input, cached-input, output and reasoning tokens; missing usage
is unknown, never zero. Do not infer billing or count feedback bytes as model tokens. Record
check count/time/bytes/executed/reused, tool calls, host load, source and executable identities,
CLI/tool versions and exact prompts. Retain private raw events and repaired files locally.

Report medians/ranges and each pair, including failures. Five heterogeneous pairs with no
within-task repetitions cannot establish statistical significance or general agent productivity.
Do not adopt/expand the interface on a speed claim from this pilot. An apparent advantage is a
hypothesis for a larger repeated study, conditional on equal correctness and actual interface use.
Model latency, prompt caching and a shared busy host may dominate; instrument test time and CLI
time separately rather than attributing total time to selection alone. Scalar tasks do not test
large-codebase context retrieval, multistep architecture, mutation robustness or property shrinking.

Use the installed authenticated CLI without copying credentials. Disable optional apps, plugins,
memories and multi-agent work, ignore user config, use ephemeral conversations and workspace-write
sandboxing. The [official CLI automation guide](https://learn.chatgpt.com/docs/non-interactive-mode)
documents JSON events and usage reporting. Workspace-write is not a secret-hiding boundary against
same-user reads. Synthetic tasks contain no user data. No publication, commit or deployment occurs.

Pre-pilot canary correction: the first canary repaired clamp but could not connect to a Unix
socket under workspace-write sandboxing (two PermissionErrors, zero delivered checks). Exclude it;
replace sockets with a bounded file mailbox inside the task workspace and rerun the canary.
Do not bypass the sandbox to make the experiment work. A guide-template brace error before
that canary launched any model was also corrected. Freeze the resulting protocol before pilot run.

## Outcome (appended after the frozen run)

All ten repairs passed independent native grading (490 fresh final case executions). Both arms
used two check calls per trial. Structured mode executed 80 visible cases and reused 20;
conventional mode executed 100. Median repair/grading time was 17.93 versus 20.24 seconds,
with overlapping ranges, five heterogeneous pairs and no within-task repetitions. Comparison
is inconclusive; no general speed/token-saving claim or adoption decision follows.
Structured feedback also delivered more bytes (5,168 versus 2,944), and total host check time
was nearly identical. See the [full analysis](../benchmarks/results/2026-11-03-agent-repair-pilot.md)
and [all trial results](../benchmarks/results/2026-11-03-agent-repair-pilot.json).
The private artifact directory retains the exact pre-run protocol/harness whose hashes appear
in the report. This appended outcome does not rewrite that preregistration.
