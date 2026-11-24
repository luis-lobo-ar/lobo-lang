# Native interface v0.1

Implemented by compiler/language 1.7, native ABI 1. This is a closed synchronous OS slice for 64-bit Linux.
The native program links its provider; it needs neither Rust nor an interpreter at execution time. The compiler
is still written in Rust. The [C11 header](../src/native_api.h) pins layout/prototypes and invocation lifetimes.
It is a contract for this provider, not a general plugin SDK. Arbitrary library/symbol names are rejected.

## Types, catalog and actions

Resource kinds are Capability, Listener and Stream, spelled `Native::Kind`. They are opaque generation-checked
handles. Raw results are variants `Ok(payload)` / `Err(I64)`: CapabilityResult, ListenerResult, StreamResult,
I64Result, BytesResult, BoolResult. The [Lobo library](../libraries/native/README.md) translates errors into
`IoResult<T>` with named IoError cases. All signatures below use Native resource/result types.

| Operation | Parameters | Result | Required action |
| --- | --- | --- | --- |
| abi_version | none | I64 | none |
| roundtrip_i64 | I64 | I64 | none |
| roundtrip_bool | Bool | Bool | none |
| copy_bytes | Bytes | Bytes | none |
| capability | none | Capability | native.authorize |
| restrict_capability | Capability, I64 mask | CapabilityResult | native.authorize |
| monotonic_ms | Capability | I64Result | clock.read |
| random_bytes | Capability, I64 count | BytesResult | random.read |
| listen | Capability, I64 port | ListenerResult | net.listen |
| connect | Capability, I64 port, I64 timeout_ms | StreamResult | net.connect |
| accept | Listener, I64 timeout_ms | StreamResult | net.accept |
| port | Listener | I64Result | net.inspect |
| read | Stream, I64 count, I64 timeout_ms | BytesResult | net.read |
| write | Stream, Bytes, I64 timeout_ms | I64Result | net.write |
| shutdown_write | Stream | BoolResult | net.write |
| close_listener | Listener | BoolResult | net.close |
| close_stream | Stream | BoolResult | net.close |

Bindings are `lobo.native.OPERATION`, ABI 1; symbols are `lobo_native_OPERATION_v1`. Each native-enabled
program includes the exact six synthetic result layouts and one `lobo.native.runtime` scope, counted under
existing program declaration limits. Artifact loading rechecks the catalog, marker, types, arities, actions,
closed layouts and visibility before code generation. Source cannot select a different provider by string.

## Authority and resource state

The executable entry acquires its root with `Native::capability()`. The runtime allows this only at call
depth one, including repeated access to the cached root. Helpers receive the handle explicitly. Action labels
are checked upper bounds, not authority. The root holds only explicit launch grants; the default has no rights.
Derived capabilities can only remove parent rights. Mask bits are clock=1, random=2, listen=4, connect=8;
0 is valid, negative/out-of-range masks and amplification fail. Grants elsewhere in the process do not widen
a derived capability. Delegated socket handles carry read/write/shutdown/close authority; read-only streams
and revocation are not implemented. Capability handles remain live until invocation exit.

A fixed registry has at most 64 live handles, including capabilities. Tokens combine slot and generation.
Wrong kind, closed, forged and stale tokens fail before OS use. Close invalidates aliases; reuse increments
the generation and generation exhaustion retires the slot. Aliasing is permitted; there is no borrow checker
or automatic local drop. Compiler 1.7.1 explicitly owns connect descriptors before handle publication. All live descriptors,
including a pending connection, close at normal or fault exit before arena cleanup. Cleanup
runs without remaining fuel or native-call budget and is idempotent. Close errors still consume the handle;
on Linux an interrupted close is not retried. This is one process, one synchronous invocation, with no threads.

## Launch policy and limits

The native executable accepts these options, each at most once. Unknown, malformed and duplicate options
fail with `invocation.cli.invalid`. Booleans must be `true`; decimal options reject leading zero spellings.

| Option | Allowed value/default |
| --- | --- |
| --native-clock | true; absent denies |
| --native-random | true; absent denies |
| --native-listen | 127.0.0.1:PORT; absent denies |
| --native-connect | 127.0.0.1:PORT or self; absent denies |
| --native-handles | 1..64; default 64 |
| --native-calls | 1..65536; default 1024 |
| --native-bytes | 0..1048576; default 1048576 |

Only exact IPv4 loopback is supported. Listen ports are 0..65535; 0 requests an ephemeral port. Connect ports
are 1..65535. A fixed grant authorizes the exact port. `self` permits connections to an active listener owned
by the same capability token. There is no wildcard, DNS or external address route. Descriptors are nonblocking
and close-on-exec; accepted streams receive those flags too. Interrupted connects must establish a matching
peer before publishing a handle. Kernel failure returns an error, without substituting another service.

