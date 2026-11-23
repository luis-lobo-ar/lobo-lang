# Standalone persistent SQLite REST API

Compiler 1.11.2 and Core 1.11 compile HTTP parsing, controller code, SQL helpers and SQLite into one native executable.
The database file is runtime data. The API serves /health, /api/users and /api/users/ID with GET/HEAD.
It remains a sequential loopback demo with bounded requests. No TLS, authentication or HTTP writes are added.
HTTP IDs use nonnegative decimal syntax up to I64 maximum. The native SQL model also exposes CLI operations.

The application owns its database connection. The shared HTTP library invokes the action-bound controller
with that connection as explicit context. Both native HTTP apps share transport policy. Bootstrap creates the users table and seeds Luis/Ada without replacing existing records.
It then closes the privileged connection, derives read-only authority and opens the connection handed to
controllers. Read-only launches can disable bootstrap. The controller validates the exact row width and uses strict typed SQL accessors with try. Each request finalizes its statements. Connection,
listener and accepted-stream owners close resources after handled outcomes.

## Windows executable

Run LoboSqliteRest.exe from a trusted writable local directory. The default launch creates or opens
lobo-users.sqlite in that directory, bootstraps it and serves http://127.0.0.1:8080/health. Try /api/users and
/api/users/1. It stops after 128 accepted connections. The default profile explicitly grants only its chosen
SQLite file and loopback listener/clock. Explicit CLI arguments inherit no demo grants.

```powershell
.\LoboSqliteRest.exe --entry example.sqlite.rest.main --arg I64:8081 --arg I64:128 --arg Bool:false --sqlite-file C:\Lobo\lobo-users.sqlite --sqlite-mode read --sqlite-calls 65536 --sqlite-steps 1000000 --native-clock true --native-listen 127.0.0.1:8081 --native-calls 65536 --fuel 100000000 --heap-cells 16000000
.\LoboSqliteRest.exe --entry example.sqlite.users.save --arg I64:3 --arg Text:4772616365 --sqlite-file C:\Lobo\lobo-users.sqlite --sqlite-mode write
```

The first command assumes a bootstrapped file. Replace Bool:false with Bool:true and mode schema for initial
setup. The launcher converts Windows wide arguments to UTF-8, including non-ASCII database paths. Linux
cross-builds and Wine tests are separate from actual Windows 10/11 execution reports.

## Build and inspect

The installed LLVM-MinGW toolchain must be supplied explicitly. No build script downloads tools.

```sh
cargo build --locked --release --bins
python3 scripts/build-sqlite-rest.py --target linux --out target/sqlite-rest-linux
python3 scripts/build-sqlite-rest.py --target windows --toolchain /absolute/llvm-mingw --out target/sqlite-rest-windows
```

The source list is [sqlite-rest-sources.json](../../../scripts/sqlite-rest-sources.json). Build output records
commands, sources, compiler/engine hashes, executable size and DLL imports. Windows output includes source,
SQLite notices/provenance, toolchain notices and a ZIP. SQLite needs no separate DLL. Linux uses its ordinary
system runtime. Native C export is self-contained; direct LLVM application lowering is validated on Linux.
The Windows compiler itself is not ported by this distribution.

For cross-file navigation and diagnostics, open
[SQLiteRest.code-workspace](SQLiteRest.code-workspace) with extension 1.11.2. Its source set contains the nine
explicit sources. This avoids replacing the existing repository editor source configuration.

Read the [SQL library](../../../libraries/sql/README.md) and [native contract](../../../spec/sqlite-interface-v0.1.md)
for query limits, role restrictions, cleanup and storage assumptions. Use a local filesystem with trusted
parent directories. The linked native engine is trusted code, not a process sandbox or a crash-recovery proof.

## Application composition in Core 1.11

`app.lobo` owns grants, optional data setup and the read-only database lifetime. It delegates the accept,
read/dispatch/write loops and listener cleanup to `serve_with` in the HTTP transport library. The callback
is a named action-bound function with an explicit database argument; it captures no state. Request and
Response are imported for the inferred callback signature. Stream/listener close errors and the database
close retain their previous precedence. Existing main/controller/model identities remain; moved helper IDs
are relocated with their definitions. See [the library API](../../../libraries/http-server/README.md).

Compiler/extension 1.11.1 retain this app's source and identities. Its context handler now uses the same
accept/retry, preparation and cleanup policies as the stateless native REST example. The shared TCP driver
can use `--native-handles 4` to verify repeated stream-slot reuse. Database and bootstrap ownership remain
explicit in the application. See [the maintained HTTP scope](../../../scripts/http-refactor-scope.json).

Compiler/extension 1.11.2 use the typed HTTP `server_permissions()` profile and SQL `one(Read)` delegation.
The workspace and build source manifest include the shared OS permissions module. Original declaration IDs remain.
