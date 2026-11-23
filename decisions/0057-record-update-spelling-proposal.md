# 0057: Prefer three dots for the record-update proposal

Date: 2026-11-23. Status: recommendation, not implemented or a completed language milestone.

The user asked whether the record-update proposal should use three dots, noting JavaScript familiarity.
The user also expressed a preference for the separate standalone try-statement improvement.
Recommend `Search { ...state, duplicate: true }`. This updates the working recommendation without changing
[the earlier pinned DX review](../docs/dx-review-2026-11-23.md). It does not implement either proposal.

[Rust](https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax) uses
`Search { duplicate: true, ..state }` for struct update, with the base last.
[JavaScript](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax)
uses three dots in object literals and allows the base before explicit properties. Prefer that familiar
base-first spelling for Lobo. This is a design judgment, not evidence of better agent performance.
The [source record](../research/record-update-spelling-2026-11-23.json) retains findings and limits.

Keep Lobo's proposed semantics bounded: one base first, the same nominal record type, one evaluation of
that base, unchanged fields retained, explicit fields overriding it, immutability preserved, and repeated
explicit or unknown fields rejected. Three dots do not imply arbitrary object merging, multiple bases,
array spreading or variadic arguments. Either spelling could lower to the same checked operations.
No token-cost, comprehension or runtime comparison was measured.

Standalone `try operation();` remains a separately proposed source rule: discard the successful value and
propagate errors under existing try rules, while retaining a final expression for the block's result.
Any implementation needs TDD, compiler/editor/skill synchronization and its own versioned language contract.
