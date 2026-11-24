# Frozen matched agent pilot

Protocol dated 2026-11-23, selected in decision 0063. Freeze: compiler/extension 1.11.2,
Core 1.11, commit `75a551fc0aec1fbc94f1fddb2a0f6a59ddb3f122`, tag `research/lobo-1.11.2`.
This is a small exploratory study, not an inferential language ranking.

## Questions and primary outcomes

- Construction: does the final candidate satisfy every shared held-out observation within budget?
- Comprehension: does a source-only answer satisfy eleven objective checks?
- Requirement changes: does a modified correct base satisfy the original and new contract?

Report whole-task acceptance for every scheduled attempt, comprehension item counts, final functional
acceptance independent of budget, diagnostic/check history, observed tokens, tool items and elapsed
wall time. Do not hide failures by conditioning the primary comparison on successful programs.

54 assignments: ledger state transitions, dependency scheduler, framed-byte parser; Lobo/Rust/Python;
construction/comprehension/change; two repetitions. `schedule()` uses seed 20261123, eighteen matched
three-language blocks; each language appears six times at each within-block position.

## Equal opportunities and recorded differences

Same model alias `gpt-6.1-sol`, high reasoning, Codex CLI 0.171.0, default provider/account. Explicit
CLI flags ignore user configuration, use ephemeral sessions, disable apps/plugins/multi-agent/memory,
and require workspace-write sandbox with no approvals. Comprehension disables shell tools and uses
read-only mode. All phases start fresh. No delegation or language implementation changes during trials.
Model aliases and backend service behavior are not immutable weights; model/pretraining familiarity
is not matched. Host instructions are inherited equally but may affect actual context and usage.

120 seconds from process launch, sixteen command-execution tool items, six visible check submissions.
Compilation and visible checks count in wall time. Final independent grading is outside that budget.
No hard token cap: record actual terminal usage, including cached input as a subset of input. Never
estimate interrupted usage or multiply a survivor's cost into a total. No pricing/billing claim.
Guides are language-specific and capped at 8000 UTF-8 bytes, not padded. Report exact bytes/hashes.
Lobo receives explicit identity-management and state-loop guidance. Rust/Python get DTO/signature,
stdlib and build guidance. Actual guide-token counts are unknown; actual whole-turn usage is measured.

## Tasks and isolation

See `BASE_REQUIREMENTS`, `CHANGE_REQUIREMENTS`, `models`, `sources`, `prediction_inputs` and
`comprehension_questions` in `study.py`. These define finite, exact-integer, pure-function contracts.
Visible and held-out inputs use distinct fixed seeds, plus hand-selected boundaries, cycles, zero
values, locking/undo, truncation, checksum failures, padding and extended-header cases.
All eighteen canonical implementations must pass the same observations. Deliberate inclusive-boundary
defects must compile and fail behavior in every task/language/base-change combination.

Construction edits only `logic.*` plus Lobo's identity sidecar. Change receives the same correct base
algorithm and an already-updated DTO, never a construction survivor. Comprehension receives all
algorithm/model/util source in its prompt, with no execution/tools, six predictions plus five
structural/witness checks. Exact JSON types matter; Boolean values cannot stand in for integers.

The host controls compilation and testing through a file mailbox. Each check snapshots candidate
files externally. Fixed models/adapters and existing Lobo IDs must stay unchanged. A basic source
policy forbids IO/process/unsafe escapes; this is cooperative experimental isolation, not a security
proof against hostile same-user processes. The frozen compiler's directory and other trial/reference
files are explicitly out of scope. Retain scope violations separately from functional correctness.

## Registration and failure policy

Offline setup runs and excluded mailbox canaries precede registration. Record all discovered harness
errors. Registration pins the harness SHA-256 and all task/guide/source hashes before scheduled use.
The controller refuses a changed harness. No scheduled attempt is retried/replaced. Keep process exit,
timeouts, partial JSONL, stderr, final files, checks, grading, usage (or null), timestamps and load.
Three consecutive infrastructure failures stop the schedule; report unattempted assignments separately.
Any material grading/protocol repair after registration requires an explicit amendment and retention of
original outcomes. Compiler diagnostics caused by candidate code count as ordinary unsuccessful work.

## Performance

