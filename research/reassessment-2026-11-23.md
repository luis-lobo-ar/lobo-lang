# Lobo research reassessment at compiler 1.11.2

Date: 2026-11-23. Input revision: e489dca976ce45ae1628416093fb489cfb234f1f.
Language Core 1.11, compiler/extension 1.11.2, native ABI 1, SQLite extension 1.

Lobo is a substantially more capable experimental platform than the Core 0.9 used in the original agent
studies. The research claim remains open: we have not established better agent construction, comprehension
or whole-task efficiency than established languages. This review revisits the original dimensions, replays
local checks, and separates those observations from the historical agent studies.

The [machine-readable assessment](../benchmarks/results/2026-11-23-research-reassessment.json) records inputs,
counts, failures and unknowns. No new model-backed trial or literature search was launched. This is an
engineering reassessment and local replication, not a confirmatory language-productivity experiment.
The [original hypotheses](hypotheses.md), [construction framework](construction-quality.md),
[comprehension framework](code-comprehension.md) and dated results retain their original scope.

## The original objective and evaluation rule

The objective is to determine whether language constructs and development interfaces designed for agents
help them construct correct software, understand existing code and preserve correctness through change.
The primary endpoint is complete requirements satisfaction at a declared total budget. Tokens, time,
retrieval and verification work measure costs. Program runtime and source length are separate outcomes.

H1 attributes a benefit to checked representations. H2 attributes it to abstractions that express the task.
H3 attributes it to relevant context and useful feedback. A combined Lobo result would need further
experiments to distinguish these explanations. An improvement obtainable in Rust or through a library
would be useful evidence for the mechanism, without establishing a distinctive Lobo advantage.

## What changed across iterations

| Checkpoint | Relevant development | Research consequence |
| --- | --- | --- |
| Initial native Core | Native compiler, evaluator reference and Rust/Python/Node arithmetic comparisons | Establishes executable behavior and scoped runtime observations. Agent productivity was not measured. |
| Core 0.9 | Records, variants, lists, modules, JSONL testing, programming skill; repair and onboarding studies | Provides the original agent evidence. Recursive/generic/higher-order limitations restricted H2 workloads. |
| Core 0.10 and 0.11 | List-guarded recursive data, AST workload and explicit generic functions | Closes specific expressibility gaps. Public development programs are not independent evaluation holdouts. |
| Core 1.0 | Generic nominal data, checked generic templates, noncapturing function values and the bounded acceptance milestone | Supports more credible reuse and dependent-change tasks. Completion was a specified language milestone, not universal replacement. |
| Core 1.1 and 1.2 | Bounded operator bindings, compact native authoring and maintained-source migration | Reduces explicit source ceremony. No new agent construction or comprehension score follows. |
| Core 1.3 | Persistent identity sidecars and explicit authoring operations | Separates authoring text from identity state. Total metadata and identity-maintenance costs must still be counted. |
| Core 1.4 through 1.6 | Generic/callback inference, constructor inference and automatic JSON builders | Removes repetitive type/constructor/builder spelling. Context needed to infer meaning may move to dependencies. |
| Core 1.7 and patches | Closed OS boundary, grants/quotas, finite TLA+ models, Windows snapshots and native HTTP | Adds executable authority and resource checks. Models cover declared finite properties, not arbitrary native-code safety. |
| Core 1.8 | Prefix try and maintained-source modernization | Reduces error-forwarding scaffolding. Cleanup ownership remains explicit. |
| Core 1.9 | SQLite boundary, typed rows, persistent HTTP example and larger source budget | Enables bounded storage/application tasks. Ecosystem availability becomes an additional comparison variable. |
| Core 1.10 | Immutable record updates and standalone try | Reduces field copying and unused success bindings. No agent outcome has been measured for these constructs. |
| Core 1.11 / compiler 1.11.2 | Action-bound callbacks, shared HTTP lifecycle and typed OS/SQL permissions | Enables shorter applications with explicit authority. Adds reusable mechanisms and indirection whose agent effects remain unmeasured. |

