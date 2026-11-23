# Lobo skill comparison, 2026-11-03

The Lobo programming skill is implemented and usable. All ten skill-arm agents read its entry
and both language/tool references, then passed independent native grading. The equivalent-guide
arm also passed ten of ten. **This comparison shows no accuracy gain or consistent token/time
advantage from skill packaging.** Median total times were higher in the skill arm on both tasks.
Three skill-arm agents made parent-scope instruction probes, recorded separately from correctness.

[Full trial data](2026-11-03-skill-evaluation.json),
[trace audit](2026-11-03-skill-evaluation-audit.json),
[registered method](../../decisions/0028-lobo-skill-onboarding-comparison.md),
[skill](../../.agents/skills/lobo-programming/SKILL.md).

## What changed

The repository skill targets Core/compiler 0.9. Its entry routes to language and tool references
and unrelated executable examples. It covers immutable data, iteration, Unicode, closed modules,
native compilation and fixed test evidence. The package is 9,995 UTF-8 bytes. Canonical specs
remain authoritative; no model retraining, new language feature, global installation or plugin
was introduced. The compiler, native runner and VS Code extension are unchanged.

Two fresh tasks complete run-length encoding over lists of records and Unicode-scalar clipping
with typed bounds failures and a byte/scalar helper repair. Neither package nor visible project
contains their solutions. The guide arm receives every package file in one flattened guide and
a non-discoverable reference directory. The skill arm receives the same information through an
explicitly invoked discoverable package. Both can read identical canonical specifications.
Thus this compares explicit activation and retrieval/organization, not extra language knowledge,
implicit discovery, a plugin, no-documentation learning or fine-tuning.

Twenty sequential conversations cover two tasks, two arms and five repetitions, alternating
pair order. Native full checking, visible/held-out criteria, budgets and model settings match.
Skill/guide/spec/client fixtures are immutable. All twenty attempts completed, without timeouts,
infrastructure failures, fixture modifications or replacements. Four-attempt checkpointed blocks
avoided the prior long-controller interruption. Both excluded canaries passed before freezing.

## Results

| Task / arm | Accepted | Median seconds | Range seconds | Median first admitted edited candidate |
| --- | ---: | ---: | ---: | ---: |
| Run-length encoding / guide | 5/5 | 35.98 | 34.92–45.51 | 25.66 s |
| Run-length encoding / skill | 5/5 | 39.95 | 34.81–46.97 | 31.59 s |
| Unicode clipping / guide | 5/5 | 29.68 | 27.93–33.85 | 22.05 s |
| Unicode clipping / skill | 5/5 | 37.26 | 26.38–42.21 | 27.04 s |

Total time includes CLI launch, knowledge loading, tools, final response and independent grading.
First admitted edited candidate includes native checking of that candidate. Initial stubs already
compile, so initial admission is not evidence that the model learned the language. No candidate
produced a syntax/type compiler diagnostic in either arm. Every first edited candidate passed
the visible checks. Nineteen attempts used one edited source bundle; one guide/RLE attempt
verified two distinct edited versions, both passing visible checks. This was a later revision,
not recovery from a failing edited candidate. Only final sources received held-out grading.
A skill/RLE attempt made an extra detail call while retaining one distinct edited source bundle.

Progress-note correction: an intermediate update described the guide's second edit as a needed
behavioral repair. The retained trace shows that its first edited version already passed visible
checks. The interpretation above corrects that mistake; no trial data or acceptance changed.

Final grading freshly executed **1,220 native cases**, all passing: 50 per RLE attempt and 72
per clipping attempt. Source algorithms stay in Lobo; Python supplies independent mathematical
expectations. No interpreted execution or reused evidence enters acceptance.

| Task / arm | Median input tokens | Input range | Median cached input | Median output tokens |
| --- | ---: | ---: | ---: | ---: |
| RLE / guide | 150,953 | 149,849–151,354 | 133,248 | 1,125 |
| RLE / skill | 143,606 | 113,518–151,857 | 120,448 | 1,343 |
| Clip / guide | 147,964 | 120,644–148,651 | 126,720 | 870 |
| Clip / skill | 151,742 | 85,195–184,490 | 130,176 | 1,114 |

Usage is the actual terminal CLI report for every attempt. Cached input is included in input;
reasoning output remains separately recorded, not added to estimate a bill. There is no billing
estimate. Lower RLE input and higher clipping input do not establish a general token benefit.
Output tokens were higher at the median with the skill on both tasks.

| Paired skill minus guide | Accepted pairs | Median seconds | Exploratory bootstrap 95% interval |
| --- | ---: | ---: | ---: |
| RLE | 5 | −0.29 | −1.87 to +12.05 |
| Clipping | 5 | +5.71 | −1.73 to +12.53 |

