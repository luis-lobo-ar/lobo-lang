# Grammar-first Lobo development skill: revision and matched comparison

Date: 2026-11-26. Decision [0071](../decisions/0071-grammar-first-skill-revision-comparison.md).
The revised skill completes 4/4 assignments within budget, versus 2/4 with the original and 4/4 without
a skill. All twelve final candidates pass their native suites. The observed gain over the original is
completion within the budget, not higher final test-case correctness. Token use versus control is
essentially unchanged at the median paired assignment, with substantial variation. The revised skill
remains a tested candidate, not a demonstrated general improvement over ordinary documentation/tools.

## Revision grounded in previous failures

The original 4,799-byte [skill](../.agents/skills/lobo-development/SKILL.md) stays unchanged. Add
[lobo-development-v2](../.agents/skills/lobo-development-v2/SKILL.md), 3,138 UTF-8 bytes: 34.6% shorter.
It puts braced else branches, reserved result, parenthesized field/compound match subjects and inference
boundaries before optional references. It explains that failed identity sync must be repaired at its
first diagnostic before retrying; a subsequent missing-ID error can be secondary. It keeps compact
requirement-based testing, native verification, independent expectations, stable IDs, cleanup and selective
lookup. It asks agents to avoid unrelated isolated-workspace chores and stop at the acceptance gate.

The revision changes content, ordering and workflow together; byte reduction does not establish token
savings. It includes no scored task algorithm or gold answer. Versioned files and invocation names preserve
the original's reproducibility. The local v2 installation is a symlink to this checkout, with physical
reference resolution and structure checked. No separate UI-discovery experiment is claimed.

Before calls, compiler probes reject unbraced else-if, a local result binding and unparenthesized field
match subjects. Corrected forms compile and yield six known native values. An identity-sync probe fails
without changing old metadata, then allocates the missing declaration after source repair while preserving
old IDs. The first match probe incorrectly assumed a direct function call needed parentheses. The compiler
accepted it; that failed assumption and source are retained, and the reminder was narrowed before
registration. The [probe receipt](../benchmarks/results/2026-11-26-skill-revision-probes.json) records the final checks.
No language/parser/tool implementation changed to accommodate the skill.

## Preregistered design

Commit `de27b40b0f0631005c9bfcf2773bbc08d5a4118a` was pushed before any participant or availability call.
The [manifest](snapshots/2026-11-26-skill-revision/registration/manifest.json) pins all delivered inputs,
prompts, both skills, seventeen code/material files, 78 common references, compiler hash, calibration,
schedule and all three contrasts. Both skills and the tasks freeze for every attempt. No retry,
replacement, fallback or outcome-driven tuning occurs. One source-free availability check passes;
it is excluded from scores and included in costs.

There are twelve fresh ephemeral CLI sessions: two tasks × two repetitions × three arms. Every session
uses GPT-5.6-Luna/high, 120 seconds, sixteen command-execution items, six visible-check submissions and
no hard token cap. Arm order rotates by task and reverses in the second repetition. Original and revised
have equal aggregate position frequencies; four blocks cannot balance all three positions exactly.
Comparisons within this run use identical configurations; alias names do not establish immutable weights
or unchanged provider/CLI conditions across dates. Do not compare absolute scores with the earlier tasks.

All arms share requirements, starter/model/util code, source IDs, visible cases, native client, basic guide
and the same selected 78-file documentation packet. Control has no skill entrypoint. Original or revised
adds exactly its corresponding skill and invocation. This tests explicit skill revision availability,
text/name/path, workflow and salience together. There is no equal-text placebo or identified effect of
an individual reminder. Both can retrieve the same library/specification/card facts.

A temporary private CODEX_HOME excludes globally installed skills; credentials/model metadata remain outside
participant workspaces/evidence and the home is removed. Memories, apps, plugins and delegation are disabled.
Only the assigned skill path appears in each skill arm's command traces; neither path appears in controls.
No external skill-home read is detected. Hidden provider instructions and full OS-level read isolation are
not proved. The common packet has known incomplete transitive links, although direct skill targets exist.
Raw Markdown publication uses exact text exports and a logical-path/hash map; it does not complete those
links or alter delivered filenames. See the snapshot README for reconstruction.

