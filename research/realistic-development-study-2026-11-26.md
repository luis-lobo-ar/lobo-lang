# Realistic Lobo development with and without v2

Date: 2026-11-26. Complete:12 attempts, all results preserved.

The question is whether the existing v2 development skill helps the same agent complete more realistic
Lobo application changes under the same limits. Lobo1.11.2/Core1.11, libraries, compiler, editor and both
skills remain frozen. This study changes application fixtures only. It does not measure a language change.
[Decision0072](../decisions/0072-realistic-development-v2-comparison.md) fixes the scope and stopping rules.

## Registered design

Preregistration `1517d17f842df52451851537f4aeba1ef9fedfe1` and raw-whitespace metadata correction
`878329fae1a72c89bd18a9426bca81447f645708` were pushed before availability or scored calls. The correction
preserves copied reference bytes with a scoped Git whitespace attribute; it changes no delivered input.
[The manifest](snapshots/2026-11-26-realistic-study/registration/manifest.json) pins exact code, delivered
files/prompts, model settings, budgets, materials, compiler and calibration. Snapshot Markdown copies use
`.md.txt` with a verified [logical path map](snapshots/2026-11-26-realistic-study/document-path-map.json).

There are12 fresh ephemeral attempts: two tasks, three repetitions, control/v2. Order alternates by
(task index + repetition), balanced across tasks. Every attempt uses GPT-5.6-Luna/high,120 seconds,
16 command-execution items and six visible check submissions, with no hard token cap. Both arms receive
identical78 reference files, requirements, working scaffolds, four editable files, explicit source bundle,
identity-authoring helper and native feedback. Only the frozen3138-byte v2 entrypoint and explicit invocation
differ. Skills are isolated with a temporary CODEX_HOME; authentication is outside workspaces/evidence and
removed. Apps/plugins/memories/delegation are disabled. No retries, replacements or fallback models.

Primary acceptance requires completion within budget, preserved fixed files/IDs and all final native
scenarios passing. Final candidate correctness is reported separately: a correct candidate at timeout still
fails primary. Final grading runs outside the agent window. One source-free availability check is excluded
from scores but included in known cost. Three consecutive infrastructure failures stop the sequence; all
unattempted assignments remain listed. Unknown terminal usage stays unknown. Match costs by task AND
repetition, use only complete pairs for token ratios, and retain failures in aggregate cost accounting.

## What makes these tasks different

| Task | Required application change | Actual execution and independent checks |
| --- | --- | --- |
| Catalog | Change controller and ID policy: active/available filtering, canonical positive IDs, exact routing, JSON values/escaping | Compile9 sources into a native loopback TCP server; inspect real HTTP status, headers, JSON and UTF-8 body length; HEAD gets a paired GET |
| Transfer | Change transfer service and key policy: persistent idempotency, replay/conflict, bound parameters and atomic ledger updates | Compile5 sources with SQLite; reuse a real file across requests; inspect both tables after EACH call; probe transaction cleanup before close; exercise real constraint/decode errors |

Four editable files per task are logic/policy source and both adjacent sidecars. Stable IDs and existing
declarations must be preserved; app/model/lib/client/bundle/references are fixed. Catalog has30 scenarios
(5 public,25 private); transfer20 (5 public,15 private), some with multiple operations. Public scenarios
are included in final grading. These are50 distinct scenarios reused across repetitions and arms, not300
independent development projects. Fixed transport and transaction-wrapper scaffolds make the changes
feasible in the existing budget; this is not from-scratch service development.

Catalog tests include escaped Unicode/control content, numeric JSON fields, disabled/out-of-stock entries,
leading zeros, overflow, signs/query/slashes/percent tokens, wrong methods and HEAD wire semantics.
Transfer tests include duplicate/reused keys, conflict, validation priority, missing accounts/funds,24/25
Unicode scalars, quoted/NUL/case-sensitive keys, successive transfers, rollback on a real engine CHECK
failure and strict typed-row decode failure. The oracle is separate Python state-transition logic and a
route specification, not the candidate or reference implementation. SQLite tables are inspected through
the host's sqlite3 reader; production execution uses the frozen native SQLite adapter.

## Calibration and measurement limits

The final [calibration](../benchmarks/results/2026-11-26-realistic-validation.json) compiles seven programs:
references pass30/30 and20/20; starters pass22/30 and7/20; canonical-ID mutant28/30, missing-ledger
mutant9/20, skipped-rollback mutant17/20. All three deliberate defects compile and fail behavioral tests.
That is170 scenario evaluations, with additional HTTP requests/SQLite calls inside some scenarios.