The median paired difference is not the difference of the two arm medians. Both exploratory
intervals include zero and timing ranges overlap. The apparent extra clipping time is worth
retaining as a negative finding; these samples do not justify a general slowdown or speedup claim.
The common host was not isolated: 16 CPUs, recorded one-minute loads 1.56 to 8.92. Model/service
latency, cache behavior and extra tool turns were not separately profiled. Host test-client work
totaled about 2.4–2.7 seconds in each five-attempt group, so it cannot explain most elapsed time.

## What the agents actually did

All ten skill agents read `SKILL.md`, `language.md` and `tools.md`; nine also read the specification
index. They did not materially narrow retrieval to one relevant reference. The flattened guide
offers the same language knowledge in fewer retrieval steps, which is a plausible source of
overhead, not an established attribution of the timing difference. Explicit successful invocation
does not prove implicit matching or cross-model skill quality.

Global host coding-skill reads were permitted equally in both arms and are retained with their
loading cost. Counts varied by attempt. This fixes the earlier study's conflicting read rule but
does not isolate the skill from all host context. Generic `reference_commands` in trial data can
include host references; they are not a count of Lobo-only reference reads. The trace audit stores
the actual Lobo-reference commands separately.

Three skill attempts, indices 4, 7 and 9 zero-based, probed `/tmp/AGENTS.md`, `/AGENTS.md`, or
searched the parent directory for instruction files. Those locations were outside the task's
declared reference scope. The absolute-file probes returned no content. Preserve the parent
search trace privately rather than publish unrelated filenames. No outside source-file changes,
MCP calls or fixture modifications were found. All attempts correctly used the client before
and after editing. Audited workflow adherence is **10/10 guide, 7/10 skill**; native acceptance
remains 10/10 in each. No trial is removed or retuned because of this audit.

This does not prove that the skill caused the scope probes; its entry does not direct them.
It does show that a correct program and loaded skill are insufficient evidence of constrained
agent behavior. Prompt instructions and skill guidance do not enforce filesystem authority.

## Interpretation and follow-up

The package makes Lobo knowledge reusable and supplies a tested invocation route. It has not
improved accepted outcomes over an equally informed guide here. Both arms already have strong
language information, and success is at a ceiling. This comparison cannot answer how either
performs without documentation. The two tasks remain small; failures in long projects, library
discovery, unfamiliar architecture and many edit/verify cycles are unmeasured.

Keep the tested package and its version constraints. A future iteration can test a smaller common
entry and stricter conditional routing to tool details when a project client already supplies the
contract. Do not optimize it mid-run, tune until a favorable sample appears, or market smaller
files as measured efficiency. A plugin and implicit-discovery comparison remain unimplemented.

## Reproducibility and checks

Private raw evidence is under `benchmarks/private-evidence/skill-study-20261103T211835/`, outside
the build directory, ignored by Git and protected by its private directory. It contains frozen
skill/spec/task material, the preregistered protocol, harness/dependency copies, all CLI events,
per-attempt sources and atomic outcomes. It does not contain copied credentials. The exported
report has hashes for large knowledge/source bundles; exact data stays in those private artifacts.
Two excluded canaries are under `skill-canary-20261103T211611/` in the same private-evidence parent.

Requested model `gpt-6.1-sol`, high reasoning; Codex CLI 0.166.0, Clang 18.1.3, Python 3.12.3.
The requested alias is not an immutable provider-weight identity. No pricing or availability
claim follows from successful authenticated local execution.

```sh
python3 benchmarks/skill-evaluation.py --init
python3 benchmarks/skill-evaluation.py --directory PRINTED_DIRECTORY --block 0
# Repeat blocks 1 through 4 sequentially. Each block makes four new model attempts.
python3 benchmarks/skill-evaluation.py --directory PRINTED_DIRECTORY --export benchmarks/results/skill-run.json
python3 benchmarks/skill-evaluation-audit.py benchmarks/results/skill-run.json benchmarks/results/skill-run-audit.json
```

Re-exporting/auditing existing evidence makes no model calls. Initializing and running blocks
uses the existing authenticated quota. A started attempt cannot be replaced silently. The
pre-run record retains the initial absent-harness TDD failure and corrected record-tag fixture.
The skill validator, its native examples, all 24 Python tests and 145 Rust tests pass. The
Python tests include source/oracle admission, information parity, skill links, atomic checkpoints,
missing usage/failure accounting and scope-probe classification. No editor checks were rerun
because neither language nor extension behavior changed.
