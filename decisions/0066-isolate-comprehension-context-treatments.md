# 0066 — Isolate comprehension context treatments

Date: 2026-11-23. Status: selected by the user; registered pilot follows validation.

The user selected prompt deduplication and standardized input documentation after decision 0065.
Keep compiler/extension 1.11.2, libraries, old skill, drivers, questions and evidence frozen. Add a
versioned research driver, fresh source/question variants, a compact language reference and a concept map.
No syntax, API or runtime change is selected. Risk mode: lean research, with retained model spend and
outcomes. No new dependencies or background services.

Use a 2×2 design: original context, deduplication only, input schema only, and both. Deduplication removes
only the second TASK/GUIDE blocks. Keep the other authoring/check noise constant to isolate this factor.
Append the same language-neutral schema once in each schema arm. Schemas describe the wire input, not
language semantics or answers. Keep language guides unchanged. The new question convention explicitly
includes the edited predicate in impact in every arm, rather than treating that convention as an input fix.

Run 72 fresh source-only assignments: four arms × three languages × three families × two reasoning
settings. Use GPT-5.6-Luna/high then low, with four balanced arm orders inside matched blocks. Each
family has one new program form, six new predictions and five other objective questions. Change constants,
helper names and scheduler selection direction from the old study. Compare treatments within these new
questions. Do not attribute old/new score differences to the interventions. The small pilot is descriptive.

Use identical 120-second wall limits and disable all tools. There is no hard token cap. Retain usage,
cached tokens, answers, command configuration, failures and unknowns. No retries or replacements. Stop
a cohort after three consecutive infrastructure failures and record all remaining assignments. Exclude
one source-free availability canary per setting from scores and include it in cost accounting.

Write tests first for exact old prompt parity, isolated treatments, grading, fresh cases, stable identity
values, configuration and schedule. Compile/run each fresh reference and policy mutant in all three
languages against an independent oracle before registration. A mutant must compile and fail behavior.
Pin code, tests, prompts, validation and frozen compiler before the first participant call. Preserve red
and intermediate failing development checks as well as green results. Do not tune tasks after outcomes.

The additive language reference distinguishes syntax/type rules, libraries, runtime and study encodings.
Every concept has authoritative source provenance. Generate machine-readable relationships and editable
draw.io XML from the same data, with a bounded topic lookup. Keep these new materials out of the trial
prompts. A map's comprehension benefit remains a separate hypothesis requiring a later selected experiment.

Report per-arm acceptance and item errors, matched discordances, actual reported token usage and wall
time. Prompt bytes are not tokens or billing. Provider caching, model aliases, task reuse and high-before-low
time order limit interpretation. Preserve every prior paper entry/pin and append the new evidence.
