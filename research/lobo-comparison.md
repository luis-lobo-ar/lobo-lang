# What we can currently say about Lobo versus existing languages

Assessment date: **2026-11-03**. This separates executed checks from hypotheses and unavailable measurements.
The [readiness assessment](comparison-readiness.md) distinguishes focused supported-task comparisons
from broad productivity claims, and identifies workload-driven language evolution before the latter.

Follow-up, 2026-11-04: Core 0.10 implements List-guarded recursive data and the native AST development
workload. [The new evidence](../benchmarks/results/2026-11-04-recursive-ast-native.json) records
source/binary identities and direct execution. Earlier compiler/fixture and agent findings retain
their original versions; this feature does not establish improved agent construction or comprehension.

Later follow-up, 2026-11-04: [Core v0.11](../spec/core-v0.11.md) implements explicit generic function
specialization for a public collection-reuse workload. Its compiler/native tests establish expressibility,
type/action checking and bounded specialization; H2 construction, cost and comprehension effects remain
unmeasured. Generic nominal types and universally checked templates are still absent.

Backend follow-up, 2026-11-04: compiler 0.11.1 now emits application LLVM directly alongside its C route.
The [paired native comparison](../benchmarks/results/2026-11-04-llvm-backend-verification.md) demonstrates
no runtime win on four public workloads, greater LLVM compile latency and equal or eight-byte-larger
executables. C is the better current default under those measurements. This backend intervention supplies
no agent construction, comprehension or token-efficiency estimate. The [paper evidence index](local-evidence.md)
retains the distinction and its preservation record.

| Question | Current Lobo comparison | Evidence limit |
| --- | --- | --- |
| H1: Checked representations improve construction | Lobo and Rust both reject the selected missing-case and invalid-constructor probes. Looser integer-tag encodings compile those defects. | Compiler checks, not observed agent construction success. No overall winner. |
| H2: Task-aligned abstractions improve construction | No comparative agent measurement. Existing Lobo workloads show expressibility, not an advantage over existing languages. | Requires matched abstraction/library interventions, not token counts or syntax preference. |
| H3: Semantic context and feedback improve construction | Lobo has an implemented JSONL verification prototype. Earlier within-Lobo studies do not establish a consistent efficiency advantage. | No cross-language context/feedback experiment; the full semantic workspace remains proposed. |
| Code comprehension | Five behavior-matched program arms and an eleven-item deterministic grader are validated. | No agent or human comprehension scores exist yet. |

The [executed calibration](../benchmarks/results/2026-11-03-language-comparison-fixtures.md) contains
750 baseline behavior checks and fifteen mutation builds. On this small family, native Lobo variants
and Rust enums provide the same observed rejection pattern. A wrong boundary condition compiles
in every arm and fails execution checks. This illustrates both the value and the limits of checking
the representation. It does not establish that Lobo is better than Rust, Python or any other language.

The available toolchains are Rust, Python and native Lobo through Clang. OCaml, Dafny, Lean and
the TypeScript compiler command are unavailable in this environment. No missing toolchain was
installed. Their absence is not a failed language result. SQL/database and proof-language baselines
need task-specific requirements; they cannot be ranked from this native state-workflow example.

## The next measurement

Measure construction and [comprehension](code-comprehension.md) separately across matched programs
and requirements. Use both Lobo/Rust representation arms to test a construct intervention, plus
Python as one established-language baseline. Include stronger Python checking or another typed
baseline before generalizing about dynamic versus static languages. A five-arm pilot is still one
small task family, not a language-maturity or general productivity benchmark.

Before running agents, freeze fresh tasks, source-only versus tool-assisted comprehension rules,
model/budget/repetitions, grading, instruction adherence and failure accounting. Align exhaustive
matching policy for a state-extension task; the public Rust calibration has a catch-all in `step`.
Preserve every candidate source, model response and grading event. Public calibration solutions
are not confirmatory holdouts. No comparative agent run was launched in this step.

The explicit request to launch additional agents is pending under this session's delegation rule.
Fixture and grader validation are complete independent of that request. Prior results and the
[three hypotheses](hypotheses.md) remain recorded even if the eventual comparison favors an existing language.
