# Native test provenance

The [manifest](manifest.json) pins unchanged upstream files, commits, URLs, SHA-256 values and licenses.
Run `python3 scripts/check-native-upstream.py` from the repository root to verify the retained bytes and
adaptation targets offline. These files are source material for adapted Lobo tests. We do not run or claim
conformance to the complete upstream test suites. Reusing cases gives established edge conditions; it does
not make the five runtimes independent validators of Lobo's native adapter.

| Project | Retained license | Adapted coverage |
| --- | --- | --- |
| Node.js 22.21.0 | [MIT and bundled notices](nodejs/LICENSE) | BigInt signed values, numeric boundaries, copied buffer ownership |
| CPython 3.14.0 | [PSF and historical notices](cpython/LICENSE) | ctypes signed-width bounds and Bool, socket Unicode/CRLF payload, empty IO and EOF |
| libuv 1.51.0 | [MIT](libuv/LICENSE) | Close/reuse, shutdown after writes, resource cleanup |
| Go 1.25.0 | [BSD-3-Clause](go/LICENSE) | Deadline, interruption and readiness behavior |
| Rust 1.90.0 | [MIT](rust/LICENSE-MIT) or [Apache-2.0](rust/LICENSE-APACHE) | Loopback, bind failures, binary IO and shutdown |

The [adapted scalar/payload corpus](adapted-cases.json) is consumed by `tests/native_boundary.rs` and
`tests/native_tcp.rs`. `tests/native/adapter_harness.c` injects interruptions, short IO, clock failures and
partial entropy reads into the actual C provider. It adds a 50,000-step deterministic handle model, all
256 parent/child permission masks, stale and wrong-kind checks, fd accounting and fault-exit cleanup.
These model and fault-injection checks are Lobo-specific extensions, rather than copied upstream programs.

Lobo preserves exact signed I64 values and rejects overflow instead of using JavaScript Number truncation
or ctypes wrapping as an oracle. Bool has only false/true. Buffer outputs use an invocation arena, rather
than Node's garbage collector/finalizer lifecycle. Sockets use synchronous bounded operations, IPv4
127.0.0.1 and nonblocking descriptors internally. Async callback ordering, UDP/IPv6, general FFI pointers,
unsigned/floating ABI conversions and upstream GC behavior are outside this contract.

The Node numeric test is also retained as evidence for differences we deliberately do not adopt. Two
attempted `test/js-native-api/test_arraybuffer/test.js` / `test_buffer/test.js` URLs did not exist at the
pinned commit. The actual `test/node-api/test_buffer/test.js` was located through the repository tree and
retained instead. No missing source is counted as borrowed coverage. All original notices remain intact.
