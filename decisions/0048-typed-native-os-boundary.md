# 0048 Add a typed native and OS boundary

Date 2026-11-22. Status implemented as the bounded compiler/language 1.7 milestone. Risk mode high assurance.

The user authorized implementing the native-boundary proposal with extensive failing-first tests and
maximum useful reuse of upstream tests. The motivating application is Google authentication and an Angular
frontend served by one native Lobo application. Existing research inputs and language 1.0 release remain frozen.

## Alternatives and selected direction

A separate Rust host is available today but does not meet the selected single-Lobo-application direction.
A general arbitrary-symbol FFI would expose pointer, ownership and ABI hazards before resource authority is
specified. A closed operation registry can pin signatures, ABI versions and actions while allowing the
compiler to lower approved operations to native symbols. Select that registry for the first implementation.
Expand the catalog through reviewed, tested bindings rather than allowing a source string to call any symbol.
No interpreter or subprocess fallback implements native calls.

Use a dedicated NativeOp checked expression and the existing compiler/backend pipelines. Do not overload
buffer-operation nodes with OS behavior. Inspectable binding IDs and ABI versions are validated on artifact
admission as well as source checking. Older language tags cannot admit new native operations. Start with
scalar/buffer ABI conformance, then opaque resources, runtime grants, bounded Linux loopback TCP, clock and
secure randomness. Keep OS types unavailable for external argument/literal construction. Native code remains
trusted process code; typed wrappers and grants do not sandbox malicious native implementations.

Native resource failures need an explicit result contract before resource operations ship. A source action
label is necessary for behavioral checking but cannot grant authority. Runtime policy defaults to deny,
resource handles need kind/generation/lifetime checks, and all live resources need cleanup on normal/fault exit.
The initial OS target is Linux; do not infer portability from Clang support alone. Concurrency and network
scope must remain explicit. Missing tools and unsupported targets fail instead of skipping or falling back.

## Evidence and acceptance

Write ABI regressions before implementation. Borrow relevant behavior/data from pinned Node, CPython, libuv,
Go and Rust test sources, retaining their license notices, exact commits and hashes. Do not describe adapted
cases as passing those upstream suites. Document semantic differences, excluded cases and why. Cases based
on JavaScript Number truncation must not weaken Lobo's exact I64 behavior.

Required coverage includes strict ABI/catalog admission, scalar extremes, buffer ownership and bounds,
source/checked-artifact type/action checks, C/LLVM parity, ASan/UBSan, default-deny grants, stale/wrong handles,
close/reuse, short IO, EOF, refused connections, timeouts, resource exhaustion and fault cleanup. Add
adversarial/model-based checks and a standalone native ABI harness where applicable. Keep editor navigation,
diagnostics, skill, current specifications, release tools and ledger synchronized. Preserve all failed attempts.
Do not call the boundary complete, secure, production-ready or formally proved based on test counts alone.

The historical [review](../research/native-interop-2026-11-22.md) remains an unimplemented proposal as recorded
at that checkpoint. Current implementation status and outcomes belong in a new result record.

## Implemented contract and outcome

ABI 1 has 17 approved operations, three opaque resource kinds and six fixed raw result types.
`program.native_abi: 1` explicitly activates the catalog; native-looking IDs in earlier programs remain user
identities. Both C and LLVM link the C provider. The Lobo source library supplies generic IoResult and named
errors. Root authority is acquired directly at entry, attenuated by a subset mask and passed to helpers.
Launch grants default to deny. Registry generations protect reuse; normal/fault exit closes descriptors.

Tests exposed interruption/deadline, root-acquisition, artifact admission, legacy namespace, malformed buffer
and interrupted-connect defects, each now guarded by regression tests. Borrowed cases retain exact versions,
licenses and mapping to Lobo semantics. The [result record](../benchmarks/results/2026-11-22-native-boundary.md)
retains all attempts and limits. This is finite engineering validation, not a security proof.

Stop at synchronous 64-bit Linux loopback, clock and randomness. General FFI, TLS, HTTP serving, OAuth,
frontend embedding and concurrency require separately chosen contracts. The earlier Rust HTTP sample remains
compatible but is not used as the implementation of this native boundary.
