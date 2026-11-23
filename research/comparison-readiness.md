# Is Lobo ready for a language comparison?

Assessment: **2026-11-03**, Core/compiler 0.9. This is a source/specification audit and research
recommendation, not a new agent experiment, a language change or an implementation commitment.

Follow-up on 2026-11-04: the user authorized evolution. [Core v0.10](../spec/core-v0.10.md) now
supports List-guarded recursive nominal values, driven by the public AST development workload.
The assessment below remains the dated Core 0.9 baseline. Direct inline cycles, user generics,
function values and wider scale/application limits remain. No agent comparison advantage is established.

Later follow-up on 2026-11-04: [Core v0.11](../spec/core-v0.11.md) adds explicit generic functions
checked per concrete specialization. Imported helpers now work with caller-owned nominal types.
Generic nominal types, function values, universal generic-body checking and wider scale remain open.
This supersedes the user-generic-function gap above; it does not measure construction or comprehension.
Both Core 0.9 and Core 0.10 compiler/source baselines are preserved. Reuse fixtures are public development
material and must stay separate from held-out H2 construction/comprehension tasks.

## Readiness depends on the claim

Lobo is ready for focused comparisons of small structured computations and their comprehension.
It is not ready to support a broad claim about replacing mature languages for general software
construction. Counting statements or reproducing another language's syntax is not the acceptance
criterion. The question is whether the tasks fit the implemented semantics without forcing unusual
representations or moving application logic into the host harness.

The earlier [language-only milestone](../docs/language-completeness.md) established representative
computation and composition under explicit limits. It did not establish mature abstraction facilities,
large-program suitability, ecosystem support or universal replacement. This assessment extends the
research scope, not the historical milestone's claims.

## What is sufficient today

Core has functions and recursive calls, local bindings, branching, state-carrying iteration, counted
folds, immutable records, tagged alternatives, exhaustive matching, persistent lists, Text/Bytes,
typed expected failures and explicit multi-module composition. Native compilation, runtime contracts,
checked action metadata and editor support are implemented. The [compiler type model](../src/lib.rs)
and [Core specifications](../spec/README.md) define the actual scope.

These support comparing:

- state transitions and error handling using variants versus integer-tag records;
- parsing, validation and collection transformations on bounded in-memory inputs;
- comprehension of branches, return values, calls, contracts and small dependency graphs;
- small feature changes and repairs with independent behavioral grading.

The [750-case calibration](../benchmarks/results/2026-11-03-language-comparison-fixtures.md) establishes
fixture equivalence and selected compiler checks. It does not establish agent construction or
comprehension advantages. Larger existing parser/collection/tree workloads provide additional
development examples, but are not independent agent-comparison results.

## Gaps that can distort a broader comparison

| Gap | Current evidence | Why it matters |
| --- | --- | --- |
| Recursive named data | Core rejects nominal layout cycles, including through Lists. The existing tree uses a parent-index node table. | Direct tree/AST/document representations cannot be compared in that form. Tables are valid designs, but can introduce explicit identity, lookup and consistency work. |
| User-defined generic abstractions | Built-in `List<T>` is parameterized; user functions/types are not. The shared reverse helper is specialized to `I64`. | Reusing typed algorithms across element types may require duplicated implementations or a different representation. |
| Function values and higher-order reuse | No function type appears in the implemented type model. Functions can be called by declaration identity. | Reusable transformations parameterized by behavior need another encoding or repeated logic. This matters especially for H2. |
| Program and execution scale | Core v0.9 admits at most 16 sources and 8,192 source tokens in aggregate. Ordinary recursive calls retain a 32-frame budget. | Large repository-comprehension and deep-recursion tasks may hit implementation limits before testing the intended hypothesis. |
| Stronger specification mechanisms | Contracts are checked at runtime; declared action bounds are metadata, not resource enforcement or arbitrary mathematical proofs. | Comparisons requiring static refinement proofs or capability isolation would compare absent mechanisms. |
| Wider application facilities | Host operations, external-library interoperability and the semantic workspace remain deferred. | Network/storage/system tasks would mix language, ecosystem and tool availability. They are outside the first language-only comparison. |

Sources: [coverage assessment](../docs/language-completeness.md),
[list layout limits](../spec/core-v0.6.md), [module/source limits](../spec/core-v0.9.md),
[tree implementation](../examples/tree.lobo), [shared reverse](../examples/sequences.lobo),
and [architecture status](../docs/architecture.md).

Rust already offers [recursive representations using indirection](https://doc.rust-lang.org/book/ch15-01-box.html)
and [generic definitions](https://doc.rust-lang.org/book/ch10-01-syntax.html). Those are relevant
baselines, not evidence that either mechanism automatically improves an agent's results.

## Recommendation by hypothesis

| Target | Compare now? | Prerequisite for a broader claim |
| --- | --- | --- |
| H1: checked representations | Yes, on supported state/failure tasks | Strong established-language baselines, matched missing-case policies and harder independent tasks |
| H2: task-aligned abstractions | Only on abstractions actually available in each arm | Workloads that expose reuse needs; distinguish library support from new syntax or checking strength |
| H3: semantic context/feedback | A small foreground-tool comparison is possible | Implement and validate any semantic retrieval/workspace mechanism before claiming its benefits |
| Comprehension | Yes, on supported small programs | Multiple modules, realistic dependency/change tasks and scale admission that does not truncate an arm's program |
| General software productivity | No broad conclusion from current fixtures | Broader language/workload coverage, relevant application facilities, multiple models and independent task families |

## Evolve selectively before the broad study

Keep the Core 0.9 evidence as the initial baseline. Develop several materially different programs
before expanding the confirmatory benchmark:

1. A nested syntax-tree or document transformation with a later schema change. Compare the current
   table representation with a direct recursive representation in an established language. Examine
   the extra invariants and implementation work; do not assume that tables are inherently worse.
2. A reusable sequence/result operation used with integers, text and a record across modules.
   Identify actual duplication and lost type information before specifying generic support.
3. A transformation whose behavior varies at several call sites. Establish whether ordinary
   named helpers suffice or function values provide a worthwhile abstraction.
4. A multi-module comprehension/change task involving shared types and dependent functions.
   Check source, declaration and execution budgets before scheduling agent trials.

The first likely language priorities are recursive named data and reusable generic abstractions.
Function values are a candidate if the behavior-parameterization workload demonstrates need.
This is a proposed order, not evidence that all three must be implemented before any comparison.
Numeric types, collection representations and other facilities should follow actual target workloads.

Develop features against public development workloads. Keep confirmatory tasks held out and evaluate
both the preserved baseline and later compiler versions. Do not repeatedly redesign the language
against the same hidden tasks and present the eventual results as independent validation.

Every implemented slice still requires its own specification, failing tests, native/reference and
resource checks, versioning, editor maintenance and agent-skill updates. The user has asked for
an assessment here; no new construct, resource model, dependency or ecosystem expansion is implemented.
