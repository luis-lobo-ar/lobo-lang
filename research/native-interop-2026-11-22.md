# Native library and operating-system interfaces

Reviewed 2026-11-22, America/Chicago. This is a primary-documentation comparison and an unimplemented Lobo
proposal. No native binding, socket API, authentication server or Angular app is implemented by this review.
The [source register](native-interop-2026-11-22.json) records URLs, selected claims and limits.

## What is being compared

A native library contains compiled machine code with callable functions. Application languages can invoke
those functions through an agreed binary interface. That interface specifies symbol names, argument and
result layouts, calling conventions, ownership and error behavior. C-compatible interfaces are common,
including when a library implementation uses Rust or C++.

Execution engines, I/O libraries and operating-system services have different responsibilities. V8 executes
JavaScript. Node supplies the wider platform, including I/O through libuv. CPython runs Python code and
exposes native extension interfaces. A native-compiled program can also include runtime support. Go's runtime
is linked support code and does not imply a virtual machine.

The operating system implements facilities such as sockets. On Linux, a C application can call `socket`
through libc. C's language specification does not itself provide a portable networking API. Libraries then
build higher-level functionality on OS facilities. Implementing an HTTP library does not require
reimplementing the kernel's TCP stack.

The [November 2026 TIOBE index](https://www.tiobe.com/tiobe-index/) supplies the literal top-ten selection below.
Its popularity measure is not a quality ranking or evidence of native-interface performance. SQL requires a
database-specific example. Visual Basic uses the documented VB.NET interface rather than classic VB.

| Rank | Language | How it reaches native code and the OS | Primary reference |
| --- | --- | --- | --- |
| 1 | Python | CPython extension modules use its C API. `ctypes` calls DLL/shared-library functions with C-compatible types. Python wrappers can expose higher-level APIs. | [ctypes](https://docs.python.org/3/library/ctypes.html), [extension interface](https://docs.python.org/3/extending/index.html) |
| 2 | C | Headers declare functions and the linker connects compiled implementations. On Linux, libc exposes `socket` and related OS interfaces. Other platforms have their own APIs. | [Linux socket interface](https://man7.org/linux/man-pages/man2/socket.2.html) |
| 3 | Java | The JVM supports JNI and the Foreign Function and Memory API. FFM calls native libraries and represents memory layouts and lifetimes explicitly. | [FFM](https://docs.oracle.com/en/java/javase/25/core/foreign-function-and-memory-api.html) |
| 4 | C++ | Native functions link directly. `extern "C"` declares C language linkage for interoperable exports/imports. Object layout and ownership still require agreement. | [C linkage](https://learn.microsoft.com/en-us/cpp/cpp/extern-cpp?view=msvc-170) |
| 5 | C# | .NET P/Invoke maps managed declarations to native functions. `LibraryImport` generates interop code, while `DllImport` is another supported declaration mechanism. | [P/Invoke](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/pinvoke) |
| 6 | JavaScript | In Node, V8 executes JS, libuv provides platform I/O support, and Node-API exposes an ABI-stable C interface for native addons. Browser JS uses the browser's exposed APIs instead. | [Node-API](https://nodejs.org/api/n-api.html), [libuv](https://docs.libuv.org/en/v1.x/design.html), [V8](https://nodejs.org/learn/getting-started/the-v8-javascript-engine) |
| 7 | R | Native packages register compiled routines. `.C`, `.Fortran`, `.Call` and `.External` expose different foreign-call interfaces and value representations. | [Writing R extensions](https://cran.r-project.org/doc/manuals/r-release/R-exts.html) |
| 8 | SQL | There is no universal SQL native interface. PostgreSQL loads compiled C-compatible functions as shared libraries through its server extension API. This says nothing about other engines' extension conventions. | [PostgreSQL C functions](https://www.postgresql.org/docs/current/xfunc-c.html) |
| 9 | Visual Basic | VB.NET uses .NET native interop. `Declare Function ... Lib ...` names a native external procedure and library. | [Declare statement](https://learn.microsoft.com/en-us/dotnet/visual-basic/language-reference/statements/declare-statement) |
| 10 | Go | Go supplies native-compiled standard libraries and runtime support. `cgo` builds packages that call C code. Ordinary Go networking does not require application authors to use cgo. | [cgo](https://pkg.go.dev/cmd/cgo), [runtime explanation](https://go.dev/doc/faq#runtime) |

Rust is an additional relevant comparison because it implements Lobo's compiler. Rust uses C ABI declarations
for foreign calls, with unsafe operations normally contained behind typed wrappers. The
[Rust FFI guide](https://doc.rust-lang.org/nomicon/ffi.html) illustrates a C-compatible interface over a C++ library.
This does not require the application's source language to be Rust.

Python's [SSL module](https://docs.python.org/3/library/ssl.html) illustrates library reuse: it wraps OpenSSL.
Node similarly has a [TLS interface](https://nodejs.org/api/tls.html). An application author works through the
language API while the native implementation performs cryptographic operations.

## Proposed Lobo boundary

This is a recommendation inferred from the comparison and repository constraints. It is not accepted syntax,
an implemented ABI, an implementation authorization or a performance result. Current Lobo 1.6 compiles pure
source to native code, links its built-in runtime helpers, and checks declared actions. It has no general
foreign-function declaration or implemented OS capability enforcement. The existing web example uses a
separate Rust host. See [compiler/backends](../docs/compiler-and-backends.md) and
[checked action bounds](../decisions/0013-checked-action-bounds.md).

The proposed route has a narrow native adapter, a Lobo library API, and linked application code. The adapter
can initially use C-compatible exports implemented in C or Rust. Higher-level HTTP parsing, routing,
response construction and application policy can use Lobo. TLS and cryptography should use established
libraries through reviewed bindings.

A schematic native adapter declaration could be:

```c
/* Proposal only: declaration, not an implemented function. */
int32_t lobo_tcp_listen_v1(
    uint64_t capability_handle,
    const uint8_t *host, uint64_t host_length,
    uint16_t port,
    uint64_t *listener_handle);
```

That declaration would need a full platform contract before use. An internal handle registry must validate
capability and listener handles, their generation, kind and lifetime. Application source must not construct
these handles as ordinary integers. The adapter must validate pointer/length combinations generated by the
compiler, bound input sizes, copy or retain data under an explicit ownership rule, and classify every failure.
The return status and output handle must have a defined success/error relationship. Platform calling
conventions, integer layouts and alignment must be checked for each supported target. No panic or exception
may unwind across the boundary. This signature is not a complete resource or concurrency design.

The public API should expose opaque resource values and typed results. Native binding metadata should pin
stable operation IDs, ABI version, signatures, library inputs and action requirements. The build must allow
only declared bindings. Existing action labels describe behavior; actual runtime checks must enforce
capabilities, permitted addresses, resource bounds and cleanup. Adding a label alone provides no authority.

A typed boundary reduces accidental misuse. Native code still shares process memory and may bypass policy
or corrupt the process. Untrusted native implementations require a separate isolation boundary or must be
rejected. Static linking does not establish memory safety or sandboxing.

## Implementation order and acceptance

1. Specify the ABI, trusted library set, ownership/error rules and platform support. Reject arbitrary symbol
   loading as the initial surface. Keep the resource model and supported blocking/concurrency behavior explicit.
2. Test a small deterministic native operation through both C and LLVM lowerings, checked-artifact admission
   and the linker. Reject missing/mismatched symbols, ABI versions and signatures. Do not silently fall back
   to an interpreter or host subprocess.
3. Add one bounded resource slice, such as a loopback TCP listener. Test denied authority, invalid/stale/wrong
   handles, close and reuse, connection failure, timeouts, resource exhaustion, partial reads/writes and cleanup.
   Test runtime enforcement as well as source action checking. Add clocks and secure randomness as separate
   explicit operations. Select concurrency only when the application requirements establish its need.
4. Implement the Lobo HTTP layer and reviewed HTTPS/cryptography bindings. Exercise malformed/truncated input,
   limits, certificate/hostname validation, identity-token validation and OAuth state/nonce/PKCE failures.
5. Build the Google authentication sample and Angular assets. Link supported native dependencies into the app
   executable and embed the frontend build. Record any remaining shared libraries and OS data dependencies.
   Credentials remain deployment configuration. Building a native dependency from source needs its compiler;
   using a shipped compatible library artifact can avoid that toolchain on an application developer's machine.

One executable is a packaging target. It is not automatically a fully static binary, a cross-platform binary,
or independent of certificate stores, OS facilities and configuration. Static versus dynamic linkage remains
an explicit per-target decision. Building the existing Lobo compiler requires Rust. Building current Lobo
programs with an already-built compiler uses the documented native toolchain; Rust is not a language-level
requirement for application source.

## Review outcome and limits

The reviewed implementations support a native-call boundary as a plausible route for Lobo. They do not
establish the exact syntax, ownership representation, capability enforcement, compatibility policy, scheduling
model or dependency choices. No cross-language programs were executed for this comparison. No agent
construction, comprehension, token, safety or performance advantage was measured.

The Google auth and Angular work is paused at this architecture discussion after the user questioned a
separate Rust server. The application remains requested, but this review does not resume the rejected host
approach or claim an OS boundary exists. The initial Rust-host choice was a workaround for current gaps;
it is not required by native compilation or by Rust being Lobo's compiler implementation language.
