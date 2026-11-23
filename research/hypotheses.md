# Three hypotheses for the Lobo study

Status: **proposals, 2026-11-03**. None has been tested by the existing Lobo agent trials.
H1 is the recommended primary hypothesis, not an accepted result or a preregistered protocol.
This document responds to the user's request to formulate three candidates before choosing a direction.
The subsequent user requirement adds [code comprehension](code-comprehension.md) under all three
hypotheses. Construction and comprehension require separate measurements. Neither is a proxy for
the other. The [current Lobo comparison](lobo-comparison.md) records what has actually been checked.

Choose a research question for its importance, testability and feasible controls. Do not choose a
claim because its expected answer favors Lobo. These mechanisms can coexist. Retain the alternatives
and report results even if existing languages perform as well as Lobo.

## H1: Checked representations improve construction quality

**Claim.** For tasks involving structured states and failures, agents using representations whose
invariants the compiler checks achieve a higher complete-task acceptance rate at a fixed budget
than agents using loosely encoded representations with the same requirement information.

**Mechanism.** A representation can reject invalid combinations and expose missing cases before
execution. This may reduce defects and repair effort. It may also increase failed construction
attempts if the agent cannot satisfy the checker; the experiment must count those failures.

**First comparison.** Within one language, compare tagged alternatives with an equivalent record
of flags and payload fields. Give both arms the same protocol requirements and test access. Use
tasks involving validation, state transitions and typed failures. Native Lobo already supports
variants and records, so this needs no new language construct. Add idiomatic Rust or another
established typed-language baseline before claiming a distinctive Lobo advantage.

**Primary outcome.** Fraction of all scheduled attempts satisfying the complete frozen requirements
within the declared budget. Count compiler failures, wrong answers and unfinished attempts as
unsuccessful construction under the frozen infrastructure-handling rules.

**Secondary outcomes.** Escaped invariant violations, time to the first accepted candidate, total
cost and correct dependent changes after a new state is introduced. Measure evolution both across
the full scheduled cohort and, separately, from common correct starting programs. Conditioning
only on each arm's successful initial programs can produce unequal survivor groups.

**Evidence against it.** A sufficiently precise estimate rules out a practically worthwhile
acceptance improvement, or the checker prevents particular defects but makes complete-task
acceptance worse. A small, uncertain or ceiling-limited experiment is inconclusive.

**Limit.** A positive result establishes a benefit of the studied representation under those
conditions. It does not establish that stronger types always help or that a new language is needed.
Relevant prior art includes [RustBelt](https://doi.org/10.1145/3158154),
[type-constrained generation](https://doi.org/10.1145/3729274) and
[VERINA](https://arxiv.org/abs/2505.23135v3). Their interventions and guarantees differ from this test.

## H2: Abstractions aligned with the task improve construction quality

**Claim.** For a defined task family, agents using abstractions that directly express the required
operation achieve higher complete-task acceptance at a fixed budget than agents implementing the
same behavior through lower-level steps, even when checking strength is comparable.

**Mechanism.** An appropriate abstraction reduces the decisions needed to translate a requirement
into code. This differs from H1: a construct can remove implementation steps without adding a
stronger static invariant.

**First comparison.** In an existing language, compare a declarative collection transformation
with explicit iteration using the same data model, libraries, requirement information and error
policy. Select tasks requiring composition and later changes, not a single call that already
solves the benchmark. Determine whether the benefit comes from the abstraction, its library
implementation, its syntax or documentation before proposing any new Lobo construct.

**Primary outcome.** Complete-task acceptance at the same declared budget.

**Secondary outcomes.** Missed requirement steps, regression rate under a hidden change, edit
iterations and whole-trajectory cost. Source length is descriptive, not the success criterion.

**Evidence against it.** A sufficiently precise comparison finds no worthwhile acceptance gain,
or apparent gains disappear once equivalent library help and documentation are supplied.

**Limit.** Some benefit may be obtainable through libraries in existing languages. A result for
one domain does not establish a universally better language. [MiniSpec](https://arxiv.org/abs/2312.14950v2)
and [CodeAct](https://proceedings.mlr.press/v235/wang24h.html) are related design precedents,
not existing evidence for this specific software-construction hypothesis.

## H3: Semantic context and feedback improve construction across languages

**Claim.** Giving agents relevant typed context and precise, actionable verification feedback
improves complete-task acceptance at a fixed budget, in both Lobo and established languages.

**Mechanism.** Agents make fewer context and diagnosis mistakes because the environment supplies
relevant facts and locates violated obligations. This could explain a combined Lobo-system gain
that would otherwise be attributed to its syntax or constructs.

**First comparison.** Hold source language and task fixed. Compare conventional file/search/compiler
tools with bounded semantic retrieval and structured verification feedback. Keep the underlying
facts, tests, permissions and model fixed. The retrieval method may change which facts are found
within budget; do not secretly add requirements or solution hints to one arm. Start with implemented
foreground tools. The semantic daemon and transactional workspace remain future work.

**Primary outcome.** Complete-task acceptance at the same declared budget, reported per language.

**Secondary outcomes.** Context omissions, diagnosis mistakes, read volume, tool calls and total
cost, including indexing and setup. Later ablations should separate retrieval from feedback.

**Evidence against it.** Equalized facts and budgets produce no worthwhile acceptance gain with
sufficient precision, or tool overhead reduces acceptance. If a construct advantage persists under
matched tools, H3 does not fully explain that advantage, even if the tools also help independently.

**Limit.** A positive result can favor better tools for existing languages rather than a new
application language. Relevant work includes [Typed Holes](https://doi.org/10.1145/3689728),
[SWE-agent](https://arxiv.org/abs/2405.15793v3) and the preliminary
[LSP-versus-grep study](https://arxiv.org/abs/2608.13568v1). Their mixed conditions require careful baselines.

## Recommended lead and selection rationale

Lead with **H1**, because it directly tests the user's software-construction question and can
start with constructs already implemented in Lobo. Its within-language comparison is easier to
interpret than a combined language, documentation, library and tool replacement. It also admits
a useful negative answer: constraints might reject defects without improving successful construction.

Keep H2 as the alternative that fewer implementation decisions matter more than additional checks.
Keep H3 as the alternative that context and feedback explain the gain. H1 can hold in Rust as well
as Lobo. If so, the appropriate claim is about checked representations, and Lobo's unique value
remains unestablished. This research question has value independently of which language wins.

Before a confirmatory experiment, freeze the exact construct, task families, budget definition,
primary endpoint, worthwhile effect size, sample size, stopping rule and infrastructure policy.
Use a separate difficulty/variance pilot to avoid another ceiling-limited study. Report uncertainty
and task-level differences. Preserve hypotheses and amendments before inspecting confirmatory results.

This step formulates hypotheses. No new model trial, language change or publication was performed.
The [paper plan](paper-plan.md), [construction-quality framework](construction-quality.md) and
[findings register](findings.json) retain the wider design and existing evidence limits.