Development calibration failures are preserved: first an improperly escaped fixture string, then a wrong
HTTP error-helper arity, then uppercase Json used instead of the lowercase json literal. After these
syntax/API corrections, SQLite returned Limit because the harness granted two handles. Capability +
database + statement need three; the corrected reference passes. None of these failed calibrations is a
participant result. Identity-authoring helper probes succeed on both initial bundles before calls. Ten
new harness/reporter tests include preserved failing-first logs, matching/unknown accounting, both sidecar
integrity, public/private counts and independent route/state checks. Existing harness/research tests run
as well. Raw calibration attempts and diagnostics remain alongside the final calibration.

The initial SQLite preflight took25.77 seconds; final calibration builds took approximately22–24 seconds.
HTTP builds take approximately2.2–2.4 seconds. Compilation is inside the120-second agent window with a
35-second build ceiling and40-second mailbox response ceiling; there is no precompiled cache. Build time
is reported separately from the agent window, and repeated builds consume real task budget. This fixed
budget can constrain testing of SQLite more than HTTP. No task/backend/budget change follows outcomes.
The shared16-core host had load5.45/7.55/7.91 before registration and other work; timing is descriptive.
Do not turn a task completion ratio into a causal model-speed or runtime-performance claim.

The pinned compiler SHA256 is `8d9f84530b8d34a6a72597d25befc249a604430a6569656b4184721e1e444d9d`.
The previous312-pass compiler receipt is reused by hash; this is not a new full compiler/editor suite.
Common reference transitive links remain incomplete, although direct v2 links are delivered. Same model
identifier/settings do not prove immutable weights, provider conditions or hidden host instructions.
Six matched pairs across two authored changes are exploratory, selected after prior failure review,
not powered/blinded evidence or production coverage for security, concurrency, TLS or deployment.

## Observed results

All12 registered assignments ran; none were replaced, retried or left unattempted.
[Machine-readable results](../benchmarks/results/2026-11-26-realistic-study.json) retain every outcome.

| Outcome | Control | v2 |
| --- | --- | --- |
| HTTP timely completion | 3/3 | 3/3 |
| SQLite timely completion | 1/3 | 1/3 |
| Overall primary acceptance | 4/6 | 4/6 |
| Final candidate correctness | 6/6 | 6/6 |
| Final native scenarios passing | 150/150 | 150/150 |
| Timeout / unknown usage records | 2 / 2 | 2 / 2 |
| Visible compiler errors | 1 | 1 |
| Direct authoring command diagnostics | 0 | 1 |

There is no observed completion or final-correctness advantage for v2 in this sample. One SQLite pair
favors control, another favors v2, and the third times out in both arms. Equality here does not establish
statistical equivalence or general noninferiority. All300 repeated scenario evaluations pass, including
the four timeout candidates. The post-registration strict JSON-type sensitivity audit changes zero
outcomes; it checks exact nested types in addition to Python equality, which can equate booleans and
integers. It does not rewrite registered scores.

| Index | Task / repetition | Arm | Status | Agent window seconds | Reported input + output tokens |
| --- | --- | --- | --- | --- | --- |
| 0 | HTTP /0 | control | complete | 105.62 | 220,764 |
| 1 | HTTP /0 | v2 | complete | 90.48 | 229,698 |
| 2 | SQLite /0 | v2 | timeout | 120.10 | unknown |
| 3 | SQLite /0 | control | complete | 114.25 | 131,341 |
| 4 | HTTP /1 | v2 | complete | 70.02 | 145,179 |
| 5 | HTTP /1 | control | complete | 89.17 | 170,289 |
| 6 | SQLite /1 | control | timeout | 120.05 | unknown |
| 7 | SQLite /1 | v2 | complete | 112.00 | 206,744 |
| 8 | HTTP /2 | control | complete | 76.88 | 145,561 |
| 9 | HTTP /2 | v2 | complete | 108.40 | 247,639 |
| 10 | SQLite /2 | v2 | timeout | 120.08 | unknown |
| 11 | SQLite /2 | control | timeout | 120.11 | unknown |

Only three token pairs are complete, all HTTP. Their v2/control ratios are1.0405,0.8525 and1.7013:
median1.0405, approximately4.0% more reported tokens, with large mixed variation. There are ZERO complete
SQLite token pairs; neither an overall paired cost ranking nor a SQLite token comparison is supported.
Known scored subtotal is1,497,215 across8/12 records, not total experiment cost. Control's known subtotal
is667,955 across4/6 and v2's829,260 across4/6; these omit different failed SQLite assignments and must
not be treated as comparable complete-arm totals. Including availability gives1,508,407 across9/13 known
records, with four unknowns. Cached input and reasoning output are subsets, not added again. Billing,
provider-internal requests, parent development tokens and immutable model weights remain unknown.

Median agent window is109.94s control and110.20s v2. These include censored timeouts, startup, retrieval,
reasoning, editing, compilation and tool turnaround. They are not inference-speed or runtime benchmarks.
Successful visible native builds have median1.98s HTTP (range1.89–2.53,n12) and27.82s SQLite
(range21.51–33.75,n8). These counts include public builds only, not final private grading or failed
compiles. Shared host load and different edit/check sequences prevent a causal timing attribution.

