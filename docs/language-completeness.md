# Current goal: a general-purpose language core

Status: language-only acceptance evidence assembled on 2026-10-18; see the requirement audit below.
Set by the user on 2026-10-17. This is not ecosystem, production, or universal-replacement readiness.
Decision: [0014](../decisions/0014-language-first-development.md).

Current bounded follow-up: [decision 0033](../decisions/0033-bounded-language-1-release.md) authorizes exactly
four families through local language 1.0. Core 0.12 development implements them and passes the three fixed
programs/changes. Language 1.0 freezes this core; its [release receipt](../benchmarks/results/2026-11-04-language-v1-release.json)
records final package/check status. The earlier audit below remains historical evidence.
Definition-time parametric checking now supersedes the instance-only boundary described for Core 0.11.

Follow-up on 2026-11-04: [Core v0.10](../spec/core-v0.10.md) adds List-guarded recursive nominal
values for the [AST development workload](../spec/recursive-ast-workload.md). The historical Core 0.9
audit below is retained as dated evidence, not rewritten to imply the later feature existed then.
Core v0.11 subsequently adds [explicit generic functions](../spec/core-v0.11.md) for the
[public reuse workload](../spec/generic-reuse-workload.md). This is checking per concrete instance,
not universally checked parametric polymorphism or evidence of improved agent outcomes.

Scope correction, 2026-11-04: [decision 0032](../decisions/0032-stop-feature-growth-at-the-evaluation-checkpoint.md)
sets implemented Core 0.11/compiler 0.11.1 as the evaluation checkpoint. Proactive feature additions stop
here. The later generic-data proposal is deferred. Missing facilities become assessment findings; they
do not automatically extend this milestone. The broader research and language ambitions remain long-term.
The [future scope proposal](future-language-scope.md) gives a finite candidate list and explicit later
completion gates. It is not an implementation commitment or an extension of the current checkpoint.

## Outcome

Make native Lobo expressive enough to implement representative self-contained programs whose computational
logic would otherwise be written in a general-purpose language. Build a small set of composable facilities,
not a checklist of constructs copied from other languages. Add a construct when demonstrated programming
patterns justify it; expressiveness does not require identical syntax, object models, or control structures.

This is a language milestone, separate from application libraries and the eventual agent workspace. It does
not promise replacement of every language's deployment model, hardware access, ecosystem, performance, or
specialized domain facilities. Those require separate requirements and evidence.

## Current boundary

Core v0.11 has scalars, UTF-8 text/byte buffers, bounded immutable records/variants, persistent growable lists,
exhaustive matches, bindings, branching, functions/recursion, state-driven iteration, counted
folds, contracts, and checked action bounds. It compiles natively and has a reference evaluator and editor support.
Expected failures can be returned as user-declared variants. Closed multi-module checking/native linking is
implemented experimentally, with explicit-source editor navigation/checking and collection/tree helper sharing.
Named recursion through Lists and explicit generic functions are implemented; direct inline cycles remain
rejected. Generic nominal types and function values remain absent. Action bounds do not provide resource operations or grants.

## Required capabilities, not prescribed syntax

- Represent compound and alternative data, including extensible structures, rather than encoding all
  application state into a fixed number of scalar parameters.
- Construct, inspect, transform, and traverse data; define allocation, copying/sharing, ownership/lifetime,
  equality, bounds, and exhaustion behavior before relying on them in native lowering.
- Express data-dependent computation through existing recursion or justified control constructs. Do not
  introduce every style of loop or mutation merely because another language has it.
- Return expected failures as ordinary typed values and handle alternatives explicitly.
- Reuse and compose algorithms and organize declarations without requiring copies of application logic.
  Specify any later generic/function-value/module facility from demonstrated limitations of the smaller core.
- Preserve explicit action bounds and pure contracts as the data model grows; any new authority still needs
  its own design. Local value allocation is not permission to introduce ambient external access.

Some of these can share one underlying mechanism. Records, tagged variants with matching, and growable
sequences are candidates, not a frozen grammar or an instruction to implement them all in one change.

## Working order

1. Define the structured-value and memory model; start with a bounded record-value slice that can pass,
   return, select fields, and carry structured state through folds.
