# Why Lobo comprehension answers failed

Date: 2026-11-23. Status: offline analysis complete. Improvements are proposals only.
Compiler and extension remain 1.11.2. No language, library, programming guide, question, grader or prior
result was changed. No new model trial was launched.

There is no single identified cause. The answers show reasoning and constraint mistakes. We found
definite prompt-delivery defects and documentation gaps. Language and API presentation remain plausible
contributors. The tested canonical programs agree with their expected outputs. The existing experiments cannot assign
a causal percentage to the language, documentation or model.

## What the evidence covers

The audit covers all 54 source-only comprehension attempts from the Sol/high baseline and the Luna/high
and Luna/low cohorts. Each language has six attempts per configuration and eleven items per attempt.
Answers, actual participant prompts and source files were inspected. The audit uses final responses and
recorded outcomes, without treating private model reasoning as an explanation of intent.

| Configuration | Lobo attempts | Lobo items | Rust attempts / items | Python attempts / items |
| --- | ---: | ---: | ---: | ---: |
| Sol/high | 6/6 | 66/66 | 6/6; 66/66 | 6/6; 66/66 |
| Luna/high | 3/6 | 62/66 | 5/6; 65/66 | 5/6; 65/66 |
| Luna/low | 2/6 | 61/66 | 5/6; 64/66 | 2/6; 62/66 |

Lobo has seven unsuccessful attempts containing nine incorrect items. Pooled across the Luna settings,
five items have the same family/question failure in another language. Four items have such a counterpart
within the same configuration. These are overlap counts, not causal shares. Shared documentation could
also produce shared mistakes.

All Lobo direct-call, predicate-location and counterexample items passed. The predicate-location question
is cued, so those successes do not establish strong fault-localization ability. No failed item directly
tests identity management. Lobo also passed every construction/change assignment across the three
configurations. Tool-assisted code writing and unaided source comprehension are distinct outcomes.

The [machine-readable audit](../benchmarks/results/2026-11-23-comprehension-failure-audit.json) records
every failed item across languages. A [prompt snapshot](snapshots/2026-11-23-comprehension-audit/prompts.json)
preserves the nine distinct participant user prompts. Hidden host instructions are not included.

## Each incorrect Lobo item

Trial indices identify the original cohort assignments. Predictions use the original zero-based keys.

| Configuration / trial | Item | Actual response | Verified result | Interpretation and limits |
| --- | --- | --- | --- | --- |
| Luna/high 15 | Frames `prediction_1`, `data="0000",limit=0` | Two empty frames, consumed 4 | One empty frame, consumed 2 | Incorrect byte/frame counting. Hex-character confusion is possible, but the response alone does not prove it. |
| Luna/high 15 | Frames `impacted` | `["run"]` | `["fits","run"]` | Omitted the function whose body changes. The question's semantic definition includes it. The word "impacted" can invite a callers-only convention. |
| Luna/high 28 | Ledger `prediction_5` | Debit 1000001 succeeds, balance 0, rejected 0 | Debit exceeds the amount limit. Balance 1000001, rejected 1 | Missed a stated domain bound even though sufficient balance exists. Python also missed this question under low reasoning. |
| Luna/high 38 | Ledger witness | Initial 9, debit 2, debit 3 | Balance 4, rejected 0, peak 9. Target requires balance 7, rejected 1, peak 9 | A valid input that fails the target constraints. This witness can be checked from the contract without knowing Lobo syntax. |
| Luna/low 15 | Frames `prediction_1` | No frames, consumed 2 | One empty frame, consumed 2 | Preserved byte consumption but dropped the valid empty payload. Rust and Python also made empty-frame errors. |
| Luna/low 15 | Frames `prediction_2`, `data="02010203",limit=2` | Payload `01`, consumed 4, error 0 | Payload `0102`, consumed 4, error 0 | Consistent with treating slice's third argument as an end offset. The unchanged success/checksum claim also makes this an incomplete explanation. |
| Luna/low 28 | Ledger `prediction_5` | Same invalid large debit succeeds | Same rejection as above | Repeated boundary error. |
| Luna/low 38 | Ledger `prediction_5` | Same invalid large debit succeeds | Same rejection as above | Repeated boundary error on a fresh conversation. |
| Luna/low 44 | Scheduler `prediction_1` | Runs job 1, spent 1 | Runs affordable, lower-ID job 0, spent 2 | Violates the explicit selection policy. Cheapest-job selection and overwriting the best candidate are both compatible explanations. Neither is identified by the answer alone. |

