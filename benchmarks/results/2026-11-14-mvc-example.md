# Simple MVC sample verification

Recorded 2026-11-14 for the user's request to show a simple MVC example.
The executable source is [examples/mvc.lobo](../../examples/mvc.lobo).

The model contains one fixed in-memory user with ID 1 and name Luis. The view produces JSON with the
existing `json_string` helper. The controller maps a found user to status 200 and a missing user to 404.
HTTP routing and transport can invoke this controller through the existing typed Response interface.
The demonstrated checks exercise native controller functions through the CLI.

Three native invocations passed through `target/release/lobo run`, using explicit
`--source libraries/http/http.lobo`. ID 1 returned application/json with body `{"name":"Luis"}`.
ID 42 returned 404 with the existing not-found body. Direct view invocation round-tripped a name containing
a quote, backslash, newline and Unicode through independent Python JSON decoding.

Exact commands, stdout/stderr, exit codes and source/compiler SHA-256 identities are retained in
`target/mvc-sample-20261114.json`. This is a source example and focused execution check. Compiler/editor
implementation, dependency and protocol work belong to their existing milestones. No agent or performance
measurement accompanied this sample. The requested sample was checked directly without a failing-first
implementation cycle. Documentation link validation uses `./scripts/check.sh`, with output retained in
`target/mvc-sample-20261114-documents.log`.
