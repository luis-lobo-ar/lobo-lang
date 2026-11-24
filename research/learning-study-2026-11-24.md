# Response contracts, language reference and graph presentation

Date: 2026-11-24. Decision [0067](../decisions/0067-response-contract-reference-and-graph-experiments.md).
All 84 registered attempts completed, with known usage for all 90 participant/availability invocations.
The explicit contract made the response rules unambiguous and improved observed shape compliance. The
flat reference showed a small, mixed comprehension gain. Graph presentation showed no consistent advantage.
Error review finds incomplete operation/grammar coverage, limiting conclusions about either presentation.
Compiler/extension 1.11.2, Core 1.11, libraries, skill and the existing reference remain frozen.

## Registered comparisons

Commit `678a7ee` was pushed before the first participant call. The
[registration](snapshots/2026-11-24-learning-study/registration/manifest.json) pins 84 delivered prompts,
13 code/material files, the compiler, validation and schedule. All three comparisons were fixed before
outcomes. Later read-only diagnostics implement already selected metrics; they do not alter grades.

Every attempt uses GPT-5.6-Luna, high then low within its stage, in a fresh ephemeral read-only session.
Tools are prohibited and shell/apps/plugins/multi-agent/memory features are disabled. The wall limit is
120 seconds with no hard token cap. All 84 attempts finished normally, with no tool use, timeout,
infrastructure failure, retry, replacement or unattempted assignment. Six source-free availability checks
are excluded from scores and included in costs. The scored schedule took about 26 minutes.

| Stage | Matched arms | Fresh tasks | Attempts |
| --- | --- | --- | ---: |
| Response format | Ambiguous wording / explicit v2 profile | Three families in Lobo/Rust/Python | 36 |
| Reference | Base guide / flat reference | Six Lobo topics, form A | 24 |
| Graph | Flat reference / identical facts as adjacency data | Six Lobo topics, form B | 24 |

The format programs use cap 83, lowest-ID scheduling, checksum seed 13, new inputs/targets/helper names
and unchanged Lobo identity values. The family/item templates are inherited. Both arms use the same
strict validator, one guide/task copy, common input documentation and optional top-level notes.
Only the explicit response-profile disclosure is added. No provider-native sampling constraint is used.

The Lobo topics cover carried state/lists, byte/scalar ranges, nominal records and inference, try,
callback/action signatures, and contracts. Each task has six value predictions and four static-admission
judgments. Metadata is correctly supplied by the host. Shared declarations appear once. Form B has
different runtime constants, helper/type names and admission cases. Both forms test related laws.

The reference uses the existing 22 semantic nodes, 22 relationships and checked example source. Flat
and graph packets have identical node facts, relationships, provenance and examples. The graph groups
edges in adjacency lists; it does not expose interactive lookup or traversal. The historical exposure
flag is omitted from both renderings. Reference presence adds information; graph presentation changes
organization/encoding. Compare arms within stages, not absolute form-A/form-B scores.

Questions were authored after reviewing earlier failures. These are new instances, not independently
blinded holdouts or a powered confirmatory sample. The
[protocol](../benchmarks/learning-study.md) describes exact procedures and limits.

## Results

| Stage and arm | Valid shapes | All tasks accepted | Correct task items |
| --- | ---: | ---: | ---: |
| Format: ambiguous | 15/18 | 9/18 | 177/198 |
| Format: explicit | 17/18 | 9/18 | 174/198 |
| Reference: base guide | 12/12 | 9/12 | 116/120 |
| Reference: flat packet | 12/12 | 10/12 | 118/120 |
| Graph: flat packet | 12/12 | 4/12 | 110/120 |
| Graph: adjacency packet | 12/12 | 5/12 | 109/120 |

The format contrast has two matched invalid-to-valid shape transitions and no reverse transition.
Whole-task acceptance has three gains and three losses. The explicit arm has three fewer correct task
items despite better shape compliance. A well-shaped answer is not necessarily correct.