## What the process evidence shows

The [process audit](../benchmarks/results/2026-11-26-realistic-process-audit.json) retains commands,
compiler diagnostics, check timings and first passing edited candidates. All six v2 sessions refer to
the skill path in commands; no control does. No command refers to either external skill home. This is
observed trace evidence, not a proof of complete instruction consumption or OS-enforced read isolation.
Initial/prompt hashes, terminal usage, native case labels, IDs and acceptance formulas were checked for
all12 assignments against the preregistration. Fixed inputs and all earlier frozen evidence stay intact.

Both arms produce a visible `compile.action.undeclared` diagnostic for omitted `db.close` in the
SQLite service. One further v2 authoring command encounters the same diagnostic before visible checking.
The query helper finalizes statements and therefore requires that declared action even if the service
body has no explicit close. The candidates repair it without changing language or library behavior.
There are no persistent final behavior failures in either arm. More failing visible checks in v2
(4 versus1) include checks of the unchanged incomplete starter; they cannot be interpreted as four
introduced implementation defects. Each arm has one visible compile error and all failures are retained.

All four timed-out sessions had already received a passing public check: indices2,6,10,11 at112.47,
103.76,103.71,85.76 seconds respectively. They do not emit terminal completion before the fixed deadline.
This is a session-completion failure under the preregistered primary measure, not evidence of incorrect
final application code. Further edits/checks occurred after some passing observations; successive source
snapshots are different, so these are not simply identical-source reruns. Index11's second queued public
check finishes at143.34 seconds, after agent termination. The unchanged controller drains pending work
before private grading; the agent receives no extra usable window, and this late pass does not rescue
primary acceptance. Retain this controller cost separately from the120-second agent window. Native final
grading is also outside that window for every arm.

These observations locate a practical constraint: full SQLite rebuilds and completion/tool turnaround
leave little margin under120 seconds. They do not isolate compiler cost, provider latency, prompting or
model behavior as the sole cause. Successful sessions also spend time after a public pass. A passing
public check does not prove all private requirements, so further edits are not automatically mistakes.

## Interpretation and proposed follow-up

The frozen language plus shared documentation/native feedback supported correct multi-module HTTP/JSON
and transactional SQLite candidates in all12 attempts. The new skill does not improve measured completion
or final correctness on these two changes, and no general token benefit is demonstrated. The earlier
small grammar-task result does not transfer into a demonstrated realistic-task advantage. Different task
instances/dates forbid a direct cross-study absolute-score comparison.

Keep v2 optional and frozen as tested. Proposed next work, requiring a separately selected experiment:

- Test a concrete library-action reminder: preserve transitive action bounds, including statement cleanup,
  rather than narrowing an action list by scanning only explicit calls in the current function.
- Isolate an explicit budget/finish policy from grammar/reference changes, and measure time after the last
  useful check. Do not assume a reminder alone can remove provider/tool turnaround.
- Measure a separately registered reusable-build workflow or longer fixed application budget in BOTH arms.
  Keep compiler/runtime changes separate; do not silently grant cached builds or extend these attempts.
- Add independent application task instances before making a general productivity claim; retain acceptance,
  candidate correctness, cost coverage and comprehension as distinct outcomes.

No proposal was implemented and no extra participant calls were made. The present study stops at12.

## Evidence and verification

The [snapshot](snapshots/2026-11-26-realistic-study/README.md) and per-trial JSON under
`benchmarks/fixtures/realistic-study-20261126/` preserve candidates, prompts, final native observations,
source hashes, process detail and unknowns. The local raw archive retains original delivered filenames,
complete CLI streams, intermediate edits, failed calibrations, native binaries and database files; its
manifest is published. It is local evidence, not an off-device backup. No authentication file is included.

Final verification uses69 harness tests (10 new),15 research tests, native calibration, byte/hash/ID and
observation audits, plus repository JSON/JSONL/Markdown checks. The previous312-pass compiler receipt is
reused, not rerun. Paper local-047 appends this result while preserving all46 earlier entry objects and168
pinned files. Lobo, libraries, extension, skills and prior studies remain unchanged.

The closed raw archive contains3454 verified members,18,742,138 bytes, SHA256
`84c66952a75ac3d4153dfbb0ea453dbe0cddc798b3ec043257971044db3d46a0`. Its [manifest](snapshots/2026-11-26-realistic-study/evidence-manifest.json) and
[verification receipt](../benchmarks/results/2026-11-26-realistic-verification.json) record coverage and limits.
Final native grading used216 actual HTTP connections (including startup probes/HEAD GET comparisons)
and204 SQLite request invocations, nested within the300 scenario evaluations.
