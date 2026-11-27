# Small checked-reference pilot

Date: 2026-11-24. Decision [0068](../decisions/0068-small-guide-and-fact-pilot.md).
All 24 registered attempts completed on frozen compiler/extension 1.11.2, Core 1.11.
Short checked topic cards improved comprehension on these selected tasks. Guide cleanup alone did not.
No language, library, editor, old guide/reference/graph or historical score changed.

## What changed

The earlier study exposed missing builtin signatures and contextual grammar/contract rules. This pilot
adds a [reading-only guide and four cards](../docs/agent-reference/compact-pilot-2026-11-24/README.md).
The reading guide preserves the old semantic paragraphs verbatim, removes authoring/check instructions,
and adds a source-only reading cue. It shrinks from 3,443 to 2,003 UTF-8 bytes. Each targeted prompt adds
one relevant card, 744–934 bytes, rather than the entire reference. These are byte counts, not tokens.

The cards state buffer parameter/result types, byte versus scalar units, runtime fault boundaries,
nominal constructors and updates, inference limits, contextual try spellings, and contract Bool/purity
rules. They explicitly distinguish static admission from runtime success and proof. The machine-readable
packet keeps source hashes and anchors outside delivered prose. It covers these four topics, not the
whole language. General rules are taught; scored source names and gold answers are absent from cards.

## Registered procedure

Commit `ef3d410` was pushed before calls. The [registration](snapshots/2026-11-24-compact-pilot/registration/manifest.json)
pins 18 code/material files, 24 exact prompts, the compiler hash, validations, schedule and contrasts.
Four fresh program/admission sets cover ranges, nominal values/inference, propagation and contracts.
Each has six value predictions and four independent static-admission judgments. Every arm gets the same
program, question, Unicode input legend and strict v2 response contract. Correct identity metadata is
host-supplied. There is no scoring relaxation, repair or posthoc replacement.

Three arms use the existing guide, the cleaned reading guide, or the cleaned guide plus its topic card.
GPT-5.6-Luna runs 12 high-reasoning attempts, then 12 low, with rotated arm order within topics. Each
attempt has a fresh ephemeral read-only session, zero tools, a 120-second wall limit and no hard token cap.
The old process runner is reused through in-memory hooks; its disk bytes stay frozen. Two source-free
availability checks are excluded from scores and included in costs. All attempts completed normally;
there were no retries, replacements, tool calls, timeouts, infrastructure failures or unattempted tasks.

These examples were authored after reviewing earlier errors. They are development-targeted transfer
instances, not independently authored blind holdouts. The three preregistered comparisons are old→clean,
clean→targeted, and old→targeted. The second estimates the addition of facts conditional on cleanup.
There is no facts-only arm or measured interaction.

## Results

| Context | Whole tasks accepted | Correct items | Value predictions | Static admissions |
| --- | ---: | ---: | ---: | ---: |
| Existing guide | 0/8 | 68/80 (85.0%) | 47/48 | 21/32 |
| Reading-only guide | 0/8 | 67/80 (83.8%) | 48/48 | 19/32 |
| Reading guide + relevant card | 8/8 | 80/80 (100%) | 48/48 | 32/32 |

All 24 response shapes were valid. Every targeted-card task passed at both reasoning settings: 4/4
high and 4/4 low. High item scores are old35/40, clean34/40, targeted40/40; low scores are33/40,
33/40 and40/40. All arms already predict values well. Errors concentrate in static admissions, so
whole-task acceptance magnifies a small number of mistakes: a task needs all ten answers correct.
The targeted arm gains eight matched tasks against either baseline, with no reverse transition.

The [results](../benchmarks/results/2026-11-24-compact-pilot.json) retain all 25 failed item checks in
the other arms. Errors include rejecting accepted payload-free `Case()` construction; inferring generic
function arguments only from an expected return type; rejecting a statically accepted out-of-range byte
operation; confusing `Text::valid(Bytes)` with Text input; admitting direct propagation in contracts;
rejecting an ordinary declared function named try; treating a false postcondition as static rejection;
and allowing an actionful call inside an unreachable contract branch. One old/low reply also predicts
a fold result instead of propagation of its error. Some rules were already partly stated in the old
semantic paragraphs; the treatment adds precision and salience. It does not identify which sentence
caused each correction or prove the model's internal reason for a wrong judgment.

