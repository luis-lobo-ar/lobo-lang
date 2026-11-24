# Prompt and input-documentation treatments at frozen Lobo 1.11.2

Date: 2026-11-23. Decision [0066](../decisions/0066-isolate-comprehension-context-treatments.md).
All 72 registered source-only attempts completed. Each arm accepted 12/18 under the primary strict
grader. Deduplication reduced reported tokens, but this pilot does not demonstrate improved comprehension.
No language, compiler, library, extension or old research fixture changed.

## Design and retained inputs

Registration commit `a2f5048` was pushed before participant calls. The
[manifest](snapshots/2026-11-23-context-study/registration/manifest.json) pins the driver, tests, compiler,
36 delivered prompts, input files, schedule and reference validation. The experiment uses
GPT-5.6-Luna/high then low, 120 seconds per attempt, fresh ephemeral contexts, read-only mode and no tools.
There is no hard token cap. Model aliases do not identify immutable weights. All 74 invocations, including
two excluded availability checks, have terminal usage records. No retries, replacements, timeouts,
infrastructure failures or tool uses occurred. The scored schedule took about 30.7 minutes.

The four arms isolate two changes. `dedup` removes only the second TASK/GUIDE copies. `schema` appends
one common input-documentation block, identical across languages within each family. `both` combines
them. `original` retains duplication. Other authoring/check noise and language-specific guides remain
constant. Input documentation describes wire encoding, not source-language syntax.

Fresh programs use amount cap 97, highest-ID scheduling, checksum seed 7 and renamed helpers. Every
Lobo identity value is preserved while helper-name bindings change. Six new predictions and five other
items reuse the earlier task/item templates. The impact convention explicitly includes the edited
predicate in every arm. Questions were authored after reviewing old failures, not by an independent
blinded author. These are fresh pilot questions, not an unseen confirmatory sample. Absolute old/new
score differences cannot identify treatment effects. Four balanced arm orders reduce position bias;
there are only three families and one program form per family.

The [protocol and commands](../benchmarks/context-study.md) describe reproduction and limits.
All [public trial bundles](../benchmarks/fixtures/context-study-20261123/trial-00.json) retain delivered
prompts, initial files, answers, grades, commands, usage and raw-evidence hashes. Trial indices 00..71
cover the complete schedule. Raw events and stderr remain in the archived private evidence.

## Primary results

| Arm | High accepted | Low accepted | All accepted | Passed items |
| --- | ---: | ---: | ---: | ---: |
| Original | 7/9 | 5/9 | 12/18 | 192/198 |
| Deduplication only | 8/9 | 4/9 | 12/18 | 189/198 |
| Input documentation only | 8/9 | 4/9 | 12/18 | 191/198 |
| Both | 7/9 | 5/9 | 12/18 | 188/198 |

Each treatment versus original has three matched pass-to-fail and three fail-to-pass transitions across
18 pairs. Equal acceptance counts do not establish equivalent quality or noninferiority. Item accuracy
is slightly lower in the treatment arms. The small, correlated sample does not support a population
ranking or an accuracy improvement claim.

Lobo accepted 11/24, Rust 16/24 and Python 21/24 across configurations and arms. Prediction accuracy is
139/144, 141/144 and 143/144 respectively. Lobo's per-arm acceptance is original 3/6, dedup 3/6,
schema 3/6 and both 2/6. These strict counts mix behavior, response shape and instruction filtering.
They do not isolate intrinsic language difficulty or establish a construction-quality result.

All 32 wrong items are retained with supplied inputs, exact answers and expected results in the
[primary data](../benchmarks/results/2026-11-23-context-factorial-study.json):

| Item | Passed | Failed |
| --- | ---: | ---: |
| Behavior predictions | 423/432 | 9 |
| Construct a witness | 57/72 | 15 |
| Filter direct calls | 67/72 | 5 |
| Semantic change impact | 71/72 | 1 |
| Cued policy location | 72/72 | 0 |
| Construct a counterexample | 70/72 | 2 |

All five call-list errors are low-reasoning Lobo responses that add `reverse`, which the question
explicitly excludes. They identify a real call but fail the requested filter. That is not evidence that
the model failed to see a dependency. The filtering exception also burdens Lobo's persistent-list
implementation differently from the comparator implementations. The policy location is cued by name
in the question, so 72/72 does not establish uncued fault-localization ability.

Behavior errors include ordering jobs incorrectly under the highest-ID policy, running an unaffordable
job, accepting a locked debit, dropping an audit entry, and rejecting a valid checksum. Some contradict
explicit requirements. Witness errors include wrong balances/checksums, missing input fields and extra
metadata. Do not assign causal shares to documentation, language or model weights from these examples.

## Response-format ambiguity and secondary sensitivity

The registered grader requires exact top-level input keys. The original and newly standardized input
documentation do not explicitly forbid additional top-level properties. This omission in the new study
documentation is a protocol defect. Several answers supply a correct input plus explanatory output fields.
Other answers provide only a desired output, omit required inputs or supply an input with wrong behavior.
Those cases must remain distinguishable.