Reference presence has two whole-task gains and one loss across 12 pairs, totaling two additional correct
items. The high-reasoning base/flat results are 59/60 versus 58/60 items and 5/6 versus 4/6 tasks. Low
results are 57/60 versus 60/60 items and 4/6 versus 6/6 tasks. The aggregate improvement is small and
does not hold in both configurations. It does not establish a general reference advantage.

Graph presentation also has two task gains and one loss. It has one fewer correct item overall. High
flat/graph results are 57/60 versus 56/60; low results are 53/60 in both. More accepted tasks but fewer
correct items reflect different error concentration. Neither metric can be chosen after seeing outcomes
to declare a graph win. No consistent comprehension advantage is demonstrated by this presentation test.

All 38 unsuccessful attempts and 72 failed item checks are retained in
[the results](../benchmarks/results/2026-11-24-learning-study.json) and
[the diagnostics](../benchmarks/results/2026-11-24-learning-diagnostics.json).
The [public bundles](../benchmarks/fixtures/learning-study-20261124/trial-00.json), indices 00..83, include
every exact prompt, answer, result, canonical example and raw-evidence hashes. Canonical witnesses are
examples; other inputs are graded against behavior, not equality to those examples.

## What failed

The four format-invalid replies comprise two non-JSON task-mode responses, one per arm, and two
ambiguous-arm witnesses containing Summary outputs instead of required inputs. The mode responses say
they cannot edit/run checks without tools. The frozen guide still contains authoring/check instructions.
Explicit disclosure also reinforces the requested response mode, so the contrast cannot identify a causal
effect of the unknown-property policy alone. The specific older pattern of a valid input plus extra
metadata did not recur. The new contract fixes that ambiguity by definition and executable validation;
this trial does not independently prove prevention of that exact older failure mode.

Nonresponses contribute eleven failed task checks each. Those failures are not eleven independent
misunderstandings of code. Remaining format-stage misses include state/order/arithmetic and witness
errors. Prediction accuracy is 189/216, including the nonresponses. The custom call filter and cued policy
location remain unchanged, so this stage does not establish uncued localization or pure language ability.

Reference-stage predictions are 141/144 correct; static judgments are 93/96. The exact callback-action
rule is a useful narrow signal: both base-guide configurations incorrectly allow a pure function reference
where an action-bearing Fn signature is expected. Both flat-reference counterparts classify that case
correctly. Other differences include arithmetic and a false-precondition static-admission mistake. This
is evidence about these cases, not an identified causal contribution of one reference sentence.

Graph-stage predictions are 139/144 correct; static judgments are 80/96. Shared errors expose gaps in
the tested packet:

- All four range-task replies reject a statically valid snippet containing `Bytes::concat`. Neither guide
  nor semantic packet supplies a complete builtin signature table. The compiler accepts the snippet.
- Three of four propagation replies reject a declared function named try called as `try(3)`. The packet
  says the parenthesized spelling is a function call, but gives no complete contextual-keyword grammar.
- All four propagation replies permit direct try in an ensures clause. The compiler rejects it. The
  short try entry does not state the prohibition on direct propagation in contracts.
- Both low-reasoning nominal replies reject the permitted payload-free `Missing()` spelling. The short
  reference emphasizes bare cases and does not explicitly teach this accepted spelling.
- Contract mistakes include permitting actionful calls inside pure contracts and treating a false
  precondition as a static rejection. The packet has no dedicated contract node or complete contract grammar.
  The question does explicitly distinguish static checking from runtime/proof; that instruction did not
  ensure correct application.

False judgments do not identify whether a model considered a name unavailable, misunderstood a type,
or applied a runtime condition. Coverage gaps are observable; their causal shares remain unknown.
These omissions exist in a selected-rule reference; the full specifications contain the rules. Source
paths/hashes are listed, but agents cannot fetch those texts in source-only trials. Packaging identical
incomplete facts as a graph cannot supply a missing signature or grammar rule. This limits conclusions
about graph usefulness and full-language comprehension.

Other errors contradict available facts. Both low-reasoning range replies return byte 231 where raw byte
offset 2 yields 169, consistent with mixing scalar and byte positions. Byte/scalar distinctions are stated
in the packet. This observation does not reveal the model's internal reasoning. The CJK scalar value,
flagged as a background-knowledge risk before these trials, was answered correctly in all four graph-stage
range replies. No item was removed or rescored.

