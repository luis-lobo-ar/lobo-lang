# 0035: Native web-server sample with an explicit host

Date: 2026-11-05. The user selected a sample web server after the bounded language 1.0 release.
This authorizes one application example, not a language/FFI/network API expansion.

Keep routing and response construction in compiled `app.lobo`. A small native Rust host owns the loopback
socket and invokes the precompiled handler using the existing typed CLI. No request compiles source or
selects an executable. The handler receives only method/path Text values and returns a checked response.
The host grants no network/filesystem functions to Lobo and does not pretend action labels enforce authority.

Alternatives: Python's HTTP host would be smaller but would add an interpreted serving component; embedding
Lobo functions through FFI would require a new supported ABI and authority design. Two native executables
using the existing boundary meet this sample without either expansion. Each request starts one handler
process; this is a deliberately simple sequential sample, not a throughput experiment.

Use isolated example Cargo.toml/Cargo.lock: httparse 1.10.1 supplies HTTP/1.x syntax validation, httpdate 1.0.3
formats the Date header, and serde_json decodes existing typed output. The first two are additional example
dependencies, justified by HTTP framing/date correctness rather than duplicating protocol parsing. They are
already cached and resolved offline. The compiler's dependencies, language syntax, release files, extension
and frozen evidence remain unchanged. No new concurrency model or network framework is added.

Bind loopback only; one request per connection, close afterward. Bound headers to 8 KiB / 32 fields and
targets to 2 KiB; use an absolute three-second header deadline and bounded writes. Require Host for HTTP/1.1,
reject ambiguous framing and transfer encoding, and reject bodies. Lobo handles GET/HEAD routes and 404/405.
The host suppresses HEAD bodies, counts UTF-8 bytes, validates response status/content type/body bounds,
and gives the native handler fixed fuel/heap budgets. This is no OS sandbox or production HTTP service.

Protocol reference: [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html), especially framing and closure.
TDD: native routing initially returns 404 and positive cases fail; HTTP boundary/serialization stubs fail
focused tests. Verify actual loopback requests and failure cases before handing off the sample.
