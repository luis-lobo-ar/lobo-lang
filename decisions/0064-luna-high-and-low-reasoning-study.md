# 0064 — Repeat the frozen pilot with Luna at high and low reasoning

Date: 2026-11-23. Status: user-selected; experiments pending.

The user selected both GPT-5.6-Luna configurations: first the same high reasoning used by the
GPT-6.1-Sol baseline, then the lowest advertised reasoning level, low. Run two separate 54-assignment
cohorts. Keep compiler/extension1.11.2, Core1.11, task/guide/source bytes, case sets, schedule, controller,
allowed edits and working limits unchanged. No language/library/editor/skill or previous evidence changes.

Use an additive driver importing the hash-verified original controller. Only model and reasoning CLI
arguments change. Prefer this over editing/copying the frozen controller, which would make equivalence
harder to audit and risk changing old evidence. Register the wrapper hash and both configurations before
any participant launch. Check model access with one excluded, source-free, shell-disabled invocation
immediately before each cohort. No fallback model or replacement trial. Candidate failures/timeouts count.
Three consecutive infrastructure failures stop that cohort; retain its remaining assignments as unattempted
and continue to the next configuration's access check. Retain all raw events, checks, final sources,
usage or unknowns, and independently grade every final candidate.

The common limits remain120 seconds,16 command-execution tool items and6 visible checks, without a hard
token cap. High-before-low order follows the user's request; time/cache/provider drift remains a confound.
Compare each cohort with the earlier baseline and with each other, while distinguishing configuration
comparisons from causal attribution. Model descriptions do not establish a measured capability ranking.
The earlier baseline may have different host metadata or provider behavior despite matching input bytes
and CLI flags. Record current/before-after versions and load. Do not pool away failed attempts or infer
billed cost from cache counts. Participant usage excludes parent/harness-development cost.

All previous38 paper entries and111 file pins remain unchanged. Add results as dated evidence. Runtime
measurements from decision0063 remain valid for their binaries; this request changes agent configuration,
not the speed of those same artifacts. Any new runtime comparison of generated programs must be separately
labelled and must retain unsuccessful-task denominators. Stop at these two cohorts and documentation.
