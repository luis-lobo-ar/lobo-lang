# Isolating v3 instructions on harder Lobo applications

Registered 2026-11-26; completed 2026-11-27 (client date, America/Chicago).
Artifact names and the registered machine-result date retain the study-start tag2026-11-26.

This study tests three proposed additions to the frozen v2 skill: requirement-to-evidence coverage,
resource/action contracts, and evidence-based finishing. It also tests their combined v3 candidate.
The user explicitly selected creation and separate tests. Lobo1.11.2/Core1.11, compiler, libraries,
editor, v2, earlier skills/cards/studies and all47 prior paper entry objects/177 pins remain frozen.
[Decision0073](../decisions/0073-isolate-v3-correctness-skill-changes.md) fixes scope and stopping rules.

## Registered design

Registration `4e071e274a920eb16e28951d79e20d7dee6a19fa` was pushed before availability/scored calls.
The [manifest](snapshots/2026-11-26-v3-study/registration/manifest.json) pins delivered inputs, prompts,
code, skill blocks, compiler, shared materials and calibration. Copied Markdown is exported as .md.txt
with original logical paths and hashes in [the map](snapshots/2026-11-26-v3-study/document-path-map.json).
The private raw input tree preserves actual delivered filenames and bytes.

| Arm | Instructions delivered |
| --- | --- |
| v2 | Frozen v2 body; common experimental alias |
| requirements | v2 plus requirement mapping, boundaries/failure-state evidence and requirement-based TDD |
| resources | v2 plus borrowed/owned resource handling, transitive actions, cleanup and original errors |
| finish | v2 plus acceptance gate, remaining-time/check planning and stopping without weakening verification |
| v3 | v2 plus all three blocks in the fixed order above |

Every arm uses the SAME skill name/path/frontmatter and explicit invocation: lobo-development-study.
Only selected appended blocks differ. Original v2 is never edited; frontmatter.name is renamed identically
for delivery. The permanent [v3 skill](../.agents/skills/lobo-development-v3/SKILL.md) has the same tested
combined body under its own versioned name. UI metadata and automatic discovery are validated separately,
not effects identified by this experiment. The [three blocks](../benchmarks/matched-study/v3_parts.json)
contain generic instructions, not benchmark solutions or domain constants.

There are20 fresh ephemeral attempts: two new tasks x2 repetitions x5 arms, four matched assignments per
arm and per contrast. Rotation/reversal is fixed in the manifest; four blocks cannot perfectly balance
five order positions. Each contrast reuses the same v2 baseline assignments, so contrasts are correlated.
This is five of eight possible factor combinations, not a full factorial interaction study. No no-skill
arm is present. Instruction content and extra text length change together. Common alias sizes are3141,
3679,3665,3687,4749 bytes for v2/requirements/resources/finish/v3 respectively.

All attempts use the same configured GPT-5.6-Luna/high,120-second agent window,16 command-execution items,
six visible checks and no hard token cap. Fixed tasks,78 shared reference files, legacy scaffolds, four
editable logic/policy source+sidecar files, explicit source bundle and identity-authoring helper are the
same. Temporary CODEX_HOME has empty external skill discovery; authentication stays outside evidence/
workspaces and is removed. Apps/plugins/memories/delegation disabled. One source-free availability check
is excluded from scoring but included in known costs. No retries/replacements/fallback or post-outcome
edits. Stop after20, or after three consecutive infrastructure failures while retaining all unattempted rows.

## New tasks and independent tests

| Task | New required behavior | Native observation |
| --- | --- | --- |
| Event feed | Strict query parser with unordered/duplicate/empty/overflow cases; published/type filtering BEFORE ascending cursor pagination; lookahead/nullable next, I64_MAX and HEAD | Compile9 sources, launch bounded loopback TCP server, parse wire status/headers/JSON/UTF-8 length, pair HEAD with GET |
| Inventory reservation | Normalize two-line carts, ignore zeros, merge duplicates safely, canonical reorder-insensitive replay/conflict, validation priority, multi-stock+ledger atomicity and rollback/cleanup | Compile5 sources with SQLite, reuse DB across requests, inspect inventory/requests after EACH operation, probe open transaction before caller close |

