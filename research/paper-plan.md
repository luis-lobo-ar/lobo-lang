# Paper direction and evidence plan

Status: **proposal, 2026-11-03**. No experiments below have been run unless explicitly linked as
existing evidence. This plan does not change the language-first roadmap or authorize model spend.

Working title: **Language Constructs for Reliable Agentic Software Construction:
A Study of Construction, Comprehension, and Cost with Lobo**.

Follow-up to the initial cost-focused framing: the user asked whether the real outcome is better
software construction given appropriate language constructs. The primary outcome is now requirements
satisfaction and correct evolution at a fixed budget. Tokens remain one cost measure. The initial
[framing is retained](snapshots/2026-11-03-initial-review/paper-plan.md.txt).
See the [construction-quality framework](construction-quality.md) and [external findings](findings.json).

The [three-hypothesis comparison](hypotheses.md) recommends checked representations as the initial
primary hypothesis, retaining abstraction and tool/context explanations. This is a recommendation
before a confirmatory study, not evidence that H1 is true or that Lobo is superior.
The user also requires [code comprehension](code-comprehension.md). Measure it as a distinct outcome,
including behavior prediction, dependency understanding, localization and change impact. Do not
infer comprehension from a passing patch or combine the two outcomes into an opaque average.
The [current comparison](lobo-comparison.md) separates compiler calibration from agent results.
The [readiness assessment](comparison-readiness.md) recommends retaining the narrow Core 0.9 baseline
while developing abstraction/data workloads before a broad language-productivity study. Recursive
named data, user generics, function values and scale limits remain material boundaries. No new
construct is assumed implemented, and feature-development tasks must remain separate from holdouts.

Implemented development slice, 2026-11-04: Core 0.10 permits nominal recursion through List storage,
with an AST transformation and native/reference/resource checks. Preserve Core 0.9 as the original
baseline. This closes one representation gap for list-guarded data; direct inline cycles, user
generics, function values and the wider workspace remain absent. The AST is public development
material and must not become a claimed unseen confirmatory task. Agent quality benefits remain unknown.

Subsequent development slice, 2026-11-04: Core 0.11 adds explicit generic functions checked per concrete
instance, retaining the pre-change Core 0.10 compiler/source baseline. The public reuse workload shows
shared collection helpers across I64/Text/caller-owned records with native output, not an H2 or code
comprehension result. Future comparisons must freeze the language version and independently author
held-out tasks; do not promote these development examples into unseen evaluation evidence.

## Defensible contribution

The strongest candidate contribution is a controlled investigation of whether appropriate checked
constructs improve construction and subsequent changes, and how context and verification interfaces
affect that result. Lobo provides an experimental native language and tooling in which to test it.
Measure what an agent successfully constructs, the guarantees actually established and the total
effort. A shorter unsuccessful trajectory is not a construction improvement.

Do not claim the first AI-oriented language, that models require no training or documentation, that
compact syntax necessarily saves billed tokens, or that native execution makes agents faster.
[Prior work](related-work.md) already covers compact agent-authored DSLs, code action interfaces,
typed context and cost-aware evaluation. A distinctive contribution needs measured behavior or a
precise new mechanism with an appropriate baseline; assembling familiar ideas is not automatically novel.

## Research questions

| Question | Manipulation | Main outcomes |
| --- | --- | --- |
| RQ1: Do languages improve construction for matched requirements and budgets? | Idiomatic Lobo and established-language tasks with equivalent acceptance criteria | Accepted tasks, hidden defects, guarantees and total effort |
| RQ2: Which constructs explain a difference? | Within-language checked versus loose representations with the same information | Relevant defect classes, acceptance, correct dependent changes and effort |
| RQ3: Which interface changes explain a difference? | Plain/structured feedback, full/affected verification, equivalent context packages | Diagnosis/edit cycles, retrieval, tool calls and usage |
| RQ4: Can agents learn the unfamiliar language economically? | Equal factual knowledge as guide versus skill, then varied documentation budgets | Onboarding cost, competence, retrieval and adherence |
| RQ5: Does quality persist through change? | Frozen hidden requirement changes, multi-module repair and refactoring | Change acceptance, regressions, first/last correct candidate and cost |
| RQ6: Does the representation improve comprehension? | Behavior-matched programs with grounded questions; source-only and tool-assisted conditions separated | Prediction, witness, dependency, localization and impact accuracy, with effort and false claims |

