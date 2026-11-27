# SQLite interface v0.1

Implemented by compiler 1.9.0, Core 1.9, SQLite extension 1 over the unchanged native ABI 1 result layout.
The pinned SQLite 3.54.1 amalgamation is compiled into selected programs. This is a synchronous 64-bit
Linux provider and a Windows x64 C-export/cross-build profile. It is not a general driver/plugin SDK.

## Closed catalog

All bindings are lobo.native.sqlite_NAME with NativeOp ABI 1, C symbols lobo_sqlite_NAME_v1.
The [C declarations](../src/sqlite_api.h) use the common [native result/buffer layout](../src/native_api.h).
The union adds a double member without changing its 24-byte size or eight-byte value offset.

| NAME | Arguments | Output | Action |
| --- | --- | --- | --- |
| capability | none | SqliteCapability | native.authorize |
| restrict | SqliteCapability, I64 mask | SqliteCapabilityResult | native.authorize |
| open | SqliteCapability | DatabaseResult | db.open |
| close | Database | BoolResult | db.close |
| prepare | Database, Text SQL | StatementResult | db.execute |
| bind_null | Statement, I64 position | BoolResult | db.execute |
| bind_i64 | Statement, I64 position, I64 | BoolResult | db.execute |
| bind_f64 | Statement, I64 position, F64 | BoolResult | db.execute |
| bind_text | Statement, I64 position, Text | BoolResult | db.execute |
| bind_bytes | Statement, I64 position, Bytes | BoolResult | db.execute |
| step | Statement | BoolResult | db.execute |
| column_count | Statement | I64Result | db.read |
| column_type | Statement, I64 index | I64Result | db.read |
| column_i64 | Statement, I64 index | I64Result | db.read |
| column_f64 | Statement, I64 index | F64Result | db.read |
| column_text | Statement, I64 index | TextResult | db.read |
| column_bytes | Statement, I64 index | BytesResult | db.read |
| finalize | Statement | BoolResult | db.close |
| begin | Database | BoolResult | db.transaction |
| commit | Database | BoolResult | db.transaction |
| rollback | Database | BoolResult | db.transaction |
| changes | Database | I64Result | db.read |

All result layouts have Ok(payload) and Err(I64). New layouts are SqliteCapabilityResult, DatabaseResult,
StatementResult, TextResult and F64Result. Result owners can differ across try when error types match.
The high-level [SQL library](../libraries/sql/README.md) exposes SqlResult<T> with a structured SqlError.
Actions are conservative bounds. db.execute can read or mutate SQL data. Runtime roles enforce authority.

## Grants and ownership

Only an executable entry at depth one can acquire Native::sqlite_capability(), including the cached root.
Its rights come from explicit launch options. restrict accepts masks 0..7 and only removes parent rights.
Rights are read=1, write=2 and schema=4. Delegated database handles retain their captured role.
An empty grant cannot open a database. SQL capability tokens and the socket/clock registry are distinct.

Source cannot choose a file path. open selects the single launch-granted absolute database path. Reject
URI/memory names, dot path components, symlinks and Windows reparse points. Validate the main file and
-journal/-wal/-shm side paths. Parent directories must remain trusted against concurrent replacement.
These checks are not a custom secure VFS. The profile assumes a local filesystem with SQLite-compatible
locking/durability. It does not detect every remote mount, OS redirection or hostile filesystem race.

Read opens read-only. Write opens an existing database read/write. Schema additionally permits creation.
The adapter configures defensive mode, untrusted schemas, memory temporary stores, DELETE journal mode,
FULL synchronization and disabled mmap before installing the authorizer. Incompatible existing journal
modes or filesystem failures can reject open. Schema administration covers main-table/index creation,
drop and reindex. ALTER, views, triggers, temp objects, ATTACH, PRAGMA and unapproved functions are denied.
Extension loading is compiled out. The function allowlist is the reviewed list in native_sqlite.c.
Known authorizer denials remain Denied even when SQLite itself reports SQLITE_ERROR for a denied function.