These add concrete requirements beyond the prior catalog/transfer instances. No model pilot selects
for failures or adapts hardness. Legacy scaffolds run but lack required behavior; no dormant complete
solution is supplied. The feed starts with5/40 scenarios passing, reservation7/30. Input requirements
are fully stated to every arm, including Unicode key length, opaque quoted/NUL keys, ignored zero lines,
overflow-safe duplicate quantities and missing/decode/stock priority. The SQLite schema deliberately
rejects ledger key 'reject' with an actual CHECK error: valid input must yield the original engine error
and roll back all prior stock decrements. Host setup can seed a wrong SQLite column type to test strict
decoding. This does not imply arbitrary-trigger/concurrency or production security coverage.

The independent Python route/state-transition oracles use exact nested JSON types from registration.
The task implementations/references do not determine expected outputs. Feed40 scenarios (5public35private),
reservation30 (5public25private), reused across arms/repeats:70 distinct scenarios and700 planned final
scenario evaluations, NOT700 independent development projects. Some SQLite scenarios contain multiple
calls; HEAD adds a separate GET comparison and each server has one startup probe. A compile failure has
no executed native suite, which must be distinguished from observed behavioral errors. Retain placeholders
for any unexecuted cases rather than claiming every planned test actually ran.

## Calibration and verification before calls

Final [calibration](../benchmarks/results/2026-11-26-v3-validation.json) has nine compiling programs:
references40/40 and30/30; starters5/40 and7/30; duplicate-query mutant37/40, cursor mutant36/40,
missing-aggregation mutant28/30, skipped-rollback mutant27/30, ignored-second-stock mutant28/30.
All five deliberate defects compile and fail behavioral checks. That is310 final calibration scenario
evaluations, with further native calls inside some scenarios. Earlier development calibration results
are preserved separately, not silently discarded or counted as participant calls.

Two fixture-authoring compile failures were corrected before calls: a parameter shadowed the ordered
function, and an if-expression used as an unbound statement had an invalid following semicolon. No
language/library change fixed either. Both identity-authoring helper probes pass. Skill frontmatter/UI
and references are validated.80 harness tests (11 new) and15 research tests pass before calls, including
preserved failing-first logs, single-factor fixture isolation/common activation, schedule, independent
oracles, canonical-cart equivalence, I64_MAX/overflow, fixture integrity, strict JSON types, repeated-task
pairing, unknown usage and nonmutating reporting.

## Acceptance, cost and limits

Primary acceptance is completion within fixed budgets AND fixture/declaration/ID integrity AND every
final native scenario passing. Report candidate correctness separately; a correct timeout stays a primary
failure. Report requirement/case coverage and compile errors separately as well. Compare cost among
correct, completed solutions descriptively, but retain all failed outcomes and unknowns in the full
accounting; do not conceal survivorship by presenting only favorable pairs. Token ratios require complete
matched task+repetition records and family coverage labels. Cached input/reasoning output are subsets,
not extra totals. Unknown terminal usage stays unknown, never zero. Billing, provider-internal requests,
parent development tokens, hidden host instructions and immutable weights remain unknown.

Full SQLite compilation remains inside120 seconds with35-second build/40-second client ceilings; no
prebuilt cache/backend/budget change. Final calibration SQLite builds are about21 seconds, HTTP about2.
Pending public controller work may drain after agent termination without granting extra usable time;
late results cannot rescue primary acceptance. Final private grading occurs outside the agent window.
The16-core shared host had load3.22/3.29/4.04 before registration, with other work; order/cache/load and
different edit/check sequences confound latency. No native runtime-speed or inference-speed claim follows.

The compiler SHA256 remains `8d9f84530b8d34a6a72597d25befc249a604430a6569656b4184721e1e444d9d`.
Reuse its previous312-pass receipt honestly; no new full compiler/editor suite. Common78 references retain
known incomplete transitive links, although direct skill links are present. Four matched assignments per
contrast across TWO authored task instances are exploratory, not powered/blinded population estimates.
Finite tests do not prove whole-contract correctness. Any extension/promotion/further trials require a
new user selection; findings never authorize automatic skill or language evolution.

## Results

All20 registered assignments ran; none were retried, replaced or left unattempted. Every session hit
the wall limit. [Full results](../benchmarks/results/2026-11-26-v3-study.json) retain all four contrasts,
all final source snapshots, native outcomes, process details and unknowns.

| Arm | Timely complete | Final candidates passing the whole suite | Passing planned scenarios | Terminal usage known |
| --- | --- | --- | --- | --- |
| v2 | 0/4 | 1/4 | 81/140 | 0/4 |
| requirements only | 0/4 | 0/4 | 53/140 | 0/4 |
| resources only | 0/4 | 0/4 | 17/140 | 0/4 |
| finish only | 0/4 | 0/4 | 53/140 | 0/4 |
| full v3 | 0/4 | 1/4 | 52/140 | 0/4 |

