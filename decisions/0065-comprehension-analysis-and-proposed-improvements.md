# 0065 — Explain comprehension failures before selecting changes

Date: 2026-11-23. Status: requested analysis completed. Improvements proposed, not implemented.

Risk mode: lean. This is a bounded offline audit and documentation task, without deployment or model spend.

The user requested an analysis of language, documentation and model contributions before changes.
Inspect all frozen comprehension responses, compare languages/configurations, audit delivered context
and replay diagnostic inputs. Make no new model calls, grader edits, language changes or guide revisions.

The audit finds seven failed Lobo attempts and nine wrong items. Five items also fail in another language when pooling configurations. Four have a same-configuration counterpart.
Canonical executions agree with all 54 prediction expectations. Model/configuration sensitivity is observed,
while causally separating language familiarity, API conventions and documentation requires interventions.

A definite prompt-assembly issue duplicates TASK/GUIDE and includes check-client and authoring instructions
in source-only comprehension. Input encoding is less explicit than it should be across language guides.
Those issues are not established causes of the scored errors. One Lobo answer omits the changed function
from impact, and one payload answer is consistent with count/end confusion. Other errors contradict
explicit numeric, scheduling or witness constraints. No compiler mismatch is demonstrated in these cases.

Prefer a versioned prompt-cleanup and common-schema experiment over immediate syntax/API changes. A
presentation and tool-assisted comprehension comparison can follow if selected. Freeze the old data and
separate new teaching examples from new held-out questions. Do not infer token savings from bytes or
attribute causal fractions from observational counts. Language/library changes require residual errors
under those controls and a separate user-selected scope.

See research/comprehension-failure-analysis-2026-11-23.md and the offline audit data. This decision grants
no authority to implement proposals, rerun agents or unfreeze 1.11.2. Append this analysis as new paper
evidence while preserving all 39 earlier entries and 118 file pins.