A separate 32-slot registry uses namespace 0x53, a one-byte slot index and a 48-bit generation. Wrong kind,
forged, closed, stale and retired handles fail before engine use. Generation exhaustion retires the slot.
Aliases are allowed. Finalize consumes its statement. Close finalizes owned statements, rolls back any
active transaction and closes the database. Invalidated aliases stay invalid after slot reuse.
Explicit close/finalize bypass the SQL call quota. Invocation/fault cleanup bypasses all user quotas,
finalizes statements before connections, rolls back active transactions and then runs the old OS cleanup
before arena release. IO failures remain possible. There is no automatic cleanup at every function return.

## Statement and transaction state

prepare accepts one nonempty UTF-8 statement up to 65,536 bytes, rejects embedded NUL and non-whitespace
tail text, and tracks ownership before a Lobo fault can strand a result. Statements start Ready. All
parameter positions must be bound explicitly before stepping, including explicit NULL. Positions are
one-based. Column indices are zero-based. Bound buffers are copied by SQLite; empty Text/Bytes stay
empty values, distinct from NULL. Parameters represent values, never SQL identifiers or SQL fragments.

step returns true for a row and false for completion. It permits Ready/Row states and transitions to
Row, Done or Failed. Done/Failed cannot silently reset or restart a statement. Binding after a step is
rejected. Writable statements with result columns, including DML RETURNING, are rejected in this slice.
The query helper requires result columns; the execute helper rejects result columns before execution.

Column type codes follow SQLite: 1=I64, 2=F64, 3=Text, 4=Bytes, 5=NULL. Typed access requires the exact
storage class and a current row. No implicit narrowing/coercion is performed. Text must be valid UTF-8;
F64 must be finite. Owned output buffers remain valid after finalization/close. SQLite's own affinity,
arithmetic and storage semantics still apply. Lobo's arithmetic rules do not replace SQL semantics.

begin requires no active transaction. commit/rollback require an active one. These operations reject live
statements on that connection. A failed commit can leave a transaction active. Do not automatically retry
writes. Explicit close provides rollback cleanup even after limits are exhausted. Native execution faults
and typed SQL errors remain distinct. The library captures operation results before finalization/close.
Finalization reports lifecycle/handle errors, not an already-reported step failure.

## Budgets

| Option | Accepted range | Default |
| --- | --- | --- |
| --sqlite-file | One validated absolute file path | No access |
| --sqlite-mode | read, write, schema | read |
| --sqlite-calls | 1..65,536 | 4,096 |
| --sqlite-bytes | 0..1,048,576 | 1,048,576 |
| --sqlite-steps | 1..10,000,000 | 100,000 |
| --sqlite-memory | 1,048,576..67,108,864 | 8,388,608 |
| --sqlite-handles | 1..32 | 32 |
| --sqlite-rows | 1..1,024 per statement | 128 |

Duplicate, malformed, unknown or unsupported options fail CLI admission. Scalar spellings reject leading
zeros. Mode without a file grants no access. prepare/binding/returned copies charge byte quotas. Operations
charge calls except root acquisition and teardown. SQLite progress callbacks consume a separate instruction
budget during preparation/execution. Busy waits return immediately. SQLite allocations use its hard heap
limit; Lobo-owned copies consume normal arena/fuel budgets. Columns/parameters are capped at 32, SQL/value
length at 65,536, expression depth at 32 and VDBE program length at 25,000 instructions.

These limits do not promise a hard wall-clock bound on OS IO, every parser step or rollback. SQLite remains
trusted C in the process. The adapter is not a memory-corruption sandbox or a proof of crash durability.

## Errors and evidence

Raw error integers encode category + 65,536 * original SQLite result code. Adapter policy/state errors have
backend code zero. Categories are 1 Denied, 2 InvalidInput, 3 InvalidHandle, 4 WrongKind, 5 Closed, 7 Busy,
8 Constraint, 9 Decode, 10 IO, 11 Limit, 12 SQL and 13 State. Code six is unused. Preserve backend details;
for example constraint subcodes distinguish the underlying violation. No raw SQL/value/credential logging
is enabled by this extension.

The [SQL library](../libraries/sql/README.md) owns buffer/statement loops and error decoding. The
[verification mapping](../verification/sqlite/README.md) records bounded lifecycle claims and deliberate
model/C defects. Concrete engine/provider/native executable tests cover additional behavior. None of these
checks establish complete SQLite correctness, arbitrary filesystem failure recovery, production readiness,
actual Windows hardware coverage or an agent-efficiency benefit.
