# Frozen Luna high/low reasoning comparison

Selected by the user in decision 0064. Two consecutive cohorts: GPT-5.6-Luna/high, then GPT-5.6-Luna/low.
Each uses the same 54 assignments as the completed GPT-6.1-Sol/high baseline: three families, three
languages, construction/comprehension/change, two repetitions. Run all high assignments before low.

`matched-study/luna.py` imports and verifies the original controller by SHA-256. The original task,
guide, source, prompt, graders, mailbox, edit rules, schedule and 120-second/16-command/6-check limits
remain unchanged. No hard token cap is available. Unit tests prove command parity except the model and
reasoning arguments and compare every registered fixture hash. Compiler 1.11.2 remains archived and pinned.
The new driver and both cohorts are registered together before participant calls.

Before each cohort, one excluded source-free, shell-disabled model-access canary checks the requested
configuration. Preserve its events and usage. Lack of access means that cohort's 54 assignments remain
unattempted; do not substitute a model. Every scored assignment starts fresh. No retries, replacements,
new hints or task tuning. Keep candidate compile/test/limit failures. Three consecutive infrastructure
failures stop that cohort, with remaining assignments explicitly unattempted; proceed to the next cohort's
access check. No outcome is silently removed. Independent final grading is outside agent time limits.

Compare whole-attempt acceptance, comprehension items, per-case outcomes, diagnostic recovery, commands,
checks, participant-reported tokens and wall time. Cache hits remain within input tokens. Missing terminal
usage is unknown, not zero. Report failure counts next to successful-only costs and source/runtime artifacts.
The original cohort and all older evidence remain separate. Both new cohorts share the frozen task set,
which is small and previously ceiling-limited. Predicate location is cued. These are not 108 independent
real-world problems and not a causal proof that reasoning or model size alone explains differences.

The chronological order was requested, so cache/service/time drift can confound high/low differences.
Comparisons with the older baseline can also reflect changes in host metadata or model aliases; versions,
executable hashes and load are retained, but immutable model weights and exact hidden host context are
unknown. This tests two requested configurations, without asserting that Luna is provably the weakest.

The new registration and private raw evidence use `luna-study-20261123`, preserving all earlier paths.
Only authorized model runs use `python3 benchmarks/matched-study/luna.py run`; unit tests and inspecting
stored evidence do not call models. Run with screen so phone/SSH disconnects do not interrupt the study.
A full run contains 108 scored agent conversations plus two excluded access checks; each conversation may
contain multiple underlying model requests. Stop after both cohorts and their documented results.

## Completed evidence

See research/luna-reasoning-study-2026-11-23.md, the normalized data in
benchmarks/results/2026-11-23-luna-reasoning-study.json and paper local-039. Public source/answer bundles
are in benchmarks/fixtures/luna-study-20261123. Replaying them and the post-hoc diagnostics needs no
model calls. All prior source, profiles, frozen archives and paper evidence remain unchanged.
