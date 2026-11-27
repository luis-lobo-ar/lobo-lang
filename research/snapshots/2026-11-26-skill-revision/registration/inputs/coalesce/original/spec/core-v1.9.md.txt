# Lobo Core v1.9

Status: implemented by compiler/extension 1.9.0. This adds a closed SQLite native extension to
[Core 1.8](core-v1.8.md). Existing source constructs and try rules remain unchanged.
Checked format remains lobo-checked/1.0. The existing native ABI remains version 1.

Modern Native::sqlite_* calls and SQLite resource/result types select program.sqlite_abi: 1 alongside
program.native_abi: 1. The compiler injects five additional result layouts only for that extension.
Ordinary native programs retain their original six layouts. Loading revalidates the marker, namespace,
closed signatures, exact result shapes, types, action bounds and resource restrictions. SQLite nodes cannot
be retagged to Core 1.8 or earlier. Core 1.0 through 1.8 artifacts remain readable under their existing rules.
Pre-native user identities that resemble new result IDs keep their original meaning without native markers.

Resource kinds are Native::SqliteCapability, Native::Database and Native::Statement. They are opaque
process-local tokens, never literals, external arguments or serializable outputs. A named database grant
and a root SQLite capability authorize database access. Socket/clock capabilities cannot authorize it.
The [SQLite interface contract](sqlite-interface-v0.1.md) defines operations, errors, ownership and limits.
The compiler emits a self-contained SQLite engine and adapter only for SQLite-selected source bundles.
Both C and direct LLVM application backends compile to native code. SQLite queries use the embedded engine.
There is no Lobo interpreter fallback for database operations. The reference evaluator rejects native calls.

The persistent HTTP/JSON/SQL application exceeded two source admission limits. Core 1.9 therefore admits
at most 16,384 aggregate bundle tokens and 64 source nominal declarations, including synthetic layouts.
Individual source lexing remains capped at 8,192 tokens. The 1 MiB bundle, 16 explicit sources, eight-level
type/layout depth and 128 specialized nominal-instance limits remain unchanged. Existing source/token and
nominal-admission negative tests exceed the new bounds. This changes capacity, not grammar or inference.

No remote database driver, NoSQL interface, ORM, SQL translation, arbitrary FFI, impure function value,
automatic function-scope cleanup or concurrency is introduced. Current Fn values remain pure. The new REST
example calls an actionful controller directly and retains the earlier pure callback helper unchanged.
See [decision 0055](../decisions/0055-first-sqlite-driver.md).
