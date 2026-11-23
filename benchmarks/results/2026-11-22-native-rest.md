# Standalone native REST API, 2026-11-22

Compiler and extension 1.7.3 retain language core/1.7 and ABI 1. The
[application](../../examples/native/rest/app.lobo), [controller](../../examples/native/rest/controller.lobo),
[HTTP parser and wire serializer](../../libraries/http-server/server.lobo) and
[transport](../../libraries/http-server/transport.lobo) compile into one executable. JSON generation uses the
existing Lobo library. No Rust HTTP host, interpreter or separate application service runs the API.
The compiler itself remains Rust on the Linux build host. The executable links trusted C runtime/OS support.

The delivered Windows x64 executable is 151,040 bytes. It imports Windows Kernel32, Winsock and UCRT system
components. The build rejects unexpected DLL dependencies. Source sidecars, build hashes, LLVM and MinGW
notices accompany the ZIP. The release is unsigned and local. See the [usage guide](../../examples/native/rest/README.md),
[decision](../../decisions/0051-standalone-native-rest.md) and [structured receipt](2026-11-22-native-rest.json).

## Application contract

The launcher supplies explicit clock and exact 127.0.0.1:8080 listen grants. Other CLI arguments retain
default-deny behavior. GET and HEAD expose /health, /api/users and /api/users/1 or /api/users/2. Responses
are ordinary JSON, without the command-line typed envelope. Unknown paths return 404 and unsupported
body-free methods return 405 with Allow. The controller is a pure Fn(Request)->Response callback.

This is a restricted HTTP/1.1 read-only local demo. It requires one nonempty Host, accepts ASCII headers and
origin-form targets, and rejects ambiguous framing, folded lines, duplicate lengths/hosts, transfer coding,
Expect and request bodies. It does not claim complete RFC 9112 support. Connections close after one response.
Content-Length counts response bytes; parsed HEAD requests receive the same length with no body.

At most 4096 header bytes are read. Fragmented reads and partial writes use independent absolute two-second
deadlines. EOF on an incomplete request returns 400, inactivity returns 408 and excessive headers return 431.
The invocation handles at most 128 connections and 1024 accept polls, with conservative byte/call/fuel/arena
limits. The arena is not recycled between requests. Socket faults close the connection; exhausted quotas or
close errors stop serving and invocation cleanup retains ownership of remaining sockets. No TLS, OAuth,
persistence, concurrent service, full Windows compiler distribution or production readiness is implied.

## Windows provider

LOBO_WINDOWS_TCP selects a separate Winsock implementation of the existing closed ABI during C cross-build.
The default snapshot clock/random provider remains separate. No language syntax, ABI operation or grant is
added. The Linux provider and covered formal inputs remain byte-identical to the preserved checkpoint.

The Windows implementation borrows the repository's Linux policy and tests, with OS-specific descriptor,
startup, error and close semantics. SOCKET/INVALID_SOCKET replace POSIX descriptors. Winsock starts lazily
after grant checks, sockets are nonblocking and noninheritable, listeners request exclusive address ownership,
and pending connect ownership precedes fault-capable waits. Registry generations reject stale handles and
retire at the maximum. Exit cleanup closes sockets and calls WSACleanup. Linux finite-model results do not
prove Windows behavior. No external implementation's source/test corpus was newly imported in this task.

## Test-first evidence and corrections

The initial real Windows loopback test failed because the snapshot provider rejected TCP launch grants.
The added provider executes the existing Lobo loopback successfully. The initial HTTP stub returned 400 for
valid GET /health; the test required 200 and failed before the parser implementation.

An injected closesocket failure exposed a Windows ownership defect in the first port. The old port retired
the slot even when closesocket returned WSAEWOULDBLOCK. A regression required the listener to remain tracked
and usable for a close retry. It failed before the correction and passes after the Windows close function
releases a slot only on success. This fault injection demonstrates the ownership path, not a claim that the
normal zero-linger application usually encounters that error.

Implementation attempts also retained an extra parser closing brace, a reserved local name, an import alias
that conflicted with a local function, omitted nominal callback signature imports, and a Rust test-sidecar type
mismatch. These produced compile failures and were corrected. One test driver consequently attempted a missing
executable after a failed build; that is a setup failure, not a protocol result. The first VS Code host attempt
lacked LOBO_VSCODE_EXECUTABLE; the configured isolated host subsequently passed. No failed attempt is treated
as a successful runtime test. Earlier prototypes and incomplete packaging directories remain retained evidence.

## Verification

- Thirty public protocol cases run identically in the reference evaluator, C native, direct LLVM native and
  both sanitizer builds. Cases cover normal routes/HEAD, framing, control characters, headers and size limits.
- The same end-to-end driver passes on Linux and Windows PE under Wine. Its 35 scenarios include the thirty
  protocol cases plus fragmentation, timeout, truncated EOF, port conflict and explicit default denial.
- The exact final Windows binary passes the no-argument launcher suite and its 128-connection lifetime limit.
- A separate final-code test build caps real recv calls to 31 bytes and send calls to seven bytes. The same
  end-to-end scenarios pass, exercising the Lobo reassembly and partial-write loops.
- Eleven Windows provider cases cover all 256 parent/child permission masks, malformed options, bounds,
  quotas, stale tokens, generation retirement, retry after close failure, real loopback and fault cleanup.
- The default Windows snapshot profile still passes 14 application and 10 provider cases with 64 permission
  pairs using the new compiler. Its previously delivered executable and release evidence remain unchanged.
- Both Linux baseline TLA+ models, eleven expected model counterexamples, 22 C witnesses and 22 C mutation
  rejections pass. These claims stay Linux-only. No source-map hashes were changed to include Windows.
- The full Rust suite passes 263 tests. The final focused tests pass after the close correction. All 161
  editor tests, the real isolated VS Code host, Clippy, formatting and both local skill validators pass.
  Workspace navigation includes the new Lobo source closure. No grammar change required editor code changes.

These counts overlap by construction. Repeated platform/build runs are not independent tasks, subjects or
agent measurements. The original snapshot's [Windows 10 user run](2026-11-22-windows10-user-run.json) is a
separate observation. Actual Windows execution of this new REST API remains unknown. Wine is not Windows
hardware. There is no runtime speed, token efficiency, comprehension or software-construction comparison.

## Sources and provenance

The task reuses the isolated LLVM-MinGW 20261117 toolchain from the snapshot build. No new download,
system package or PATH change occurred. Build/test scripts require installed tools explicitly. Archive SHA-256
is e7a15a49a2ea95e9f3d8a9d0cfabe5052a4b10fb4777cc54de5a8c27b64bb701.
Primary references consulted on 2026-11-22:

- [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html): request framing, Content-Length, Transfer-Encoding and connection closure.
- [WSAStartup](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsastartup): explicit Winsock initialization and direct return codes.
- [WSAPoll](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsapoll): readiness flags, millisecond waits and socket error reporting.
- [ioctlsocket](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-ioctlsocket): nonblocking socket configuration.
- [WSASocketW](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsasocketw): SOCKET type and noninheritance flag.
- [closesocket](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-closesocket): close errors and linger semantics.
- [Exclusive address ownership](https://learn.microsoft.com/en-us/windows/win32/winsock/using-so-reuseaddr-and-so-exclusiveaddruse): explicit listener bind policy.

Raw commands, failed attempts, PE binaries, Linux/model results, prior/current source snapshots and hashes are
retained under benchmarks/private-evidence/native-rest-20261122. This local archive is not a publication,
commit, independent audit or off-device backup.