The [material-coverage audit](../benchmarks/results/2026-11-24-learning-material-coverage.json) was recorded
after registration and before any reference/graph outcome. All 297 applicable submitted witness and
counterexample executions agree with the independent oracles across the three reference implementations;
see [observed-input replays](../benchmarks/results/2026-11-24-learning-observed-input-replays.json).
No compiler mismatch is demonstrated by these observations.

## Costs

Ratios below are medians of matched reported input-plus-output token totals. All failed attempts are
included. They are not byte ratios, billing estimates or total study-development costs.

| Contrast | Pairs | Median token ratio | Observed increase | Median wall ratio |
| --- | ---: | ---: | ---: | ---: |
| Explicit / ambiguous | 18 | 1.0247 | 2.5% | 0.9011 |
| Flat reference / base | 12 | 1.2420 | 24.2% | 0.9716 |
| Graph / flat facts | 12 | 1.0682 | 6.8% | 1.0103 |

The reference adds useful facts, irrelevant facts, examples and provenance overhead. The graph encoding
adds tokens while preserving facts. These results favor testing smaller complete context packets before
assuming a larger reference or adjacency dump is efficient. They do not establish superiority of flat
text for interactive retrieval or repository-scale work.

Wall ratios have wide ranges: 0.433–2.233, 0.491–1.806 and 0.785–2.248 respectively. Service latency,
cached prefixes, provider behavior and reasoning length vary. There is no inference-speed or native
runtime-performance claim. No locally heavy compiler checks ran during participant timing.

Scored usage is 1,473,635 input plus 70,272 output tokens, totaling 1,543,907. Cached input is 718,336
of the input; reasoning output is 62,314 of the output. Six availability checks add 84,701 tokens.
All 90 invocations total 1,628,608 reported tokens, with zero unknown terminal-usage records. Cached
input and reasoning output must not be added again. Billing, immutable model weights, underlying provider
request counts and parent-agent development usage remain unknown. High-before-low ordering and repeated
related tasks limit causal reasoning and population-level interpretations.

## Decisions and next proposals

Keep the explicit [v2 response contract](../benchmarks/response-contract-v2.md). It has one executable
field definition for rendering and validation, gives explanations a defined location, and preserves
invalid replies. Its structural clarity is verified; model compliance remains imperfect.

Keep the current reference and graph frozen as the tested materials. The next candidate improvement is
complete checked context: builtin signatures with units/fault behavior, contract typing/purity/proof
boundaries, and accepted contextual/constructor spellings. Add worked examples that are distinct from
scored tasks. A concise mode-specific reading guide could remove authoring/check noise. These are
documentation/context proposals, not new syntax or a reason to expand the language.

A later selected study can compare a checked complete packet with the current packet on independently
authored fresh tasks. Bounded fact retrieval or tool-assisted verification should be a separate contrast
from graph presentation. More tasks, repetitions and models are needed before a general recommendation.
The present source-only studies do not measure reliable repository construction, human comprehension,
graph traversal quality or production software correctness.

## Verification and preservation

TDD retained initial missing-module/syntax failures, a freshness regression, repeated shared declarations,
their corrections and green tests. The registered code had 14 focused tests; two later read-only diagnostic
tests also pass. The frozen compiler suite passes 312 tests, with no failures or ignored tests. Earlier
research/harness tests remain green. Final validation includes 1,908 cross-language oracle comparisons,
144 native/evaluator value observations, 48 static admissions and 21 killed compiled defects. Two earlier
development validation rounds are retained separately and are not additional independent trial evidence.

The [verification receipt](../benchmarks/results/2026-11-24-learning-verification.json) records prompt,
grade, source, graph-fact and archive checks. Paper local-042 preserves all 41 earlier entries and 131 pins.
All original primary scores, the older posthoc sensitivity, registered prompts and failures remain intact.
No language, library, extension, reference-fact or old-fixture change is part of this milestone. Stop at
these 84 attempts; further documentation changes or model trials require a new selection.