RQ3 starts with mechanisms implemented in the repository. A semantic graph/transaction interface
cannot be evaluated until implemented. Use a staged study rather than an unaffordable full factorial
of every language, interface, model and task combination. Follow the first experiment with focused
ablations that test its strongest alternative explanations.

For a later semantic-context experiment, include lexical search and conventional language-server
retrieval, not only a weaker file-dump baseline. Measure setup/indexing and per-query overhead.
The [LSP-versus-grep preliminary study](https://arxiv.org/abs/2608.13568v1) shows why precision alone
does not guarantee token savings. Type-constrained decoding is a separate possible intervention,
requiring compiler/decoder integration; it is not implemented by our current language skill.

## Existing Lobo evidence

The [result register](../benchmarks/results/README.md) links all datasets and qualifications.
The [local paper evidence index](local-evidence.md) maps those records to supported claims and unmeasured
outcomes. Its [preservation receipt](verification-local-evidence.json) pins the local evidence bundle.

- Execution microbenchmarks compare native/evaluator/Rust program behavior and runtime. They do not
  measure agent development cost or establish a general language-speed ranking.
- The scripted verification prototype checks agreement and reuse across six scenarios. It does not
  contain real coding-agent productivity measurements.
- The real-agent pilot, repeated library study and skill study scheduled **60 attempts** in total.
  There were **59 accepted completed repairs and one controller interruption**, retained without
  replacement. This is a collection of small studies, not 60 independent programming workloads.
- All agent arms used Lobo. Feedback, test selection and knowledge packaging varied; the application
  language did not. No agent-productivity comparison with Rust or Python has been performed.
- All completed repair studies had easy first-candidate outcomes. The skill trial had no compile
  diagnostics and no demonstrated accuracy or consistent time/token benefit over an equivalent guide.
  Ceiling effects make stronger recovery and onboarding conclusions unavailable.
- Context deviations and interrupted/missing usage remain recorded. Acceptance, adherence and
  infrastructure reliability are separate outcomes, not reasons to discard inconvenient records.
- The Core 0.10/0.11 data and generic-function developments have versioned native/tooling evidence.
  These public workloads establish implemented expressibility, not an agent abstraction benefit.
- Compiler 0.11.1's C/direct-LLVM comparison holds the source language fixed. On four public programs,
  two local rounds demonstrate no runtime improvement; final LLVM compile medians are 18.3% to 30.5%
  higher, with equal or eight-byte-larger executables. C remains the current default. Use this in the
  systems/controls/limitations discussion, not as an H1/H2/H3 or comprehension result. Preserve the busy-host,
  warm-cache, process-startup and unavailable-hardware-counter qualifications in the linked records.

The current implementation includes native compilation, a reference evaluator, immutable data,
closed module composition, checked action metadata, runtime contracts and a foreground JSONL test
runner with snapshots/reuse. The wider semantic workspace, transactional daemon, capability sandbox,
formal state-machine verification and plugin are proposed. Describe them as future work.

## Minimum credible language comparison

1. Freeze a protocol before running: hypotheses, tasks, exclusions, timeout/budget rules, primary
   endpoints, stopping rule and analysis. Include an explicit record of protocol amendments.
2. Build matched task specifications and independent tests for Lobo, Rust and Python. Include
   implementation, repair and multi-module changes. Have independent review verify equivalent
   behavior, integer/Unicode/error semantics and feasible idioms. Avoid forcing a syntax translation
   that makes one language artificially awkward. Restrict the first comparison to the shared
   supported problem domain; ecosystem-heavy applications would answer a different question.
   Add credible existing baselines for the construct under test, such as strict TypeScript or
   OCaml for typed alternatives and Dafny or Lean for proof-required tasks. Do not treat Python
   as the only alternative to a new language. Keep native-only requirements explicit.
3. Use common agent scaffolding, initial information, tool authority and available test knowledge.
   Freeze compiler/interpreter, dependencies, harness, model identity, tokenizer and documentation.
   Record whether API model names are immutable versions or moving aliases. Treat language knowledge
   and pretraining exposure as measured/limited assumptions, not known absence of training data.
4. Use fresh isolated workspaces and conversations, randomize/interleave arm order, and repeat across
   task families and multiple model families. Choose sample size from a separate variance/difficulty
   pilot and a worthwhile effect size. Repeated samples of two tasks do not establish generality.
5. Freeze held-out grading before agents run; keep it inaccessible during editing. Use independent
   behavioral oracles, boundary cases and mutation checks. Retain the same grading for failed,
   interrupted and successful attempts. Record cancellation and system failures separately.
6. Persist the exact source snapshot for **every candidate before checking it**, with hashes,
   timestamps, tool events, input identities and versioned test results. Grade intermediate snapshots
   offline to measure first success and later regressions. Existing logs often retain intermediate
   hashes but not all source snapshots; do not claim retrospective reconstruction from hashes alone.
7. Analyze paired task-level outcomes and uncertainty, clustering repeated trials by task. Report
   individual task/model results and sensitivity to workflow deviations. Do not pool all repeats as
   independent evidence. Preserve the predeclared primary analysis alongside corrections.

## Token, cost and time accounting

| Quantity | Meaning and required caveat |
| --- | --- |
| Final-source tokens | Lexical tokens and model-tokenizer counts separately; also source bytes. This is representation size. |
| Input usage | Entire trajectory; separate cached and uncached tokens where supplied. Repeated prefixes still matter. |
| Output usage | Generated code, prose and reasoning usage where exposed. Record whether reasoning is included or separately reported. Never sum overlapping fields. |
| Tool feedback | Bytes and tokenizer counts, plus actual subsequent model input usage. Avoid counting the same payload twice as billed input. |
| Retrieval | Knowledge/context read, retrieval calls and failures; include onboarding in the primary whole-task budget. |
| Time | End-to-end wall clock plus model, tool, compile/test and queue components where observable. Mark unknowns. |
| Monetary cost | Provider, price date, cache discounts and complete usage. Do not infer historical billing from current prices. |
| Runtime | Compiled/interpreted program time, memory and binary size in a separate execution experiment. |

Report task success at a fixed budget and the success/cost tradeoff. Compute aggregate cost per
accepted task as **cost of all scheduled attempts / number accepted** under the declared handling
of infrastructure failures. Do not average only successful-attempt cost or count an unknown interrupted
usage value as zero. If nothing is accepted, the ratio is undefined; report the failure directly.
Report final correctness separately from whether any intermediate candidate was correct. Do not
substitute median source length for median trajectory cost or compare equal-cost and unlimited-cost
success numbers as if the budgets matched. These requirements follow the concerns raised by
[Tokenmaxxing](https://arxiv.org/abs/2607.22807v2) and
[AI Agents That Matter](https://arxiv.org/abs/2407.01502v1).

## Threats and interpretation

Main threats include language familiarity, documentation quality, unsupported library work,
unequal scaffolds, biased task translation, insufficient tests, contamination, prompt leakage,
model/provider drift, shared context, cache state, task-order effects and infrastructure interruption.
Record interventions and retain unsuccessful outcomes. Keep hidden tests out of onboarding examples.

A null result can be valuable if the comparison is fair: it may show that interface/context costs
dominate syntax or that a novel language's onboarding offsets compactness. A positive combined-system
result needs ablations before attributing it to the language. Claims about safety need specific
enforcement/proof evidence; native execution and action declarations alone do not supply it.

## Artifact and writing plan

Keep the compiler/specification, agent skill, frozen task packages, test identities, harness, prompts,
sanitized trajectories, raw usage accounting and analysis runnable from pinned versions. Document
what cannot be shared, why, and what evidence remains. Private local evidence is not a public archive.
Check task/data licenses and sanitize credentials before any release; no publication is performed here.

A future paper can present: the hypothesis and related work; implemented language/tool mechanisms;
controlled methodology; successes and negative results; ablations; limitations; and a reproducibility
appendix. Select a venue after the actual contribution and evidence are clear. The present artifact
is a research foundation and proposed study, not a submission-ready empirical paper.