All outcomes, delivered prompts, exact answers, canonical answers and raw-file hashes are public in the
[trial bundles](../benchmarks/fixtures/compact-study-20261124/trial-00.json), indices00..23. No negative
result was removed. The grade replay agrees for every attempt, including every unsuccessful reply.

## Costs and timing

Ratios are medians of eight matched reported input-plus-output token totals, including failed attempts.
They are neither ratios of guide bytes nor billing estimates.

| Contrast | Item difference | Median token ratio | Median wall ratio |
| --- | ---: | ---: | ---: |
| Reading / existing | -1 | 0.9764 (2.4% lower) | 0.8748 |
| Targeted / reading | +13 | 1.0009 (0.09% higher) | 0.5971 |
| Targeted / existing | +12 | 0.9778 (2.2% lower) | 0.6251 |

Cleanup alone saves some reported tokens without improved accuracy. Adding precise facts to the cleaned
guide substantially improves these answers with effectively unchanged median token use. Against the
existing guide, the combined condition improves accuracy while using slightly fewer median tokens.
That combined token ratio ranges0.8451–0.9861; targeted/clean ranges0.8675–1.0100. Aggregate arm totals
are131,213,127,577 and125,184 tokens respectively. Aggregate reductions and median paired reductions
are different summaries and must not be substituted for each other.

The targeted condition has fewer output tokens (2,447 versus7,589 old and6,309 clean); this partly
offsets its additional facts. Reported reasoning is a subset of output. Cached prefixes, model-service
behavior and reasoning length affect costs/timing. The combined median wall ratio0.6251 is an observed
37.5% reduction in response wall time, not native program speed or an established inference-speed gain.
No local heavy compiler suite ran during participant timing.

Scored usage is367,629 input plus16,345 output, totaling383,974 tokens. Two availability calls add28,240.
All26 calls total412,214 reported tokens. Cached input310,656 is part of395,859 input; reasoning15,625
is part of16,355 output. Neither is added again. All terminal usage records are known. Billing, parent-agent
development tokens, underlying provider-request counts and immutable model weights remain unknown.

## Interpretation and next step

The promising change is compact, task-relevant, compiler-checked rules. Simply removing text did not
improve comprehension. Keep the packet available as a tested research artifact; do not replace frozen
study inputs or silently promote it to a universal reference. A later selected trial should use independent
fresh questions, broader topics and another model, then separately test bounded lookup/tool-assisted use.
No further participant run is part of this task.

The observed 100% is a small-pilot ceiling on four selected program instances repeated across reasoning
settings. Items share programs, and high precedes low; they are not80 independent samples. Prior errors
informed topic selection and card content. Whole-task0/8→8/8 must not be compared directly with older
studies' different tasks. We cannot claim full-language comprehension, generalization, noninferiority,
causal reasoning-level differences, better repository construction, Rust/Python superiority, production
correctness or native runtime gains. This supports a context-delivery hypothesis, not a language-feature claim.

## Verification and preservation

TDD preserves initial missing-module failures for the new driver and reporter, followed by six passing
new tests. Final harness tests pass40/40 and research tests15/15. Before participant calls, the frozen
compiler validates48 native/evaluator value observations,16 static cases,28 positive/negative signature
probes for14 operations, and four compiled behavior mutations that fail the expected results. These
are calibration observations, not extra model attempts or independent research replications.

The compiler SHA-256 is unchanged. Its earlier312-pass regression receipt is reused under decision0068;
that full compiler suite was not rerun here. No extension check is newly claimed. The
[verification receipt](../benchmarks/results/2026-11-24-compact-verification.json) records exact checks,
registration/prompt audits, code/export hashes and raw-archive integrity. Paper local-043 preserves all42
prior entries and141 pins, including the older scores and limitations. Public snapshots contain the
registered materials and development outcomes; local raw archives also retain native binaries and CLI
streams. Local archives are not off-device backups. Product and tested materials stay frozen. Stop at24.
