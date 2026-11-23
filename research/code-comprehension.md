# Code comprehension in the Lobo comparison

Status: framework and executable calibration, **2026-11-03**. Comparative agent scores are unknown.
The user's comprehension requirement applies alongside construction under all three hypotheses.
Comprehension means grounded answers about existing code, not fluent explanations or a passing patch.

## Measure distinct abilities

| Ability | Task | Grounding |
| --- | --- | --- |
| Behavioral reasoning | Predict a return value or fault, or construct an input witness | Independent oracle plus actual execution |
| State and invariant reasoning | Identify a reachable invalid state or violated requirement | State-transition oracle and counterexample |
| Dependency understanding | Identify direct calls and affected callers | Reviewed dependency graph, with dynamic traces where appropriate |
| Fault localization | Name the responsible function and supply a distinguishing input | Seeded mutation and reproduced failure |
| Change-impact understanding | Identify obligations and behaviors affected by a new requirement | Frozen change specification and independent changed-program checks |
| Uncertainty calibration | Abstain or flag missing context instead of inventing an answer | Deliberately unanswerable cases scored separately |

Report each ability and whole-task acceptance. Fix the mixture of question types before comparing
an aggregate. Report unsupported claims and false positive locations, not only correctly named
symbols. Require concise evidence such as a source location, dependency or counterexample rather
than private reasoning transcripts. A claimed call edge is different from an executed call trace.

## Agent and human comprehension are different studies

Agent comprehension is the first target. Use the same model, requirements, program behavior and
question semantics across languages. Separate source-only reasoning from tool-assisted investigation.
If executing the target is allowed, output prediction measures tool-assisted problem solving; do
not describe it as unaided understanding. Record setup, retrieval, language-guide and tool costs.

Human comprehension would need actual human participants, controlled familiarity, randomized order
and time/accuracy measures. An agent's prose judgment is not a human-readability measurement.
No human study has been performed here. Keep human reviewability as an explicit separate outcome.

## Connection to the hypotheses

- **H1:** Do checked alternatives help agents infer states, failures and missing cases? They may
  reveal structure while adding unfamiliar notation. Measure answers, not assumed readability.
- **H2:** Does an abstraction express the needed behavior clearly, or conceal important details?
  Include boundary and failure questions, not only the abstraction's obvious successful case.
- **H3:** Do typed context and precise diagnostics help agents localize causes and assess impact?
  Give both arms access to the same facts, with retrieval and setup charged to the budget.

A construction advantage may coexist with worse comprehension. Preserve that tradeoff rather
than combine it into a score that hides the weakness. Test unseen edits after comprehension, but
also score comprehension before any editing so that a repair does not substitute for an explanation.

## Current executable evidence

The [calibration fixtures and grader](../benchmarks/language-comparison/README.md) cover five
language/representation arms. All 750 initial behavioral observations agree with the common oracle.
Selected invalid constructs are rejected by native Lobo and Rust variant checks; a wrong numeric
boundary compiles in every arm and is detected by execution. Those are compiler/fixture observations.

The deterministic grader checks predictions, a witness, calls, impact, fault location and a
counterexample, with eleven items. Its reference answer passing proves grader plumbing, not that
any agent understood the source. The six harness tests include negative answers and real compiled
program execution. A TDD run caught an ambiguous Python mutation target, fixed by using unique context.
No H1/H2/H3 agent conclusion follows, and there are no measured comprehension rankings yet.

## Related work and scope

[CRUXEval](https://proceedings.mlr.press/v235/gu24c.html) evaluates input/output reasoning separately
from generation on 800 short Python functions. It motivates keeping comprehension distinct from
construction. It does not provide a Lobo comparison. Key: `gu2024cruxeval`.

[CRUXEVAL-X](https://aclanthology.org/2025.acl-long.1158/) extends execution reasoning across 19
languages with test-guided translations. It is relevant to matching behavior and respecting language
semantics; translation does not automatically eliminate all confounds. Key: `xu2025cruxevalx`.

[RepoQA](https://arxiv.org/abs/2406.06025v1) evaluates multilingual function localization from natural
language descriptions. Retrieval/localization is one aspect of comprehension, not a substitute for
behavioral or change-impact understanding. Abstract/metadata review only. Key: `liu2024repoqa`.

The [paper plan](paper-plan.md) and [hypotheses](hypotheses.md) remain proposals pending actual
comparative trials. Broader programs and multiple task/model families are needed after calibration.
