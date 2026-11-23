# Native boundary state-machine verification proposal

Date: 2026-11-22. Status: recommendation, not implemented or model-checked. No language or runtime change.
The user asked whether state-machine models and TLA+ should follow native boundary 1.7. Recommend two small
models before expanding networking or adding concurrency. Existing deterministic C model tests remain useful;
they are sampled executions, not a completed TLC exploration. Earlier testing-workflow TLA+ ideas in
[decision 0026](../decisions/0026-repeated-library-repair-study.md) also remain unimplemented.

## Selected targets

1. Authority and resource lifecycle: root acquisition, subset-only delegation, allocation, resource kind,
   generation, alias invalidation, close/reuse, generation retirement and normal/fault cleanup. Link transitions
   to capability, lookup, begin, insert, release and cleanup functions in `src/native_api.c`. No delegated
   authority may exceed the parent. An old token must never access a reused slot. Invalid handles and denied
   grants must not reach OS calls. Quota/fuel exhaustion must preserve cleanup behavior.
2. Operation lifecycle: acquire descriptor, pending connection, peer validation, publish handle, short IO,
   interruption, deadline, shutdown, failure and cleanup. Track descriptors acquired before registry publication;
   collapsing acquisition/publication into one idealized step would conceal a failure window. Link transitions
   to connect, wait, accept, read, write and cleanup. Only a validated connected peer may produce a usable stream.
   Interrupted waits must not reset deadlines or spend more than available budgets.

The current runtime is synchronous. Model its real sequential steps and nondeterministic syscall results;
do not introduce fictional thread interleavings. For progress claims, explicitly assume modeled syscalls
return, account for fuel and state how time advances. A model that allows time to stand still cannot silently
assume every deadline expires. Distinguish language-fault cleanup, normal exit and kernel process teardown.

## Verification workflow and finish line

- Write the states, transitions and properties in plain language, then encode them in TLA+ and check with TLC.
- Start with explicitly small domains, for example 2–4 slots, tiny generation limits and budgets, to force reuse,
  retirement and exhaustion. These are candidate configurations, not checked bounds or cutoff proofs for 64 slots.
- Map every transition/atomic step to source functions and retain covered source hashes. Review that mapping
  whenever the provider changes. A correct abstract model alone does not establish implementation conformance.
- Exercise every intended transition. Inspect reachable/action counts so disabled transitions cannot make an
  invariant pass vacuously. Check that deliberately broken generation, authority and publication rules fail.
- Translate counterexamples and representative model traces into tests of the actual C provider. Confirm
  that corresponding implementation mutations are caught. Keep ASan/UBSan, ABI and backend tests.
- Pin the tool version; provide one repeatable command and structured agent-readable results: invariant ID,
  result, bounds, assumptions, state counts, source identity and counterexample steps. This wrapper is proposed.
- Preserve failed and corrected runs separately. Require completed exploration of each selected finite model;
  timeout/resource exhaustion is inconclusive. Record liveness assumptions rather than asserting unconditional
  eventual success. Complete these two models and their implementation checks before selecting another target.

TLA+ checks design behavior. It does not automatically check the C implementation's pointer validity, ABI
layout, undefined behavior or actual kernel semantics. No production-readiness, full security, whole-language
correctness, or agent productivity claim follows from a model pass. This proposal adds no Lobo syntax and
requests no broad language-feature expansion. No formal-verification result or state count exists yet.

## Primary sources consulted

Accessed 2026-11-22:

- [Official TLA+ Toolbox model overview](https://dl.tlapl.us/tlatoolbox/doc/model/overview-page.html): initial/next-state specifications, invariant/deadlock/temporal checks and explicit fairness.
- [Official TLA+ Toolbox results](https://dl.tlapl.us/tlatoolbox/doc/model/results-page.html): reachable-state/action statistics, accidentally disabled actions and preserving results before another run.
- [Lamport, Specifying Systems](https://lamport.azurewebsites.net/tla/book-02-03-04.pdf), chapter 14: finite models and TLC model checking. The chapter was located through primary-source search; no new formal experiment was executed.
