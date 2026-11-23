# SQL helpers

Core/compiler 1.9.0 supplies a first SQLite adapter. This Lobo library adds bound parameters, owned typed
rows, statement cleanup and structured errors. It is currently SQLite-specific. Common names do not promise
SQL dialect portability or a stable multi-driver ABI. No ORM or connection pool is implemented.

Select the helpers and types used by the source and supply sql.lobo explicitly. Native launch grants select
the database file. See [the executable users example](../../examples/native/sqlite/README.md) and
[the persistent REST example](../../examples/native/sqlite-rest/README.md).

```lobo
fn find(db:Native::Database,id:I64)->SqlResult<List<Row>>
actions {db.execute,db.read,db.close}
{
  query(db,"SELECT id,name FROM users WHERE id=?1",params1(Integer(id)))
}
```

This excerpt assumes selected imports. The executable example and tracked sidecars provide complete code.
Use no_params(), params1(value) and params2(first,second). Cell values are Null, Integer(I64), Decimal(F64),
String(Text) and Binary(Bytes). A Row owns List<Cell> values in column order. An empty result list is distinct
from a row containing Null. Decoders should validate count, storage classes and domain requirements.

open/restrict/close and begin/commit/rollback return SqlResult<T>. query returns bounded List<Row>;
execute handles statements without result columns and returns SqlResult<Bool>. query/execute bind every
parameter, capture the result and finalize the statement before returning it. Explicit teardown works after
query budgets are exhausted. Statement errors and SQLite backend codes are preserved in SqlError.
error_details decodes raw native errors; error_code reverses that encoding for native-result boundaries.

A transaction helper must retain an owner that closes the connection after a failed operation. The users
example captures setup/save results, then closes the database. Close rolls back an active transaction.
Try does not imply automatic cleanup. Current pure Fn callbacks cannot execute arbitrary database actions.

Compile to native C or direct LLVM application code. The reference evaluator can run pure decoding helpers,
but database calls require a native executable. Launch with an explicit absolute --sqlite-file and role.
Read is the default role. Write requires an existing file; schema permits creating a file and main tables/
indexes. New files must be in an existing trusted directory. See [the native contract](../../spec/sqlite-interface-v0.1.md)
for the closed SQL policy, limits, error codes and supported platforms.