No addition shows a timely-completion or whole-candidate correctness advantage over v2 in this sample.
Full v3 and v2 each produce one fully passing HTTP candidate in different repetitions. Neither produces
a fully passing reservation candidate. The isolated additions produce none. This does not establish
equivalence, population inferiority of the additions, or that the advice is generally unhelpful. The
primary measure has a floor: every attempt exhausts the same120-second window. Four assignments per
contrast across only two task instances cannot support a general ranking.

Passing scenario totals are diagnostic, not a substitute objective: inherited starter behavior contributes
passes, and compile failures have no executed suite. They must not be treated as hundreds of independent
software-construction samples or used to trade a critical invariant for many easy assertions. There are
256 passing observations out of700 planned final scenarios;610 actually execute. Three reservation
candidates fail compilation, leaving90 planned scenarios unexecuted. All10 HTTP suites and7 SQLite suites
execute:450 real HTTP connections (including startup/HEAD comparisons),301 SQLite request invocations.

| Index | Task / repeat | Arm | Final stage | Final suite | Source unchanged |
| --- | --- | --- | --- | --- | --- |
| 0 | HTTP /0 | v2 | behavior failure | 34/40 | no |
| 1 | HTTP /0 | requirements | behavior failure | 34/40 | no |
| 2 | HTTP /0 | resources | behavior failure | 5/40 | yes |
| 3 | HTTP /0 | finish | behavior failure | 34/40 | no |
| 4 | HTTP /0 | v3 | whole suite passes | 40/40 | no |
| 5 | SQLite /0 | requirements | unchanged starter | 7/30 | yes |
| 6 | SQLite /0 | resources | unchanged starter | 7/30 | yes |
| 7 | SQLite /0 | finish | unchanged starter | 7/30 | yes |
| 8 | SQLite /0 | v3 | compile failure | not executed | no |
| 9 | SQLite /0 | v2 | unchanged starter | 7/30 | yes |
| 10 | HTTP /1 | requirements | unchanged starter | 5/40 | yes |
| 11 | HTTP /1 | v2 | whole suite passes | 40/40 | no |
| 12 | HTTP /1 | v3 | unchanged starter | 5/40 | yes |
| 13 | HTTP /1 | finish | unchanged starter | 5/40 | yes |
| 14 | HTTP /1 | resources | unchanged starter | 5/40 | yes |
| 15 | SQLite /1 | resources | compile failure | not executed | no |
| 16 | SQLite /1 | requirements | unchanged starter | 7/30 | yes |
| 17 | SQLite /1 | v2 | compile failure | not executed | no |
| 18 | SQLite /1 | v3 | unchanged starter | 7/30 | yes |
| 19 | SQLite /1 | finish | unchanged starter | 7/30 | yes |

Every row is a timeout, including the two whole-suite successes. “Source unchanged” compares logic and
policy bytes with the initial scaffold, not notes/identity serialization. Twelve candidates remain
unchanged; their starter failures are unfinished construction, not introduced defects. Eight candidates
change source: five HTTP candidates compile (two pass; three fail ordering), and all three edited SQLite
candidates stop at compilation. This limits what the study tells us about higher-level resource reasoning:
many attempts never reach that implementation stage.

## Failures and process observations

The [audit](../benchmarks/results/2026-11-26-v3-audit.json) checks all30 registered code pins, all20
input/prompt/usage/primary/case labels, existing declarations/IDs and unchanged product paths. Every
session refers to the common skill path in commands; none references either external skill home.
Observed path reads do not prove complete instruction consumption or OS-enforced read isolation.

The three34/40 HTTP candidates (indices0,1,3) fail the same six multi-item ordering checks: correct items,
count and cursor, but array order is reversed. This is a concrete Lobo API-use mistake rather than evidence
of a missing rule: the unchanged [JSON guide](../libraries/json/README.md) explicitly says serialized array
elements follow insertion order, while builder internals prepend to reversed lists. Reading the internal
representation alone can suggest an incorrect extra reversal. The finite observations do not isolate
whether retrieval, attention, model knowledge or task timing caused that mistake.