The dated [Core 0.9 readiness assessment](comparison-readiness.md) remains a historical assessment. Its
absence-of-generics/function-values/OS statements must not be read as descriptions of Core 1.11.
Current specifications and implementation receipts supersede those implementation gaps. Earlier experiments
retain their versions; later features cannot retrospectively improve their measured outcomes.

## Construction and requirement evolution

The original three agent studies scheduled 60 attempts: ten scalar repairs, thirty library repairs and
twenty skill/guide attempts. Fifty-nine were accepted, and one controller interruption remains recorded.
Usage is known for 59 attempts. The unknown interrupted usage is not zero. All arms used Lobo, with the
requested model alias gpt-6.1-sol. These are repeats of a small collection of tasks, not 60 independent
programming workloads and not a cross-language construction comparison.

The tasks had ceiling-limited first-edited-candidate outcomes. They establish that the supplied workflow
could support those repairs. They do not establish difficult diagnostic recovery, current-language onboarding,
repository-scale construction or robustness through independently hidden requirement changes.

Current records document working native applications and passing compiler/runtime checks. That materially
improves feasibility for a new study, but contributor-authored examples and implementation tests cannot be
substituted for independent agent tasks. We still lack a current Lobo/Rust/Python construction or requirement-
evolution trial with matched requirements, total budgets and frozen independent grading.

## H1: Checked representations

The unchanged five-arm calibration was rerun with the current compiler. All 750 baseline observations agree
with the independent state-transition oracle. The fifteen mutation builds preserve the original pattern:
Lobo variants and Rust enums reject the selected missing-case and invalid-constructor mutations. The loose
record/tag encodings admit them. An incorrect business-rule boundary compiles in every arm and is caught by
execution. See [the fresh calibration](../benchmarks/results/2026-11-23-research-reassessment-calibration.json).

This supports the selected checking mechanisms and their limits. It supplies no agent acceptance estimate.
The fixtures retain their historical source spelling. They test compatibility and semantics, not whether
the newest authoring syntax improves an agent’s work; future agent programs should use idiomatic current syntax.
Rust is an essential comparator because the observed type-checking benefit is not unique to Lobo.
The frozen Rust step function uses guards and a catch-all arm. Align exhaustive-matching policy before
using these programs for a controlled state-extension trial.

Later typed permissions additionally reject integers or the wrong permission domain at the library boundary.
Runtime provider checks enforce grants and attenuation. Declared actions bound source operations; they do
not grant authority. Capabilities do not statically encode every right or prevent every resource-lifetime
mistake. Runtime contracts are not mathematical proofs of arbitrary requirements. A study must identify
which guarantee is being compared and evaluate the actual requirements independently of agent-authored contracts.

Verdict: implemented and calibrated mechanisms; H1's agent-construction effect remains unknown.

## H2: Abstractions and developer experience

Generics, named callbacks, inference, JSON helpers, try, record updates and HTTP helpers now provide usable
abstractions absent from the initial comparison platform. The application can request server_permissions()
and delegate listener lifetime to serve or serve_with. SQLite delegation names one(Read). Stable IDs and
error/cleanup policies were preserved by the recorded migrations.

A fresh lexical inventory observes 46 maintained Lobo files and 249 functions, compared with 43 files and
214 functions before the Core 1.9 DX review. Candidate forwarded fields change from 45 to 37 and unused
bindings from 29 to 4. The cleanup-pattern detector changes from 13 matches to zero. Its specific spelling
includes a discarded let before try-close, so standalone try can remove a match without removing cleanup.
These are lexical indicators on an expanded corpus, not defect counts, AST complexity or agent improvement.
The maintained-source audit also passes for all 46 sources with zero remaining checked-equivalent rewrites.

Library extraction reduces decisions at a call site but introduces dependency traversal. Inference removes
spelling while requiring knowledge of expected types. These can help or hinder comprehension. Compare
matched abstractions and equal library help before attributing a difference to Lobo syntax.

Verdict: stronger and more useful candidate mechanisms; H2 remains unmeasured in agents.

## Source size, identities and tokens

