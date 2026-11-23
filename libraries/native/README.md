# Native OS helpers

Compiler/language 1.7 supplies a closed Linux native interface. This Lobo module adds `IoResult<T>` and
named `IoError` cases around its raw results. Supply [native.lobo](native.lobo) and [permissions.lobo](permissions.lobo) explicitly; no package lookup
or library download happens. [The contract](../../spec/native-interface-v0.1.md) defines all limits and errors.

Acquire the root capability directly in the executable entry, then pass it to helpers:

```lobo
fn clock()->IoResult<I64> actions {native.authorize,clock.read} {
  let cap=Native::capability();
  os.monotonic_ms(cap)
}
```

The complete [probe](../../examples/native/probe.lobo) includes imports and persistent identities.
From the repository root, choose a fresh output path:

```sh
mkdir -p target/native-samples
./target/release/lobo compile examples/native/probe.lobo --source libraries/native/native.lobo --source libraries/native/permissions.lobo -o target/native-samples/probe
./target/native-samples/probe --entry native.probe.clock
./target/native-samples/probe --entry native.probe.clock --native-clock true
./target/native-samples/probe --entry native.probe.random --arg I64:32 --native-random true
```

The first clock call returns `Err(Denied)`; the granted call returns `Ok(milliseconds)`. Grants belong to the
compiled executable's invocation, independently of source action labels. The compiler's `run` convenience
command does not forward grant flags; compile then invoke the executable. No Rust process hosts these calls.

Wrappers preserve the native signatures for `restrict_mask`, `monotonic_ms`, `random_bytes`, `listen`,
`connect`, `accept`, `port`, `read`, `write`, `shutdown_write`, `close_listener` and `close_stream`, replacing
raw result types with `IoResult<T>`. `from_error_code` and `to_error_code` preserve unknown codes. The enum
contains Denied, InvalidInput, InvalidHandle, WrongKind, Closed, TimedOut, WouldBlock, ConnectionRefused, AddressInUse,
OsFailure, LimitExceeded and Unknown(I64). Invocation faults such as fuel exhaustion remain faults.

Only native entry code can call `Native::capability()`; wrapping it in an ordinary helper is rejected at
execution. The raw ABI encodes clock=1, random=2, listen=4, connect=8. Derived capabilities cannot add rights.
Resource values are opaque, invocation-local and nonserializable; pass them through typed parameters.
Close invalidates aliases. All remaining descriptors close at invocation exit, including fault exit.

The [loopback example](../../examples/native/README.md) uses the raw catalog to demonstrate IO loops, EOF and
cleanup. This raw OS module supplies no HTTP, TLS or OAuth implementation. Native provider code is trusted process
code; grants restrict the approved provider, rather than sandboxing arbitrary native libraries.

## Typed permissions in 1.11.2

`Permission` names ClockRead, RandomRead, Listen and Connect. `Permissions` is a typed set.
Use `none()`, `one(ClockRead)` and `allow(requested, RandomRead)` to construct it.
Repeated permissions have no additional effect. Every combination, including the empty set, is valid.
The set describes a request. It never confers authority.

```lobo
use access { Permission, one, allow };
use os { IoResult, restrict_capability };
```

The snapshot calls `os.restrict_capability(root, allow(one(ClockRead), RandomRead))`.
The helper returns `IoResult<Native::Capability>`. `access.restrict(root, requested)` returns the native
result with an I64 error for applications that already use native error results.
`abi_mask` is the explicit ABI encoder. Conversion does not allocate native handles or consume native call quotas.
`restrict_mask` retains the old integer API and its stable ID for deliberate ABI callers.
Maintained consumers use typed helpers. Launch grants, opaque capabilities, subset checks and quotas remain unchanged.
