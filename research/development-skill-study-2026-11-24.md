# New Lobo development skill: matched pilot

Date: 2026-11-24. Decision [0070](../decisions/0070-matched-development-skill-pilot.md).
The new skill did not improve development results in this small pilot. Control passes 6/6 tasks;
skill passes 5/6, with one timeout leaving an uncompiled candidate. Every complete known-usage pair
uses more reported tokens with the skill. Keep these findings separate from the earlier source-only
comprehension gain. Compiler/extension 1.11.2, Core 1.11 and all tested skills/cards remain unchanged.

## Registered comparison

Commit `4e8fd802294d565df291b3010646e3de7ade4278` was pushed before the first participant call.
The [registration](snapshots/2026-11-24-development-study/registration/manifest.json) pins all delivered
prompts/fixtures, ten code/material files,78 shared reference files, compiler hash, native calibration,
schedule, metrics and limits. A preliminary preparation snapshot is retained privately. Before publication
and before any participant call, the comprehension-only reading guide was excluded from both development
arms so that its no-tool instruction cannot interfere. Task requirements/oracles and the skill were not
changed after calls began. No participant retry, replacement or fallback occurred.

Twelve fresh ephemeral CLI sessions use GPT-5.6-Luna/high: three tasks × two repetitions × two arms.
Each attempt has 120 seconds,16 command-execution tool items and six visible native-check submissions,
with no hard token cap. Each task has control-first and skill-first pairs. One source-free availability
check passes; it is excluded from scores and included in usage. All 12 assignments are attempted,11
complete normally and one reaches its wall limit. No infrastructure stop or unattempted assignment occurs.
The participant windows sum to approximately 16.8 minutes; preparation/calibration/publication are separate.

Control and skill get identical requirements, initial source/identity files, five visible cases, check
client, basic language guide and 78 reference files. These include all four compact checked cards, normative
specifications, library sources/guides and existing programming-skill references. Neither arm receives the
older programming SKILL.md. The treatment adds the exact frozen new SKILL.md plus one explicit invocation
sentence. It tests availability, activation, instruction/salience and retrieval together; there is no
flattened equal-text placebo or isolated estimate of a particular workflow sentence.

A temporary private CODEX_HOME has copied authentication/model metadata and an empty skills directory,
so the globally installed Lobo skill is outside discovery. Credentials never enter participant workspaces
or evidence; the temporary home is removed. Apps, plugins, memories and multi-agent features are disabled.
The control workspace has no new skill entrypoint. All six skill traces explicitly reference its path;
none of the six controls do. No trace contains an external skill-home path. This verifies supplied files
and observed reads, not every hidden provider instruction or an OS-level proof of read isolation.

The unchanged frozen process runner is loaded in memory with new fixture/grader hooks. A separate function
code object changes only the exact common prompt constant for the treatment invocation. Original source
bytes stay frozen. Registered-prompt and initial-file audits agree for every attempt. Visible checks compile
and execute optimized native Lobo; final hidden grading runs after the agent window. Direct compiler probes
share the command/wall limits. All acceptance criteria are independent of candidate output.

## Fresh tasks and outcomes

| Task | Development work | Control | Skill |
| --- | --- | ---: | ---: |
| Unicode splice | Construct typed window replacement, removed text and scalar/byte deltas | 2/2 | 2/2 |
| Batch conversion | Construct ordered conversion/aggregation and first-error propagation | 2/2 | 1/2 |
| Last-entry resolution | Change a working active filter to last-entry/tombstone semantics | 2/2 | 2/2 |
| Total | Complete acceptance within fixed limits | 6/6 | 5/6 |

These are new requirements and implementations, authored after reviewing past failures. Bounds/error laws
relate to earlier topics; these are development-targeted instances, not independently blinded holdouts.
Generic reversal is supplied equally as scaffolding. Model/util sources and existing declaration IDs are
fixed; logic and its sidecar are editable. Optional scratch tests cannot alter the grading criteria.

