# Frozen repair-study source inputs

These three files preserve the exact source bytes that the repair-study driver read before the maintained
examples migrated to language 1.2 syntax. Their IDs, task transformations and seeded faults are unchanged.
The driver now reads these fixed copies instead of mutable application examples.

The source identities and SHA-256 values are pinned in `scripts/source-migration.json` at the repository root.
The migration verifier checks the copies before accepting the maintained corpus. Do not migrate or reformat
these research inputs. Existing archived study packages remain unchanged.
