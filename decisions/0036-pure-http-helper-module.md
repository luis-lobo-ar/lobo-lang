# 0036: Pure reusable HTTP helpers

Date: 2026-11-05. User selected helpers for the web-server sample. Scope is a small source library,
not a new language/runtime/network API or a complete HTTP/JSON framework.

Move typed response construction, GET/HEAD classification and UTF-8 prefix matching into
`libraries/http/http.lobo`. Preserve Response's existing stable ID when moving its declaration between
modules, so the native host contract remains identical. Imports explicitly supply the library source.
Migrate the sample and builder/test callers together; no duplicated legacy response API remains in the app.

Provide text responses, already-serialized JSON document responses, 404/405 helpers, a tagged prefix result,
and JSON string escaping. Response constructors admit only the sample host's 200/404/405 statuses. The JSON
document helper does not parse/validate arbitrary JSON: callers supply a serialized document, using the string
quoting helper for dynamic values. Quoting preserves UTF-8 and escapes every JSON control/quote/backslash.

Build quoted bytes with persistent lists then one reverse/conversion, rather than repeatedly copying a
growing immutable text buffer. Existing caller fuel/heap budgets remain authoritative. Library conformance
uses explicit 200,000 fuel / 100,000 heap; the live sample's existing 10,000/10,000 budgets must still pass its
maximum-target case. The helper grants no IO authority or budget exemption. No new dependency is needed.

Alternatives: continue repeated record literals (small but amplifies policy/type changes), or add compiler
intrinsics/framework machinery (unnecessary for existing Text/Bytes/List/variant constructs). Use source-level
helpers and preserve the frozen language/compiler/editor. TDD first demonstrates response/prefix/quoting
failures against valid stubs, then checks evaluator, C, LLVM and sanitized native paths plus live sample HTTP.
