# Pure HTTP helpers for the native web example

This is a reusable Lobo 1.2 source module. The host owns sockets and HTTP transport.
Use [http.lobo](http.lobo) with explicit `--source libraries/http/http.lobo`. No dependency installation or
compiler extension is required. The native host still owns IO and enforces its response policy.
The repository's explicit VS Code source list includes both the web app and this module, so Ctrl-click can
follow helper calls and match cases across files. In another project, list both files in `lobo.sourceFiles`.

| Helper / stable ID | Behavior |
| --- | --- |
| `text_response(status, body)` / `lobo.http.text_response` | Response with plain UTF-8 text |
| `json_document(status, document)` / `lobo.http.json_document` | Response with `application/json`; document is already serialized JSON |
| `not_found()` / `lobo.http.not_found` | Standard 404 text response |
| `method_not_allowed()` / `lobo.http.method_not_allowed` | Standard 405 text response; host writes Allow header |
| `read_method(method)` / `lobo.http.read_method` | True for exact GET or HEAD |
| `strip_prefix(value, prefix)` / `lobo.http.strip_prefix` | `PathMatch::Matched(Text)` or `NoMatch`, preserving valid UTF-8 |
| `json_string(value)` / `lobo.http.json_string` | Serialized JSON string, including surrounding quotes and escaping |

For typed objects, arrays, scalar values and complete serialization, use the
[JSON helper module](../json/README.md). Its `json_response(status, value)` adapter accepts a typed Json
value. Supply both helper sources explicitly when compiling that module.

`Response` retains ID `example.web.response` when moved out of the application. Import it by that ID.
`PathMatch` has ID `lobo.http.path_match`. Response constructors require status 200, 404 or 405, matching the
current sample host. An unsupported status fails its precondition; it is not silently changed to 200.
The host's body/content-type checks still apply. Extending status support requires changing that host policy too.

```lobo
module hello;
use http { Response, json_document, json_string };

fn greeting(name: Text) -> Response {
  let document = "{\"name\":" + (json_string(name) + "}");
  json_document(200,document)
}
identities {
  module: "hello",
  http.Response: "example.web.response",
  http.json_document: "lobo.http.json_document",
  http.json_string: "lobo.http.json_string",
  greeting: "hello.greeting"
}
```

`json_document` does not parse or validate arbitrary JSON text. Use `json_string` for dynamic string values;
do not concatenate raw user text into JSON. The quoting helper escapes all U+0000–U+001F controls with `\u00xx`,
escapes quotes/backslashes, and preserves other UTF-8 bytes. It builds persistent byte lists and converts once,
avoiding repeated copies of a growing text buffer. It serializes a string, not arbitrary records or a JSON DOM.

Prefix matching uses byte slices of valid Text values. When an exact valid UTF-8 prefix matches, its end is a
UTF-8 boundary; the suffix is checked again during Text conversion. Empty prefix matches the entire input;
an exact match returns an empty suffix. URL percent decoding belongs to the host and is not repeated here.
The application decides whether an empty suffix is an acceptable route parameter.

All functions are pure and use existing language/runtime budgets. Conformance checks use 200,000 fuel and
100,000 heap cells for escaping cases; that is not a budget grant or a guarantee for every possible input.
The live sample still uses 10,000/10,000 and passes its original 2 KiB maximum-path test. Large JSON strings
can require a larger caller budget; helper use never bypasses host output limits or grants network authority.

Run `cargo test --locked --test http_helpers`. It reloads checked artifacts and tests evaluator, C, LLVM and
both sanitized native routes. The [web example](../../examples/web-server/README.md) imports these helpers.
The maintained source uses 1.2 syntax and preserves its original IDs and checked behavior.
The frozen 1.0 release and research fixtures remain unchanged.

For a complete executable local demo with HTTP parsing and socket loops in Lobo, use the
[native HTTP sources](../http-server/README.md) and [REST app](../../examples/native/rest/README.md).
The older Rust-hosted example remains historical. Existing pure helper status preconditions are unchanged.