The current MVC and the preserved older spelling were compiled with the same HTTP/JSON libraries through
both C and LLVM. Twenty controller invocations, including missing users and I64 boundaries, match independent expected
statuses/bodies and agree across the four programs. The 404 helper intentionally returns plain text.

| Complete application representation | Bytes |
| --- | ---: |
| Preserved older MVC source | 1,037 |
| Current MVC source | 503 |
| Current identity sidecar | 465 |
| Current source plus sidecar | 968 |

The source file is 51.5% smaller. The combined representation is 6.7% smaller. Comments, formatting,
inference, JSON syntax and identity extraction all contribute; this is not an isolated syntax experiment.
Both counts exclude common library inputs. The compiler/editor can read sidecars without an agent reading
them, so total stored bytes also cannot be substituted for actual model context consumption.

No model-tokenizer package is installed for this measurement, and none was added. Byte counts are not
lexical tokens, model tokens or billed usage. There is no new whole-trajectory token-saving estimate.
Measure actual reads, prompts, diagnostics, retries, cached/uncached input and final correctness together.

## Onboarding, skills and adherence

The original skill trial accepted ten of ten skill attempts and ten of ten equivalent-guide attempts.
It demonstrated no accuracy gain or consistent token/time advantage from packaging. All skill attempts read
the entry and both references rather than narrowing retrieval. Three skill attempts made out-of-scope parent
instruction probes; correctness and adherence were scored separately. This is not evidence that a skill
caused those probes, nor that loaded instructions enforce filesystem authority.

The exact four original knowledge files were recovered from the hash-verified Core 0.9 archive. Every hash
matches the original trial's knowledge manifest. Comparing those same four filenames gives:

| Knowledge input | Bytes |
| --- | ---: |
| Original four-file package | 9,995 |
| Current same four files | 37,323 |
| Entire current programming-skill directory | 40,857 |

The same-file volume is 3.73 times larger. Coverage expanded considerably, so this is not an equal-information
experiment or proof of worse onboarding. It identifies a cost that future trials must measure. A smaller
source file can coexist with more documentation to retrieve. Bounded task-relevant guidance is a candidate
intervention, not an implemented or measured improvement in this review.

[The frozen input manifest](snapshots/2026-11-23-reassessment/knowledge.json) and exact .txt snapshots make
this accounting reproducible without exposing private agent traces. Model training exposure remains unknown;
a novel language and a working skill do not establish absence from proprietary training data.

## H3: Testing, feedback and context

The original six scripted verification scenarios were replayed against the current native test runner.
All 384 affected/full case comparisons agree, including expected regressions. Affected selection executes
128 cases and reuses 256 passing outcomes; forced-full execution executes 384. Comments, unreachable edits,
leaf/transitive edits, contract edits and layout additions retain their intended behavior.
See [the fresh selection replay](../benchmarks/results/2026-11-23-research-reassessment-selection.json).

This is a reproducible execution-reuse mechanism, not a 66.7% agent-cost reduction. Native compilation,
model/session work, retrieval and tool turns remain separate costs. The historical pilot delivered more
feedback bytes with JSON; the repeated library study had mixed task-level time/token results. A structured
interface alone did not establish general efficiency.

The compiler supplies machine-readable diagnostics, explicit bundles, managed identities and a JSONL
verification session. Editor navigation and diagnostics have implementation evidence. The full semantic
workspace, transactional graph-edit interface and general context-selection engine remain future work.
Source/identity snapshots provide part of the state discipline, not the complete original workspace vision.
The earlier [proposal to model the verification-session protocol](../decisions/0026-repeated-library-repair-study.md)
remains unimplemented. The later native/SQLite TLA+ models cover different systems; they do not prove the
plan/snapshot/evidence-reuse protocol correct.

The original six scenarios do not independently establish cache soundness for every newer callback,
permission or database pattern. Future trials should exercise current multi-module changes and compare
against credible file/search and conventional language-server workflows with the same underlying facts.

Verdict: the original scripted mechanism replicates; H3's current agent-productivity effect remains unknown.

## Code comprehension

The eleven-item deterministic grader still validates predictions, witnesses, dependency sets, change impact,
fault location and a distinguishing input. Negative answers are rejected by the harness tests. Its reference
answer passing is grader verification, not an agent score. There is still no cross-language agent comprehension
ranking and no human-participant study.

