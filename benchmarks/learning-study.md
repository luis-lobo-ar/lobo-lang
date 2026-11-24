# Separate response, reference and graph studies

Decision [0067](../decisions/0067-response-contract-reference-and-graph-experiments.md) authorizes a bounded
84-assignment pilot. Compiler/extension 1.11.2, Core 1.11, libraries, skill, reference source and all earlier
evidence remain frozen. Code and tasks are additive. All stages are registered before participant calls.

| Stage | Arms | Question instances | Assignments |
| --- | --- | --- | ---: |
| Format | Ambiguous / explicit response contract | Three program families in Lobo/Rust/Python | 36 |
| Reference | Base guide / flat reference | Six Lobo topics, form A | 24 |
| Graph | Flat reference / graph representation | Six Lobo topics, form B | 24 |

Each stage runs Luna/high then Luna/low, with balanced matched arm order. Every assignment starts a new
ephemeral source-only session with no tools and a 120-second wall limit. No hard token cap is imposed.
The six source-free availability checks are excluded from scores and included in cost accounting.
No retries, replacements or fallback model are allowed. After three consecutive infrastructure errors,
the cohort stops and remaining assignments are retained as unattempted.

The format stage retains the previous family/item templates with fresh programs, inputs and targets.
The new programs use amount cap 83, lowest-ID scheduling and checksum seed 13, with new helper names and
preserved Lobo identity values. Both arms have one guide/task copy, common input documentation and the
same strict validator. Only explicit response-profile disclosure is added. Shape validity is this stage's
primary outcome. Task checks and whole-task acceptance remain separate. Old absolute rates cannot be
compared causally with these new instances and context choices.

The [v2 contract](response-contract-v2.md) defines explanation placement and exact closed shapes.
It is an executable research profile, with kind/required/optional descriptors. Its JSON representation is
generated from the validator's definitions. It is not enforced by a provider-native sampling constraint.
The original custom direct-call filter and cued policy-location question remain in the format stage,
so the response-contract intervention does not also change those measurements.

The Lobo topics are state/list traversal, byte/scalar ranges, nominal records and constructor inference,
try propagation, callback/action signatures, and static contracts. Each has six I64 result predictions and
four type/check-acceptance judgments. The latter are checked without executing their bodies. Shared
helpers/types appear once. Each admission snippet substitutes for p0..p5 in that shared context; the host
supplies correct identity metadata. Complete generated modules are retained for reproduction.

Form B has different runtime constants, helper/type names and admission cases. Both forms test related
language laws. Questions are newly authored, but the author saw the preceding studies. This is not a
blinded or confirmatory unseen sample. Compare matched arms within each stage. Do not interpret the
absolute form-A/form-B difference as a graph effect.

The reference rendering uses the existing source-pinned semantic nodes, relationships and checked example
source. The flat and graph renderings contain identical facts and examples. Flat cards express each
relationship in a sentence-like row; the machine-readable graph groups edges in adjacency lists. Exact
node contents and reconstructed relationships are checked. Version/coverage/provenance are included.
The historical experiment-exposure flag is omitted because it describes the earlier study, not semantics.

Reference presence changes available facts and token count. Graph presentation changes their organization
and encoding while preserving facts. These source-only tests do not measure interactive graph traversal,
bounded lookup behavior, retrieval quality or tool-assisted comprehension. All guides, programs and
response contracts within a pair are otherwise identical.

Before registration, validate each cross-language reference and compiled policy mutant against an
independent oracle. Evaluate every Lobo runtime answer in native C and the evaluator, check every static
admission, and kill a compiled semantic defect in each microprogram. The full frozen compiler regression
suite must pass before model calls. Retain every TDD red, intermediate and green result.

The [driver](matched-study/learning.py) pins all code, tests, prompts, reference facts, compiler and
validation. The [reporter](matched-study/learning_report.py) replays grades and hashes, reports each stage
separately, and publishes every answer and error. All reported input/output tokens include failed attempts;
cached input and reasoning output are subsets. Missing usage and actual billing remain unknown.
High-before-low order, provider caches, alias drift, shared host and small sample size limit inference.

Run local tests without model calls:

```sh
python3 -m unittest discover -s benchmarks/matched-study -p 'test_learning*.py'
```

The validate/register/run operations are explicit experiment operations with fresh output directories.
Do not run a published attempt again in place, tune a question after outcomes, or modify frozen inputs.
