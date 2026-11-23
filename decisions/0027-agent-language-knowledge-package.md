# 0027: Supply versioned Lobo knowledge to agents

- Status: skill implemented/evaluated in 0028; plugin remains a proposal
- Date: 2026-11-03
- Authorization: document the proposal and all results. No new experiment or distribution is started.

The user asked whether a skill or plugin would help models that have not been trained on Lobo.
Treat built-in Lobo knowledge as unavailable. We have no verified claim about any model's
training corpus. Runtime documentation and tools are the immediate way to supply the language.

## Evidence and uncertainty

The [scalar pilot](../benchmarks/results/2026-11-03-agent-repair-pilot.md) and
[repeated study](../benchmarks/results/2026-11-03-library-repair-study.md) already provided a
language guide, requirements, existing source and test commands. All 39 completed repairs across
the two studies passed independent native grading. The additional interrupted attempt remains
an infrastructure failure. These results demonstrate agents using supplied Lobo knowledge;
they do not measure a dedicated skill or plugin, cold language discovery, or fine-tuning.

A skill may reduce syntax guessing, repeated documentation searches and incorrect tool use.
That is a hypothesis. A large skill that repeats the entire specification could increase context
cost. The repeated study also exposed global-skill reads outside declared task directories,
so allowed reference locations and skill activation must be explicit in the next evaluation.

Official [skill documentation](https://learn.chatgpt.com/docs/build-skills) describes skills as
reusable instructions with optional resources/scripts, loaded progressively. Plugins distribute
skills and optional connectors. These mechanisms support the packaging proposal; they do not
establish that this package will improve Lobo task outcomes.

## Proposed package

Start with a repository skill for developing Lobo programs. Keep its entry short and link the
canonical language specifications, focused executable examples and tool contracts on demand.
Record supported Core/compiler/tool versions and reject mismatches rather than invent syntax.
Use another entry for compiler implementation only if its workflow warrants a separate skill.

The program-development skill covers immutable bindings, records, alternatives, lists, text,
explicit module inputs and native compilation. Its workflow uses independent expected outcomes,
TDD, bounded verification, failure drill-down and a final forced-full gate. It explains that a
clean protocol exit is not a passing check, pending means unestablished, and replay reproduces
an observation rather than certifying a repair. Preserve fixed criteria, explicit snapshot bases,
environment assertions and resource budgets. Supply small examples without benchmark fixes.

Tools remain the authority for parsing, type checking, execution and evidence. A skill teaches
the agent to use them; its instructions cannot enforce permissions, freeze criteria against a
hostile process or make stale evidence valid. Host checks and a possible formal state model
address those guarantees separately.

An optional plugin can later distribute the skill with a thin adapter to existing CLI/session
operations. Avoid a second compiler or alternative verification semantics. Keep state and
evidence in the host, provide bounded tool responses, and fetch exact retained source only when
needed. The CLI remains usable by shell agents. No MCP server or plugin is necessary merely
to supply the first language skill. The existing VS Code extension remains a separate editor tool.

## Alternatives and evaluation

Continue with the current inline guide as the baseline. Compare it with a versioned skill that
offers equivalent information and on-demand references. Add plugin distribution/tool adaptation
as a separate comparison if needed, so format, knowledge and execution changes are not conflated.
Retraining is not a prerequisite for this evaluation; no fine-tuning experiment is authorized here.

Use new tasks that are not solved by copying the examples. Keep all permitted references inside
the declared task context or explicitly authorize their locations in every arm. Hold the requested
model, independent acceptance checks, budgets and repetitions fixed. Record first valid compile,
syntax/type errors, retrieval and tool calls, actual usage, accepted repairs and total time.
Preserve every failed or interrupted attempt. Count skill/reference loading as part of the task.
Measure whether activation occurs and whether the agent uses the intended workflow.

If the skill improves onboarding but not hard debugging, record both findings. Do not advertise
speed or token savings from smaller files alone. Maintain the skill with language/tool changes
once implemented, using canonical references and executable examples to detect drift.

## Recording rule

The user explicitly requested documenting all results all the time. [AGENTS.md](../AGENTS.md)
now requires result records before task completion, including negative and unknown outcomes,
original evidence for analysis corrections, and distinct labels for proposals. The
[result register](../benchmarks/results/README.md) indexes current experiments and their limits.
Documentation/links are checked for this change; no skill/plugin behavior is claimed as verified.

## Follow-up authorization and evidence

The user subsequently requested creating the skill and measuring a change. [Decision 0028](0028-lobo-skill-onboarding-comparison.md)
records that implementation and a frozen twenty-attempt comparison. The skill is usable, but
shows no accuracy or consistent efficiency advantage over an equally informed guide in that
study. The proposal and original documentation-only authorization above remain historical context.
