# Persistent users in SQLite

This executable Lobo model uses the SQL library to create, save, find and list users. It has no server or
interpreter process. SQLite 3.54.1 is compiled into the executable. The compiler still requires Rust to build
and Clang to compile programs. Execution does not require Rust or an installed SQLite SDK.

```sh
cargo build --locked --release --bins
./target/release/lobo compile examples/native/sqlite/users.lobo --source libraries/sql/sql.lobo -o target/sqlite-users
./target/sqlite-users --entry example.sqlite.users.setup --sqlite-file /absolute/private/users.sqlite --sqlite-mode schema
./target/sqlite-users --entry example.sqlite.users.save --arg I64:3 --arg Text:4772616365 --sqlite-file /absolute/private/users.sqlite --sqlite-mode write
./target/sqlite-users --entry example.sqlite.users.get --arg I64:3 --sqlite-file /absolute/private/users.sqlite
./target/sqlite-users --entry example.sqlite.users.list --sqlite-file /absolute/private/users.sqlite
./target/sqlite-users --entry example.sqlite.users.undo --sqlite-file /absolute/private/users.sqlite --sqlite-mode write
```

Create the private parent directory first. setup is idempotent and seeds Luis/Ada without overwriting
existing names. save uses bound parameters and a transaction. get/list return SqlResult<List<Row>> with typed
cells. undo deliberately deletes inside a transaction and rolls back, so users remain. typed demonstrates
NULL, I64, F64, empty Text and empty Bytes. F64 wire values use the existing 0x-prefixed binary64 spelling.

An invocation without --sqlite-file returns a structured Denied error. Read-only writes fail. Errors return
SqlError with a kind and original backend code. Setup/save capture their operation result and close the
connection before returning it. Close rolls back failed active transactions without needing remaining query
budgets. The native contract does not guarantee rollback success under arbitrary storage failure.

Run with --backend llvm at compile time for direct LLVM application bodies. All source IDs live in adjacent
tracked identity files. Database paths are launch grants, not source imports. The
[SQLite interface](../../../spec/sqlite-interface-v0.1.md) defines policy and limits. The
[persistent REST executable](../sqlite-rest/README.md) also contains these model entries for CLI use.
