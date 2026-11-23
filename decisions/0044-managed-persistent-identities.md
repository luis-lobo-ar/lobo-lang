# 0044 Persistent identities move out of ordinary source

Date 2026-11-22. Status implemented. Risk mode standard.

The user selected automatic identity bookkeeping and requested the final MVC example without visible IDs.
This reopens only the authoring metadata, compiler input, editor and documentation scope. Variant semantics
remain unchanged. The full semantic workspace remains future work.

## Alternatives and selected boundary

Deriving IDs from names, paths or source hashes would silently change identity during ordinary maintenance.
That contradicts persistent references and evidence. A central semantic database could own this state, but
would require unrelated workspace lifecycle and protocol work. A tracked adjacent sidecar supplies durable,
reviewable state now with explicit source bundles for agents.

Select `.lobo.ids.json`, explicit `identities init`, `sync` and metadata-only `rename`, and source bundle 0.2.
The [1.3 contract](../spec/core-v1.3.md) defines allocation, validation, publication and failure behavior.
Checks/builds stay read-only. Authoring validates the full proposal before sidecar writes.
Keep inline tables for compatibility and staged extraction. An inline/external pair must agree exactly.

IDs are assigned once using OS randomness, then retained. New imports resolve only from the explicit source
closure, with exactly one matching module. Existing imported targets remain pinned. Stale metadata fails.
Names identify bindings within the sidecar but do not generate persistent identity values.

## Acceptance and scope

TDD covers pure managed snapshots, duplicate/missing/stale/conflicting metadata, read-only checks, one-time
allocation, idempotent authoring, explicit import allocation, declaration renames, source/sidecar moves,
interrupted locks and malformed state. The MVC retains all twelve bindings and exact checked-program equality
with its frozen legacy representation. Evaluator, C/LLVM and both sanitizer routes retain 200/404 behavior.
Editor tests include unsaved metadata, original positions, invalidation and real extension-host navigation.
The [result record](../benchmarks/results/2026-11-22-managed-identities.md) retains all attempts and limits.

Only the maintained MVC and a skill example move to sidecars in this change. Other maintained inline sources
continue to work. Frozen fixtures, historical specifications, reports and releases remain unchanged.
Language/compiler advance to 1.3, extension to 1.3.0. Checked format and runtime behavior remain unchanged.

This is explicit tooling, not a transactional semantic workspace or automatic semantic refactoring.
Moving a file requires moving its sidecar. Source renames precede the metadata rename command.
Deletion needs a reviewed metadata edit. Cooperating writer locks and atomic single-file publication do not
provide atomic multi-source commits, hostile-filesystem isolation or guaranteed power-loss recovery.
The authoring allocator currently requires `/dev/urandom` and a filesystem supporting hard links and rename.
Those limits are intentional and documented. No dependency or new runtime is added.

No agent token, construction-quality or comprehension result follows from removing visible IDs.
Those research outcomes remain unmeasured. Stop at this selected scope.
