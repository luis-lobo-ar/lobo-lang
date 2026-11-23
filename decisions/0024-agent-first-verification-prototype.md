# 0024: Prototype agent-first verification as a bounded foreground session

- Status: executable local prototype; local verification and scripted comparison passed
- Date: 2026-10-18
- Risk: standard, with adversarial checks around evidence reuse and acceptance integrity.

The user authorized prototyping the agent-facing testing interaction, not a daemon, ecosystem integration,
new language syntax, production deployment or a claim of faster agent repairs. Preserve native execution.

## Contract and alternatives

Success: pin explicit requirements/checks, verify a candidate, reuse only eligible in-session passing evidence,
report compact structured failures and pending/uncovered checks, and replay an exact retained failure. Compare
affected checking against forced-full checking on seeded edits. Count executions, response bytes and elapsed
time; real agent tokens/time-to-correct-repair still require a controlled agent study.

A JSON wrapper over Cargo alone does not offer application-level dependency selection or immutable checks.
A daemon/content-addressed evidence database adds authority/lifecycle before this interaction is validated.
A persistent foreground Rust CLI session keeps computed evidence in memory rather than trusting editable
cache receipts. Choose this local alternative analysis, not independent prototypes. No runtime dependency
or MCP server is needed. A future MCP adapter may wrap the same actions if a client actually needs it.

## Prototype surface

`lobo-test --session --clang /absolute/path/to/clang` reads versioned JSON lines and emits one JSON response
per request. Actions: describe, open, verify, explain, replay, close. EOF/close cleans only session-owned
temporary files. Open fixes requirement IDs, descriptions, test IDs, entry IDs, typed CLI arguments, expected
typed values/runtime faults, and fuel/heap limits. No test/expectation replacement action is accepted.
Starting a different session is not the same approval identity; this is not an OS boundary against a hostile
same-user process. Expected values must come from requirements/independent oracles, not candidate outputs.

Verify names an explicit base snapshot and supplies a complete in-memory source set plus environment identity.
Check the entire candidate first. Invalid source/entries/arguments never inherit a passing report. Canonical
checked call-closure bodies/signatures/contracts/actions, global nominal layouts, test plan and resource
budgets determine reuse. Include all static calls, including unexecuted branches and contract calls; follow
cycles with a visited set. Unknown graph shapes, nonempty actions, or changed environment disable reuse.
Environment identity is a host-supplied immutable configuration assertion, not automatic detection of all
toolchain/kernel/library changes. Native compilation is lazy if all checks can be reused. Full mode always
executes every check, providing a distinct final integration gate.

Store exact canonical data rather than relying on an unreviewed custom hash. Snapshot/report IDs are opaque
session-local handles, not cryptographic signatures or persisted content addresses. Explain/replay resolves
only retained handles and can return exact source/check/outcome details within bounded input/output sizes.
No on-disk cache receipt, caller claim of success or arbitrary shell command is imported as evidence.

## Bounds and failures

Bound request bytes, sources, requirements, cases, retained snapshots, arguments, process output, compile/run
deadlines and per-request executed cases. Pending means not established, not skipped/pass. Value mismatch,
unexpected runtime fault, resource exhaustion, infrastructure failure and invalid request stay distinct.
Uncovered requirements prevent acceptance. Reports identify reused versus freshly executed checks and
state that passing finite checks is not proof of arbitrary correctness. Structured differences are bounded;
larger exact detail is requested explicitly. Native subprocesses execute only emitted code, never supplied
paths/scripts. Direct child termination is not an OS sandbox or a descendant/process-memory containment claim.

## Validation

TDD first: absent CLI, then frozen checks, transitive/contract/recursive dependency changes, unrelated edits,
type/environment/action invalidation, compile/argument errors, expected faults versus exhaustion, pending and
uncovered requirements, exact replay, version/unknown-field rejection, oversized IO and process failures.
Use native/reference/sanitized fixtures for ordinary Lobo check functions. Existing language/editor suites
remain required; no language syntax or extension runtime changes are planned. Measure seeded edits with full
and affected modes; preserve missed-regression evidence, not just execution savings.

