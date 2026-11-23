# 0056: Review DX and centralize SQL row validation

Date: 2026-11-23. Status: bounded library improvement implemented; language/tooling recommendations proposed.

The user requested a repository-wide complexity/DX review and allowed utility libraries or fitting language
improvements. The [review](../docs/dx-review-2026-11-23.md) records a tracked-code inventory, compiler/editor
probes, actual recurring patterns, alternatives and a finite proposed sequence. It does not treat brevity
as measured construction, comprehension or token efficiency.

Implement six pure helpers inside the existing SQL library. Keep exact classes, NULL policy, width/index
checks, structured Decode errors, stable identities and the current HTTP boundary. Migrate the SQLite
controller to explicit try calls. Preserve the older REST app, frozen research inputs, native provider,
ABI, language version and compiler semantics. Correct stale current-version guidance and maintain the
programming skill/editor regression at the same time.

A general callback transaction/with-resource helper is not expressible under the current pure Fn rule.
Changing that rule, adding scoped cleanup or changing error conversion needs semantic design and dedicated
failure tests. Immutable record update is the first recommended language prototype because it addresses
45 lexical candidates without introducing mutation or inheritance. A shared project description and richer
diagnostics are higher-priority tooling proposals. No entire roadmap or new ecosystem is implemented here.

The acceptance evidence includes strict row cases on the evaluator, C, LLVM and sanitizers, independent
HTTP invalid-row checks, the original HTTP corpus on Linux/Wine, navigation and real editor-host checks,
current-source modernization and preserved old IDs/fixtures. See the
[result record](../benchmarks/results/2026-11-23-dx-review.json). Retain failed/corrected attempts and report
finite review coverage honestly. Native/provider/model files are not changed by these pure library helpers.
