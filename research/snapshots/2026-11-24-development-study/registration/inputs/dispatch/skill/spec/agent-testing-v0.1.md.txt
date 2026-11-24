# Agent-first testing protocol v0.1

Status: executable local prototype, separate from Core 0.9 and proposed LTP 0.2. No new language syntax,
runtime authority, daemon, persisted evidence service, package resolver or MCP server is implemented.
Decision: [0024](../decisions/0024-agent-first-verification-prototype.md).

## Job and interface

Let an agent ask what checks a candidate invalidates, run those checks natively, obtain bounded failure
evidence, and replay a retained failure. Assertions may be ordinary Lobo functions returning Bool or richer
values; the host provides explicit inputs and independent expected outcomes. It does not implement the
program's computation or generate an oracle by copying its actual output.

Build `cargo build --locked --release --bin lobo-test`. Start:

```sh
./target/release/lobo-test --session --clang /usr/bin/clang
```

The configured Clang path is an absolute trusted executable, not a shell expression. Every request is one
UTF-8 JSON line with this strict envelope; unknown fields/actions/versions fail explicitly:

```json
{"format":"lobo-test/0.1","action":{"op":"describe"}}
```

Every response includes `format` and an opaque session identity. Snapshot handles `s1`, `s2`, ... resolve
only inside that live foreground process, not across sessions or from on-disk receipts. They are not hashes,
signatures or externally approved revisions. Process exit 0 means clean transport termination, **not that
tests passed**; clients must inspect each response. Flush occurs after every response. Clients should drain
stdout while writing or use a request/response loop. Close/EOF removes only session-owned temporary files.
No background process remains. On Unix, the private directory is 0700 and source/capture files are 0600;
Clang's executable lives inside that private directory.

`describe` supplies compact action contracts, versions, bounds and workflow. No compiler operation occurs
for describe. A separate client adapter can construct source envelopes from explicitly selected files;
the source contents need not travel through the LLM's context. This prototype itself only reads supplied
source strings, never source labels as paths. No adapter or file scan is silently performed.

## Open pins criteria

```json
{"format":"lobo-test/0.1","action":{
  "op":"open","root":"demo",
  "sources":[{"name":"demo","text":"module d @id(\"d\"); function inc @id(\"inc\")(n: I64) -> I64 { n + 1 }"}],
  "requirements":[{"id":"increment","description":"Increment by one"}],
  "tests":[{"id":"inc.41","requirement":"increment","entry":"inc","arguments":["I64:41"],
    "expect":{"kind":"value","value":{"type":"I64","value":"42"}},"fuel":1000,"heap_cells":1000}]
}}
```

Open checks the whole closed source bundle and each test's entry/arity/typed argument shape. Scalar parsing
alone is not type validation. Expected values use Lobo typed JSON and are normalized structurally, never
derived from execution. Expected runtime faults use:

```json
{"kind":"fault","code":"runtime.integer.zero_divisor","phase":"execute","subject":"app.divide"}
```

Phase may also be `precondition` or `postcondition`. Fault expectations must match all three fields; resource
exhaustion and infrastructure errors cannot be declared successes. No native tests run at open. Its first
successful response is `opened`, snapshot `s1`. A second open is rejected as `plan.frozen`; a rejected open
does not fix invalid criteria or consume a snapshot. Test/requirement IDs are unique nonempty ASCII IDs,
at most 128 bytes, using letters/digits/`_.:-`. Every test references a declared requirement.

The plan remains fixed until exit. There is no action to alter expectations, lower budgets or import caller-
supplied success evidence. A different process has a different session identity; it is not the same approval.
An external host must enforce who may approve a plan. This CLI is not an OS boundary against a hostile
same-user process that can edit/instrument binaries or start a different session.

## Verify explicit candidate and base

```json
{"format":"lobo-test/0.1","action":{
  "op":"verify","base":"s1","root":"demo",
  "sources":[{"name":"demo","text":"module d @id(\"d\"); function inc @id(\"inc\")(n: I64) -> I64 { n + 1 }"}],
  "environment":"pinned-local-job-v1","scope":"all","max_cases":256,"wall_ms":5000
}}
```

Validate the entire candidate before reusing evidence. Compilation/entry/argument failures return explicit
errors and create no snapshot. A successful candidate creates the next snapshot, even if tests fail or remain
pending. `scope:all` forces all selected checks to execute; `affected` can reuse eligible passing results.
There is no hidden moving base: each operation supplies a retained base handle. Checks not run because of
case/time/observation allowances remain pending, never silently skipped/passed.

Reuse requires the same frozen test, arguments/budgets, nonempty environment identity and exact equality of
the entry's canonical transitive function definitions (signatures, bodies, actions, contracts), global nominal
record/variant layouts, and Core/artifact version. Traverse every static call, including unreachable and
contract-only calls; cycles terminate by visited IDs. Any nonempty action set or unknown graph shape disables
reuse. Layout changes conservatively invalidate all checks. The complete program is still checked, including
unused declarations. A typo/invalid dependency therefore cannot inherit a green result.

Store/compare exact canonical structures instead of trusting a custom hash or caller receipts. Comment/source-
label changes can preserve semantic evidence. No native compilation is needed if all checks reuse eligible
results. Other candidates compile once lazily and run each check as a separate native invocation with its
declared fuel/heap allowances, without the interpreter. No shell, arbitrary program path or setup script is
accepted in a request. The Clang executable and generated C/native runtime are trusted tools.