Automatic property generation/shrinking, durable evidence, remote sessions, protected external acceptance
services, semantic workspace transactions and agent-level efficiency claims are not established by this prototype.

## Implementation and observed failing tests

The Rust `lobo-test` binary implements this contract independently from Core syntax and LTP. It shares the
existing compiler/native lowering, not its reference evaluator for test execution. The first TDD run failed
because the binary target did not exist; the next failed because open/verify were unsupported. Native
change/reuse/failure/explain/replay then passed before adversarial tests were expanded. An explicit
`default-run = "lobo"` preserves existing `cargo run -- ...` commands; the second binary initially made
those commands ambiguous. No package/runtime dependency was added and Core/compiler/artifact stay 0.9.

TDD findings fixed without changing existing numerical/workload oracles:

- Candidate compilation errors initially appeared as generic invalid requests. They now have structured
  compiler diagnostics and no new snapshot, separate from invalid invocations or infrastructure failures.
- Missing Clang initially counted two cases as executed. Infrastructure now stops the run, counts no
  completed native observation and leaves remaining cases pending; full-execution status cannot be forged.
- Scalar argument parsing accepts typed atoms but is not call-signature validation. The runner now checks
  entry arity and all scalar/compound nominal types before accepting a plan/candidate.
- Temporary directories initially inherited group/world access from umask. Private creation modes now
  restrict directories/source/capture files; a regression checks 0700/0600. No broad recursive cleanup is used.
- Serde's internally tagged unit actions did not reject unexpected fields as intended. Zero-field struct
  variants now enforce the strict contract, including describe/close. Malformed and oversized frames are tested.
- A large unexpected field name produced a verbose protocol error. Its message now has a 1,024-scalar preview
  and explicit truncation flag, preventing that trivial feedback flood.
- An observation-budget boundary originally retained the crossing outcome then stopped silently. It now
  records bounded `tool.observation_limit` metadata and leaves later cases pending rather than hiding the boundary.

Only exact in-process canonical structures/results authorize reuse. No caller-supplied receipt is accepted.
Session identities distinguish separately pinned plans; they are not signatures or external approval. The
environment assertion and trusted-compiler assumptions remain load-bearing, documented limits, not solved
by these tests. Plain functions in `examples/testing/checks.lobo` express reverse-involution and duplicate-
tree-count laws; reference/reloaded-artifact/optimized/sanitized checks independently exercise their outcomes.

## Local verification

Linux x86_64; Cargo 1.92.0, Clang 18.1.3, Node 22.19.0:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 145 tests pass per profile
  (142 integration tests, one library unit and two runner process/privacy units). Fourteen new integration
  tests exercise the session contract, real native reuse, transitive/contract-only/recursive/imported calls,
  layout/environment/action invalidation, fixed criteria, invalid candidates/arguments, expected runtime
  faults versus exhaustion, missing compiler, pending/uncovered checks, replay, retention and input limits.
- Process units verify direct-child timeout, output-limit termination, nonzero exit, exact output cleanup,
  and private temporary modes. Full language native/reference/ASan/UBSan/leak/instrumentation gates remain.
- Release builds for both native binaries, all-target Clippy with warnings denied, formatting, JSON/JSONL,
  local links and whitespace checks pass. The `lobo` release hash remains unchanged.
- 104 Node editor tests pass; the additional fixture navigates ordinary law functions' imported helpers/
  inferred tree-summary fields. Real VS Code host checks emit the completed marker and exit 0 with no
  incidental SIGPIPE in this run. No syntax/extension runtime changed, so extension 0.11.0/VSIX remains valid;
  no normal-profile installation or publication was performed.
