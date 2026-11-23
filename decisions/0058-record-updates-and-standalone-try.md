# 0058: Implement record updates and standalone try

Date: 2026-11-23. Status: implemented and verified bounded scope, compiler/extension 1.10.0, Core 1.10.

The user explicitly requested implementing both improvements after selecting three-dot record-update
spelling and expressing a preference for standalone try statements. This supersedes the implementation
status of [proposal 0057](0057-record-update-spelling-proposal.md). Preserve that proposal and the pinned
DX review as historical evidence.

Implement one base-first same-type immutable record update and discard of successful prefix-try values.
Preserve exact types, override order, one evaluation, errors/postconditions, actions, native compilation,
existing IDs and frozen fixtures. The [normative rules](../spec/core-v1.10.md) define the contract.
No automatic cleanup, general expression statements, Unit, object/list spreading, implicit conversions,
actionful callbacks or other DX backlog items are selected.

Use existing checked operations. Source inference expands a record update into one fold iteration with
ordered override bindings and complete record construction. This gives a single base owner and keeps
backend/evaluator logic aligned. An alternative duplicating the base expression per copied field would
repeat side effects. Adding a new checked update node would expand the serialized/backend contract when
the existing operations are sufficient. The expansion adds ordinary charged operations and can increase
fuel/artifact size; it is a source convenience, not an optimization claim.

Standalone try uses an unused ordinary binding internally. Preserve the final block value and reject silent
discard of Result calls. Generated names use per-function ordinals rather than source positions, so whitespace
and existing checked-program identity stay independent. Reject temporary source-base fields on artifact load.

TDD covers positive/negative/generic/module cases, order/fault/propagation/postconditions, one native call
under a one-call quota, stable checked bytes under whitespace, original override ranges, editor navigation
and host repair. The migration is limited to the SQLite family and a new independent skill example. Native
provider/model files and the old checked migration checkpoint remain unchanged. Record all outcomes,
including incidental compilation/fixture errors, in the
[result record](../benchmarks/results/2026-11-23-source-conveniences.json).


The user asked about spreading an object with no values during implementation. Clarified that this is a
same-type complete-record update, not dynamic object merging. No overrides preserves the base fields;
empty text/list/None values remain fields and explicit empty values still override. Untyped empty objects
are rejected. The conformance corpus exercises these cases without adding a partial-patch feature.
