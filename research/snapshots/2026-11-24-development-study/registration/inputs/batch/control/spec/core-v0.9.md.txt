# Lobo Core v0.9: explicit module composition

Status: implemented experimentally in compiler/API/CLI and extension 0.11.0, including collection/tree
shared helpers. Specified before implementation; broader language coverage is assessed separately.
Versions: `core/0.9-draft`, `lobo-checked/0.9`, compiler 0.9.0. Inherits [Core v0.8](core-v0.8.md).
Native invocation/adapter 0.5 is unchanged. No package registry, filesystem resolver, or host operation is added.

## Source imports and identities

After a module header, top-level declarations may include `import local_name @id("declaration.id");`.
The alias may name a function, record, or variant from another supplied module. Top-level declarations are
importable; there is no visibility/export syntax yet. Imports are direct, not wildcard imports or implicit
re-exports. A dependency's dependencies do not automatically become visible names. Imports have no runtime
initializers, actions, or authority. Declaring an imported actionful function still requires its bounds on
every caller; contracts still require pure callees.

Module and declaration IDs are globally unique across the supplied program. Source names are module-local;
two modules may each declare `Pair` or `reverse`, with distinct IDs. An imported alias shares the local
declaration namespace and cannot collide with declarations/imports or visible bindings. Importing a module
ID, an unknown declaration, or a declaration owned by the importing module is invalid. Dependencies may be
mutually recursive because linking checks signatures before bodies and there is no initialization order;
recursive named data layouts are still rejected under the existing rules.

All function calls, constructors, explicit type annotations, and list element types must resolve through
the module's own declarations or explicit imports. Inferred nominal types may flow through imported function
signatures and field selection without an additional alias, but naming/constructing that type needs an import.
Imported nominal types preserve the original identity; copying a declaration is not required or equivalent.
Ordinary function/parameter names `import` remain legal outside the top-level import form.

## Closed source set and limits

The compiler API `compile_bundle(root, sources)` takes a root source label and explicit `{name, text}` sources.
Labels identify diagnostics, not paths to open. The library performs no source IO, import searching, network
access, or mutation. Validate all supplied modules, including unused ones; a root selects program identity,
not a private/export boundary. Every function remains an invocable stable-ID entry, as before.

Accept 1–16 uniquely labeled sources, at most 1 MiB UTF-8 source bytes and 8,192 lexical tokens in aggregate.
Bound linked declaration/import metadata to 1,024 slots before constructing validation contexts, including
on artifact load; source token admission already implies that bound for valid source programs. Module
contexts are shared among their declarations rather than copied per declaration.
Retain per-expression/nominal layout/temporary budgets across the whole linked program. Module/declaration
IDs must not conflict; a source can contain only types/imports, but a linked program needs at least one
function. Single-source `compile` retains its public behavior and unlabelled diagnostics.

Canonicalization orders modules by module ID and each declaration category by its module's source order. Input source
ordering, labels, comments, and coordinates are not semantic identity. Reordering the supplied source list
with the same root yields identical artifact JSON and emitted C. No moving revisions or hidden resolver inputs.
Syntax/body diagnostics locate the correct supplied text and label. Unattributable cross-module structural
errors must remain unlocated rather than attach another file's coordinates.

## Checked artifact

The existing flat native/evaluator declaration tables remain. `program.scopes` records each module's `name`,
`id`, owned `declarations` (stable IDs), and `imports` (`name`, target `id`). `program.module`/`id` name the
root. Every declaration has exactly one owner; every scope/alias/target and explicit reference is revalidated
on load. Local names are not globally mangled, and IR cannot bypass visibility merely by spelling a stable ID.
All source labels/spans are ephemeral, not paths or serialized authority. Old artifacts require recompilation.

## Compiler/agent tools and editor snapshots

The compiler CLI accepts explicit additional files with `--source FILE` on check/compile/run/eval. All
source bytes are read before linking, within the aggregate allowance; at most 15 additional files accompany
the root. Byte admission precedes UTF-8 decoding at this read boundary, including oversized malformed files.
Checked artifacts are self-contained and reject additional source options. `check --stdin` remains
single-source with no additional arguments or source IO. `check --bundle-stdin` accepts this JSON shape:

```json
{"format":"lobo-source/0.1","root":"a.lobo","sources":[{"name":"a.lobo","text":"module a @id(\"a\"); function f @id(\"f\")() -> I64 { 42 }"}]}
```

The envelope is capped at 4 MiB before UTF-8 decoding, then sources obey the aggregate language limits.
Unknown fields/versions, duplicate labels, missing root, and unsupplied imports fail explicitly. This path
only checks caller-supplied text, without files, Clang, execution, or temporary output. The equivalent library
entry is `compile_bundle_json`; `compile_bundle` accepts typed `SourceFile` values directly.

Extension 0.11.0 provides remote function/type/field/case definition targets and dependency-aware compiler
checking for explicit `lobo.sourceFiles` lists. Lists are complete closed source sets of workspace-relative
`.lobo` paths, not import resolution maps. Imports are never paths. Only listed documents participate;
ordinary files retain single-source support. Trust is required for dependency IO and compiler execution.
Reject traversal, globs, duplicate paths, unsupported URI schemes and canonical targets outside the workspace.
Read bounded regular files as strict UTF-8, overlay unsaved documents, preserve exact target/error sources,
and invalidate snapshots on dependency/configuration changes, including external file events. Never map a
dependency's range onto another file. Existing process cancellation/output/time/concurrency limits remain.
The structural navigation catalog is not the compiler's type checker. Invalid/ambiguous shapes fail closed;
no workspace scan, package download, compiler-driven execution, or temporary source write is introduced.
See [decision 0023](../decisions/0023-explicit-editor-source-snapshots.md) for boundaries and evidence.

Acceptance: collection/tree callers share one reverse helper and cursor type in `examples/sequences.lobo`;
stable public function IDs survive moving ownership, and compatibility wrappers contain no copied algorithm.
Imported nominal values,
aliases, local-name collisions across modules, dependency failures, action/contract propagation, artifacts,
source attribution, deterministic linking, native/reference/sanitizer output, budgets, and maintained editor
behavior. The full language goal also requires its final coverage audit; finite tests do not imply Turing completeness.