2. Add alternative values and their elimination/handling, driven by typed failures and tree-shaped data.
3. Add growable data and text/bytes with specified allocation, lifetime, indexing, and budget behavior.
   Revisit steps 1–2 as needed before recursive or heap-backed layouts are admitted.
4. Address reuse and program organization as representative programs expose concrete limitations.
5. Evaluate language coverage and identify repeated awkward patterns before adding further constructs.

The order is revisable. No particular syntax, ownership strategy, package system, concurrency model, or
compiler release is committed by this document. Each implementation slice needs its own normative rules,
failing tests, native/reference evidence, and matching editor support.

The first bounded record slice is now specified and implemented in [Core v0.4](../spec/core-v0.4.md), using
immutable value copies and bounded inline native layouts; see [decision 0015](../decisions/0015-immutable-record-values.md).
Alternative values and exhaustive handling are implemented in [Core v0.5](../spec/core-v0.5.md), including typed
failure returns; see [decision 0016](../decisions/0016-tagged-alternative-values.md). Persistent growable lists,
allocation accounting, and bounded serialization are implemented in [Core v0.6](../spec/core-v0.6.md) and
[decision 0017](../decisions/0017-persistent-growable-lists.md). Text/bytes are implemented in
[Core v0.7](../spec/core-v0.7.md) and [decision 0018](../decisions/0018-text-and-byte-values.md). The arithmetic
parser workload now uses these facilities without compiler changes; see
[decision 0019](../decisions/0019-parser-workload-in-existing-lobo.md). The tree workload now uses ordinary
node-table values; see [decision 0020](../decisions/0020-tree-workload-in-existing-lobo.md). State-driven
termination is implemented in [Core v0.8](../spec/core-v0.8.md) and [decision 0021](../decisions/0021-immutable-state-driven-iteration.md),
including converted parser reduction/tree search and an early-match list search. Closed module composition
and explicit-source editor integration are implemented in [Core v0.9](../spec/core-v0.9.md) and
[decision 0023](../decisions/0023-explicit-editor-source-snapshots.md). Recursive named types remain unsupported.
Unicode properties/normalization/grapheme processing are not implied by basic text support.

## Acceptance evidence

Freeze acceptance inputs and outcomes before implementation. Implement at least three materially different,
self-contained workloads in Lobo, such as:

- parsing an in-memory input into structured data and evaluating it;
- transforming/filtering/aggregating collections and reporting malformed input as typed failures;
- constructing and traversing a graph or tree with data-dependent state.

Exact cases are now frozen in [the workload specification](../spec/language-milestone-workloads.md). Collection
workload B and its odd-predicate variation run in [collections.lobo](../examples/collections.lobo), with native/
reference/sanitizer evidence. Parser workload A runs in [parser.lobo](../examples/parser.lobo), including checked
arithmetic/typed offsets and independent expression variations. Tree workload C runs in
[tree.lobo](../examples/tree.lobo), with dynamic construction, parent edges, in-order/reverse traversal,
height, and duplicate handling. Inputs may be supplied by the invocation/test harness;
the harness must not perform the computation on Lobo's behalf. No Rust/Python/Node escape hatch for the
workload's logic, compiler edits tailored to individual cases, or mandatory network/database dependency.

For each workload, verify native output against independent expectations and the reference path; include
negative/boundary/exhaustion cases, preserve structured diagnostics and artifacts, and exercise corresponding
editor behavior. An agent should be able to implement a variation using the supported language without
requiring a new compiler feature for that variation. Report remaining gaps rather than declaring universal
replacement from three successful examples.

Turing completeness is a separate theoretical claim about an abstract computation model. Do not infer it
from recursion, benchmarks, or finite tests; any such claim needs an appropriate simulation argument and
explicit storage assumptions. Real execution may continue to enforce finite resource budgets.

## Coverage assessment after the three workloads

Historical Core 0.9 assessment, retained from the 2026-10-18 milestone. The Core 0.10 follow-up above
supersedes its recursive-data boundary only; other limits and the original evidence remain recorded.

The workload gate alone was not the entire language goal. Current evidence separates capability from limitations:

| Required capability | Evidence | Remaining boundary |
| --- | --- | --- |
| Structured and growable data | Parser state/stacks, collection summaries/errors, actual tree parent edges | Recursive named values are unsupported; trees currently use an allowed node table. |
| Construction and traversal | All three executable workloads; independent outputs and native/reference/sanitizer checks | Tree child lookup scans the node table; this is not a performance claim. |
| Data-dependent computation | State-carrying while, parser reductions, tree search/traversal, early-match search, numeric recursion | Loops use fuel without growing call frames; ordinary recursive calls retain the 32-frame budget. No termination proof is implied. |
| Typed expected failures | Parser offsets/overflow and collection first-invalid alternatives | Runtime faults/resource exhaustion remain diagnostics, not catchable application values. |
| Reuse and program composition | Closed source bundles share functions/nominal types; actual collection/tree/module callers use one reverse/cursor; explicit-project editor navigation/checking | No package resolver, user generics, or function values; add abstractions only for demonstrated needs. |
| Actions and native behavior | Empty workload action sets; existing action/pure-contract corpus and native gates | No host resource access, grant, OS sandbox, or ecosystem facility is implied. |

Two concrete gates followed from existing requirements, not a feature-parity checklist. Both now have evidence:

1. Remove the demonstrated cross-file helper duplication through explicit program composition. Two callers
   must share one implementation and its nominal data types, with unambiguous identities, deterministic
   checking/lowering, missing/conflicting dependency failures, and maintained editor navigation. Choose
   source loading/authority rules before implementation; do not quietly add package downloads or ambient
   filesystem/network resolution. The compiler/API/CLI portion is implemented in [Core v0.9](../spec/core-v0.9.md)
   and [decision 0022](../decisions/0022-closed-module-composition.md), with ownership/import revalidation and
   native/reference/sanitizer fixtures. Collection/tree/module callers now delegate to `examples/sequences.lobo`.
   The public reverse entry IDs remain; a compatibility wrapper costs ordinary call/contract fuel, not free
   host work. Existing resource oracles and allowances pass without increases. One linked Cursor remains;
   unused internal `example.tree.cursor`/`example.modules.cursor` definitions were removed. Explicit-project
   editor tests cover remote nominal targets, dependency overlays/errors, staleness, trust and external changes.
2. State-driven termination: implemented as immutable carried-state `while` in Core v0.8 after comparison
   with recursive/tail-call and mutable-loop alternatives. Parser reduction/tree search stop without a
   predetermined count; early-match search returns using the same execution fuel on short/long unused tails
   after input admission. Native/reference/sanitizer tests cover exact fuel/heap, contracts, actions, and
   failure ordering. Some counted folds remain appropriate; converting all folds is not a requirement.

User generics, function values, recursive types, and additional numerical constructs remain possible later
responses to demonstrated needs, not automatically required implementations. The full audit follows;
no universal replacement claim follows.

## Language-only requirement audit — 2026-10-18

Scope is the user-selected general-purpose computational core, not the later workspace/product target.
The acceptance cases above were frozen before their implementations. This audit checks those requirements
and the additional composition/termination pressures; it does not redefine success as merely passing a suite.

