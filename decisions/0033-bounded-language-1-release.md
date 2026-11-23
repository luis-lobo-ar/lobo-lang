# 0033: Bounded language 1.0 release

Date: 2026-11-04. Status: implementation authorized and goal active; release not yet achieved.
The user requested implementation through completion and version 1.0, then explicitly made it the goal.
This opens exactly the four-family scope in `docs/future-language-scope.md`, superseding decision 0032's
feature freeze for that scope only. The prior baseline and paper evidence remain immutable.

Implement generic records/variants; definition-time checking under declared type requirements; pure,
non-capturing named function references with checked signatures/action bounds; and binary64 F64 arithmetic,
conversions, comparisons and interchange. Keep concrete specialization and native compilation. No new Cargo
dependency or runtime model is assumed. Source and artifact compatibility may break and must be versioned.

The private pointer ABI of the LLVM backend remains an implementation detail. Both native routes and the
reference evaluator must implement every admitted type/construct. Definition-time checking establishes
name/type validity under declared requirements, not contract truth or universal termination. Concrete layout,
resource admission and code generation are rechecked after specialization.

Freeze the three public acceptance programs and planned changes in `spec/language-v1-acceptance.md` before
production changes. Keep arbitrary host-library logic out of the programs. Existing specifications remain
the authority until the matching v1 rule supersedes them. Each slice needs real failing-before evidence,
source/artifact/module/resource tests, native/reference/sanitizer agreement, editor and skill maintenance.

The release is local language 1.0, not the broader agent-workspace/product 1.0 or a publication. Assign the
final version only after all four families, three programs/changes and documented release checks pass.
External host services/FFI, capturing closures, concurrency, macros, self-hosting, further backend tuning,
paid model trials and publication remain outside scope. Completion does not require outperforming Rust or
Python; no agent construction/comprehension result is implied by compiler conformance.