IO/random counts and single IO buffers are bounded to 0..65536 bytes. The pure copy_bytes operation
uses the existing Bytes, fuel and heap bounds rather than the native IO byte quota. Timeout arguments are 0..2000 milliseconds; 0 polls
without blocking. Deadlines are absolute across interruption and spurious readiness. There is no aggregate
wall-clock deadline; the caller can impose an outer process timeout. Existing fuel/heap limits also apply.
Resource-operation call and IO byte budgets are invocation-wide. A resource operation with a valid handle
consumes one call before permission/range checks; root retrieval and pure ABI operations do not consume that quota. Byte quota reserves the requested IO/random length,
including short IO and timeout, rather than charging only bytes transferred. Work is charged before IO.
Budgets are conservative and operations are not transactions: some bytes can be transferred before an error.

Read/write can be short. Callers loop explicitly. A successful read with positive requested count and empty
bytes means EOF; read(0) is an empty success without establishing EOF. Write returns the number transferred.
Write-side shutdown permits draining previously written bytes to EOF. Clock is monotonic milliseconds, not
calendar time. Random uses Linux getrandom with nonblocking entropy readiness, retries EINTR under budget,
and returns WouldBlock when unavailable. There is no fallback PRNG. Buffers returned by native code are
owned by the invocation arena; input pointers are borrowed only for the call. copy_bytes makes an independent
copy and rejects a nonempty null buffer. Resource handles cannot cross process invocations.

## Errors and limits of the contract

Raw errors: 1 Denied, 2 InvalidInput, 3 InvalidHandle, 4 WrongKind, 5 Closed, 6 Timeout, 7 WouldBlock,
8 Refused, 9 AddressInUse, 10 OsFailure, 11 Limit. Library unknown codes retain their I64 value. Invalid CLI,
fuel/heap exhaustion, root acquisition outside entry and unsupported execution/platform are invocation faults,
not IoError variants. System errno details are not exposed as a portable API.

Both C and LLVM use the same C provider. Backend agreement and ASan/UBSan do not prove provider correctness,
absence of races in future concurrent versions or security against malicious native code. This milestone has
no arbitrary FFI, filesystem, TLS, HTTP parser/server, public networking, OAuth, static asset embedding or
async scheduling. Those need separate specifications and acceptance gates. The earlier Rust HTTP host remains
a working historical sample, not a requirement for the native boundary.

## Windows snapshot distribution profile (compiler 1.7.2)

The [standalone Windows x64 snapshot](../examples/native/windows/README.md) cross-links the same checked
application with a restricted clock/random/capability provider. It preserves ABI layouts and fail-closed grants
and quotas. Network operations are unavailable and fail with invocation.native.operation_unsupported; network
grant flags are rejected. No complete Windows compiler or network provider is claimed. The Linux host compiler
can export C for this profile; its ordinary compile/run commands remain host-native. The Linux TLA+ claims do
not extend to this Windows provider. Wine tests and actual Windows coverage are recorded separately.

## Explicit Windows TCP application profile

Compiler 1.7.3 C export adds a separate provider selected with `LOBO_WINDOWS_TCP` during cross-compilation.
It implements the closed ABI with Winsock, GetTickCount64 and BCryptGenRandom. The default Windows snapshot
profile remains clock/random only. This is an application cross-build profile, not a Windows compiler host port.

Rights, exact loopback grants, conservative call/byte reservations, generation handles, and invocation cleanup
follow the existing ABI contract. Sockets use SOCKET/INVALID_SOCKET and closesocket. Winsock starts lazily after
permission checks. Listener bind requests exclusive address ownership. IO is nonblocking with monotonic wait
deadlines. A failed explicit closesocket retains the registry entry for retry and exit cleanup. Pending connect
ownership is recorded before waits that can exhaust fuel. Cleanup closes pending and registered sockets and
calls WSACleanup. Existing Linux TLA+ claims do not establish correctness of these Windows operations.

The [native REST sample](../examples/native/rest/README.md) uses a pure Lobo controller and bounded Lobo HTTP
transport. Its Wine checks and user-reported Windows snapshot success remain distinct evidence categories.


Core 1.9 optionally composes the separate [SQLite extension 1](sqlite-interface-v0.1.md). Its three opaque
resources, five extra result shapes and 22 calls are admitted only with the SQLite marker. The older
native ABI version and non-SQLite layouts remain unchanged. This document remains the original OS catalog.
