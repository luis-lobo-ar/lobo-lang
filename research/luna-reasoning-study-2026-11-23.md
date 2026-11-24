# Frozen Lobo study with Luna at high and low reasoning

Date: 2026-11-23. Status: completed; all scored attempts, failures and usage retained.

The user requested two repeats of the frozen comparison: first GPT-5.6-Luna at the same high reasoning
setting as the GPT-6.1-Sol baseline, then at its lowest advertised setting, low. This is a comparison of
configured coding agents, without claiming an independently measured weakest-model ranking.

## Registered design

[Decision 0064](../decisions/0064-luna-high-and-low-reasoning-study.md),
[the protocol](../benchmarks/luna-reasoning-study.md) and
[the registration](../benchmarks/results/2026-11-23-luna-study-registration.json) fix both cohorts before
participant launches. Each has 54 assignments: three families, Lobo/Rust/Python, construction,
source-only comprehension and requirement changes, two repetitions. All high assignments precede low,
as requested. Each conversation starts fresh. No participant is retried or replaced.

The driver imports the original hash-verified controller rather than changing its behavior. Unit checks
verify CLI parity except the model/reasoning arguments and byte-identical task, guide and source hashes.
The compiler, extension, libraries, skills, source archive and old research remain frozen at 1.11.2,
Core 1.11. The old baseline is recorded in [the earlier report](matched-agent-study-2026-11-23.md).

Budgets remain 120 seconds, 16 command-execution items and six visible checks; compilation and checks count
in agent time. Final grading is outside that budget. There is no hard token cap. Report actual cumulative
input/output usage; cached input remains within input, reasoning output remains within output. A missing
terminal usage record is unknown, not zero. Parent orchestration/harness-development usage and billing
remain unknown. An agent conversation can contain multiple underlying model requests.

Before each cohort, one excluded, source-free, shell-disabled access check invokes the requested model
and reasoning setting. Access failure leaves that cohort's assignments unattempted; no model fallback.
Three consecutive infrastructure failures stop a cohort, while candidate compile/test/limit failures are
ordinary measured outcomes. Every result, source, check, timeout, unknown and unattempted assignment stays
in the record. The controller then proceeds to the next configuration's access check.

## Outcomes

Both cohorts completed every registered assignment: **108 attempted, 93 accepted**, with no retries,
unattempted assignments, infrastructure interruptions, budget overruns or missing terminal usage. Both
excluded access checks succeeded. All 110 participant/access-check CLI invocations have reported usage.
The old baseline and its separately archived infrastructure restart remain unchanged.

[The full results](../benchmarks/results/2026-11-23-luna-reasoning-study.json) retain every attempt,
check, failure, source bundle, raw-evidence hash, paired outcome and cost. Accepted assignments below
combine all three phases; each language/configuration has 18 assignments from the same three families.

| Configured model / reasoning | Lobo | Rust | Python | Total |
| --- | ---: | ---: | ---: | ---: |
| GPT-6.1-Sol / high (earlier baseline) | 18/18 | 18/18 | 18/18 | 54/54 |
| GPT-5.6-Luna / high | 15/18 | 17/18 | 17/18 | 49/54 |
| GPT-5.6-Luna / low | 14/18 | 16/18 | 14/18 | 44/54 |

The phase breakdown separates code writing from source-only comprehension. Every construction/change
candidate compiled. All final failures arose from incorrect executed behavior or comprehension items.

| Luna setting | Construction | Requirement changes | Comprehension attempts | Comprehension items |
| --- | ---: | ---: | ---: | ---: |
| high | 18/18 | 18/18 | 13/18 | 192/198 |
| low | 17/18 | 18/18 | 9/18 | 187/198 |

Lobo passed all 12 construction/change assignments at each setting. Rust passed 12/12 at high and 11/12
at low; Python passed 12/12 at both. Lobo comprehension accepted 3/6 at high and 2/6 at low, with 62/66
and 61/66 individual items correct. Rust accepted 5/6 at both settings, with 65/66 and 64/66 items. Python
accepted 5/6 at high and 2/6 at low, with 65/66 and 62/66 items. Whole-attempt acceptance requires all eleven
items; one wrong witness can reject an otherwise correct comprehension response.

All 6876 held-out program observations were executed. High passed 3438/3438. Low passed 3415/3438;
the 23 incorrect observations belong to one scheduler implementation. No failed assignment was removed
from cost or quality denominators. Full source/answer bundles for all 108 attempts are public and hashed.

## Concrete failure evidence

Low-reasoning Rust construction trial 09 compiled and passed the visible checks, then failed 23/94 held-out
observations. Its `completed` vector tracks jobs by storage position, but dependency IDs index that
vector as though positions and IDs coincide. Arbitrary input order exposes the mismatch.

A read-only post-hoc replay minimized the problem to two jobs, in this storage order:

```text
jobs = [{id:1,cost:1,deps:[]}, {id:0,cost:1,deps:[0]}]
budget = 2
expected: order=[1],   spent=1, blocked=1
observed: order=[1,0], spent=2, blocked=0
```

