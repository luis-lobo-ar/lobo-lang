# 0070 — Matched development comparison with and without the new skill

Date: 2026-11-24. Status: selected; register before participant calls.

The user requests a small fresh-task comparison with equal model/reasoning/budgets, frozen Lobo and
complete documentation. Run exactly12 attempts: three fresh task instances × two repetitions × two arms.
Use GPT-5.6-Luna/high in every attempt,120 seconds,16 command-execution items and six visible native
check submissions. No hard token cap. Rotate within-pair order so each arm goes first once per task.
No participant retry, replacement or fallback. One source-free availability check is excluded from scores
and included in costs. Stop after three consecutive infrastructure failures and record remaining tasks.

Compare control with explicit use of the unchanged lobo-development skill. Both arms get identical
requirements, starter/model/util sources,78 common reference files, visible inputs, test client and
acceptance criteria. The skill arm adds only the exact SKILL.md and an invocation sentence. Load it from
a copied project-local skill directory. Use a temporary private CODEX_HOME containing authentication and
model metadata but an empty skills directory, so the global installed skill does not enter controls.
Credentials are not copied into task workspaces or evidence. Disable memories/apps/plugins/multi-agent
features and use fresh ephemeral sessions. Record exposure/read traces and any detected contamination.
This is explicit activation/availability/workflow/salience together, not an equal-text packaging ablation.

Tasks are Unicode window splicing with typed errors/length deltas; ordered batch conversion with first-error
propagation; and a change from active-entry filtering to last-entry/tombstone resolution. They are fresh
requirements/implementations authored after earlier error review, not independently blinded holdouts.
Two tasks are construction and one is a requirement change. Supply an existing generic reverse helper.
The common introductory guide remains identical between arms. Keep private oracles/references out of task
workspaces. Permit optional scratch tests/probes; final native acceptance uses independent Python oracles.

Primary outcome: complete final hidden native acceptance plus fixture/identity/budget integrity. Report
partial case scores, every failed/interrupted/unattempted assignment, visible compile/behavior failures,
first visible passing edited candidate, command/check use, reported tokens including cache, wall time and
unknown usage. Pair by task AND repetition. Do not select a favorable metric after outcomes. Costs include
failed attempts and availability; parent development usage, billing and immutable weights remain unknown.

Calibrate every reference against70 native cases, demonstrate three compiled semantic defects fail, and
retain starter failures. Use TDD for the harness and reporter, preserving initial mistakes/corrections.
Reuse unchanged process/timeout/mailbox/terminal-usage code through an isolated loaded module; a separate
function code object changes only its exact prompt constant for explicit invocation. Tests and audits must
prove the registered prompt and fixtures are exactly delivered. Do not edit old driver files.

For this research-only task, reuse the identical compiler hash and earlier312-pass receipt. Run focused
harness/research tests, native calibration and repository checks. Keep compiler/extension1.11.2, all product
files, old/new skills, cards, studies, primary scores and prior paper entries/pins unchanged. Append results.
The host has other load; alternate pair order and treat latency descriptively. Two repetitions per task
are insufficient for a reproducible speed ranking or population-level advantage. Stop after these12 attempts.
