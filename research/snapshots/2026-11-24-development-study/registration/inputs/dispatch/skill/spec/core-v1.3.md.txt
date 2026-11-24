# Lobo language 1.3

Compiler 1.3.0 accepts modern source with persistent identity metadata outside the program text.
The syntax, values, types, contracts, actions and execution rules inherit [language 1.2](core-v1.2.md).
Checked format remains `lobo-checked/1.0`. Source compilation emits `core/1.3`.
Artifact admission revalidates core/1.0, core/1.1, core/1.2 and core/1.3.

## Source and identity metadata

```lobo
module sample;
fn increment(n: I64) -> I64 { n + 1 }
```

The file loader reads `sample.lobo` and its adjacent `sample.lobo.ids.json`:

```json
{
  "format": "lobo-identities/0.1",
  "identities": {
    "module": "stable.module",
    "increment": "stable.increment"
  }
}
```

The suffix is appended to the complete filename. Track both files in version control.
Keys bind `module`, each local function/record/variant, and each selected `group.member` import.
The existing ASCII stable-ID syntax and global uniqueness rules apply. Missing or unused bindings fail.
The outer object admits exactly `format` and `identities`. Duplicate keys, unknown fields, invalid values
and unsupported versions fail. A manifest admits at most 2,048 bindings and 1 MiB of UTF-8 JSON.
Source plus metadata share the existing 1 MiB bundle allowance. Each simple metadata binding charges four
tokens; a dotted binding charges six, against the existing 8,192-token aggregate limit.

The 1.2 trailing table remains supported. If both representations are supplied, their maps must match exactly.
This permits extracting existing IDs before removing the source table. External metadata is rejected for
deprecated annotation-based source. Frozen legacy sources remain accepted without external metadata.

`check`, `compile`, `run` and `eval` never allocate or modify identities. Missing metadata is an error.
File inputs must be regular files; the CLI rejects source/sidecar symlinks and special files.
The loader does not discover additional source files or follow imports through paths.

## Explicit authoring commands

```sh
lobo identities init sample.lobo
lobo identities sync sample.lobo
lobo identities rename sample.lobo increment bump
```

All commands accept explicit `--source FILE` dependencies and validate the complete proposed bundle before
writing any metadata. `init` extracts existing table IDs or allocates missing IDs. `sync` adds bindings for
new declarations while preserving existing IDs. Repeating either command without changes preserves file bytes.
Neither command removes a source table or edits source text.

New declaration/module IDs use 128 random bits from `/dev/urandom`, written as `lobo.` plus 32 hexadecimal
digits. They are allocated once, never derived from a name, path or body. This authoring implementation
requires a Unix host with `/dev/urandom`. Compiling existing metadata does not require the random source.

For an unbound selected import, authoring requires exactly one matching explicitly supplied module.
The group must match that module's declared name or source filename stem, and the selected declaration must
exist. Its persistent ID becomes the pinned import target. Existing imports are never silently rebound.

`rename` retargets a local declaration's metadata key after the source declaration and references have been
renamed. It preserves the old ID and rejects ambiguous, missing or conflicting bindings. It does not perform
source refactoring. Changing a module's source name needs no metadata-key change. Move a file with its sidecar.
Stale bindings make `init` and `sync` fail rather than guessing a rename. For deliberate deletion, remove the
confirmed obsolete metadata key in a reviewed edit. There is no automated deletion command.

Each writer uses a create-new `.ids.json.lock`, a synced `.ids.json.tmp` file, and atomic publication of one
sidecar. Initial publication uses a create-new hard link; updates use rename. Cooperating writers cannot
overwrite one another. The source text is rechecked before publication. Readers reject an observed lock.
Multi-file publication is not atomic, and this mechanism is not an OS sandbox against hostile filesystem races.
It does not guarantee recovery after a filesystem or power failure. Retain interrupted lock/temp files for
review, verify the source and manifest, then remove confirmed stale files before retrying. Restore lost IDs
from version control instead of initializing replacement identities for an established module.

## In-memory tools and editor snapshots

`compile(source)` and `check --stdin` perform no filesystem access. Supply inline metadata or use the explicit
`lobo-source/0.2` bundle with optional per-source `identities`:

```json
{"format":"lobo-source/0.2","root":"main","sources":[{"name":"main","text":"module sample; fn increment(n:I64)->I64 {n+1}","identities":{"module":"stable.module","increment":"stable.increment"}}]}
```

The map has the same validation and compilation semantics as the sidecar. Source labels remain diagnostic
names, never files to load. Bundle 0.1 remains accepted without external maps and rejects managed metadata.
The Rust `SourceFile` API has an optional `identities` field. The existing verification-session source object
also accepts that field; absent fields preserve old serialization. Its checked-program evidence includes the
resolved IDs. This is a backward-compatible input extension in the current foreground prototype, not a new
daemon state protocol. Older strict readers may reject it.

Extension 1.3.0 reads adjacent sidecars only for explicit project members or the current workspace file in a
trusted workspace. It overlays unsaved source and metadata documents, watches their exact paths, and
invalidates results on changes. Reads retain bounded, strict UTF-8 and workspace-containment checks.
Managed diagnostics use compiler 1.3 and bundle 0.2. Navigation uses the same supplied IDs and original source
positions. Highlighting works without metadata; managed definition navigation needs its trusted snapshot.
No editor ID allocation, automatic file-move handling or F2 rename provider is implemented.

The [decision](../decisions/0044-managed-persistent-identities.md) records alternatives and limits.
