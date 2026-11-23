# 0032: Stop feature growth at the evaluation checkpoint

Date: 2026-11-04. Status: current scope rule, following the user's correction about an open-ended feature list.
This supersedes the immediate continuation recommendation in `docs/next-language-milestone.md`.
Past authorized implementations and their evidence remain valid and unchanged.

The project had language goals and workload gates, but continued recommendations did not enforce a clear
finish line. A demonstrated possible improvement must not automatically become another prerequisite for
evaluating Lobo. The next-feature proposal repeated that scope-management mistake.

## Current stopping point

Use implemented Core 0.11 and compiler 0.11.1 as the current language evaluation checkpoint. Stop proactive
additions of syntax, type constructs, runtime facilities, backend optimization and ecosystem services.
Generic data declarations, stronger generic checking, function values and the rest of the wish list are
deferred proposals, not remaining requirements for this checkpoint.

The bounded acceptance basis is representative self-contained parsing/evaluation, collection transformation
with typed failures, and tree/AST computation, with module composition, documented resource/failure behavior,
native/reference agreement and maintained tooling. Existing public workload and verification records provide
that engineering checkpoint. This is enough to evaluate the stated scope, not universal language completeness,
production readiness or proof that Lobo replaces every other language.

## Work within the checkpoint

Allowed continuation is review, study design, documentation, reproducibility work and fixes to violations of
the existing specification. No model-agent experiment starts automatically. A missing facility discovered
during assessment is recorded as a limitation or outcome; it does not trigger a language extension.

Before any study, fix its task set, outcomes, versions, budgets and stopping conditions. During a measured
round, do not change the language/compiler, alter acceptance criteria or add features to rescue an arm.
Retain failures and interruptions. A necessary integrity correction follows an explicit revised protocol,
with original evidence and any exclusions retained.

A later language expansion requires the user to select a new, bounded feature milestone in context, with a
concrete unsupported use case, acceptance conditions and its own stopping point. Do not manufacture another
mandatory feature stage from the roadmap. A question about how to continue is not blanket authorization for
feature growth, paid trials or ecosystem work. Clear contextual authorization still applies without repeated
permission requests.

## Enforcement and limits

The roadmap owns the active checkpoint and AGENTS.md directs future agents to this rule. The old continuation
proposal is explicitly deferred. This is a scope/authorization judgment: a compiler or lint cannot infer
whether an arbitrary code change is an authorized new milestone. No mechanical enforcement is claimed.
The fixed research snapshots remain immutable references even as later documentation changes.
