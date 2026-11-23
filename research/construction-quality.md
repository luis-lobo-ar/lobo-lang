# Language constructs and software construction quality

Research direction, **2026-11-03**. This extends the initial cost-focused review in response to the
user's question. It proposes measurements, not an observed improvement or a new language feature.

## The outcome is software that meets requirements

The main question is whether an agent constructs software that satisfies its requirements and
continues to satisfy them after change, under a stated resource budget. Token use measures part
of the cost. Fewer tokens are beneficial when acceptance and required guarantees are preserved.
Spending more tokens may be worthwhile if it produces substantially more dependable software.

There is no universal scalar for "better software." Define the required behavior and guarantees
before choosing a language. Report outcomes separately rather than hide tradeoffs in a weighted
quality score. More compile errors can mean a stronger checker rejected defects earlier, or that
the agent struggled. Count accepted complete tasks as well as intermediate diagnostic categories.

| Outcome | Observable measure | Limit |
| --- | --- | --- |
| Requirements satisfied | Independent frozen tests, including invalid and boundary inputs | Finite tests do not prove all behavior |
| Specified properties established | Proof obligations and exact assumptions, or explicitly labeled runtime checks | An incomplete specification can certify the wrong program |
| Defects prevented or escaped | Seeded defect classes rejected before acceptance, plus hidden failures after acceptance | Compare equivalent defect opportunities, not raw compiler-error counts |
| Safe boundaries | Attempts to violate resource, representation or authority rules | Declaring an action does not enforce capability policy |
| Change correctness | Success and regressions on an initially hidden requirement change | Include realistic dependent changes |
| Code comprehension | Grounded prediction, witness, dependency, localization and change-impact accuracy | A passing patch or fluent explanation is insufficient evidence |
| Construction effort | Whole-trajectory tokens, cost, time, retries and interventions | Include failures and onboarding; retain unknown usage |
| Maintainability | Correctness and effort on later changes; separately, blinded reviewer assessment if used | Short source or a style score is insufficient evidence |

Two useful primary comparisons are **acceptance rate at a fixed budget** and **cost at a fixed
acceptance/guarantee level**. Publish the tradeoff across budgets. Keep program runtime, memory
and binary size separate unless the task requirements explicitly constrain them.

## Existing languages supply concrete mechanisms

These are semantic mechanisms and candidate baselines, not an empirical ranking of agents.
Documentation establishes what a construct does; an experiment must establish its effect on construction.

| Requirement | Existing language or system | Mechanism and bounded claim |
| --- | --- | --- |
| Native resource management with memory safety | Rust | Ownership and borrowing reject invalid resource uses. Safe abstractions depend on sound unsafe implementations and compiler behavior. Logical correctness and deadlock freedom do not follow. |
| Explicit alternatives and missing-case feedback | OCaml, Rust | Variants/enums and pattern matching represent distinct states. OCaml reports non-exhaustive matches as warnings by default; configure warning policy before comparing rejection rates. |
| Verified algorithm properties | Dafny | Preconditions, postconditions, invariants and termination measures support proof of declared obligations. Wrong assumptions or weak contracts still permit wrong requirements. |
| Proof-carrying executable definitions | Lean | Dependent types and checked proofs connect programs to propositions. Audit axioms, admitted proofs and noncomputable definitions; kernel checking is not an end-to-end machine-code proof. |
| Persistent relational invariants | SQL with PostgreSQL constraints | NOT NULL, foreign keys, uniqueness and CHECK enforce declared properties. This is a language-plus-database baseline, not a pure language feature or full application specification. |