All 54 canonical prediction executions, across the three languages, agree with the existing oracle.
The ledger witness was executed and fails its target. An isolated copy of the frame parser with the
existing strict-boundary mutation confirms both impact answers: `fits(0,0)` changes from true to false,
and the same empty-frame input changes the result of `run`. The original program and grader stay frozen.

Removing the disputed impact item alone would not make its attempt accepted. That response also has an
incorrect frame prediction. No answer was reclassified or removed from the original results.

## Documentation and prompt delivery

### The prompt repeats its task and guide

This is a confirmed assembly defect in [the frozen prompt builder](../benchmarks/matched-study/study.py#L700).
The controller first inserts `TASK.txt` and `GUIDE.txt`, then adds
copies through the `FILE` loop after merging the task-directory files. All 54 comprehension prompts have
two exact copies of each. The same loop adds `check.py` and `AGENTS.md` text.

| Language | Participant prompt bytes across the three families | Duplicate task/guide bytes |
| --- | ---: | ---: |
| Lobo | 13448 to 14997 | 5470 to 5952 |
| Rust | 9689 to 10534 | 4013 to 4492 |
| Python | 9285 to 10248 | 3940 to 4418 |

Exact task/guide duplication represents 39.7% to 43.1% of these user-prompt bytes. This is not a percentage
of total model context or billed tokens. The reported input includes host metadata and provider-specific
processing. No tokenizer or accuracy intervention measured a saving.

The source-only prompt says not to use tools, but its guide still instructs participants to edit files,
run checks and synchronize identities. Lobo receives a longer authoring guide, including an absolute
identity-command path. These instructions are irrelevant to the comprehension phase. The phase disables
shell tools, the task forbids tool use, and no tool use was recorded. The effect of conflicting cues on
accuracy is unknown.

The defect affects all languages. Lobo has more absolute repeated text, while duplicate shares are
similar across languages. Sol answered correctly despite the noise. These observations do not establish
that prompt duplication caused the Luna misses, nor do they invalidate the frozen observed scores.

### The input representation needs a common schema

The question says that bytes in answers use hexadecimal strings. It does not explicitly define the
hexadecimal representation of input `data` in the same sentence. The Python guide additionally explains
that the wrapper handles hexadecimal input/output; the Lobo and Rust guides lack that sentence.

Ledger inputs use numeric event pairs. Only the Python model directly assigns numeric values to event
kinds. Lobo and Rust readers must infer the JSON-code mapping from declaration order and the examples.
The failures do not demonstrate that this caused confusion, but the representation is less explicit and
less evenly documented than it should be.

### Several allegedly missing rules were already present

The shared task explicitly says that an empty payload is valid, that invalid amounts reject and that
selection uses the lowest eligible ID. The [Lobo guide](../benchmarks/matched-study/study.py#L492) explicitly defines
`Bytes::slice(data,start,count)`.
The impact question explicitly asks about different return values for identical direct arguments.

We should not describe those semantics as undocumented. The problem is whether models retrieve and apply
the stated facts reliably. Adding more repetitions of the same rules is not an evidence-based remedy.

## What the language could contribute

The supplied Lobo implementations require explicit list traversal, carried-state records, record updates
and prepend/reverse handling. Rust and Python often express those operations through their standard
collections and mutable locals. That creates an implementation and library confound in source reading.

For example, the supplied scheduler's logic has 1742 UTF-8 bytes in Lobo, 728 in Rust and 566 in Python.
Those are source-description measurements, not normalized complexity or model tokens. The longest Lobo
example is the scheduler, yet five of its six comprehension attempts pass. Length alone does not explain
the failure pattern. Different families also differ in semantic difficulty.

Two Lobo-specific risks merit testing:

- `Bytes::slice` takes a count. Readers familiar with end-based ranges could transfer the wrong convention.
  The guide already says count. A clearer example or a more explicit future API name is a testable idea,
  not a demonstrated fix. The bad answer also fails to reconcile its shortened payload with checksum logic.
- The frame header's `payload` field is an offset, while a local `payload` variable holds bytes. Names such
  as `payload_start`, `payload_byte_count` and `payload_bytes` could reduce that distinction's reader cost.
  The current names occur in other reference implementations too, so this is not solely a Lobo defect.

The record/variant syntax, stable IDs and immutable updates are not established causes. There was no
compiler-output discrepancy in the audited cases. Failing to infer a numeric result does not prove a
syntax-design defect, and passing construction with compiler feedback does not prove source readability.

## Attribution that the data supports

| Candidate explanation | Evidence | Conclusion |
| --- | --- | --- |
| Incorrect language implementation | Every canonical prediction matches execution; the impact mutation behaves as the grader expects | No demonstrated implementation defect in these audited cases. This is not a proof that the language has no bugs. |
| Model/configuration sensitivity | Same participant input bytes yield 66/66 Lobo items on Sol, 62/66 on Luna/high and 61/66 on Luna/low. Similar failures occur in the other languages | Strong descriptive association. No causal share or universal model ranking is identified. |
| Prompt delivery and documentation | Duplicate task/guide, irrelevant check client and edit instructions, uneven explicit input encoding | Definite design issues. Their effect on the recorded errors remains unmeasured. |
| Lobo source and API unfamiliarity | Explicit state traversal, count-based slicing and one shorter-payload answer | Plausible interaction with model reasoning. No isolated representation or naming intervention establishes causation. |
| Scoring and question conventions | One answer omits the changed predicate itself; witness generation has its own failure modes | Report abilities separately and make conventions explicit. Preserve original scoring. |

This study cannot distinguish a model's capability from prior language familiarity, source presentation
and guide quality. Model aliases and hidden host context are not immutable. The earlier baseline and
chronological high/low order add time/cache/provider confounds. Two repetitions of three small families
do not identify a population effect or justify arbitrary syntax additions.

## Proposed improvements, in order

None of the following has been implemented or tested with new agents.

1. **Correct prompt assembly in a separately versioned study driver.** Supply each task, guide and source
   file once. Exclude the check client and edit-only instructions from source-only comprehension. Keep
   the frozen controller and results intact. Proposed regression checks should assert one copy of each
   section, no tool-use instruction in the no-tools arm and preservation of every relevant source definition.
2. **Use one language-neutral input schema.** Explicitly define event-code pairs, hexadecimal input bytes,
   byte counts, empty bytes and a list containing one empty payload. Give all languages the same facts.
   Explain that impact includes the edited function and callers whose outputs can change. Use unrelated,
   verified teaching examples rather than copying the scored questions into a guide.
3. **Test presentation before changing language semantics.** Use clearer variable names and ordinary
   formatting in new, equivalent reference programs. Separate language-reading material from authoring
   and identity-management instructions. Existing syntax suffices. A proposed naming example is below.
4. **Evaluate tool-assisted comprehension as a separate condition.** Allow a small, equal number of read-only
   execution or semantic-query operations across languages. Keep unaided comprehension as its own measure.
   Any trace must show byte ranges, consumed offsets and rejection reasons. It should validate witnesses
   without exposing a hidden answer key. This is a proposed experiment, not a new tool implementation.
5. **Revisit a library/API change only after a persistent error survives those controls.** For example, an
   explicitly count-named slicing operation could be considered if fresh tests still show count/end confusion.
   Do not introduce named-argument syntax, new loop forms or identity changes on the basis of one response.

Proposed common input notes for a future study, not an adopted guide patch:

```text
Input data is hexadecimal text. Each pair of hexadecimal digits represents one byte.
For example, ab00fe represents three bytes: ab, 00 and fe.
An empty byte value is "". [] contains no payloads. [""] contains one empty payload.
Ledger events use [code, amount]: 0 Credit, 1 Debit, 2 Lock, 3 Unlock.
For impact answers, include the edited function itself and affected callers.
```

These are representation notes, not new scored answers. Teaching and evaluation examples must remain
separate in any follow-up.

Proposed naming for future examples, without a grammar change:

```lobo
let payload_start = header.payload;
let payload_byte_count = header.length;
let payload_bytes = Bytes::slice(data, payload_start, payload_byte_count);
```

A follow-up should first compare original and deduplicated prompts with all facts unchanged. Then test
explicit common encoding documentation separately. Use fresh held-out inputs and questions, separate
teaching examples, matched models/limits and counterbalanced order. Compare accuracy, each ability,
witness validity and actual participant token/time use. Do not claim that improved scores on questions
whose answers were added to the documentation demonstrate better comprehension.

Language changes require a residual Lobo-specific failure across fresh tasks and models after the prompt
and documentation controls. The next step proposed here is a prompt/documentation experiment. No new
model run, guide edit or product change is selected by this audit.