Job 0 depends on itself, so it should remain blocked. This is a semantic identifier/index mapping error;
the program remains type-correct and memory-safe in the tested execution. A representation keyed by
job ID could avoid this particular mismatch in any language. One generated Rust mistake is not evidence
that Rust generally produces worse software. The minimized witness is diagnostic, not a newly scored
input or participant hint. The candidate was never repaired.

Comprehension misses include empty-frame counting, rejecting amounts above the ledger's upper bound,
change-impact selection and witness generation. An additional read-only audit ran all 54 prediction inputs
against the frozen canonical executables: every output agrees with the existing oracle. For example,
one Lobo response predicted two empty frames consuming 4 bytes for the 2-byte input `0000`; a Rust response
omitted its valid empty frame; a ledger witness with initial 9 and debits 2 then 3 does not meet the requested
balance/rejection target. These examples describe responses, without attributing the cause solely to syntax.
See [the diagnostic observations](../benchmarks/results/2026-11-23-luna-study-diagnostics.json).

## Participant time and token observations

Times below are median participant wall seconds over six attempts per phase/language. They include
visible build/check work; final independent grading is outside the participant budget. Failed responses
remain included. These are time spent producing responses, not time to a correct solution after hidden
failures are repaired.

| Configuration | Phase | Lobo seconds | Rust seconds | Python seconds |
| --- | --- | ---: | ---: | ---: |
| Sol high | construction | 36.60 | 28.95 | 23.42 |
| Sol high | comprehension | 9.11 | 8.92 | 8.98 |
| Sol high | change | 27.96 | 26.84 | 21.99 |
| Luna high | construction | 98.04 | 57.03 | 54.13 |
| Luna high | comprehension | 25.15 | 28.61 | 31.91 |
| Luna high | change | 65.21 | 52.28 | 59.94 |
| Luna low | construction | 57.20 | 29.10 | 25.26 |
| Luna low | comprehension | 14.68 | 13.96 | 15.40 |
| Luna low | change | 42.09 | 33.95 | 31.12 |

Matched-pair medians below compare the same assignment, pooling 18 language/family/repetition pairs per
phase. A token ratio sums reported input and output, including cached input, once. It is not a billing
ratio or equal-compute comparison. These are descriptive costs, including unsuccessful attempts.

| Comparison (left / right) | Phase | Reported-token ratio | Participant-wall ratio |
| --- | --- | ---: | ---: |
| luna-high / sol-high | construction | 1.462 | 2.256 |
| luna-high / sol-high | comprehension | 0.906 | 2.964 |
| luna-high / sol-high | change | 1.327 | 2.518 |
| luna-low / sol-high | construction | 0.970 | 1.057 |
| luna-low / sol-high | comprehension | 0.869 | 1.569 |
| luna-low / sol-high | change | 1.065 | 1.426 |
| luna-low / luna-high | construction | 0.665 | 0.499 |
| luna-low / luna-high | comprehension | 0.960 | 0.536 |
| luna-low / luna-high | change | 0.747 | 0.555 |

Low used less participant time and reported fewer tokens than high on these paired assignments, while
acceptance decreased from 49/54 to 44/54. Construction's median matched token ratio was 0.665 and wall
ratio 0.499; requirement changes were 0.747 and 0.555. Successful-only distributions and their denominators
are separate in the full data and do not replace these all-attempt outcomes.

High reported 6,812,089 input-plus-output tokens; low reported 4,819,402. Across 108 scored attempts,
input totals 11,431,575, including 10,144,896 cached input tokens, plus 199,916 output tokens. Reported
uncached input totals 1,286,679. Including two access checks yields 11,659,722 reported input-plus-output
tokens over 110 CLI conversations. Reasoning output is already part of output, not an additional chargeable
quantity to add. High reported 75,423 reasoning-output tokens and low 20,016; effort labels do not equate
reasoning-token counts or compute across different models. Billing and total study-development cost are
unknown, so these figures establish no dollar saving.

No new generated-program runtime benchmark was requested or run. The older native/Python runtime data
still describe their original artifacts; changing an agent setting does not change those binaries.

## Interpretation boundaries

The high/high comparison changes the requested model. The Luna high/low comparison changes requested
reasoning on the same model. These are named settings, not equal amounts of reasoning computation or
immutable model weights. The chronology can confound results through cache, load, service or time drift.
The old baseline was measured earlier; frozen input bytes and flags do not guarantee identical hidden
host instructions or provider behavior. Current versions, executable hashes and before/after load are
recorded. Model descriptions are not a capability benchmark.

The three task families are small, and the earlier baseline had a correctness ceiling. Repetitions do
not create 108 distinct real-world programming problems. Supplied comprehension contracts can reveal some
predictions, and the predicate-location item is cued. No mechanism ablation isolates H1/H2/H3 or syntax
from onboarding, collections, tool behavior and prior familiarity. Descriptive subgroup comparisons do
not establish a population language ranking.

Scope rejections and compiler failures are distinguished from executed functional failures. A rejected
workspace can leave algorithm behavior ungraded. Final source passing after a wall limit remains an
unsuccessful in-budget attempt. All-attempt costs include failures; any successful-only costs retain
their denominator and cannot replace primary acceptance results.

This request concerns agent configurations. Earlier timings still describe their original binaries;
newly generated programs have not received a new performance benchmark. No runtime advantage follows
merely from switching the model. No language/tooling feature is changed or automatically selected.