Primary semantic sources, accessed 2026-11-03:
[Rust ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html),
[OCaml variants and pattern matching](https://ocaml.org/docs/basic-data-types),
[OCaml warning controls](https://ocaml.org/manual/5.5/comp.html),
[Dafny guide](https://dafny.org/latest/OnlineTutorial/guide),
[Lean axioms and computation](https://lean-lang.org/theorem_proving_in_lean4/Axioms-and-Computation/),
and [PostgreSQL 18 constraints](https://www.postgresql.org/docs/18/ddl-constraints.html).
These are supporting semantic references, not additional empirical studies.
The [Rust concurrency chapter](https://doc.rust-lang.org/book/ch16-03-shared-state.html) explicitly
describes deadlocks that remain possible with safe shared-state abstractions.

For example, a payment protocol represented as named alternatives can associate different data with
pending, authorized and settled states. A loose record of flags permits inconsistent combinations
unless separately validated. Both representations can implement the requirement correctly. The
hypothesis is that an appropriate representation reduces opportunities for agent mistakes and helps
it respond correctly when a new state is added. This example is illustrative, not executed.

For sorting, a postcondition that output is ordered still permits an empty output. Preserving the
input multiset is a separate requirement. A proof against the weaker contract does not establish
sorting correctness. [VERINA](https://arxiv.org/abs/2505.23135v3) separates code, specification and
proof evaluation. Its specification scores can rely on testing when proofs are inconclusive; do
not label every reported specification acceptance a universal equivalence proof.

## What existing research establishes

[RustBelt](https://doi.org/10.1145/3158154) provides a machine-checked safety model for a realistic
Rust subset. It establishes a form of language assurance, not that agents complete Rust tasks more
often or cheaply. [DafnyBench](https://arxiv.org/abs/2406.08467v1) measures verification-hint generation
for existing programs, rather than construction of a whole solution from requirements.

[AutoCodeBench](https://arxiv.org/abs/2508.09101v1) observes different model scores across languages,
but its problem pools and generation/translation procedures differ. Its columns cannot establish
that a language improves the same agent on the same requirements.
[VeraBench](https://github.com/aallan/vera-bench/tree/c33fe27d49ce15fa71db324347dbbde348c53c7c)
is close prior art with ceiling, specification-mode and grading caveats. Its interpretation of a
typing advantage is not a controlled ablation of static typing.

The [code-quality reproduction study](https://arxiv.org/abs/1901.10220v2) shows why observational
rankings need caution. Project domain, teams, libraries, selection and statistical choices can
explain apparent differences. Human repository studies do not settle effects on agents.
The [findings register](findings.json) distinguishes observations, formal results and hypotheses.

## Experiments that would answer the construct question

1. **Matched language comparison.** Give agents the same behavioral requirements and equivalent
   acceptance tests, with idiomatic solutions in Lobo and established languages. Include strict
   TypeScript or an ML-family language for typed-data work, and a verification-oriented language
   for proof-required work. Use Rust and Python where they match the task. If native-only execution
   is required, exclude or separately label configurations that cannot meet it.
2. **Construct comparison within a language.** Compare enums with loose string/flag encodings, or
   checked contracts with equivalent information in prose. Supply the same facts and test knowledge.
   This isolates a mechanism more closely than a whole-language ranking. A restricted idiom is an
   intervention, not proof that the entire language is inferior.
3. **Requirement evolution.** After the initial solution, introduce a frozen hidden change, such as
   a new protocol state or validation rule. Measure dependent updates, regressions and cost.
4. **Specification versus implementation.** Compare supplied correct specifications with agent-authored
   specifications in separate arms. Evaluate both against independent requirements. Prevent weakened
   preconditions, vacuous postconditions and bypasses from counting as successful verification.

Use task families that exercise each mechanism. State the relevant defect class and an independent
test or proof obligation that detects it. Keep compiler feedback available to all arms. Do not remove
the benefit of a checked language by evaluating only one unchecked emission. Freeze toolchains and
policies for warnings, casts, unsafe blocks, admitted proofs, exceptions and runtime contract fallback.
Record trusted components and permitted escape routes.

These experiments remain proposed. A successful Lobo result would demonstrate improvement on
specified outcomes against credible existing mechanisms. It would not show that every requirement
needs a new language, or that Lobo should copy every construct in the comparison languages.
The [code-comprehension framework](code-comprehension.md) adds a separate required outcome. An
[executed calibration](lobo-comparison.md) now validates fixtures and graders, not agent rankings.
