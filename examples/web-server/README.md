Compiler 1.7 now supplies [native loopback TCP](../native/README.md) without this Rust host. This earlier
HTTP sample still uses the host below; it has not been rewritten into a native Lobo HTTP implementation.

# Native Lobo web-server sample

Lobo handles routing and constructs typed HTTP responses in [app.lobo](app.lobo). A small native Rust
[host](host.rs) owns the socket, parses HTTP and calls the already compiled handler using its typed CLI.
Both executables contain native machine code. No Lobo interpreter runs and no source compiles per request.
Lobo 1.2 itself has no network API. The host boundary remains explicit.

The app imports [reusable HTTP helpers](../../libraries/http/README.md) for typed text/JSON responses,
method checks and UTF-8 prefix matching. The builder supplies that module explicitly. To compile the handler
yourself, add `--source libraries/http/http.lobo`; importing by ID never discovers files automatically.

Build from the repository root, with Cargo, Python 3 and the existing Clang/LLVM tools installed:

```sh
python3 examples/web-server/build.py
./dist/web-server/server
```

The builder builds the maintained 1.6 compiler from the checkout. `--compiler PATH` selects an already-built
1.6 compiler. It never selects the frozen 1.0 compiler for migrated source. `--out-dir NEW_DIRECTORY` directs
the server and handler into a separate output directory, which permits testing without replacing old artifacts.
The Rust host uses an isolated Cargo manifest/lock and cached dependencies with `--offline --locked`.
If those dependencies are unavailable on another machine, Cargo reports the missing cache; an explicit
`cargo fetch --locked --manifest-path examples/web-server/Cargo.toml` fills it. The builder never installs tools.
Use `--backend llvm` to build the Lobo handler with direct LLVM instead of C/Clang.

To run the acceptance tests against a separate output directory, set `LOBO_WEB_EXAMPLE_DIR` to that directory.
`LOBO_COMPILER` selects the maintained compiler for direct handler tests. Both settings accept absolute paths.

Open **http://127.0.0.1:8080/**, or try:

```sh
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/greet/Luis
curl 'http://127.0.0.1:8080/greet/%C3%A9'
curl -I http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/missing
```

| Request | Response |
| --- | --- |
| GET `/` | 200, `Hello from native Lobo!` |
| GET `/health` | 200, JSON status/language |
| GET `/greet/<name>` | 200, UTF-8 greeting |
| GET unknown path | 404 |
| HEAD | Same routing/length headers, no body |
| Other method, without a body | 405 with `Allow: GET, HEAD` |

For another loopback port, run `./dist/web-server/server --listen 127.0.0.1:9090`. Stop with Ctrl+C.
Copy `server` and `handler` together to run from another directory without Lobo, Cargo, Python or Clang.
The host locates `handler` beside its own executable; `--handler FILE` explicitly selects a trusted executable.
Do not rebuild over a running sample; stop it, rebuild, then restart. This is local build tooling, not a rollout system.

## Boundaries

The host accepts loopback listeners only and serves one request per connection sequentially, then closes it.
Headers are limited to 8 KiB / 32 fields, targets to 2 KiB, with an absolute three-second header deadline and
a three-second write timeout. Paths are percent-decoded as UTF-8; queries are ignored and controls rejected.
HTTP/1.1 requires a single nonempty Host header. Ambiguous framing/transfer encodings are rejected; request
bodies are unsupported. These are explicit sample restrictions, not a complete HTTP implementation.

Each request runs the fixed Lobo handler with 10,000 fuel and heap cells. Only method and decoded path cross
that boundary. Status/content type/body size are checked before writing HTTP. Lobo receives no socket handle,
filesystem API or host-library access; action labels alone still grant no authority. The host/handler are trusted
local executables, not an OS sandbox. There is no TLS, persistent storage, keep-alive, concurrent request pool
or production deployment. Starting a native child for each request favors a clear example over throughput.

## Verify

```sh
cargo test --offline --locked --manifest-path examples/web-server/Cargo.toml --target-dir target/web-server
cargo clippy --offline --locked --manifest-path examples/web-server/Cargo.toml --target-dir target/web-server --all-targets -- -D warnings
python3 examples/web-server/test_server.py
```

The tests cover native C/LLVM routing, UTF-8, actual HTTP responses, HEAD byte lengths, 404/405, malformed
framing, body/target/header limits, timeout and rejection of public listeners. See
[decision 0035](../../decisions/0035-native-web-server-sample.md) and the dated result record for evidence.
