# Checked artifact v0.3

Historical format, extended by the current format in the [specification index](README.md).
The version-specific rules below describe the 0.3 reader, not the current loader.

Format `lobo-checked/0.3`, language `core/0.3-draft`. Inherits [v0.2](checked-artifact-v0.2.md), adding a
mandatory `actions` array of strings to each function. Empty arrays represent pure bounds. Arrays contain
at most 64 IDs satisfying [Core v0.3](core-v0.3.md), strictly increasing in ASCII lexicographic order.
Unknown fields, missing arrays, duplicates, invalid IDs, noncanonical ordering, and bound violations are
rejected on reload. The loader never trusts a producer's checked claim. No cached inferred action set or
authority grant is serialized. Calls still target stable function IDs; source positions remain ephemeral.

Readers accept only this format/language pair. Recompile source from older versions rather than retagging
an old envelope; the new mandatory field and checks require a new artifact version. Native executables
contain neither the tree nor an interpreter. This is still not canonical semantic workspace graph storage.
