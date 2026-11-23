# 0051: Standalone native REST demo

Date: 2026-11-22
Status: Selected for implementation by the user's request to build and fully compile a REST API in Lobo.

The outcome is a local JSON REST API whose parser, request handling, routing and response serialization are
written in Lobo. Deliver a Windows 10/11 x64 executable and validate the same Lobo sources on Linux.
Use high-assurance review for socket ownership and untrusted framing. This is a bounded sequential demo.

Considered retaining the historical Rust HTTP host, adding an external C HTTP library, and using the existing
closed native ABI with Lobo HTTP code. Select the existing ABI. It keeps HTTP/application behavior in Lobo
and requires no new language construct, ABI operation, runtime package or application host.

The existing Windows snapshot profile stays unchanged. Add an explicit build-time Windows TCP profile,
selected by LOBO_WINDOWS_TCP, with Winsock equivalents of the existing Linux ABI. Keep its implementation
separate from the byte-identical Linux provider and its finite model claims. Test real Windows PE execution
under Wine separately from user-reported Windows execution. No complete Windows compiler port is implied.

The app binds only 127.0.0.1 with explicit listen/clock grants, serves read-only /health and /api/users routes,
and closes each connection. Reject ambiguous framing, unsupported request bodies and oversized headers.
Use deadlines across fragmented reads and partial writes, deterministic content lengths, and bounded requests,
polls, bytes, calls, fuel and arena memory. Close accepted sockets after every outcome and the listener at exit.

Windows socket descriptors use SOCKET and INVALID_SOCKET, not POSIX int/-1. Initialize Winsock explicitly,
use nonblocking IO and monotonic deadlines, close with closesocket, reserve exclusive address ownership,
and track unpublished descriptors before operations that can fault. Generation tokens reject stale handles.
No outbound non-loopback access, TLS, authentication, storage, concurrent serving or performance claim.

A failing injected-close test exposed a policy difference from POSIX. Winsock close failure must retain
registry ownership, so the Windows provider releases its generation slot only after closesocket succeeds.
The regression fails before the correction and passes after it. This does not change the Linux provider.