- The executable JSONL demo passes its initial checks, introduces a real `42` versus `43` increment mismatch,
  reuses the unrelated constant pass, and reproduces the same failure from the retained snapshot.

## Scripted differential experiment

The [recorded report](../benchmarks/results/2026-10-18-agent-testing-prototype.json) is one local run per
scenario after a passing baseline, with independent expected integer results. Both strategies use the same
runner/compiler. Every one of 64 retained case statuses and complete observations is compared—not only the
eight visible failure capsules. All 384 affected/full case pairs agree in these six scenarios.

| Edit | Fresh affected cases | Reused | Fresh full cases | Affected / full ms |
| --- | ---: | ---: | ---: | ---: |
| Comments only | 0 | 64 | 64 | 2 / 286 |
| Function unreachable from checks | 0 | 64 | 64 | 2 / 297 |
| Leaf regression | 16 | 48 | 64 | 186 / 305 |
| Transitive regression | 32 | 32 | 64 | 218 / 288 |
| Contract-only regression | 16 | 48 | 64 | 174 / 283 |
| Global layout addition | 64 | 0 | 64 | 289 / 287 |

Zero differing outcomes is finite experimental evidence, not proof of selector soundness or missed-bug absence.
The layout fallback demonstrates that selection need not be faster. Timings include validation/build/execution,
exclude obtaining the already-passing baseline, are not hardware-isolated and have no repeated-run confidence
intervals. Both modes already use compact output; byte counts do not establish superiority over other frameworks.
No AI agent tokens, time-to-correct-repair or escaped defects were measured. That controlled study and automatic
property generation/counterexample shrinking remain next investigations, not implemented capabilities.

## Artifact evidence and remaining boundary

Local release artifacts, not a published release:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/lobo-test` | 993,336 | `dbd7506830098211f505c394f6be3a85b57db95bd194124c0047e5e2e2f489cf` |
| `target/release/lobo` | 1,078,512 | `8476c9530b58ec963fb02400ecaa4d8d835c5d880a4204c7526a9002a83d41b3` |

Input SHA-256: `src/agent_testing/mod.rs`
`359c598c64e7a6016de602a300376a5f3830fa47be9e9ff918af5b83c8b45a84`;
`src/agent_testing/process.rs` `b008879f8838f0a19ab8358588d1f26956a33c3a996c7f18565d11989fd0562f`;
`src/bin/lobo-test.rs` `b515be623037786da54bc49b3e7880b535d1d46a672379b007617d7d2c099904`;
`tests/agent_testing.rs` `b2d63e267e6265222a54854bb6946f6eefbd9ad6cbf4133fe3350658e50cdac7`;
comparison report `4fbe7f54531045dbf15175c13894f63579d06639c97755c72c9ff7d5689ab6a8`.
The report also records runner/source/case identities and host/tool metadata.

No commit, deployment, network/storage/search integration, FFI, OS sandbox, background daemon, MCP server,
durable cache, or external authorization service was introduced. This closes the requested first prototype,
not the larger agent-workspace design. Keep full checks and independent oracle review as acceptance gates.

## Research informing the contract

These sources motivate design choices, not claims about this prototype's measured agent productivity:

- [SWE-agent ACI research](https://arxiv.org/html/2405.15793v3): interface/feedback design matters; use concise
  actions and bounded observations, without claiming a direct framework comparison.
- [LLM-generated oracle study](https://arxiv.org/abs/2410.21136): generated assertions can mirror actual rather
  than intended behavior; pin independent criteria and distinguish checks from requirement coverage.
- [Observed coding-test bypasses](https://openai.com/index/chain-of-thought-monitoring/): do not trust mutable
  agent-authored receipts or treat changes to expectations as the same acceptance identity.
- [Infrastructure noise experiments](https://www.anthropic.com/engineering/infrastructure-noise): expose
  environment/resource assumptions and keep infrastructure outcomes distinct from assertion failures.