`environment` is a **host assertion of immutable compiler/toolchain/runtime/OS/resource configuration**,
not proof that the CLI detected every environmental change. Change the identity whenever those conditions
change; an empty identity disables reuse. Unknown/uncontrolled environments should use empty identity or
full mode. The runner sanitizes child environment variables but does not pin the linker/system libraries,
CPU or kernel. Reuse presumes the compiler implements the advertised semantics; it is not a method for
proving absence of code-generation bugs triggered by unrelated changes. Retain forced-full integration gates.

## Results and drill-down

Verify returns `checks_satisfied`, `failed`, or `incomplete`, plus:

- Snapshot identity and elapsed time (includes validation, selection, build and execution).
- Counts: completed native outcomes (`executed`), reused passes, total passed/nonpassing/pending.
- At most eight compact failures, including check/requirement IDs, mismatch path, expected/actual previews,
  and replay handles. Runtime faults, resource exhaustion and infrastructure failures have distinct case statuses.
- At most eight pending check IDs, omitted counts and uncovered requirement IDs.
- `full_execution`: true only if full mode completed every native outcome, with no reuse or pending check.
  It does not mean success; inspect status as well.

Compiler launch/read/deadline/protocol failures are infrastructure, not code assertions. They stop further
execution and leave the rest pending; they are not counted as completed native outcomes. Observation retention
exhaustion keeps a bounded infrastructure diagnostic and leaves remaining checks pending. A requirement with
no declared check prevents `checks_satisfied`. Having a passing check is not proof of complete requirement
coverage; only the declared finite cases have evidence. Typed failures returned by application code remain
ordinary values and may be expected as such.

`explain` and `replay` take `{op, snapshot, test}`. Explain returns exact retained source bundle, frozen check,
environment identity, observation and originating snapshot (or null for pending). Replay reruns that retained
snapshot/check, bypassing result reuse, and reports whether status and observation match. It does not restore
historical OS state, automatically update evidence, or convert a pending check into a retained pass. A replay
of a previously infrastructure-failed invocation may now differ; that is evidence, not permission to hide it.

## Admission and lifecycle limits

- Requests: 4 MiB per JSON line including line ending; oversized frames return `request.limit` and end the
  session to avoid treating a trailing fragment as another request. JSON decoding uses Serde's nesting guard.
- Responses: 8 MiB hard serialized admission; verify summaries use bounded previews and detail counts.
- Sources: inherited 1..16 labels, 1 MiB aggregate UTF-8 and 8,192 tokens; no implicit IO.
- Plans: 1..64 requirements, descriptions 1..1,024 bytes; 1..256 tests; expected value at most 64 KiB each.
- Arguments: at most 64 atoms, each at most 32 KiB, combined at most 64 KiB; no NULs; shape/arity/type checked.
- Invocation fuel: 1..10,000,000; heap cells: 0..1,000,000. Existing input/result/runtime limits still apply.
- Retention: eight snapshots including open; no silent eviction. Start another separately approved session
  when full. Each snapshot retains at most 4 MiB native observations plus bounded limit/error metadata.
- Verify: 0..256 fresh completed cases, 1..30,000 ms request allowance; native invocation deadline is additionally
  at most one second. Replay has five seconds including a needed build. In-process checking/selection is
  frontend-bounded, not preemptible; wall allowance is observed between operations and while polling children.
- Child stdout/stderr: each limited to 1 MiB at capture/read; excess becomes infrastructure error. Capture
  uses temporary files, avoiding pipe-drain deadlocks; limits are polled, not hard disk/process-memory quotas.
  Direct children are killed/waited on deadline/excess; descendant containment and hard real-time are not promised.

Invalid source uses `compile_error` with structured compiler diagnostic/source coordinates. Invalid test
invocations use `invalid_candidate`; plan/protocol violations use `invalid_request`. No error certifies a
candidate. EOF, close and handled output failure drop exact owned artifacts; abrupt SIGKILL/host crash may
leave a private temporary directory. This is not a durable evidence or crash-recovery database.

## Reproduce and evaluate

```sh
cargo test --locked --test agent_testing --bin lobo-test
./target/release/lobo-test --session --clang /usr/bin/clang < examples/testing/agent-session.jsonl
python3 benchmarks/agent-testing.py
```

The executable session intentionally introduces an increment regression: one check fails while the unrelated
constant check reuses its passing observation. Explain/replay retains the exact failing source and input.
`examples/testing/checks.lobo` expresses reverse involution and duplicate-tree-count laws in ordinary Lobo;
tests supply the tree/shared-sequence sources explicitly. No host tree algorithm supplies their answers.

The scripted comparison uses 64 checks across six seeded edit scenarios and compares every affected-mode
case status/observation with forced-full execution, not just the first displayed failures. It records executions,
elapsed time, response bytes, source/case/runner SHA-256 and tool metadata. This measures incremental-selection
behavior, not agent tokens, time-to-correct-repair, mutation coverage, or general soundness. Both strategies
share the runner/compiler; independent expected values guard the initial baseline. No test shrinking, random
property generation, durable approvals, changed-file adapter, or agent-level efficiency study is implemented.
