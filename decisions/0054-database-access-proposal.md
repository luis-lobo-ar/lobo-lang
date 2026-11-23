# 0054: Database library and driver proposal

Date: 2026-11-23. Status: proposed, not selected for implementation.

The user asked how Lobo should connect to databases and whether to define a SQL/NoSQL layer with drivers.
Recommend a thin SQL library and a SQLite adapter implemented together for the current standalone demo.
Define ownership, bound parameters, typed rows, failures, transaction cleanup, grants and limits first.
A later PostgreSQL adapter can test which SQL operations are genuinely shared.

Use separate interfaces for SQL, key-value and document stores. Share relevant resource-policy and
observability conventions. Keep SQL dialects and driver features visible. Defer a universal query language,
ORM, automatic SQL rewriting, dynamic plugin SDK and pooling until concrete workloads require them.

The existing closed native catalog has no database or file API. Implementing this proposal would require
explicit native catalog/resource, compiler/backend validation, driver dependency/build, library, editor,
skill, test and documentation changes. The current native catalog is not an arbitrary FFI mechanism.
SQLite can be linked into a native application. That does not make an in-process C library a security sandbox.
Try has no automatic function-scope cleanup, and current function-value callbacks remain pure.

This question authorizes investigation and a recorded proposal. It does not reopen the implementation
scope in decisions 0052/0053 or authorize a dependency, ABI operation, language construct or new driver.
Compiler/extension 1.8.1, Core 1.8 and ABI 1 remain unchanged. No runtime tests or database connections were run.
See [the proposal and test-first gates](../docs/database-access-proposal.md) and
[primary sources](../research/database-access-2026-11-23.json).
