# Native HTTP helpers

The [REST example](../../examples/native/rest/README.md) uses these Lobo sources with the closed native ABI.
`server.lobo` parses a bounded HTTP/1.1 request and serializes a response. `transport.lobo` owns the socket IO
loops. The `serve` API uses a pure `Fn(Request) -> Response` callback and owns its listener. Core 1.11 also supports explicit
context and an action-bound database handler through `respond_with` and `serve_with`.

Compile the application, its controller, both sources, `libraries/json/json.lobo` and `libraries/http/http.lobo`
as one closed bundle. Preserve the adjacent identity files. The root app imports Request and Response because
the contextual callback signature uses those nominal types.

The local demo accepts origin-form ASCII targets, token method names, ASCII headers, exactly one nonempty
Host, and absent or zero Content-Length. Header names are case insensitive. It rejects folded lines, invalid
names or controls, duplicate Host/Content-Length, transfer coding, Expect, and bodies. These are deliberate
subset restrictions. This is not a complete RFC 9112 server. Every response has a byte Content-Length and
Connection: close. HEAD preserves the representation length and omits its body for parsed requests.

At most 4096 header bytes are read. Read and write loops use separate absolute two-second deadlines.
An incomplete request at EOF returns 400, an idle incomplete request returns 408, and excessive headers
return 431. Socket faults close the connection. Fatal quota/close errors end serving and invocation cleanup
owns remaining sockets. Short reads accumulate. Short writes advance by the returned count.

The invocation arena is not recycled per request. Connection, poll, fuel, byte and call limits bound the
example. The pure demo is a sequential read-only local API. The SQLite demo adds persistent reads through a separate
closed driver. Both retain the HTTP subset above, without TLS, authentication or body decoding.

Run the regression suite with `cargo test --locked --test native_rest`. Its thirty public cases run through
the reference evaluator, C and direct LLVM executables, and both sanitizer builds. The standalone TCP driver
checks real socket behavior on Linux and, separately, Windows PE executables through an isolated Wine prefix.
See `scripts/test-native-rest.py` and `scripts/test-windows-tcp.py`.

## Context and database handlers

Compile the [SQLite REST bundle](../../scripts/sqlite-rest-sources.json) with Core 1.11. The application calls:

```lobo
let outcome = serve_with(cap, port, max_requests, db, handle);
try Native::sqlite_close(db);
outcome
```

`serve_with<C>` in `transport.lobo` takes capability, port, request count, explicit context C and a handler
of type `Fn(C, Request) actions { db.close, db.execute, db.read } -> Response`. C is inferred from context
and handler inputs. Callback signatures include exact action sets. This entry supports that concrete DB
bound; it is not action-polymorphic. The listener-based entry remains available as serve_listener with its original ID and signature. Passing a context
value grants no additional native permissions. Import Request and Response in the calling module because
the inferred callback signature names those nominal types.

The helper listens, serves sequential connections and closes its listener after capturing the serving
result. Shared helpers receive headers, send bytes, count accepts and close streams. Each connection keeps
its separate read/write deadlines. Close failures take precedence; recoverable connection errors keep the
previous serving policy, and quota error 11 remains fatal. Native faults still use invocation cleanup.
The library does not own or close the supplied database. The app chooses grants, bootstrap/schema policy,
read restriction, database open and database close. Do not replace the captured result with an early try
that skips those closes. The thin pure and DB adapters share the accept state machine, request preparation and cleanup policies.
Generic action variables and closures remain outside this milestone.

The four former app helper IDs move with their logical definitions: dispatch becomes respond_with;
serve_one becomes serve_one_with; serve becomes serve_context; with_database becomes serve_with. Their
persistent IDs retain the old example prefix. Existing pure API IDs and app main/initialize IDs remain.

`cargo test --locked --test action_callbacks --test http_context_callbacks --test native_rest --test sqlite_programs`
checks callback semantics and protocol/native composition. Real socket, persistence, malformed-row and
Windows-through-Wine checks use the existing executable drivers. See [Core 1.11](../../spec/core-v1.11.md).

## Maintained serving APIs in 1.11.1

| Entry | Inputs | Listener ownership |
| --- | --- | --- |
| serve | cap, port, count, pure handler | Library opens and closes |
| serve_with | cap, port, count, context, DB-bound handler | Library opens and closes |
| serve_listener | listener, cap, count, pure handler | Caller owns supplied listener |
| serve_context | context, listener, cap, count, DB-bound handler | Caller owns supplied listener |

The [stateless app](../../examples/native/rest/app.lobo) and [SQLite app](../../examples/native/sqlite-rest/app.lobo)
use the owning APIs. Both retain their capability policy. SQLite retains bootstrap and database lifetime.
The old serve declaration becomes serve_listener with its unchanged stable ID and signature; the owning
serve has a fresh ID. Existing tracked imports targeting the old ID retain its listener-based meaning.

There is one native accept call site and one accept/retry loop in next_connection. Its returned Connection
carries the updated progress after retries. serving defines request/poll/error termination once. prepare_request
centralizes read faults, header rejection and parsed-request dispatch. finish_connection and finish_listener
centralize close-error precedence. Thin adapters drive pure or DB callbacks under their exact action sets;
they do not duplicate accept, timeout, rejection or cleanup policy. Read/write loops and the 4096-byte,
two-second, 1024-poll limits remain unchanged. This is an implementation refactor under Core 1.11.

All 45 maintained Lobo sources have explicit review roles in
[the scope manifest](../../scripts/http-refactor-scope.json). Run
`node scripts/check-http-refactor.cjs --out NEW_JSON_FILE` and
`node --test scripts/test-http-refactor.mjs` to check scope/policy ownership. New maintained sources need a
recorded role. The raw loopback/OS examples exercise the ABI directly; the older Rust-hosted handler has its
own protocol contract. Frozen/comparison/illustrative exclusions follow the existing maintainedPath rule.
The audit is a structural check, separate from compilation and runtime tests.

`cargo test --locked --test http_lifecycle --test modern_source` verifies actual listener-slot reuse,
unchanged pure action bounds, quota behavior, stream/listener close precedence and shared termination limits.
An injected C timeout source proves 1024 accepts and rejects a deliberate 1025-poll mutant in optimized and
ASan/UBSan builds for both handler adapters. The real TCP driver accepts `--native-handles 4`, the minimum for
the apps' two capabilities, listener and one stream. Repeated connections then detect leaked stream slots.
See [decision 0060](../../decisions/0060-complete-maintained-http-refactor.md).

## Permission profile in 1.11.2

`server_permissions()` returns the typed OS set containing ClockRead and Listen. The app calls
`restrict(root, server_permissions())` before `serve` or `serve_with`. Supply
[libraries/native/permissions.lobo](../native/permissions.lobo) in the explicit source closure.
The helper requests no random or outbound-connect rights. The invocation must grant clock and listen.