New inference and helper indirection create important new comprehension questions: can an agent recover
which variant/type is meant, find the actual cleanup owner, distinguish declared actions from granted rights,
and identify affected callers after a permission or response-schema change? Score those questions before
editing and separate source-only from tool-assisted conditions. A passing repair cannot stand in for understanding.

## Safety, formal verification, tooling and maturity

Native grants/quotas, typed resources, explicit cleanup, SQLite roles and finite state-machine verification
are substantial engineering additions. The recorded native operation model exposed a real pending-socket
fault-cleanup defect, reproduced before its fix, with deliberate model/C mutations checking the witnesses.
That is a concrete formal-workflow finding; see [the original result](../benchmarks/results/2026-11-22-native-formal-verification.md).
Fresh targeted tests replay the two native C counterexample groups
and verify SQLite source correspondence. The full TLC campaigns are not rerun; covered implementation/model
sources are unchanged. Their existing bounded assumptions, trusted native code and platform limitations remain.
Neither model checking nor sanitizer success proves that agents construct safer complete applications.

The 182 editor tests pass again. The earlier real editor-host, Linux/Wine executable and 312-test Rust receipts
apply to byte-identical product inputs; those full campaigns were not repeated for this analysis-only work.
Fresh benchmark-harness tests pass 31 cases, and the new accounting collector passes four regression tests.
Current Windows hardware behavior remains unknown for this patch; Wine and the earlier user report stay distinct.

The current language is sufficient for a broader shared-domain study than Core 0.9: recursive-data
transformations, generic reuse, parsing/validation, controlled callbacks and multi-module change are feasible.
The 16-source, 16,384 aggregate-token and 64-nominal limits, execution budgets and recursion limits still bound
tasks. Function values remain noncapturing named references; action sets are exact rather than polymorphic.
HTTP/JSON/SQLite enable bounded application work, while the ecosystem and external-library catalog remain limited.
Do not generalize this milestone to unrestricted production replacement or to every mature-language guarantee.

## Native execution and backend costs

The original five-way driver was recovered unchanged from the comparison branch and run on an isolated
snapshot of the current compiler sources. Only its four frozen benchmark files were overlaid; compiler
sources match the current product. Each of thirty series has 100 measured batches, for 3,000 checked samples.
Single-call and 100-call regimes are retained. Sustained native/Rust samples use one million iterations;
the evaluator/Python/Node samples use ten thousand. These reduced sustained counts are explicit, so the
separately dated historical values are not a controlled regression estimate.

| Sustained route | Increment ns/call, median [min, max] | Absolute ns/call, median [min, max] |
| --- | ---: | ---: |
| Native Lobo | 4.745 [4.567, 6.487] | 4.678 [4.413, 8.136] |
| Rust | 0.771 [0.727, 1.385] | 0.920 [0.846, 1.735] |
| Reference evaluator | 4,542.530 [4,360.155, 5,325.146] | 4,695.935 [4,496.901, 6,287.845] |
| Python | 263.211 [248.138, 330.033] | 154.466 [146.908, 254.663] |
| Node BigInt | 59.961 [53.958, 418.190] | 95.637 [78.951, 608.209] |

[All samples and ranges](../benchmarks/results/2026-11-23-research-reassessment-fiveway.json) retain the exact
iteration counts, checksums, tools, affinity and warmup policy. The native route includes host fuel/diagnostic
bookkeeping absent from the direct Rust arithmetic. Python checks signed-I64 bounds; Node uses BigInt rather
than Number. Dynamic warmup is a fixed budget, not proof of JIT steady state. Single-call measurements are
near clock/first-use overhead. Sustained results include loop/checksum costs. Compilation, startup, argument
parsing and output are outside these timed regions. These two tiny functions do not establish a general
language ranking or the latency of lobo run, which also compiles its program.

