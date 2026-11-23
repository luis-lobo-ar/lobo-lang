# Pinned SQLite source

SQLite 3.54.1 is the first storage-engine dependency. The
[implementation decision](../../decisions/0055-first-sqlite-driver.md) records the need and alternatives.
The official amalgamation ZIP was checked against the SHA3-256 on the SQLite download page before
extraction. [provenance.json](provenance.json) pins its URL, archive identity and file hashes. Source headers
are unchanged. [NOTICE.txt](NOTICE.txt) records public-domain provenance.

[src/sqlite_profile.h](../../src/sqlite_profile.h) defines the reviewed single-threaded profile. The compiler
embeds the source only into SQLite-selected C/runtime exports. This grows the compiler by the dependency's
source data and increases database build work. Ordinary non-database programs do not embed the engine.
No system SDK, runtime database DLL or download step is required for a compiled database program.

The provider, owned buffer conversions, generic Lobo helper, native backends and executables are tested.
Selected upstream test sources are retained under tests/upstream/sqlite with adaptation provenance.
The complete SQLite upstream test suite, every compile-option combination, exhaustive fuzzing and arbitrary
filesystem crash recovery have not been verified by this project.