The frozen process/mailbox/timeout implementation is reused through isolated in-memory hooks. Normalization
is extended only for the new payload-free case, with None/absent-payload tests; old drivers and scores are
unchanged. Checks compile and execute native machine code against independent Python oracles. Final grading
runs after the agent window. IDs, fixed files and budgets are acceptance criteria, not self-reported success.

## Fresh tasks and outcomes

| Task | Requested development | Control | Original | Revised |
| --- | --- | ---: | ---: | ---: |
| Scalar-sequence search | Construct first-match lookup, typed cases, prefix/suffix and byte count | 2/2 | 0/2 | 2/2 |
| Inclusive-span coalescing | Change a filter into overlap/adjacency merging with invalid-span handling | 2/2 | 2/2 | 2/2 |
| Completion within budget | Primary outcome | 4/4 | 2/4 | 4/4 |
| Final candidates passing tests | Separate functional observation | 4/4 | 4/4 | 4/4 |

Each final suite has five repeated public examples and 65 private cases. The same case set per task is
reused across arms/repetitions; these are not independent task samples. Each arm's candidates pass 280/280
full-suite checks, including 260/260 private checks. All 840 executed final native observations agree with
the Python oracle (780 private and 60 public repetitions). Required source signatures also match.
The two original search sessions reach the wall limit, so their correct candidates do not satisfy the
registered completion criterion. No late completion, rerun or posthoc grade repairs those outcomes.

Original search repetition zero has no visible client submission before timeout; its final static check
passes. Repetition one has a passing visible native check, then attempts git diff in a non-Git isolated
workspace and receives exit129 before timing out. Both candidates are captured after termination and pass
final native grading. These traces illustrate remaining verification/finalization overhead; they do not
identify which skill sentence or service delay caused the timeout. The revised skill's stopping/recovery
wording is a plausible contributor, not an isolated causal result.

Versus the original, revised has two completion gains and no reverse transition. Versus control, all four
pairs complete in both arms. This small development-targeted sample supports the observed revision result,
not population success rates, noninferiority or a general advantage from adding skills.

## Diagnostics and workflow observations

Revised has no visible compiler or authoring diagnostic failures in these four attempts. Control has three
visible compiler errors; original has two visible compiler errors and two authoring diagnostics. They
include the same unbraced else-if and reserved-result mistakes seen earlier. All recover sufficiently to
produce passing final code. No complex-match failure occurs in scored work, so its reminder is validated
by probes, not a demonstrated scored-task prevention effect. Combined revision changes and small samples
prevent assigning error avoidance to the grammar paragraph alone.

Three revised attempts check the untouched starter before editing, versus one original and no controls.
The other revised coalescing attempt exposes an edited behavior failure before repair. These observations
show some test-first/checking behavior, not proof of complete TDD adherence or newly authored independent
tests. Revised median command use is 6.5, original 9 and control 7.5; visible submissions are 2, 1.5 and
1.5 respectively. Per-attempt check histories, first passing edited-check times, commands and diagnostics
are retained in [all results](../benchmarks/results/2026-11-26-skill-revision-study.json) and
[public bundles](../benchmarks/fixtures/skill-revision-20261126/trial-00.json), indices 00..11.

## Tokens, timing and unknowns

Ratios below are medians of matched reported input-plus-output totals, not ratios of byte counts or
unpaired arm medians. Failed attempts stay in acceptance/wall summaries. Missing terminal usage is
unknown, never zero or an imputed pass.

| Contrast | Complete token pairs | Median token ratio | All observed wall pairs | Median wall ratio |
| --- | ---: | ---: | ---: | ---: |
| Original / control | 2/4 | 1.8469 | 4/4 | 1.4433 |
| Revised / control | 4/4 | 0.9977 | 4/4 | 1.0199 |
| Revised / original | 2/4 | 0.6912 | 4/4 | 0.7015 |