Each final candidate has a 70-case suite: five repeated public examples and 65 private cases. The same
private cases per task are reused across both arms/repetitions; these are not independent task samples.
Control passes 420/420 full-suite checks; skill 350/420. On the private subset those scores are 390/390
and 325/390. The missing 70 full-suite checks are unexecuted because the timed-out candidate fails
compilation, rather than 70 separate wrong runtime values. All 770 executed final native observations
(715 private and 55 public repetitions) match the Python oracle. Acceptance also requires
preserving fixtures/IDs and staying within budgets. Captured labels and required source signatures are
audited without changing the registered grades. No additional outcome is silently repaired or substituted.

There are five pairs where both arms pass, one control-pass/skill-fail pair, and no reverse transition.
The sample is too small to establish a population-level skill disadvantage or dependable success rate.
It supplies no demonstrated construction or change-quality gain from this skill under these conditions.

## Workflow observations and failure

Median command use is 7 control versus 9 skill; median visible-check submissions 1 versus 2.5. Three skill
attempts check the untouched starter before editing, compared with none of the controls. Two proceed to
an edited passing check; one is the timeout. This is a narrow observed test-first pattern, not proof that
agents authored new independent tests or followed the entire skill. All twelve final source bundles and
per-check observations are retained in the [public bundles](../benchmarks/fixtures/development-study-20261124/trial-00.json),
indices 00..11. Exact command traces, first visible passing edited-candidate times and diagnostics are in
[the results](../benchmarks/results/2026-11-24-development-study.json) and
[read-only audit](../benchmarks/results/2026-11-24-development-study-audit.json).

The unsuccessful skill batch attempt checks the failing starter, edits the implementation and repeatedly
fails identity authoring. Three diagnostics report unsupported unbraced `else if` syntax; another reports
the reserved local name `result`. Identity sync validates source before writing metadata, so these syntax/
name failures prevent allocation for the new RunState record. At 120 seconds, the candidate still lacks
that identity and final compilation reports it. The missing-ID symptom should not be mistaken for four
independent identity mistakes. Its repeated diagnostic-guided edits do not reach an accepted candidate.
The second scheduled skill batch repetition succeeds; it is an independent registered assignment, not a retry.

These mistakes are not exclusive to the skill arm: controls also encounter reserved-name, nested-if and
complex-match/contextual-constructor errors and recover. Visible check-client compile errors total 2 control
and 3 skill; authoring commands expose another 4 control and 6 skill diagnostics. Keep these categories
separate from the failed candidate's final compilation. Skill visible behavior failures total 4, including
three untouched-starter failures and one edited requirement-change failure that is repaired. Controls have
no visible behavior failure, but their edited compile failures remain counted. More checking did not yield
higher final acceptance here.

The trace shows observed errors and recovery, not why the model selected a particular reference or syntax.
We cannot attribute the timeout solely to language design, documentation, the model or one skill sentence.
The native compiler and authoring tools return the documented diagnostics; no compiler mismatch is shown.

## Tokens and wall time

Across the five pairs with complete terminal usage, the median skill/control reported input-plus-output
ratio is 1.4119: 41.2% more tokens. The range is 1.1539–1.5156, so every known pair uses more with the skill.
The sixth pair has unknown skill usage because its timeout lacks a terminal usage record. It remains a
failure in acceptance and timing; its tokens are not imputed as zero or removed from the study inventory.
The paired token summary is a complete-record subset, not an exact six-pair total or a billing estimate.

| Quantity | Control | Skill |
| --- | ---: | ---: |
| Terminal usage known | 6/6 | 5/6 |
| Known reported token subtotal | 1,140,183 | 1,258,437 |
| Median reported tokens among known records | 180,953.5 | 224,396 |
| Median commands | 7 | 9 |
| Median visible check submissions | 1 | 2.5 |

The skill's token subtotal covers fewer attempts and omits its failed attempt; do not label it a complete
arm total. All 11 known scored records total 2,398,620 tokens. The availability check adds 11,186. The known
subtotal across 12 of 13 invocations is 2,409,806:2,367,689 input plus 42,117 output. Cached input 2,173,696 is
included in input; reasoning 22,931 is included in output. Neither is added again. Full participant tokens,
parent development usage, billing and underlying provider-request counts are unknown.