The preregistered primary scores above remain unchanged. A separately labeled
[posthoc sensitivity](../benchmarks/results/2026-11-23-context-format-sensitivity.json) removes only unknown
top-level properties from witness/counterexample inputs. It never invents missing fields, changes values,
relaxes nested shapes or alters the other checks. Seven items change and four attempts become accepted:

| Arm | Strict accepted | With top-level metadata ignored |
| --- | ---: | ---: |
| Original | 12/18 | 13/18 |
| Deduplication only | 12/18 | 12/18 |
| Input documentation only | 12/18 | 14/18 |
| Both | 12/18 | 13/18 |

The sensitivity total is 52/72 attempts and 767/792 items, versus primary 48/72 and 760/792.
This is an assessment-interface sensitivity, not a replacement score or a new intervention result.
Every failed item is still published. All 69 applicable original/projected failure-input replays agree
with the independent oracle across Lobo native C, Rust and Python references. No compiler mismatch is
demonstrated by these cases.

## Token usage and time

Deduplication removes 40.05–43.49% of user-prompt UTF-8 bytes across these fixtures. That is not a token
or billing percentage. Median paired reported input-plus-output token ratios, each over 18 pairs, are:

| Treatment versus original | Median token ratio | Median input-only ratio | Median wall-time ratio |
| --- | ---: | ---: | ---: |
| Deduplication only | 0.9323 | 0.9302 | 1.0256 |
| Input documentation only | 1.0076 | 1.0081 | 1.0096 |
| Both | 0.9473 | 0.9386 | 1.0813 |

Deduplication uses about 6.8% fewer reported tokens at the median pair. Both uses about 5.3% fewer.
Lobo's corresponding median token reductions are about 7.0% and 6.4% across six pairs. There is no
observed median wall-time improvement. Service latency, caching, temporal order and generated reasoning
vary; this is not a runtime benchmark or a causal inference about inference speed.

Scored attempts report 1,184,190 input tokens and 90,812 output tokens, totaling 1,275,002.
Cached input is 771,712 of those input tokens; reasoning output is 76,690 of those output tokens.
Do not add these subsets again. Availability checks add 28,228 reported tokens. All 74 invocations total
1,303,230 reported tokens, with zero unknown usage records. Actual billing and parent-agent development
usage remain unknown. High/low scored totals are 653,028 and 621,974, with acceptance 30/36 and 18/36.
High-before-low order and alias/provider drift prevent a clean causal reasoning comparison.

## Language reference and useful next improvements

The additive [agent reference](../docs/agent-reference/1.11.2/README.md) contains 22 concepts and 22
relationships, source hashes, semantic rules, pitfalls and bounded topic lookup. An editable draw.io map
is generated from the same data. It separates language syntax, libraries, runtime resources, tooling and
study input encoding. Seven native reading checks pass. A count/end-offset mistake is killed by the
slice example. XML structure and graph correspondence are checked. The host has no draw.io viewer;
the attempted open fell back to text, so rendered layout remains visually unverified.

The reference is excluded from every trial prompt. Its graph format, retrieval and examples have no
measured comprehension benefit yet. Provenance checks detect source drift; they do not prove every
English summary is complete. The normative specifications remain authoritative.

Keep the deduplicating builder and common input documentation as versioned, inspectable tools. Their
clearer contract and lower duplication are verified; an accuracy benefit is not. Before another selected
study, specify an explicit response envelope and additional-properties policy. Separate input from
optional expected-result explanations. Replace the arbitrary excluded-helper call-list question with
an explicitly classified or unfiltered call graph. Test those changes independently with new forms.

A graph experiment should compare the same facts in flat and graph-retrieved form under matched
retrieval budgets. A tool-assisted comprehension arm could test whether evaluation catches constructive
mistakes, separately from source-only understanding. Those are proposals, not automatic new model runs.
No result here justifies adding syntax, changing Bytes APIs or unfreezing Lobo.

## Verification and preservation

TDD retained the initial missing-module red checks, an invalid scheduler counterexample, an arm-order
imbalance, subsequent corrections and green tests. Ten prompt/grading/configuration tests, five reporting
tests, three sensitivity tests and seven reference tests cover the new code. Eight earlier research tests
also pass. Eighteen reference/mutant builds pass 1,908 oracle comparisons; all nine compiled policy
mutants are killed by behavior. Seven standalone native example outputs and a slice-mistake mutation
are checked separately. A prepublication command-metadata correction retains its original record.

The [verification receipt](../benchmarks/results/2026-11-23-context-study-verification.json) records final
repository checks, compiler tests, source/prompt/grade audits, artifact hashes and the verified raw archive.
Paper entry local-041 preserves all 40 earlier entries and 123 pins. No prior score, report, task or
fixture is replaced. The registered driver and old product remain frozen after this pilot.
