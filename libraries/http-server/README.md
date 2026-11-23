# Native HTTP helpers

The [REST example](../../examples/native/rest/README.md) uses these Lobo sources with the closed native ABI.
`server.lobo` parses a bounded HTTP/1.1 request and serializes a response. `transport.lobo` owns the socket IO
loops. A pure `Fn(Request)->Response` callback supplies application behavior.

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
example. It is a sequential read-only local API, without TLS, authentication, body decoding or persistence.

Run the regression suite with `cargo test --locked --test native_rest`. Its thirty public cases run through
the reference evaluator, C and direct LLVM executables, and both sanitizer builds. The standalone TCP driver
checks real socket behavior on Linux and, separately, Windows PE executables through an isolated Wine prefix.
See `scripts/test-native-rest.py` and `scripts/test-windows-tcp.py`.