C/direct-LLVM was also rerun on sum, parser, AST and generic workloads with five alternating compile samples
and seven runtime samples per side. All outputs match independent expectations. Observed LLVM compile
medians are about 20% to 30% higher. Runtime medians show no consistent gain and include startup/noise floors.
[Complete observations](../benchmarks/results/2026-11-23-research-reassessment-llvm.json) include ranges and
child CPU times. Separate process traces identify the extra LLVM runtime-compilation/link stages. perf
profiling is denied by the host, so counter values and a detailed hardware bottleneck attribution are unknown.
No new universal backend-performance or longitudinal-regression claim is made. C remains the existing default.

| Program | C executable bytes | LLVM executable bytes | C compile median ms | LLVM compile median ms |
| --- | ---: | ---: | ---: | ---: |
| sum | 14664 | 18760 | 149.790 | 185.910 |
| parser | 47480 | 47480 | 452.737 | 544.723 |
| ast | 22888 | 22896 | 233.757 | 302.701 |
| generic | 22888 | 22896 | 237.570 | 295.024 |

These are optimized program executables. The Rust comparison harness embeds the compiler/evaluator, so its
file size is not an equivalent standalone Rust-program size comparison.

Program execution performance remains a separate experiment from software-construction quality and agent cost.

## Research decision and next experiment

The evidence supports implemented mechanisms, local behavioral agreement and a broader comparison platform.
It does not yet establish any of H1/H2/H3 as an agent-level advantage. The most important new tradeoffs are
smaller authoring text versus total metadata, and broader language coverage versus larger onboarding inputs.

The next recommended milestone is an evaluation checkpoint at the current version. No additional language
feature is selected by this assessment. A separate difficulty pilot should produce tasks that require real
investigation and multiple candidates, followed by a preregistered study with:

1. Independent, frozen requirements and grading for matched Lobo/Rust tasks, with Python where appropriate.
2. Distinct construction, pre-edit comprehension and hidden requirement-evolution endpoints.
3. Whole-trajectory budgets that include documentation, retrieval, tools, failures and unknown usage.
4. Within-language representation/abstraction comparisons to separate H1 and H2 from language familiarity.
5. A later controlled feedback/context comparison for H3, including strong existing-tool baselines.
6. Multiple task families and model families, interleaved arms, retained intermediates and explicit stopping rules.

Do not reuse these public calibration solutions as unseen tasks. Equal guides cannot erase different
pretraining familiarity. The original easy trials should remain historical pilots instead of being repeatedly
rerun and treated as decisive productivity evidence.

## Reproduction and preservation

Run the accounting collector with a fresh output path:

```sh
python3 research/reassess.py --out NEW_ACCOUNTING.json
python3 -m unittest discover -s research -p test_reassessment.py
python3 -m unittest discover -s benchmarks -p 'test_*.py'
python3 benchmarks/language-comparison.py --output NEW_CALIBRATION.json
python3 benchmarks/agent-testing.py > NEW_SELECTION.json
python3 scripts/dx_inventory.py --out NEW_INVENTORY_DIRECTORY
python3 benchmarks/llvm-backend.py --out NEW_BACKEND_DIRECTORY --compile-repeats 5 --run-repeats 7
```

The five-way receipt pins the current base commit, frozen comparison-branch commit, four overlay hashes,
isolated snapshot commit and binary hashes. Recreate that snapshot and use its recorded compare.py command;
do not silently replace main's three-way script or rebuild a historical compiler as if it were the current one.

Raw outputs, profile attempts and intermediate failures are retained in target/research-reassessment.
The ignored preservation directory is benchmarks/private-evidence/research-reassessment-20261123. It includes
source inventories/archives, the isolated five-way snapshot bundle, benchmark artifacts and raw command receipts.
All 36 earlier paper entries and their pinned bytes remain unchanged; the new result is indexed separately.
The historical calibration script embeds its original date, retained inside a wrapper with this run's date.

TDD and correction logs retain: absent collector, an intermediate unpacking syntax error, a failing empty-usage
regression, an incorrect JSON assumption for the MVC's plain-text 404, an exploratory probe-schema mismatch,
and denied perf access. These are analysis/harness corrections or unavailable profiling, not hidden language
failures. No old report, criterion or recorded trial was replaced.