Keep runtime observations separate from agent measures. Canonical references form the complete cohort;
accepted agent implementations form a separately labelled, success-conditioned cohort. Use identical
inputs and independently verified outputs, optimized native Rust and Lobo C builds, Python stdlib.
Measure full CLI startup/parse/compute/output once and a checksum-consuming 100-operation batch with
varying parameters. Interleave at least seven runtime repeats per configuration and five native compile
repeats. Record binary/source sizes, actual compile flags, versions, CPU/load, fuel/heap and all errors.
One process's 100-operation batch is not 100 fresh process launches. Do not claim server throughput or
language-wide speed rankings from these small programs. Profile outside reported timing if possible;
identify startup/runtime/allocation work and report unresolved bottlenecks.

## Reproduction and evidence

`study.py` holds the deterministic tasks, source fixtures, graders, mailbox controller and schedule.
`python3 -m unittest discover -s benchmarks -p test_matched_study.py` checks the harness.
Run raw model experiments only with explicit authorization; this user supplied it. Raw evidence lives
under ignored `benchmarks/private-evidence/matched-study-20261123`; public reports contain assignments,
normalized observations, hashes and limitations. Raw prompts/tool transcripts stay private. Frozen older
research fixtures and paper pins are unchanged. The source archive and compiler hashes live in the
freeze manifest; registry hashes and public summaries allow checking correspondence.

## Amendment 1: transient sandbox mountpoints

The first cohort was interrupted after assignments 0–4 completed and assignment 5 began. All five
completed attempts passed final grading, but assignment 4 received false visible scope errors for
empty `.codex`, `.agents`, `.aws` and `.git` directories created transiently by the CLI sandbox.
Those checks never compiled the candidate. Assignment 5 was interrupted by the controller; terminal
usage is unknown unless a retained terminal event proves otherwise. Assignments 6–53 were not attempted.
This cohort is retained separately and excluded from the matched primary comparison.

Amend the infrastructure audit to ignore only those four **empty, regular directories**. Nonempty
entries and symlinks remain violations. Add a regression proving both cases; extend excluded mailbox
canaries to run that audit while inside a live sandboxed check. Keep task contracts, guides, oracles,
budgets and schedule byte-for-byte unchanged. Register a new harness hash and rerun the full 54-assignment
schedule as a distinct amended cohort. This is an infrastructure restart, not a language-driven task or
budget adjustment. Original trials, the interrupted attempt and missing usage remain in total accounting.

## Completed evidence and replay

The [completed report](../../research/matched-agent-study-2026-11-23.md),
[agent observations](../results/2026-11-23-matched-agent-study.json) and
[runtime observations](../results/2026-11-23-matched-performance.json) retain the primary and original
cohorts separately. Public JSON bundles under `benchmarks/fixtures/matched-study-20261123` contain
`files` maps for programs or an `answer` for comprehension. Their hashes are in the agent observations.
The archived harnesses and tests are in `research/snapshots/2026-11-23-matched-study/harnesses.json`.
No model access is needed to extract, compile or grade those stored programs.

For example, from the repository root with compiler 1.11.2 installed:

```sh
python3 - <<'PY'
import json
from pathlib import Path
bundle = json.loads(Path('benchmarks/fixtures/matched-study-20261123/trial-53.json').read_text())
folder = Path('target/matched-replay')
folder.mkdir(parents=True, exist_ok=False)
for name, text in bundle['files'].items():
    assert Path(name).name == name
    (folder / name).write_text(text)
PY
lobo compile target/matched-replay/app.lobo --source target/matched-replay/model.lobo --source target/matched-replay/util.lobo --source target/matched-replay/logic.lobo -o target/matched-replay/program
./target/matched-replay/program --entry study.frames.entry --fuel 100000000 --heap-cells 4000000 --arg Bytes:02010102 --arg I64:2
```

The final record contains one payload `0101`, consumed offset4 and error0. Use a fresh destination for
each replay. Full held-out inputs and their expected values are deterministic in `study.py`; the public
bundle replay verification independently rebuilt one agent program per language and regraded all stored
comprehension answers. The original source archive can be rebuilt from tag `research/lobo-1.11.2` using
the compiler-build instructions; the local ignored freeze manifest records the exact original binaries.