Two edited SQLite candidates (8,15) have first final diagnostic `compile.syntax.invalid`, `expected }, got Symbol(";")`, from a conditional expression followed by further statements without binding its
result. The author hit and corrected this same syntax pattern during pre-call reference calibration.
Candidate17 instead reports `compile.name.unresolved`, “unknown value: Ok”. Its untyped conditional
mixes a try-unwrapped Bool branch with Ok(true), with no resolved case owner. Merely qualifying Ok
would not establish branch-type agreement. These are first diagnostics, not all defects hidden
behind parsing/resolution. The generic v3 blocks do not remove these Lobo-specific authoring failures.
No compiler/library change is made to accommodate any candidate.

The fully passing edited public checks arrive at117.05s for v3/HTTP0 and103.02s for v2/HTTP1. Neither
session emits terminal completion before the deadline. One finish-only public check drains at120.75s,
after termination; its failing result does not grant extra usable time or rescue primary acceptance.
There is no automatic elapsed/remaining-time field in the common check response; actors can use available
shell tools for timing, but the finish-policy text alone does not supply telemetry. This is a workflow
limitation and a proposal for a separate test, not a cause identified by the experiment.

Every attempt uses at most7 command-execution items and3 visible checks, below both configured caps.
Wall time is the binding limit. Some attempts perform a baseline native failing check and never reach
source edits. Full SQLite engine builds occupy part of the window, but HTTP attempts time out too;
compiler time alone does not explain the result. Provider/tool turnaround and reasoning duration are
not independently identified. All observed window durations are censored near120s; do not use them
as a speed ranking.

## Tokens, interpretation and next proposals

All20 scored terminal usage records are UNKNOWN. There are zero complete token pairs for every contrast,
including every task family. No token-efficiency comparison, complete scored token total or cost per
correctly completed task can be calculated. The accounting's scored known subtotal0 describes an EMPTY
known set; it is not zero actual usage. The single source-free availability call reports11,187 input+
output tokens, so the21-call experiment has one known record and20 unknowns. That is an availability-only
known subtotal, not a participant-cost estimate. No imputation from wall time, output length or other
studies is performed. Parent development/billing/provider-internal costs remain unknown.

The result does not justify choosing v3 over v2. Keep both as versioned artifacts; no default replacement
or post-outcome tuning is performed. The experiment reaches a difficult operating point where the primary
completion measure floors and many implementations never start. It tests these blocks under these limits,
not unconstrained software quality or the language's maximum capability. Do not compare absolute rates
with earlier, different tasks and call that a skill or language effect.

Proposed follow-up, not implemented or run:

- Target the observed Lobo facts with concise executable examples: JSON builder insertion order;
  conditional-result binding before subsequent statements; contextual constructors and consistent
  branch types after try unwraps a result.
  Verify each example against the frozen compiler and use fresh evaluation tasks. The JSON fact is
  already documented, so test retrieval/presentation rather than claiming the language manual lacks it.
- Establish agent feasibility on separate development tasks before freezing new evaluation instances.
  Keep budgets equal across arms; avoid selecting or simplifying evaluation tasks from their own results.
- Test a shared, source/compiler-pinned public failing-baseline receipt so native TDD red evidence can
  be reused instead of rebuilt in every short attempt. Retain independent expected behavior and native
  post-edit verification; do not weaken tests or silently grant extra time to one arm.
- Separately test explicit remaining-time feedback and a finish policy. Preserve candidate correctness
  and timely session completion as distinct outcomes, rather than rewarding a premature final answer.

No additional participant calls, skill edits, caching, budget changes or language evolution follow from
these proposals. The study stops at20.

## Evidence closure

New v3 remains alongside unchanged v2.80 harness tests (11new),15 research tests, skill validation,
native calibration and repository JSON/JSONL/link checks pass. The unchanged compiler's prior312-pass
receipt is reused, not a new full compiler/editor run. The published snapshot retains failed calibrations,
red/green logs, registered delivery bytes, environment and all final candidates/native observations.
Original filenames, full event streams, intermediate edits, native binaries and SQLite files remain in a
verified local raw archive. It is local evidence, not an off-device backup; no authentication file is included.
Paper local-048 appends the completed study while preserving all47 earlier entry objects and177 pins.

The closed raw archive has5323 verified members,21,584,431 bytes, SHA256
`be2576f92012751c2373c95618f5019db316b76d98e53f282db514fdee0f93cc`. The [archive manifest](snapshots/2026-11-26-v3-study/evidence-manifest.json)
and [verification receipt](../benchmarks/results/2026-11-26-v3-verification.json) preserve closure/coverage.
Completion is recorded2026-11-27 using the supplied client date; file names and registered machine
results retain the2026-11-26 study-start tag.
