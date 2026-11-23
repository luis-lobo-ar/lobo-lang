# Database access proposal

Date: 2026-11-23. Status: proposal only. Compiler and extension remain 1.8.1, language Core 1.8 and native ABI 1.
Lobo has no database driver today. Its current closed native catalog supplies neither database handles nor
file access, remote database transport or TLS. Existing socket support is restricted to loopback.

## Start with SQLite and a thin SQL library

Implement one SQLite adapter together with the first SQL library. This fits the current standalone local
REST example. SQLite is an embedded database with a C interface and no separate database server. Its core
operations cover connection ownership, prepared statements, parameter binding, stepping, column access
and finalization. See [SQLite use cases](https://www.sqlite.org/whentouse.html) and
[the C interface introduction](https://www.sqlite.org/cintro.html).

The proposed sequence is a Lobo application calling an ordinary Lobo SQL helper, which calls a closed
native adapter linked with SQLite. Application behavior and row decoding stay in Lobo. SQLite supplies the
storage engine. Linking a compatible static SQLite build can retain one executable without a Rust server.
This requires a reviewed dependency, build/license provenance and platform validation. No dependency or
build profile is added by this proposal. The compiled Lobo program remains native; SQLite evaluates SQL.

For a first application that needs a remote shared database, PostgreSQL would be the more direct first
target. Its [libpq C client](https://www.postgresql.org/docs/current/libpq.html) is a possible adapter boundary.
That target also requires authentication, allowed destinations, TLS, connection deadlines and transport
failure semantics beyond the existing native slice. SQLite is the simplest recommendation for the current
local demo, not a universal database choice or a concurrency/performance finding.

## Share driver mechanics within each database family

| Family | Proposed public operations | Example later driver |
| --- | --- | --- |
| SQL | Prepare and bind, bounded query, execute, explicit transactions | PostgreSQL through libpq |
| Key-value | Get, put, delete, explicit expiry and conditional operations where supported | Redis through hiredis |
| Document | Find and update documents, explicit filters and indexes | Select only for a concrete application |

These are proposed library families. Do not define one query language that promises identical SQL,
key-value and document semantics. They differ in queries, atomicity, types, consistency and expiration.
Redis documents [hiredis as a C client](https://redis.io/docs/latest/develop/clients/hiredis/).
Share capability policy, quotas, structured diagnostics and audit-event conventions where they fit.

Within SQL, share connection/statement lifecycles, parameters, typed values, rows, errors and transaction
operations. Keep dialect and backend features explicit. SQLite parameter syntax and PostgreSQL's $1-style
parameters are not automatically interchangeable. PostgreSQL documents separate parameter values in
[PQexecParams](https://www.postgresql.org/docs/current/libpq-exec.html). No SQL translation or portable
query builder is proposed for the first driver. Prove the common interface with a second real SQL adapter
before claiming database portability. Keep the initial adapter statically selected. No plugin SDK is needed.

Application functions such as find_user and save_user can hide persistence choices behind application
contracts. That separation does not require an ORM or compiler syntax for tables and queries.

## Define ownership and failures before implementation

The first closed contract should specify the following behavior:

- Connections, statements and transactions have opaque handles and validated lifecycle states.
  Close/finalize invalidates aliases. Driver-owned buffers never escape as raw pointers.
- Parameters are bound separately from query text. A parameter represents a value, not a table name,
  column name or SQL fragment. Dynamic identifiers require a separate allowlisted construction rule.
- Scalar values distinguish NULL, I64, F64, Text and Bytes. Row decoding checks missing columns,
  NULL, invalid encodings, type mismatches and numeric range. SQL NULL is not a missing row.
  Later backend-specific types need explicit codecs and loss rules.
- Queries have row, result-byte, parameter, statement-size and execution limits. The first helper can
  materialize bounded owned rows and finalize the statement internally on success or failure.
- Transactions expose begin, commit and rollback with defined failure states. A helper using try can
  return an operation result to an owner that commits or rolls back explicitly. Invocation fault cleanup
  must roll back active transactions and release resources without relying on remaining user budgets.
- Errors distinguish constraint failure, busy/lock contention, decoding, denied access, exhausted limits,
  cancellation and IO. Preserve backend codes as details. A later remote driver must distinguish an
  unknown commit outcome from a confirmed failed write. Never retry writes solely from a generic error.

Lobo's current Fn callbacks are pure and non-capturing. A callback-based transaction helper that runs
arbitrary database actions would require more design. Use existing functions, result values and explicit
transaction ownership for the first implementation. Try does not introduce automatic scope cleanup.
No new grammar, exceptions, borrow checker or ORM is implied by this proposal.

## Enforce permissions inside the adapter

New database actions would describe possible operations. Explicit grants would authorize a named database
and a role such as read, write or schema administration. These names and grant syntax are not finalized.
Connections retain delegated authority. Action declarations alone never grant access.

For SQLite, specify allowed database files and required journal/WAL/temporary side files. Define path,
symlink, URI and memory-database policy before opening anything. Deny unapproved ATTACH targets, extension
loading and SQL features that escape the selected policy. Inspect actual operations through the engine's
[authorization callback](https://www.sqlite.org/c3ref/set_authorizer.html), not a SQL-prefix string test.
Apply the appropriate [SQLite defensive settings and limits](https://www.sqlite.org/security.html).

Lobo expression fuel does not meter SQLite execution inside one native call. The adapter needs its own
execution controls. SQLite supplies a [progress callback](https://www.sqlite.org/c3ref/progress_handler.html)
that can interrupt work. Design lock waits, IO, allocations and cancellation separately. A progress callback
alone does not prove a hard wall-clock deadline or bound every engine allocation.

A linked C library remains trusted code in the same process. Capability checks constrain calls through the
adapter; they do not sandbox memory corruption or a malicious native library. Stronger isolation is a
separate architecture choice. Audit events should use query identities, operation names, counts and timings.
Do not log credentials, bound values or sensitive SQL by default. No new global logging subsystem is proposed.

## Use a closed test-first milestone

A proposed first milestone is persistent user lookup in the current native REST example using SQLite,
with an explicit database fixture/setup step and a reviewed read/write policy. Success means the same Lobo
application source compiles to a Linux native executable and a Windows executable with a linked adapter.
Actual Windows and Wine results must remain separate. This milestone is not authorized for implementation yet.

Write failing tests against an independent contract before the adapter. Acceptance should cover typed
roundtrips, empty rows versus NULL, binary/Unicode/large values, parameter injection attempts, multi-statement
rejection in single-statement APIs, commit/rollback/persistence after reopen, lock contention and cancellation.
Test denial, read-only/schema roles, ATTACH/extension bypass attempts, side-file policy, stale handles,
resource exhaustion and cleanup after every failure boundary. Test schema mismatch and constraint errors.
Use independent SQLite queries/files to check results, not only the Lobo wrapper's own decoder.

Retain upstream test provenance and license terms for any borrowed cases. Run native C/direct LLVM,
sanitizers and real executable tests. Build a bounded connection/statement/transaction state model if
implementation is selected. Review source correspondence, kill deliberately broken cleanup/commit variants,
and translate counterexamples into adapter tests. Such a model would not prove SQLite, filesystem crash
recovery, the whole language or a future remote driver's network semantics.

No implementation, database connection, dependency installation, performance benchmark or agent experiment
was performed in this review. Primary-source findings and proposed choices are distinguished in the
[source record](../research/database-access-2026-11-23.json). See
[decision 0054](../decisions/0054-database-access-proposal.md).
