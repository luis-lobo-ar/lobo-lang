# 0064 — Repeat the frozen pilot with Luna at high and low reasoning

Date: 2026-11-23. Status: completed; both requested cohorts retained without retries.

The user selected both GPT-5.6-Luna configurations: first the same high reasoning used by the
GPT-6.1-Sol baseline, then the lowest advertised reasoning level, low. Run two separate 54-assignment
cohorts. Keep compiler/extension 1.11.2, Core 1.11, task/guide/source bytes, case sets, schedule, controller,
allowed edits and working limits unchanged. No language/library/editor/skill or previous evidence changes.

Use an additive driver importing the hash-verified original controller. Only model and reasoning CLI
arguments change. Prefer this over editing/copying the frozen controller, which would make equivalence
harder to audit and risk changing old evidence. Register the wrapper hash and both configurations before
any participant launch. Check model access with one excluded, source-free, shell-disabled invocation
immediately before each cohort. No fallback model or replacement trial. Candidate failures/timeouts count.
Three consecutive infrastructure failures stop that cohort; retain its remaining assignments as unattempted
and continue to the next configuration's access check. Retain all raw events, checks, final sources,
usage or unknowns, and independently grade every final candidate.

The common limits remain 120 seconds, 16 command-execution tool items and 6 visible checks, without a hard
token cap. High-before-low order follows the user's request; time/cache/provider drift remains a confound.
Compare each cohort with the earlier baseline and with each other, while distinguishing configuration
comparisons from causal attribution. Model descriptions do not establish a measured capability ranking.
The earlier baseline may have different host metadata or provider behavior despite matching input bytes
and CLI flags. Record current/before-after versions and load. Do not pool away failed attempts or infer
billed cost from cache counts. Participant usage excludes parent/harness-development cost.

All previous 38 paper entries and 111 file pins remain unchanged. Add results as dated evidence. Runtime
measurements from decision 0063 remain valid for their binaries; this request changes agent configuration,
not the speed of those same artifacts. Any new runtime comparison of generated programs must be separately
labelled and must retain unsuccessful-task denominators. Stop at these two cohorts and documentation.

## Completed observations

All 108 scored assignments completed, with no missing terminal usage, budget overruns or infrastructure
interruptions. Luna/high accepted 49/54 and Luna/low 44/54. Code-writing outcomes were 36/36 and 35/36;
comprehension accepted 13/18 and 9/18, with 192/198 and 187/198 items. Lobo passed all 12 construction/change
assignments in both cohorts. Low used less participant time and reported fewer tokens, with more failures.
A low Rust scheduler has a confirmed storage-index/job-ID mismatch. Preserve its failing code.
Source/command hash audits and public replays pass. See research/luna-reasoning-study-2026-11-23.md and
paper entry local-039. These observations select no language feature, further model trial or runtime study.