Both complete revised/original token pairs are coalescing repetitions: 30.9% fewer at their median,
range 0.5220–0.8605. Original search usage is unknown for both timeouts. This is not a complete four-pair
cost comparison or total savings estimate. Revised/control median is approximately unchanged (0.23% lower),
with ratios from 0.5524 to 2.2680 and two token gains/two losses. No consistent token advantage over control
is demonstrated. Aggregate revised/control totals differ by 7.8%; that is a different weighting and must
not replace the preregistered matched summary to promote a favorable result.

| Arm | Known terminal records | Known reported token subtotal |
| --- | ---: | ---: |
| Control | 4/4 | 764,203 |
| Original | 2/4 | 476,306 |
| Revised | 4/4 | 704,296 |

The original subtotal excludes both timed-out attempts and is not its complete cost. Ten known scored
records total 1,944,805 tokens. Availability adds 11,190. The known subtotal across eleven of thirteen
invocations is 1,955,995: 1,916,668 input plus 39,327 output. Cached input 1,763,328 is part of input;
reasoning 20,812 is part of output. Neither is added again. Full participant tokens, billing, parent
skill/harness-development tokens and provider-request counts remain unknown.

Revised/original observed wall ratios are 0.5626–0.9370; the median is descriptively 29.9% lower. Two
original windows are capped by timeout, so these are observed window costs rather than uncensored times
to successful completion. Revised/control ratios span 0.6067–1.9739, median 1.0199. These are not native
program speed or reliable inference-latency claims. Session windows include startup, lookup, reasoning,
service latency and verification. Other host load and managed caches remain; two repetitions per task are
not a reproducible speed benchmark. No heavy local compiler suite runs during participant timing.

## Interpretation and next scope

The new candidate is smaller, teaches verified failure-specific rules, avoids observed grammar diagnostics
in these attempts and improves budget completion relative to the original on identical fresh work.
All arms' final code is functionally correct on the suites, and ordinary documentation plus native feedback
matches revised completion with essentially equal median paired tokens. This is not evidence that every
Lobo task needs a skill, that shorter prompts always win, or that the language is superior to Rust/Python.
The earlier source-only card study and prior development study retain their different tasks, controls and
limitations; no historical score is replaced.

Keep v2 available as `$lobo-development-v2` and preserve v1 as tested. More independently authored task
instances, other requirement classes and controlled content/workflow ablations would be needed for a
broader claim. Those are proposals only. Stop at these twelve assignments; no further model calls,
language changes or post-outcome skill edits accompany this result.

## Verification and preservation

TDD retains missing-module failures and the corrected match-probe assumption. All 59 harness tests
(nine new) and 15 research tests pass. Six reference/starter/mutant programs execute 420 calibration
observations: 140 references, 140 starter outcomes and 140 mutation outcomes. Both compiled defects are
caught. Three corrected grammar examples yield six native values; sync failure/repair preserves old IDs.
Initial incorrect probe evidence remains separate from final validation and model results.

The compiler SHA-256 remains `8d9f84530b8d34a6a72597d25befc249a604430a6569656b4184721e1e444d9d`.
Its earlier 312-pass receipt is reused, not a fresh full compiler run; no new editor suite is claimed.
[Verification](../benchmarks/results/2026-11-26-skill-revision-verification.json) and
[read-only audit](../benchmarks/results/2026-11-26-skill-revision-audit.json) cover exact inputs/prompts,
usage, source signatures, native labels, snapshots and archive integrity. Paper local-046 preserves all
45 prior entry objects and 160 pins. The original skill, cards, product and old studies remain byte-identical.
Public exports preserve logical names/bytes and failed observations. The local raw archive retains original
input trees, intermediate candidates, binaries and CLI streams; it is not an off-device backup. No credential
home is included. Model aliases, incomplete reference closure and finite case samples remain limitations.