All six matched wall ratios are available: median 1.1136, range 0.7830–1.3627. This is descriptively 11.4%
more wall time at the median paired task, with wins and losses. The unpaired arm medians are 81.0s control
and 79.2s skill; they are a different summary and must not replace the matched result. The host has other
load on 16 logical CPUs, and sessions include CLI startup, reference loading, model service, tool feedback
and reasoning. No native runtime speed or reliable inference-latency ranking is established. Two repetitions
per task do not provide a reproducible performance benchmark. Full compiler tests run neither during nor
after participant timing; their existing receipt is reused below.

Larger context and more tool rounds are visible contributors to reported token consumption. The exact
causal contribution of each is not isolated; there is no provider-side profiling or billing measurement.
First-visible-pass timings exist for six controls and five skill attempts, with the failed attempt censored.
Every value is retained; successful-subset timings must not be promoted over the primary failure outcome.

## Interpretation and proposals

The earlier source-only card experiment supplied missing facts and improved selected comprehension answers.
This comparison already gives both arms those facts and native feedback. Adding the development skill
shows no gain in final quality or reported token use here. The studies ask different questions and their
absolute scores cannot be compared. This does not invalidate the earlier observations or establish that
skills generally harm coding, nor does it compare Lobo with Rust/Python or identify language superiority.

Keep the tested skill available and unchanged as an evidence artifact. Candidate improvements for a later
selected revision are a small mandatory grammar reminder (braced else branches, reserved result, complex
match subjects), fewer repeated workflow instructions, and an explicit recovery rule to fix the first
identity-sync diagnostic before continuing. A future fresh-task comparison could separate that grammar
content from workflow packaging. These are proposals, not implemented fixes, extra trials or a language
unfreeze. Stop at the twelve registered attempts.

## Verification and preservation

TDD retains initial missing-module failures, a nested fixture-writer mistake, wrong record-list CLI argument
encoding, an incomplete report-test fixture and their corrections. A preliminary registration is retained;
only the final published registration is delivered. Final harness tests pass 50/50 (ten new) and research
15/15. Calibration compiles all nine reference/starter/mutant programs and executes 630 native observations:
210 for correct references,210 starters and 210 deliberate mutations. All three behavioral mutations are
caught. Earlier failed calibration is kept separately and is not extra independent model evidence.

The compiler SHA-256 remains `8d9f84530b8d34a6a72597d25befc249a604430a6569656b4184721e1e444d9d`.
The earlier 312-pass compiler receipt is reused, not newly rerun; no new editor suite is claimed. Product,
SKILL.md, cards, old studies and historical scores stay frozen. The
[verification receipt](../benchmarks/results/2026-11-24-development-study-verification.json) records inputs,
prompts, usage, case-label/source-signature audits, checks and archive integrity. Paper local-045 preserves
all 44 earlier entry objects and 153 file pins. All commands, failures, raw streams, candidate changes and
binaries remain in the local archive; public snapshots/bundles preserve the inputs and results with hashes.
The local archive is not an off-device backup and no credential home is included. Model aliases are not
immutable weights, hidden provider instructions are not fully observed, and finite held-out samples do not
prove all stated input domains or production correctness.

## Reference-packet and publication limitation

The common packet supplies 78 selected files, not a full checkout. Its normative documents have some
transitive links to omitted guides, decisions and examples. Both arms receive the identical packet;
all direct links from the new skill's entrypoint are present. Completeness of every reference chain is
not established. This can affect retrieval and limits interpreting the treatment as an ideal fully
self-contained development environment. No material or score is repaired after the trial.

The final publication link check also encounters those links in byte-identical raw input copies. Its
failures are retained, including an ineffective scoped-ignore attempt. For publication, raw Markdown is
exported with `.md.txt` filenames and an exact logical-path/SHA-256 map. The original named directory
layout remains in registration commit `4e8fd80` and the local raw archive. Registration hashes and the
filenames actually delivered to participants remain unchanged. Canonical repository docs are checked
at their original paths. Passing that check must not be described as completing participant links.
