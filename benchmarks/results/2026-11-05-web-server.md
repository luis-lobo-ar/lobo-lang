# Native web-server sample

Status: implemented and verified. The user selected this application after language 1.0 completion.
Routing/response construction is compiled Lobo; transport/framing is a small native Rust host. Two native
ELF executables use the existing typed CLI, not a new Lobo socket API, FFI or interpreter. No compiler/editor
source or language 1.0 release artifact changed. See decision 0035 and `examples/web-server/README.md`.

## Evidence

- `web-server-handler-red.log`: valid initial Lobo stub returned 404; five expected positive/method cases
  failed before route implementation. `web-server-handler-green.log` then passed the original route test.
- `web-server-host-red.log`: three focused HTTP parser/serialization tests failed against valid stubs.
  `web-server-host-green.log` and `web-server-host-final.log`: all three passed after implementation.
- `web-server-http-first.log`: the new Python test client mistakenly used HTTPConnection as a context manager;
  six request subcases errored before exercising the server. Corrected resource handling to `closing`, without
  changing expected behavior. `web-server-http-second.log`: all four initial acceptance tests passed.
- `web-server-http-boundaries.log`: an added maximum-target test exposed HTTP 500 from handler fuel exhaustion.
  Caching Text::length alone still failed (`web-server-http-length-fix.log`). Existing Text slices charge the
  entire source length, so use byte slicing for the literal ASCII prefix and checked UTF-8 conversion of the
  remainder. The prefix comparison establishes the ASCII boundary. Kept both 2 KiB path and 10,000 fuel/heap
  ceilings unchanged. `web-server-http-final.log`: all six acceptance tests passed, including longest path,
  actual HTTP routing/HEAD/UTF-8/framing/limits/deadline, public-listener rejection and handler startup failure.
  The direct route test now checks seven cases with both C and LLVM native handlers.
- Isolated host Clippy with warnings denied and formatting pass (`web-server-clippy.log`); build uses locked
  offline dependencies. `web-server-core-regression.log`: 185 compiler Rust tests pass. Documentation/link
  checks and skill validation pass (`web-server-doc-check.log`). No syntax changed, so the extension is retained.
- Live `/health` and `/greet/Luis` returned HTTP 200 with exact expected bodies at 127.0.0.1:8080; the live-check
  receipt records headers and payloads. Demo process is intentionally left running locally for the user.
- All 86 frozen language 1.0 artifact hashes were reread successfully. Compiler src/Cargo and editor input
  hashes still match that release. New sample/docs are a subsequent working-tree addition, not a rewrite of
  the frozen source snapshot. No release retagging, publication or user-profile installation.

Final sample binaries are `dist/web-server/server` (504,600 bytes) and `dist/web-server/handler` (22,896 bytes).
These are file sizes, not throughput/latency measurements. Root compiler dependencies are unchanged. The
example's own lock has httparse/httpdate plus serde_json; these were resolved from the existing cache offline.
There is one native process invocation per request and sequential, loopback-only service. No TLS, request
bodies, keep-alive, external exposure, persistent storage or production-readiness claim.

The [machine-readable receipt](2026-11-05-web-server.json) pins source, tool identities, binaries and all raw
results. This is public engineering evidence, not an independent agent/token/comprehension experiment.
