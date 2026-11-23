# SQLite test provenance and adaptations

The five unchanged Tcl files come from the official SQLite 3.54.1 source ZIP. The ZIP SHA3-256 was verified
against the official download page before extraction. [provenance.json](provenance.json) records the archive
and individual file hashes. Upstream source disclaimers and notices remain in each file.

These files are source material for adaptations, not a claim that Lobo ran the complete upstream Tcl suite.
The actual C adapter corpus is [sqlite_harness.c](../../native/sqlite_harness.c). It adapts bound value/NULL/
empty-buffer behavior, authorization denial and transaction persistence/rollback patterns. Lobo-specific
capabilities, limits, token generations and fault cleanup add independent cases.

- bind.test bind-1 and bind-2 families inform prepared parameter binding and value separation.
- bind.test bind-13 families inform explicit NULL behavior and repeated parameter semantics.
- auth.test auth-1 families inform denied operations through the engine authorizer.
- trans.test trans-1/trans-2 families inform independent connection observations and transaction rollback.
- misc1.test and malloc.test are retained for additional boundary/failure review. Their complete cases are
  not currently executed or claimed as adopted coverage.

The adapter deliberately differs from raw SQLite: every parameter must be bound; statements do not reset
implicitly; only one statement and a restricted SQL policy are admitted; and returned values are copied and
validated against Lobo's finite F64/UTF-8 invariants. SQLite-side expectations are checked through direct C
calls and separate Python sqlite3 reads. Failures and exact executed cases are recorded in the release evidence.
