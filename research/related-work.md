# Related work: languages and development interfaces for agents

Reviewed **2026-11-03**. See [method and review limits](README.md), [machine-readable sources](sources.json)
and [BibTeX](references.bib). Findings below describe the cited versions; Lobo implications are our
interpretations. An annotation is not a replication or a claim that all conditions were inspected.

## Main findings

There is direct prior work on languages written by LLMs and on programming interfaces adapted to
agents. A claim that Lobo is the first agent-oriented language would be unsupported. Existing evidence
also distinguishes compact generated programs from inexpensive development trajectories. Our proposed
contribution should isolate language, context and verification costs, rather than attribute every
improvement to syntax. The [paper plan](paper-plan.md) turns this into testable questions.
The follow-up [construction-quality framework](construction-quality.md) makes requirements satisfaction
and successful change the primary outcomes. Selected external results are in the [findings register](findings.json).

## Languages written by agents

### TypeFly / MiniSpec — Chen et al., arXiv v2, revised 2024

[TypeFly: Flying Drones with Large Language Model](https://arxiv.org/abs/2312.14950v2)
introduces MiniSpec, a compact language for LLM-authored drone plans. Section 7.3 reports an average
34% reduction in plan-output tokens versus Python, up to 42%. Response-time reductions reach 32%
without streaming interpretation and 62% with it; streaming is a separate contribution from syntax.
This is direct precedent for agent-oriented language design, but its domain-specific plans and skills
do not establish general software-development savings. The later journal version has a different
title; its claims are not combined here. Key: `chen2024typefly`.

### Synoema — Bubnov, technical note, April 2026

[Synoema v0.1.0-alpha.3](https://zenodo.org/records/19514230) proposes BPE-aligned syntax, structured
diagnostics, contracts, integrated testing and type-constrained decoding. It is particularly close
design prior art. Peer review is not established. Selected evaluation sections distinguish static
source counts, runtime tests and model-generation studies. The revised 16-task static table averages
92.5 source tokens for Synoema versus 92.9 for Python; selected large reductions are not a universal
46% saving. Some imperative tasks are larger. Full integration of the proposed decoding tiers remains
incomplete. These are author-reported results, not independently verified agent-trajectory savings.
Key: `bubnov2026synoema`.

### Djinnlang — Henniger, Chong and Amin, September 2026 preprint

[Djinnlang](https://arxiv.org/abs/2609.23954v1) puts an LLM inside a compiler pipeline for unambiguous
specifications, generating Dafny implementation and verification obligations. It is relevant to
separating intent from generated implementation and checking untrusted generation. Abstract/metadata
review only: its claims should not be treated as verified token savings or evidence for Lobo's native
backend. Key: `henniger2026djinnlang`.

## Language choice and token costs

### Tokenmaxxing — Wu, Anderson and Guha, October 2026 preprint v2

[The Best Programming Language for Tokenmaxxing](https://arxiv.org/abs/2607.22807v2) is the closest
empirical study to our cost question. It compares 100 problems across Python, Java, Rust and OCaml
with five models: 2,000 trajectories. It examines intermediate candidates, debugging phases and work
after a passing candidate. Final-program token counts explain only a small part of whole-trajectory
cost. Its representation counts use a common proxy tokenizer; they are not every model's billed
token count. The method motivates preserving every intermediate source snapshot and measuring
failed attempts. Language familiarity remains a confound for a new-language comparison.
Key: `wu2026tokenmaxxing`.

### TokDrift — Li, Deng and Nie, 2025 preprint

[TokDrift: When LLM Speaks in Subwords but Code Speaks in Grammar](https://arxiv.org/abs/2510.14972v1)
studies sensitivity to semantics-preserving transformations, including changes affecting token
boundaries. It makes tokenizer choice and surface representation relevant experimental controls.
It does not establish that shorter syntax improves full agent repair trajectories. Abstract/metadata
review. Key: `li2025tokdrift`.

### EsoLang-Bench — Sharma and Chopra, 2026 preprint v2

[EsoLang-Bench](https://arxiv.org/abs/2603.09678v2) evaluates reasoning in unfamiliar esoteric
languages. It highlights the danger of assuming that a model transfers its coding ability to an
unfamiliar language. Esoteric languages are intentionally difficult; the results cannot isolate
training familiarity from language design or predict Lobo performance. Abstract/metadata review.
Key: `sharma2026esolang`.

## Agent interfaces and semantic context

### SWE-agent — Yang et al., NeurIPS 2024

[SWE-agent](https://arxiv.org/abs/2405.15793v3) treats the agent-computer interface as an experimental
variable: search, navigation, editing and feedback can be designed around agents. This supports
studying Lobo's tools separately from its language. A better interface does not require changing
the application language, so it is also an essential alternative explanation for a combined-system
gain. Key: `yang2024sweagent`.

### CodeAct — Wang et al., ICML 2024

[Executable Code Actions Elicit Better LLM Agents](https://proceedings.mlr.press/v235/wang24h.html)
uses executable Python as the agent action representation instead of text or JSON actions.
Code combines tool calls and computation in a reusable action space. It informs Lobo's tool
interface, but agent action code and application implementation are separate experimental layers.
An interpreted action interface does not imply that Lobo programs need an interpreter.
Key: `wang2024codeact`.

### Typed Holes — Blinn et al., OOPSLA 2024

[Statically Contextualizing Large Language Models with Typed Holes](https://arxiv.org/abs/2409.00921v1)
uses expected types, type definitions and relevant headers to construct completion context, with
type/syntax feedback. Hazel and a TypeScript adaptation make this close prior art for Lobo's proposed
semantic context engine. Some TypeScript retrieval was manually emulated because of tooling limits;
this is not a fully automated repository-agent result. The published record is
[PACMPL, DOI 10.1145/3689728](https://doi.org/10.1145/3689728).
Key: `blinn2024typedholes`.

### CodePlan — Bairi et al., 2023 preprint; later FSE 2024 publication

[CodePlan](https://arxiv.org/abs/2309.12499v1) combines repository-level planning with incremental
dependency analysis and change-impact information. This precedes parts of our proposed semantic
workspace and motivates dependency-aware context baselines. The preprint and later publication
have different evaluation counts, so no merged numerical result is used here. Abstract and primary
artifact/publication records reviewed. Key: `bairi2023codeplan`.

### Type-constrained generation — Mündler et al., PLDI 2025

[Type-Constrained Code Generation with Language Models](https://doi.org/10.1145/3729274) constrains
decoding with prefix automata and type information, going beyond syntactic validity. Its TypeScript
evaluation covers synthesis, translation and repair. The experimental setup emulates support for
features beyond the implemented subset by retaining compiling unconstrained samples and resampling
others; this matters when interpreting results. It suggests an incremental compiler/completion
interface as a research direction. Decoder-level integration is a distinct mechanism from supplying
an agent with a language guide or checking completed source. Key: `mundler2025typeconstrained`.

### LSP versus grep — Xu, 2026 preliminary preprint

[Does a Language Server Save Tokens for Coding Agents?](https://arxiv.org/abs/2608.13568v1)
compares lexical and semantic retrieval in Python/TypeScript repositories. It reports conditional,
often negative token results: precise references can require extra calls and file reads, while
lexical matches include comments/strings needed by some rename tasks. Small repositories, task
classes, model choices and one harness limit generalization. This is a direct challenge to assuming
that Lobo's proposed semantic context is automatically cheaper. Include lexical baselines, indexing
costs and task-specific success checks. Its tokens-to-success formula averages successful rollouts
only, unlike our proposed all-attempt cost-per-acceptance. The limitations retain coverage claims
inconsistent with reported TypeScript/edit results; inspect the artifact before adopting numerical
conclusions. Key: `xu2026lsptokens`.

## Skills and repository instructions: mixed findings

### SkillsBench — Li et al., 2026 preprint v4

[SkillsBench](https://arxiv.org/abs/2602.12670v4) evaluates 87 tasks across eight domains and 18
model/harness configurations. Its v4 reports average success rising from 33.9% to 50.5% with curated
skills. Invocation and resolution are distinct; self-generated skills need not improve results.
These heterogeneous tasks do not establish the effect of an unfamiliar-language onboarding skill.
Earlier versions have different task/configuration counts. Key: `li2026skillsbench`.

### SWE-Skills-Bench — Han et al., October 2026 preprint v2

[SWE-Skills-Bench](https://arxiv.org/abs/2603.15401v2) reports 3,920 paired evaluations of 98 skills
across four configurations. It finds no significant impact for 92 skills; aggregate pass ratio
changes from 76.6% to 76.9%. Its explicit invocation and paired setup make it relevant to our
guide-versus-skill comparison. The findings concern these skills and tasks, not every possible
knowledge package. v1 cost figures circulating in snippets are not imported into this v2 summary.
Key: `han2026sweskills`.

### AGENTS.md efficiency — Lulla et al., 2026 preprint v2

[On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents](https://arxiv.org/abs/2601.20404v2)
studies 124 pull requests across ten repositories. Its abstract reports associations with 28.64%
lower median runtime and 16.58% fewer output tokens, with comparable task-completion behavior.
Output tokens are not total billed input/output cost, and this does not establish a universal causal
benefit from repository instructions. Abstract/metadata review. Key: `lulla2026agentsmd`.

### AGENTS.md evaluation — Gloaguen et al., ICLR 2026 MemAgents workshop

[Evaluating AGENTS.md](https://www.sri.inf.ethz.ch/publications/gloaguen2026agentsmd) finds no overall
success improvement and more than 20% higher inference cost in the evaluated settings. Extra
requirements can cause additional exploration. This contrasts with the preceding study but uses
different tasks and conditions. The primary institutional abstract was accessible; the linked
OpenReview page presented a verification challenge. This is a workshop publication, not ICLR main
conference. Key: `gloaguen2026agentsmd`.

## Languages for orchestrating LLMs and structured messages

These address a different direction of programming: people or systems specify model behavior.
Their benefits do not by themselves establish a better application language for an agent to write.

### LMQL — Beurer-Kellner, Fischer and Vechev, PLDI 2023

[Prompting Is Programming](https://arxiv.org/abs/2212.06094v3) introduces a query language combining
prompts, control flow and output constraints. It demonstrates that compilation and constrained
execution can improve LLM programming. Its cost results concern its prompt workloads and execution
strategy, not autonomous software development in a new native language. Abstract/metadata review.
Key: `beurerkellner2023lmql`.

### SGLang — Zheng et al., NeurIPS 2024

[SGLang](https://proceedings.neurips.cc/paper_files/paper/2024/hash/724be4472168f31ba1c9ac630f15dec8-Abstract-Conference.html)
combines a structured model-programming interface with execution optimizations such as shared
prefix/KV-cache reuse. Reported throughput gains are runtime-serving measurements, not equivalent
reductions in agent token counts. It motivates recording cached and uncached input separately.
Key: `zheng2024sglang`.

### LLMON — Hind et al., 2026 preprint v4

[LLMON](https://arxiv.org/abs/2603.22519v4) proposes an LLM-native markup interface to represent
structure and semantics. It is relevant to typed context and machine-readable boundaries but is
not a general-purpose native application language. Abstract/metadata review.
Key: `hind2026llmon`.

## Evaluation and correctness

### AI Agents That Matter — Kapoor et al., 2024 preprint

[AI Agents That Matter](https://arxiv.org/abs/2407.01502v1) argues for cost-controlled evaluation,
clear holdouts and reproducibility rather than accuracy alone. It supports reporting success/cost
tradeoffs, including failed attempts, and avoiding conclusions from uncontrolled scaffold changes.
Selected evaluation-method sections reviewed. Key: `kapoor2024agents`.

### MultiPL-E — Cassano et al., arXiv v4, 2022

[MultiPL-E](https://arxiv.org/abs/2208.08227v4) translates programming benchmarks and tests into
multiple languages. It offers a model for matched behavioral tasks when comparing Lobo with existing
languages. Translation validity and equivalent test strength still require checking. It evaluates
code generation, not an entire repair workflow. Abstract/metadata review. Key: `cassano2022multiple`.

### EvalPlus — Liu et al., 2023 arXiv v3

[Is Your Code Generated by ChatGPT Really Correct?](https://arxiv.org/abs/2305.01210v3) shows that
expanded tests reveal errors missed by conventional code-generation benchmarks and can change model
rankings. It motivates independent held-out, boundary and mutation-sensitive tests for Lobo trials.
Passing our visible fixtures is not a correctness proof. Abstract/metadata review.
Key: `liu2023evalplus`.

### LiveCodeBench — Jain et al., 2024 arXiv v2

[LiveCodeBench](https://arxiv.org/abs/2403.07974v2) uses temporally collected contest tasks and
evaluates capabilities beyond initial generation, including repair. It motivates recording task
provenance and model/date cutoffs. Temporal filtering reduces some contamination risks; it cannot
guarantee that every future model is unfamiliar with every task. Abstract/metadata review.
Key: `jain2024livecodebench`.

### Clover — Sun et al., arXiv v4, 2024

[Clover](https://arxiv.org/abs/2310.17807v4) checks consistency among code, documentation and formal
annotations, using verification and LLMs. Its textbook-level Dafny evaluation is relevant to checked
generation and specification quality. Its dataset results are not a universal absence-of-defects
guarantee. Lobo's runtime contracts and finite tests currently offer different guarantees.
Abstract/metadata review. Key: `sun2024clover`.

## Authority and runtime safety

### AgentSpec — Wang, Poskitt and Sun, ICSE 2026

[AgentSpec](https://cposkitt.github.io/files/publications/agentspec_llm_enforcement_icse26.pdf)
uses a DSL to describe triggers, predicates and runtime enforcement for agent behavior. It is
related to Lobo's proposed permissions, rate limits and cross-cutting policy hooks. Checked action
labels alone do not implement such enforcement. Primary author abstract/introduction reviewed.
Key: `wang2026agentspec`.

### CaMeL — Debenedetti et al., 2025 arXiv v2

[Defeating Prompt Injections by Design](https://arxiv.org/abs/2503.18813v2) separates control/data
flow and applies capability-based policies to tool use. This informs the proposed separation of
actions, authority and untrusted data. The authors' artifact notes implementation limitations;
neither that prototype nor Lobo's current metadata establishes complete prompt-injection resistance.
Abstract and primary artifact limitations reviewed. Key: `debenedetti2025camel`.

## Follow-up: constructs and software construction quality

### AutoCodeBench, Chou et al., 2025 arXiv v1

[AutoCodeBench](https://arxiv.org/abs/2508.09101v1) contains 3,920 problems across 20 languages.
Language-specific generation and approximate translation supply different problem pools. Its score
columns do not establish which language improves the same agent on matched requirements. A later
ICLR 2026 record exists; results here remain tied to the preprint. Key: `chou2025autocodebench`.

### VERINA, Ye et al., 2026 arXiv v3

[VERINA](https://arxiv.org/abs/2505.23135v3) separates implementation, specification and proof tasks
using 189 Lean problems. An agent's weak specification can miss the actual requirement. Its evaluator
combines proofs and tests, retaining inconclusive outcomes. Proof success and functional test success
are different metrics. Selected methods and abstract reviewed. Key: `ye2025verina`.

### DafnyBench, Loughridge et al., 2024 arXiv v1

[DafnyBench](https://arxiv.org/abs/2406.08467v1) evaluates verification hints for existing programs,
with error-feedback retries. It supports a verifier-driven repair study but does not directly measure
construction from requirements or superiority over another language. Abstract/metadata review.
Key: `loughridge2024dafnybench`.

### Language and code quality, Berger et al., TOPLAS 2019

[On the Impact of Programming Languages on Code Quality](https://arxiv.org/abs/1901.10220v2)
reproduces and reanalyzes an observational GitHub study. Modeling flaws and very small remaining
effects undermine broad rankings. These are human-project data, not agent trials. Control task and
domain rather than infer causality from aggregate scores. Key: `berger2019codequality`.

### RustBelt, Jung et al., POPL 2018

[RustBelt](https://doi.org/10.1145/3158154) supplies a machine-checked safety proof for a realistic
Rust subset and obligations for unsafe libraries. It shows that language design can establish
specific guarantees independently of who writes the code. It does not show that agents generate
correct Rust more often or cheaply. Primary abstract/introduction reviewed. Key: `jung2018rustbelt`.

### Vera, Allan, author software artifact, 2026

[Vera at the reviewed commit](https://github.com/aallan/vera/tree/f6d45a787ba66912fe6604089d3e977cefd26031)
has typed positional references, required contracts, explicit behavior declarations and contract
verification/runtime checking. These are author artifact claims, not an independent soundness audit.
The current implementation differs from the benchmark compiler version. Peer review and universal
absence from training data are unestablished. Key: `allan2026vera`.

### VeraBench, Allan, author software artifact, 2026

[VeraBench at the reviewed commit](https://github.com/aallan/vera-bench/tree/c33fe27d49ce15fa71db324347dbbde348c53c7c)
compares Vera with established and other agent-oriented languages. Qualifications include ceiling
effects, specification-mode differences, grading asymmetry and T1-T4 comparison guidance. Raw attempts
were not obtained. Typing causality was not isolated. The [findings register](findings.json) retains
headline means as author-reported results. Key: `allan2026verabench`.

## Follow-up: code comprehension

### CRUXEval, Gu et al., ICML 2024

[CRUXEval](https://proceedings.mlr.press/v235/gu24c.html) uses input and output prediction on 800
short Python functions to evaluate reasoning separately from generation. It motivates testing
grounded understanding rather than equating code-generation success with comprehension. Primary
proceedings abstract/metadata review. Key: `gu2024cruxeval`.

### CRUXEVAL-X, Xu et al., ACL 2025

[CRUXEVAL-X](https://aclanthology.org/2025.acl-long.1158/) extends input/output reasoning across
19 languages with test-guided generation/translation. Language-pair semantic differences need
explicit treatment when building a Lobo comparison. It is a multilingual reasoning precedent,
not evidence of Lobo performance. Primary publisher abstract/metadata review. Key: `xu2025cruxevalx`.

### RepoQA, Liu et al., 2024 arXiv v1

[RepoQA](https://arxiv.org/abs/2406.06025v1) evaluates finding functions from descriptions across
repositories and languages. It motivates repository-level comprehension tasks, while showing why
function localization is narrower than full behavioral understanding. Primary abstract/metadata
review. Key: `liu2024repoqa`.