| Requirement | Authoritative implementation and observed evidence | Assessment |
| --- | --- | --- |
| Native machine code, not interpreted application logic | `src/native.rs`, C runtime, `tests/native.rs` CLI/default corpus, independent generated native corpus; direct collection/tree executable smokes below | Met on advertised Linux x86_64/Clang target. Evaluator is a separate correctness reference. |
| Structured, alternative, and extensible data; specified lifetime/sharing/equality | Core 0.4–0.7; records/variants/lists/buffers implementation and `tests/records.rs`, `variants.rs`, `lists.rs`, `text.rs` | Met with immutable records/variants, persistent Lists and UTF-8/octet buffers. Named layouts are acyclic; extensible trees use node tables. |
| Construct/inspect/transform/traverse rather than fixed scalar encodings | Full parser stacks/state/errors, collection summaries, actual parent-index tree edges; `tests/workloads.rs`, `lists.rs`, `tree.rs` | Met. No host parser, sort/tree routine, FFI, or fixture-specific intrinsic supplies the computation. |
| Data-dependent general computation | Functions/branching/recursion, fold and immutable-state while; numeric algorithms, parser reduction, tree search, early-match variation in `tests/while.rs` | Met within explicit fuel/heap/call-depth budgets. State-driven iteration does not grow call frames; no termination proof is implied. |
| Expected failures as ordinary typed values | Variant matches, first-invalid collection index/value, parser byte offsets/overflow; exhaustive/rejection/fault tests | Met. Runtime faults and exhaustion are structured diagnostics, not caught application values. |
| Reusable abstractions and program organization without copied algorithms | `src/modules.rs`, selected stable-ID aliases, canonical ownership IR; 17 module tests plus shared collection/tree algorithm/cursor and ordinary wrappers | Met. Imports preserve nominal identity, actions, and direct visibility. Source sets are closed caller inputs; no resolver authority. |
| Semantics before features; constructs justified by patterns; TDD | Core specifications 0.4–0.9, decisions 0015–0023 and ledger record data model, parser/tree pressures and observed failing regressions | Met for these slices. Loops/modules address stopped-state/duplication evidence; no feature-parity requirement was introduced. |
| Native/reference conformance, negative/boundary/exhaustion evidence | 129 Rust tests in each debug/release profile, reloaded artifacts, independent expectations/generated cases, ASan/UBSan with leak checks and active-instrumentation probes | Met for advertised semantics and finite corpus, not a proof over every program. Frozen numerical/workload expected results were preserved. |
| Maintained editor and agent access | Extension 0.11.0, 103 Node tests, real VS Code definition/diagnostic command checks; CLI source sets and no-IO agent bundle checking with empty PATH | Met. Remote function/type/field/case catalog targets, unsaved overlays, cancellation, bounds, trust and source attribution are tested. Structural navigation is not a second authoritative type checker. |
| Stable diagnostics, compatibility and history | Core/artifact 0.9 load revalidation/version rejection, exact source labels/ranges, existing standalone executables, CHANGELOG and decisions | Met. Old checked artifacts require recompilation; no retagging. Normal editor installation/publication is not automatic. |
| Preserve explicit actions and authority boundaries; defer ecosystem | Action/contract corpus and imported-edge tests; no compiler/runtime dependency addition, resource operation, FFI, package downloader or daemon in this slice | Met. Editor dependency IO is explicitly configured/trusted; no Lobo runtime capability is granted. Native execution is not an OS sandbox. |
| Separate expressiveness from Turing completeness | Resource limits remain specified; no abstract-machine simulation proof or unbounded-storage assumption was supplied | Met by not claiming Turing completeness. Finite workloads do not prove it. |

Verification commands: `cargo test --locked --quiet`, `cargo test --locked --release --quiet`,
`cargo build --locked --release --bin lobo`, `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, `git diff --check`,
`npm --prefix editors/vscode run package`, and the real host command in the extension README. All pass
locally. The final host reports its completion marker and exits 0. Actual collection/tree sources also have
bundle-definition fixtures, alongside the real-host unsaved/external dependency scenarios.

Direct native CLI smoke results: collection `[5,2,8]` with the even predicate yields `[2,8]`/count 2/sum 10;
tree `[3,1,3,2]` yields `[1,2,3]`/count 3/height 3. Source imports are supplied explicitly. The stripped
executables are 18,784 and 31,080 bytes. Exact source/artifact identities and failure findings are in
[decision 0023](../decisions/0023-explicit-editor-source-snapshots.md).

Conclusion: the defined language-first milestone has the required local evidence. Lobo can express these
representative self-contained programs natively with composition and matching tools. It is still experimental:
finite frontend/runtime limits, invocation-scoped arena retention, acyclic named layouts, I64/Bool numerical
primitives, and a structural editor catalog are material limits. Generics/function values, richer numeric types,
recursive named data, optimization and broader workloads remain possible language evolution, not claims of
implemented support. This is neither universal replacement nor the application/agent-workspace v1.0 gate.

## Deferred, not abandoned

Network/storage/search integrations, general FFI, a third-party package ecosystem, application hosting,
runtime capability/quota/audit adapters, and the semantic-workspace daemon/context engine come later.
They are not required to pass this language-only milestone and are not claimed implemented. Native
compilation, TDD/conformance, maintained VS Code support, compatibility/versioning, and the change/decision
ledger continue throughout. Self-hosting and additional backends are not prerequisites.
