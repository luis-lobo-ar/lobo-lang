# 0028: Create and evaluate the Lobo programming skill

- Status: skill implemented; all 20 attempts accepted; no demonstrated productivity advantage
- Date: 2026-11-03
- Authorization: user requested creating the skill and measuring a change; no plugin or publication.

This implements the skill portion of [0027](0027-agent-language-knowledge-package.md).
The repository skill is `.agents/skills/lobo-programming`, targets Core/compiler 0.9, and
routes to language/tool references and unrelated executable examples. Canonical specifications
remain the authority. The skill teaches source bundles, native execution and test evidence;
it does not grant authority or create another compiler. No model retraining is involved.

## Frozen comparison

Two new task types: implement run-length encoding over immutable lists of records, and implement
Unicode-scalar text clipping with typed bounds failures and repair a byte/scalar helper. These
are new completion/repair tasks, not the earlier collection/tree defects. Neither skill examples
nor task-visible files contain their solutions. The host independently computes expected values.

Two arms: `guide` receives one flattened guide containing all skill entry/reference/example
content; `skill` receives the same content in its discoverable skill package, explicitly invoked
as `$lobo-programming`, with references loaded on demand. Both can read identical canonical
specifications. This tests explicit activation plus organization/retrieval, not extra knowledge,
implicit skill selection, plugin transport, no-documentation learning or fine-tuning. Model
skill-list metadata differs by design. Record whether the entry and references are actually read.

Two tasks x two arms x five repetitions = **20 sequential conversations**. Alternate arm order
by task/repetition. Use `gpt-6.1-sol`, high reasoning, existing authentication, ephemeral sessions,
workspace-write sandbox and the previous optional-feature disables. Each agent has 120 seconds.
Each four-attempt repetition is a separate controller invocation, below the previous long-run
interruption window. Atomically checkpoint each outcome; retain running/interrupted states and
never replace a started attempt. Excluded canaries validate both arms before freezing the run.
No study-specific tuning after measured trials start. Preserve every failure and missing value.

All arms use the same JSON full-checking client/native backend, fixed visible cases and resource
budgets. Record first admitted *edited* candidate, compiler syntax/type diagnostics, attempts,
list/detail/replay calls, feedback bytes, actual terminal input/cached/output/reasoning usage,
end-to-end time through fresh final grading, source/tool/knowledge identities and host load.
Initial stubs already compile, so baseline admission is not evidence of language learning.
Final acceptance independently checks visible plus additional inputs in a fresh full session.

The only editable files are the two named Lobo sources. Skill/guide/spec/client fixtures remain
immutable and integrity-checked. Global host coding-skill instructions may require reading the
host skill directories; authorize those reference locations equally in both arms and record
actual external references/tokens. Other project reads, network access, packages and delegation
remain outside the task. This controls the earlier instruction conflict without pretending
the sandbox gives strict read isolation. No skills or account settings are installed globally.

## Evidence and decision

Primary: independently accepted repairs / all scheduled attempts, per task/arm. Also report
workflow adherence, actual skill/reference use, syntax/type failures, admitted candidate count,
time and usage. Skill loading is included in the measured session. Unknown usage/time is not
zero; interrupted trials remain failures with censored/unknown latency. Give all rows and paired
medians/ranges, with exploratory paired resampling only where both outcomes/times are known.
Do not pick a winner if correctness differs or timing gaps overlap ordinary variation. Two small
related tasks, five repetitions, one requested model and a shared host cannot establish broad
agent productivity. Even successful explicit activation does not prove implicit discovery works.

Keep the skill if it supplies correct, usable versioned guidance; call any time/token benefit a
hypothesis until measured. If no gain appears, record that result rather than removing failures,
changing the baseline or claiming smaller reference files prove efficiency.

Store private raw evidence outside the build directory under ignored `benchmarks/private-evidence/`
so ordinary build cleanup does not erase it. Retain frozen skill, specifications, harness,
requirements, case plans, all CLI events, final sources and analysis. Export sanitized results
and update the result register, decision and ledger before completion. No credentials are copied.

## Pre-run validation

The skill frontmatter validator passes and the Lobo checker accepts the unrelated example module.
Behavioral/native examples, benchmark oracles, information parity, activation, fixture integrity
and checkpoint behavior are checked before measured runs. These checks establish package and
experiment wiring, not an agent productivity gain.

TDD first failed because the evaluation harness was absent. Native oracle admission then caught
an incorrect lowercase JSON record-type tag in the RLE fixture; correcting it allowed the intended
stub-failure/reference-pass checks. Six evaluation tests pass, including information parity,
native examples and checkpoint/unknown-value handling. Excluded RLE/skill and clip/guide canaries
both passed independent grading. The skill canary explicitly read the entry and references.
The final harness adds reporting/metadata to this validated behavior; freeze before measurement.

## Outcome (appended after the frozen run)

All twenty attempts passed independent full native grading (1,220 cases), with no compiler
diagnostics, timeouts or infrastructure failures. All ten skill agents read the entry and both
references. Accuracy equals the equivalent-guide baseline. Median total time was higher with
the skill on both tasks, while paired timing intervals include zero; token effects vary by task.
Three skill-arm parent-instruction probes are separately flagged workflow deviations, without
changing primary acceptance or removing attempts. The audited workflow count is 17/20.

Keep the correct, version-targeted skill; do not claim a measured efficiency improvement or
implicit-discovery result. See [findings](../benchmarks/results/2026-11-03-skill-evaluation.md),
[all trial data](../benchmarks/results/2026-11-03-skill-evaluation.json) and
[trace audit](../benchmarks/results/2026-11-03-skill-evaluation-audit.json). Private evidence retains
this protocol's exact pre-run version. The later appendix does not rewrite the frozen input.
