# Lobo change ledger

## 2026-11-23 — Freeze 1.11.2 and run matched agent experiments

Frozen Core 1.11/compiler-extension 1.11.2 at tag `research/lobo-1.11.2`, with no product changes.
Locally registered matched tasks, budgets and source-only comprehension; all 54 primary attempts pass.
Lobo has higher median paired construction token/time costs, and no quality advantage is established.
A separately retained infrastructure restart preserves five completed attempts, one interrupted attempt
with unknown usage, and 48 unattempted assignments. All 270 builds, 162 warmups and 1134 timed runtime
runs validate; native hardware counters remain unavailable. Public source bundles, hashes, profiles and
paper entry local-038 preserve every outcome. See [the full report](research/matched-agent-study-2026-11-23.md).

## 2026-11-23 — Reassess the research across language iterations

Reviewed the original construction/comprehension/cost hypotheses against compiler 1.11.2. Fresh local
checks pass 750 calibration observations, reproduce fifteen mutation-probe outcomes and six affected/full scenarios, and validate twenty
MVC controller invocations, 31 benchmark-harness tests, four accounting tests, three native/SQLite witness
and correspondence tests, and 182 editor tests. Repeated C/LLVM and five-way arithmetic observations
remain scoped runtime results. Source/metadata accounting and hash-matched knowledge snapshots expose
tradeoffs; no new agent-language advantage, token-saving estimate or comprehension score is claimed.
Original reports, failures, unknown usage and frozen fixtures are preserved. See
[the review](research/reassessment-2026-11-23.md) and [data](benchmarks/results/2026-11-23-research-reassessment.json).

## 2026-11-23 — Publish current language and corrected history

Published all pending source/docs in commit 37a6f1ba598a and atomically updated all
three branches and both baseline tags with explicit remote leases. GitHub confirms that every source-history
author and committer belongs to luis-lobo-ar, and the repository is public. The changelog, original checkpoint
evidence and three worktrees are preserved. Private evidence and generated binary caches stay local.
See [the publication receipt](benchmarks/results/2026-11-23-git-publication.json).

## 2026-11-23 — Correct Git identity and simplify example commands

Restored the missing per-directory lobo-ar Git config and verified a fresh repository uses the personal
luis-lobo-ar identity. Repository-local author/committer settings and push credentials use the same account.
At the user’s explicit request, rewrote the six existing commits across branches and tags while preserving
trees, messages, dates and all three worktrees. Original history remains in a local backup bundle;
[the mapping](benchmarks/results/2026-11-23-git-history-map.json) connects frozen checkpoint IDs to new IDs.
The README examples now use lobo directly. All 20 Lobo commands pass; the two remaining Cargo commands
are confined to compiler build/test instructions. Earlier changelog entries and pinned evidence remain unchanged.
See [the identity and example checks](benchmarks/results/2026-11-23-git-identity.json).

## 2026-11-23 — Make the GitHub repository public

Changed luis-lobo-ar/lobo-lang from private to public at the user’s explicit request. Verified the setting
through the owner account and an anonymous GitHub API request. Existing local work remains unpushed.
See [the visibility receipt](benchmarks/results/2026-11-23-repository-public.json).

## 2026-11-23 — Implement typed permissions

Compiler/extension 1.11.2 add separate OS/SQL Permission variants and finite Permissions sets on unchanged
Core 1.11. HTTP apps use server_permissions(); the snapshot names ClockRead/RandomRead; SQL delegates one(Read).
Old raw helper IDs/signatures remain at restrict_mask. The 46-source role audit rejects raw delegation in
maintained consumers; build/editor closures, programming skill and guides include the new module.
The complete suite passes 312 Rust tests and 182 editor tests plus the real host.
TDD failures, corrections, full verification and preserved evidence are recorded in
[the result](benchmarks/results/2026-11-23-typed-permissions.md) and [decision 0062](decisions/0062-typed-permissions.md).
No grammar/provider/model expansion or agent-performance claim is made.

## 2026-11-23 — Propose named capability permissions

Clarified that the HTTP sample's raw mask 5 selects clock=1 plus listen=4 from existing root authority.
Recommend an existing-variant Permission enum and a typed permission set/library profile, with raw numeric
conversion confined to the ABI adapter. Capability tokens remain opaque. This is a proposal only; no
language, library, runtime or test behavior changes. See [decision 0061](decisions/0061-typed-native-permissions-proposal.md).


## 2026-11-23 — Complete the maintained HTTP refactor

Compiler/extension 1.11.1 retain Core 1.11. Both native HTTP apps use library-owned listeners; one accept/retry
state machine and shared preparation/close policies support thin exact-bound callback adapters. The old serve
ID/signature remain at serve_listener, and all prior IDs/frozen evidence are preserved. The role audit covers
45 maintained Lobo files and rejects the old scaffolding. 309 Rust/180 editor tests, the real host,
minimal-handle C/LLVM/Wine HTTP runs and persistence checks pass. A valid 1025-poll mutant is rejected.
Version verification also catches and repairs current and historical editor lock metadata corruption without
changing installed dependencies. Old snapshots remain. No language, native provider or model change is made.
See [decision 0060](decisions/0060-complete-maintained-http-refactor.md),
[report](benchmarks/results/2026-11-23-maintained-http-refactor.md) and
[receipt](benchmarks/results/2026-11-23-maintained-http-refactor.json).


## 2026-11-23 — Clarify HTTP refactor coverage

A follow-up source audit confirms the SQLite extraction is complete. The original pure REST app still
owns listener open/close, and the library retains separate pure and DB-bound context drivers over shared
transport helpers. Low-level native demos and frozen fixtures remain intentional exclusions. No additional
implementation or language expansion is made. See the [scope audit](benchmarks/results/2026-11-23-http-refactor-scope.json).


## 2026-11-23 — Core 1.11 callback actions and HTTP library extraction

Implemented explicit finite action bounds in named callback types, exact signature revalidation and checked
indirect invocation in compiler/extension 1.11.0. Omitted bounds stay pure. The HTTP library takes explicit
context for the concrete DB handler bound, owns serving/socket cleanup and preserves the original pure API.
The SQLite app retains grants, bootstrap and database lifetime; its four extracted helper IDs move unchanged.
TDD found an old diagnostic-range regression; the maintained-layout gate also caught six formatting changes.
Both are corrected with original assertions retained. 304 Rust and 178 editor tests, the real host,
native/sanitizer checks and Linux C/LLVM/Wine HTTP/database gates pass. Prior evidence and provider/model
inputs remain. No action polymorphism, closures, new native authority or agent-efficiency measurement. See
[Core 1.11](spec/core-v1.11.md), [decision 0059](decisions/0059-action-bound-callbacks-and-http-library.md),
[report](benchmarks/results/2026-11-23-action-callbacks.md) and [receipt](benchmarks/results/2026-11-23-action-callbacks.json).


## 2026-11-23 — Review the HTTP application/library boundary

Recommend moving repeated connection/dispatch/listener machinery into the existing HTTP server library,
while application code retains capabilities, database lifetime, bootstrap policy and controller selection.
The current pure Fn restriction prevents the database handler from using the generic serve loop. A complete
callback-based API needs a separately selected action-bound callback design. No language or implementation
change is made. Cleanup/error precedence and existing transport/database behavior remain required regression
gates. See the [review record](benchmarks/results/2026-11-23-http-library-boundary.json).


## 2026-11-23 — Refresh the SQLite MVC sample

Reformatted the model, controller/view and server examples around Core 1.10 syntax and added role comments.
The before/after checked programs are byte-identical, the existing identity sidecars are unchanged, and all
12 bundle navigation tests pass. Updated the installed VS Code extension from 0.3.0 to 1.10.0 and confirmed
the SQLite workspace window with the controller selected. Initial launches lacked DISPLAY; the existing
user Xorg session accepted the final launch. See the [receipt](benchmarks/results/2026-11-23-mvc-sample-refresh.json).


## 2026-11-23 — Core 1.10 record updates and standalone try

Implemented both selected source forms in compiler/extension 1.10.0. Same-type immutable updates evaluate
one base and source-ordered overrides; standalone try discards success, preserves error propagation and
requires a final block value. Existing checked operations, ABI and limits remain. Migrated the SQLite
family with unchanged IDs/cleanup, synchronized editor/skill/docs and retained failures/corrections.
296 Rust tests, 173 editor tests, the real editor host and C/LLVM/sanitizer/Linux/Wine gates pass.
Whitespace-generated-name, precise-range, legacy-diagnostic and parser-stack regressions were found
and fixed without weakening old oracles. Empty values remain values; partial merging is excluded. See
[Core 1.10](spec/core-v1.10.md), [decision 0058](decisions/0058-record-updates-and-standalone-try.md),
[report](benchmarks/results/2026-11-23-source-conveniences.md) and
[receipt](benchmarks/results/2026-11-23-source-conveniences.json). No agent/performance claim.


## 2026-11-23 — Compare record-update spellings

Verified Rust struct-update and JavaScript spread documentation. Recommend three-dot, base-first record
update syntax for Lobo and record the user's preference for standalone try statements. These remain
proposals; no compiler/editor behavior or performance measurement changed. See
[decision 0057](decisions/0057-record-update-spelling-proposal.md) and
[sources](research/record-update-spelling-2026-11-23.json). Earlier pinned evidence remains unchanged.


## 2026-11-23 — Review DX and add strict row utilities

Reviewed tracked code across the compiler, libraries, examples, native providers and agent/editor/build
tools. The baseline inventory covers 263 code/configuration files, including 43 maintained Lobo files
with 214 functions. Recorded cleanup, state copying, result adaptation, callback, project-selection and
diagnostic patterns. These static observations are not agent productivity or token measurements. Added
six pure SQL row helpers, migrated the SQLite controller and corrected stale version guidance. Existing
IDs, Core 1.9/compiler/extension 1.9.0, native providers and frozen evidence remain. New syntax is proposed
only. See [review](docs/dx-review-2026-11-23.md), [decision 0056](decisions/0056-dx-review-and-row-utilities.md)
and [results](benchmarks/results/2026-11-23-dx-review.json).


## 2026-11-23 — First native SQLite driver

Compiler/extension 1.9.0 and Core 1.9 add the closed optional SQLite extension, separate file/role grants,
prepared bindings, typed owned rows, transactions and quota-independent teardown. SQLite 3.54.1 is pinned
and embedded only for database programs. A persistent users REST demo compiles into standalone Linux and
Windows executables. The real composition justifies bounded 16,384 bundle tokens and 64 source nominals;
other bounds and older checked-program meanings remain. All 286 Rust and 168 editor tests pass, alongside
the real editor host, C/LLVM/sanitizer, model/mutation and Linux/Wine executable gates. Preserved failures
include permission classification, schema indexing, cleanup charging, legacy-ID compatibility and harness
assumptions. Existing paper entries and frozen artifacts remain intact. See
[decision 0055](decisions/0055-first-sqlite-driver.md), [report](benchmarks/results/2026-11-23-sqlite-driver.md)
and [receipt](benchmarks/results/2026-11-23-sqlite-driver.json). No agent-efficiency or production-security claim.


## 2026-11-23 — Database access proposal

Reviewed SQLite, PostgreSQL/libpq and Redis/hiredis primary documentation. Recommend a thin SQL library
and first SQLite adapter together, with separate key-value/document interfaces and shared resource-policy
conventions. Recorded ownership, parameters, transaction cleanup, native budgets, permissions and TDD gates.
This is a proposal only. Compiler/extension 1.8.1, Core 1.8 and ABI 1 remain unchanged; no driver, dependency,
database connection or runtime test was added. See [decision 0054](decisions/0054-database-access-proposal.md),
[proposal](docs/database-access-proposal.md) and [sources](research/database-access-2026-11-23.json).

## 2026-11-22 — Modernize maintained source

Compiler/extension 1.8.1 retain Core 1.8 and ABI 1. Reviewed 38 sources and migrated 20 to inferred calls/constructors,
named callbacks, Text +, concise arms, automatic JSON helpers and compatible try forwarding. Declaration IDs,
frozen fixtures and the old checked checkpoint remain intact. Error conversion/retry/cleanup stays explicit.
274 Rust tests, 166 editor tests, the real VS Code host, source-tool checks and Linux/Wine native application
checks pass. Initial tool/fixture/host failures and their corrections are preserved; no agent/performance
effect is measured. See [decision 0053](decisions/0053-maintained-core-1.8-source-modernization.md) and
[results](benchmarks/results/2026-11-22-modernize-source.md).

## 2026-11-22 — Implement try error propagation

Compiler/extension 1.8.0 and core/1.8 implement prefix try over exact two-case Ok/Err result shapes with
matching error payload types. The evaluator, C and LLVM paths evaluate once, return from the current
function body, run postconditions on errors and restore frame context. Contracts reject direct propagation.
Editor highlighting, success-payload navigation and Core 1.8 diagnostics are synchronized. The snapshot
uses try with unchanged IDs; explicit REST cleanup remains unchanged. Earlier source/checked versions and
frozen evidence are preserved. See [the rule](spec/core-v1.8.md), [decision 0052](decisions/0052-error-propagation-proposal.md)
and [results](benchmarks/results/2026-11-22-try-propagation.md).

## 2026-11-22 — Select try for result propagation

The user prefers prefix try over postfix ?. Updated decision 0052 to use one selected spelling and retained
the original proposal as history below. Success yields the value; failure returns from the current function.
Detailed type/cleanup rules remain proposed. Compiler 1.7.3 and existing executables are unchanged.

## 2026-11-22 — Result propagation proposal

Reviewed the snapshot's repeated error-forwarding matches. Proposed postfix ? with exact error typing,
single evaluation and explicit function-boundary/cleanup rules. Named permission masks remain a separate
API question. This is a proposal only: no language, compiler, editor or executable change. See
[decision 0052](decisions/0052-error-propagation-proposal.md). Agent effects remain unmeasured.

## 2026-11-22 — Standalone native REST API

Compiler/extension 1.7.3 retain core/1.7 and ABI 1. A Lobo parser, pure controller, JSON serializer and bounded
socket loops compile into a 151,040-byte Windows executable using an explicit Winsock build profile. The
read-only loopback API needs no Rust server or interpreter. A failing injected-close regression corrected
Windows handle ownership on close failure. The snapshot profile and Linux formal coverage remain separate.
Protocol, real TCP, short IO, permissions, fault cleanup, default launch, sanitizer, model and editor gates
pass. Actual Windows execution of the REST API is unobserved. See
[the report](benchmarks/results/2026-11-22-native-rest.md) and [usage](examples/native/rest/README.md).

## 2026-11-22 — User-reported Windows 10 snapshot run

The user supplied successful typed snapshot output with 16 random bytes and corrected the OS from Windows 11
to Windows 10. This is a user-reported execution. Command, exit code and executed artifact hash were not
independently observed. The original Wine-only release evidence stays unchanged. See
[the run record](benchmarks/results/2026-11-22-windows10-user-run.json).

## 2026-11-22 — Standalone Windows snapshot executable

Compiler/extension 1.7.2 package the unchanged snapshot Lobo source as one Windows x64 executable, with a
bounded Windows clock/random provider and explicit default app launch policy. Direct native arguments still
default to deny. No Rust host/interpreter/WSL is needed at runtime. Windows networking/full compiler support
remain outside the profile. The old unsupported-target cleanup stub contained Linux-only names; the failing
Windows build exposed them and the fallback was corrected. Existing Linux provider code is byte-identical.

The exact artifact passes 14 application and 10 provider cases under Wine, including 64 permission pairs,
fail-closed unsupported operations, quotas, API faults and overflow. Actual Windows hardware validation is
unknown. Linux models/witnesses, 262 Rust, 161 editor, real host and conventional gates pass. Portable tool
provisioning, source/tool hashes, notices, failed builds and UX checks are retained. See
[report and scope](benchmarks/results/2026-11-22-windows-snapshot.md).

## 2026-11-22 — Windows testing instructions

Reviewed the existing Linux/64-bit native-ABI guard and official Microsoft WSL/Rust installation guidance.
Documented a WSL 2 route to build/run the snapshot from current source, including tracked identity sidecars
and explicit launch grants. No native Windows adapter was implemented and no Windows/WSL execution was
performed; the earlier sample verification remains Linux-only. See [the guide](docs/windows-testing.md).

## 2026-11-22 — Single native snapshot program

Added an executable example that narrows a capability to clock/random rights and returns a typed snapshot
with monotonic milliseconds and 16 random bytes. IDs are tracked in a sidecar. Ten invocations across C and
LLVM verify granted success, default/partial denial and typed byte-quota failure. The initial 30-second compile
check timed out; a fresh run with a larger verification deadline passed. No compiler, library or extension
behavior changed. See [source](examples/native/snapshot.lobo) and [results](benchmarks/results/2026-11-22-native-snapshot-example.json).

## 2026-11-22 — Native state models and pending-descriptor fix

Implemented source-linked Authority and Operations TLA+ models, eleven intentional model defects, concrete
C witnesses and eleven isolated C mutations. Baseline checks complete at 11,991 and 1,815 distinct states;
all expected model violations and optimized/sanitizer C rejections occur. A negative checker test caught
and fixed removable Python assertions under -O; acceptance now uses explicit checks. A source-drift guard rejects stale
mappings, and a local verification skill documents maintenance. The model produced a real C failure before
fixing pending connect-descriptor cleanup under fuel exhaustion. Compiler/extension patch 1.7.1 preserve
language core/1.7, ABI 1 and stable identities. No new syntax or networking scope.

Recorded preliminary/model/tool errors and corrected runs; 262 Rust, 161 editor, 31 Python, four codemod and
six HTTP tests pass, with real host, release examples, skill and preservation gates. Bounds, assumptions,
manual witness translations and the absence of a C-refinement/security proof are explicit. See
[results and receipt](benchmarks/results/2026-11-22-native-formal-verification.md).

## 2026-11-22 — Native state-machine/TLA+ recommendation

Reviewed the native 1.7 contract, provider functions, earlier unimplemented TLA+ proposal and official TLC
documentation. Recommended two bounded models: authority/resource lifecycle and IO/failure lifecycle, with
source mappings, mutation checks and counterexample replay against the C provider. Explicitly include
descriptors before handle publication and assumptions about time/syscalls. No model, tool installation or
formal-verification pass is claimed. See [proposal and sources](research/native-formal-verification-plan-2026-11-22.md).

## 2026-11-22 — Typed native OS boundary

Compiler/language 1.7 adds a closed versioned native catalog, explicit ABI activation, opaque resources,
default-deny runtime grants and subset-only delegated capabilities. C and LLVM link clock, randomness and
bounded loopback TCP into native executables. The Lobo library exposes named IoResult errors. Earlier
artifact IDs, MVC code/identities and frozen fixtures remain compatible. No runtime dependency is added.

Thirty new Rust tests include pinned Node/CPython/libuv/Go/Rust adaptations, sanitizer routes, a 50,000-step
handle model, all 256 permission pairs and injected syscall failures. Regressions guard real bugs found in
deadlines, root authority, artifact admission, legacy namespaces, null buffers and interrupted connections.
Extension/skill/specs are synchronized; 259 Rust, 161 editor, 31 Python, four codemod and six HTTP tests pass,
as do the real editor host, release/smoke and preservation checks. Failed test fixtures, tooling mistakes and
a corrected accidental lockfile metadata edit are recorded. See [results and limits](benchmarks/results/2026-11-22-native-boundary.md).
TLS, HTTP serving, OAuth and the Google-auth/Angular application are not implemented by this milestone.

## 2026-11-22 — Native library and OS interface review

Recorded a primary-documentation comparison of the November 2026 TIOBE top ten, plus Rust, and an
unimplemented Lobo native-boundary proposal. Distinguished execution engines, linked runtime support,
OS services, ABI bindings and packaging. Corrected the earlier Rust-server workaround as an optional host
choice. The Google-auth/Angular application is paused at the architecture discussion. No compiler feature,
new dependency, OS interface or application is implemented by this review. See [findings and sources](research/native-interop-2026-11-22.md).

## 2026-11-22 — Automatic JSON literal builders

Compiler/language 1.6 binds `json { ... }` builders by stable ID from explicitly supplied sources. Json and
functions called in source remain selected. Reserved generated imports expose dependencies in checked code;
ordinary names cannot redirect literals. Correct explicit imports remain compatible. The current generic MVC
removes only two unused import bindings and preserves all eleven retained IDs. Frozen fixtures are unchanged.

Added failing-first compiler/editor regressions, native/checked equivalence, closed dependency and action
checks, real editor-host navigation and unsaved error/repair coverage. Extension 1.6.0, skill and docs match.
Final gates pass 229 Rust, 155 editor, 31 Python, four codemod and six HTTP tests, plus quality/host/skill checks.
The optional legacy exporter still requires explicit generic-call/constructor forms; its full inferred-MVC
failure is retained. See [decision 0047](decisions/0047-json-literal-helper-dependencies.md) and
[results](benchmarks/results/2026-11-22-json-auto-helpers.md). Agent outcomes remain unmeasured.

## 2026-11-22 — Constructor inference and contextual case names

Compiler/language 1.5 infers generic constructors from fields, payloads and expected nominal types.
The MVC model now uses `Found(...)` and `Missing`. Expected variant identities choose owners; local/import
precedence is retained and ambiguous owners are rejected. Explicit forms, legacy read rules, runtime behavior,
artifact validation and stable IDs remain supported. Extension 1.5.0 navigates contextual cases and inferred fields.

Added failing-first compiler/editor regressions and native equivalence coverage. Updated the skill, current
docs and specification. Final checks pass 226 Rust, 153 editor, 31 Python benchmark, four codemod and six HTTP
tests, real VS Code host and quality gates. See [decision 0046](decisions/0046-constructor-inference.md) and
[the result record](benchmarks/results/2026-11-22-constructor-inference.md). Agent outcomes remain unmeasured.

## 2026-11-22 — Local call inference and contextual callbacks

Compiler/language 1.4 accepts `respond_json(find_user(id), render_user)`. Modern arguments determine generic
function types and complete expected Fn signatures resolve named callbacks. Explicit syntax remains available.
The new bounded source pass precedes existing parametric checking and specialization. Concrete checked code,
native runtime behavior, purity and persistent IDs retain their contracts. Frozen legacy source remains explicit.

Added the generic MVC, failing-first compiler/editor regressions, native equivalence and growth checks.
Updated extension 1.4.0, skill examples/references, current docs and the specification. Test scratch names now
include an atomic sequence after a collision in the first full suite. Final verification passes 219 Rust,
148 editor, 31 Python benchmark, four codemod and six HTTP checks, plus the real host and quality gates.
See [decision 0045](decisions/0045-local-call-inference.md) and
[the result record](benchmarks/results/2026-11-22-call-inference.md). Agent outcomes remain unmeasured.

## 2026-11-22 — Generic MVC response pattern verified

Verified a generic lookup and JSON response adapter using existing 1.3 constructs. User and Article views
share the missing-to-404 policy. Eight C/LLVM cases pass, including a missing value that must not invoke its
renderer. An incompatible renderer is rejected. No language, compiler, library or editor behavior changes.
See [the result record](benchmarks/results/2026-11-22-generic-mvc.md) for retained source and observations.

## 2026-11-22 — Managed persistent identities

Compiler/language 1.3 accepts tracked adjacent `.lobo.ids.json` state and explicit bundle 0.2 metadata.
Added native `identities init`, `sync` and metadata-only `rename`. Checks and builds stay read-only.
IDs are allocated once and preserved through explicit renames and source/sidecar moves. Invalid or stale
state fails. Inline tables retain read support and can be extracted without changing identity values.

The MVC and small skill example omit visible identity tables while retaining all twelve and two bindings.
Extension 1.3.0 supports trusted sidecar snapshots, unsaved metadata, navigation and native diagnostics.
Updated the skill, specification and native workflow. Frozen fixtures, reports and release evidence remain intact.

Verification passes 212 Rust, 145 editor, 31 Python benchmark, four codemod and six native HTTP checks,
real VS Code host, Clippy, formatting and skill validation. Release C/LLVM MVC behavior remains identical.
Failed-first regressions, infrastructure failures, corrections and preservation hashes are retained in
[the result record](benchmarks/results/2026-11-22-managed-identities.md). See
[decision 0044](decisions/0044-managed-persistent-identities.md). Agent outcomes remain unmeasured.

## 2026-11-14 — Maintained libraries and examples migrated

Migrated 23 source files and three embedded testing-session programs to modern 1.2 spelling. Preserved
157 module/declaration IDs, 18 selected import bindings and every complete checked-program fingerprint.
Added a guarded codemod, pinned manifest, no-write verification and a Rust integration gate.
Frozen acceptance/research sources, specifications, old reports and releases remain unchanged.

The repair-study driver now reads three byte-identical fixed copies rather than mutable examples. Only its
input directory changes. Updated exact editor anchors and added all-import corpus coverage. Maintenance
extension 1.2.1 retains the same runtime and grammar. The web builder now uses the maintained compiler and
supports isolated output paths. Updated current README/library/web examples and skill references.

Verification passes 207 Rust, 142 editor, 31 Python benchmark, four codemod and six web checks, plus the real
VS Code host and intentional-failure testing-session replay. Negative attempts and integrity checks are in
[the result record](benchmarks/results/2026-11-14-source-migration.md). See
[decision 0043](decisions/0043-maintained-source-migration.md). No language/compiler semantics or agent-study claims change.

## 2026-11-14 — Native 1.2 syntax and deprecated legacy spelling

The Rust compiler now parses modern source directly. No marker or authoring projection is required.
Modern declarations retain generic data/functions, callbacks, contracts and actions. JSON shorthand remains
Text-valued. Module headers select modern or deprecated legacy spelling per source. Existing libraries and
regression fixtures remain readable through the same checker, checked representation and native backends.
The old spelling is frozen, with no new legacy workflow, warning output or removal date.

Compiler/language and extension versions advance to 1.2.0/core/1.2. Checked format stays 1.0 and loaders
revalidate core/1.0, core/1.1 and core/1.2. The editor sends original source for native diagnostics and supplies
marker-free highlighting/navigation. Removed the old diagnostic projection/mapping path. Migrated the skill's
18 declaration identities without changing its checked behavior. Updated current docs and marked prototype
instructions historical. Frozen research and release evidence remain intact.

Five failing-first native cases and editor/grammar red tests drove adoption. Final verification passes
206 Rust, 141 editor and 31 Python tests, real VS Code host checks, Clippy, native CLI and skill examples.
See [decision 0042](decisions/0042-native-compact-syntax-and-legacy-deprecation.md) and
[the result record](benchmarks/results/2026-11-14-native-syntax.md). Agent outcomes remain unmeasured.

## 2026-11-14 — Compact MVC authoring prototype

Implemented the user-selected compact syntax as an opt-in source projection. Added `fn`, grouped selected
imports, qualified calls, arrow match arms, Text-valued JSON objects and a trailing persistent-ID table.
The original MVC source remains the comparison fixture. Exact checked-artifact equality and native 200/404
responses pass. Compiler/language versions remain 1.1. Stock CLI commands require generated canonical source.

The shared projector drives the authoring CLI, original-position navigation and mapped compiler squiggles.
Extension 1.1.1 includes compact highlighting and passes real VS Code local/project checks. Updated the
programming skill, independent executable example, explicit source configuration, references and usage guide.
201 Rust, 138 editor and 31 Python tests pass. Failed-first tests, oracle corrections and packaging failure
are retained in the [result record](benchmarks/results/2026-11-14-compact-syntax.md).
See [decision 0041](decisions/0041-compact-mvc-authoring-prototype.md). Agent outcomes remain unmeasured.

## 2026-11-14 — Language complexity review, proposal only

Reviewed the MVC/JSON authoring overhead and current specification entry points. Identified repeated identity
and import bookkeeping, explicit JSON wrappers, and a stale Core 0.11 authority statement in the older language
design overview. Recommended a finite first pass around a current guide, managed stable identities and grouped
selected imports. Generic inference and schema-driven encoder generation remain later candidates.
See [proposal 0040](decisions/0040-language-complexity-review.md). Comparative agent outcomes are unmeasured.
Documentation validation output is retained in `target/language-complexity-review-documents.log`.

The user's follow-up prioritizes syntax conventions and source ceremony. Proposal 0040 now also records a
compact source-style prototype around fn declarations, qualified calls, data construction and managed
identity metadata. Example spellings remain illustrative proposals pending an implementation milestone.

## 2026-11-14 — Typed JSON construction and serialization

Added [JSON source helpers](libraries/json/README.md) with immutable objects/arrays, scalar variants,
full-range I64 formatting, validated decimal text, string/key escaping, duplicate-key rejection and a typed
HTTP response adapter. Serialization uses a work list and chunks. The MVC view now returns Json instead of
serialized Text, and its controller uses the new adapter. Existing 200/404 response bodies are retained.
The earlier MVC source is preserved with this change's evidence. Added explicit editor sources and navigation
fixtures, plus programming-skill guidance. Four focused tests and full Rust/editor/Python/host checks pass.
See [decision 0039](decisions/0039-typed-json-source-helpers.md) and
[the verification result](benchmarks/results/2026-11-14-json-helpers.md).

## 2026-11-14 — Simple MVC source example

Added [an MVC example](examples/mvc.lobo) for the user's sample-code request. It combines an in-memory
user lookup, a JSON view using the existing quoting helper, and a controller selecting 200 or 404 responses.
Native CLI checks cover found/missing users and quotes, backslashes, newline and Unicode in the view.
See [the result](benchmarks/results/2026-11-14-mvc-example.md). This sample uses the existing 1.1 language.

## 2026-11-08 — Typed operator bindings and Text concatenation in 1.1

The user selected the bounded C++-inspired operator-function proposal for implementation. Added exact typed
binary arithmetic bindings to pure local functions, nominal ownership, explicit target-import availability,
bundle coherence and a 64-binding admission bound. Text addition resolves to concatenation. Source lowering
emits existing checked calls/intrinsics with canonical identities, contracts and ordinary resource behavior.
Compiler/language and extension versions advance to 1.1.0/core/1.1. The checked format retains revalidated
1.0 input support. The original 1.0 release remains the historical acceptance baseline.

Updated highlighting, symbol/target navigation, explicit-project resolution, compiler diagnostics and the
programming skill. Failing-first compiler/editor tests drove the implementation. Verification, failed attempts,
native examples and retained evidence are recorded in the [operator result](benchmarks/results/2026-11-08-operators.md).
See [decision 0038](decisions/0038-explicit-typed-operator-bindings.md) and [language 1.1](spec/core-v1.1.md).

## 2026-11-06 — C++ operator-function comparison, proposal only

Reviewed the C++ working draft in response to the user's follow-up. Recommended evaluating fixed, exact typed
operator bindings to pure named functions, with inspectable identities and explicit ownership. Recorded open
grammar, symbol and generic-constraint questions in [proposal 0037](decisions/0037-symbolic-shortcuts-proposal.md).
No implementation, experiment, release or extension change. Agent benefits remain unmeasured. Documentation
validation uses `./scripts/check.sh`, with output at `target/operator-shortcuts-cpp-doc-check.log`.

## 2026-11-05 — Symbolic-shortcut discussion, proposal only

Reviewed the user's operator-overloading question against current Core rules and primary Rust/Swift
documentation. Recommended a bounded Text-concatenation candidate for future evaluation, with exact typed
resolution, canonical operations and defined resource/evaluation behavior. No compiler syntax, release,
operator binding, study or package changed. Agent/construction/comprehension/token benefits remain unmeasured.
See [proposal 0037](decisions/0037-symbolic-shortcuts-proposal.md). Language 1.0 remains frozen.

## 2026-11-05 — Reusable pure HTTP helpers

The user requested helpers for the web-server sample. Added `libraries/http/http.lobo` for typed text/JSON
responses, standard errors, method classification, tagged UTF-8 prefix results and safe JSON string quoting.
Moved Response while retaining its stable identity, then migrated the app, builder and tests to explicit
source imports. No language, compiler, native host ABI, dependency or frozen release change. Valid initial
stubs failed three behavior tests before implementation; five-route native/reference/sanitizer and HTTP
validation is recorded in the dated helper result. See [decision 0036](decisions/0036-pure-http-helper-module.md)
and [helper API](libraries/http/README.md).

## 2026-11-05 — Native web-server sample

The user selected a web-server application after the bounded 1.0 language release. Added a pure native Lobo
route handler and isolated native Rust HTTP host, with explicit loopback/resource/typed-process boundaries.
No language syntax, compiler dependency, FFI, extension behavior or frozen release artifact changed.
The example adds locked, cached HTTP parsing/date dependencies only in its own manifest. Routing/parser
tests failed before implementation; actual HTTP checks and final validation are recorded separately.
See [decision 0035](decisions/0035-native-web-server-sample.md) and [sample instructions](examples/web-server/README.md).

This is the human-readable history of meaningful changes and decisions. Entries distinguish implemented work
from proposals and link to the reasoning, compatibility impact, and verification evidence. Dates on earlier
milestones come from existing decision records; they are not newly verified release or publication dates.

For every meaningful change, add a dated entry with what changed, why, implemented/proposed status, compatibility
impact, and links to decisions and verification. Keep earlier entries; corrections and reversals receive explicit
follow-up entries. Git records file changes; this ledger explains the project history.

## Unreleased — 2026-11-04

### Language 1.0.0 — bounded local release

The user-selected four families are implemented, and all three public programs plus their planned changes
pass evaluator/C/LLVM and both sanitized native routes. Prerelease verification passed 185 Rust tests,
122 editor tests, 30 Python tests, seven optimized floating-point tests, real editor-host checks, Clippy,
documentation/skill validation and six native examples. Version 1.0.0 freezes this bounded language contract,
`core/1.0` and `lobo-checked/1.0`; old checked artifacts require recompilation. Extension/skill target 1.0.
The [release receipt](benchmarks/results/2026-11-04-language-v1-release.json) and its immutable manifest record
final optimized artifact/check status; a pending/failed receipt is not release acceptance. The reproducible
[local release builder](scripts/release.py) retains every final command/output, checks source drift and tests
six packaged programs with the release compiler and both native backends. See [language 1.0](docs/language-1.0.md).
This closes language feature growth at the fixed contract; external libraries/workspace and agent studies
remain separate. No Git commit/tag, publication, installed-profile change or performance/agent-advantage claim.

### Generic data, parametric definition checks and pure function values — development

Compiler 0.12.0-dev / Core 0.12 / checked format 0.12 implements three of the four authorized families.
Generic nominal data lowers to concrete existing layouts; every generic definition is checked with rigid
symbolic parameters. Pure named function values retain signatures and invoke compiled bodies on both native
routes. Old artifacts require recompilation, and unconstrained generic arithmetic is now rejected even unused.
Extension 0.14.0 and the agent skill follow the syntax and scope changes. See [Core 0.12](spec/core-v0.12.md)
and [the running evidence record](benchmarks/results/2026-11-04-language-v1-progress.md).
F64, three fixed programs/planned changes, final adversarial gates and version 1.0 remain pending.

### F64 and artifact admission correction — development

F64 now implements binary64 arithmetic/comparisons, checked I64 conversion, exact bit interchange and
classification on evaluator/C/LLVM, including recursive IEEE equality. Execution requires validated nearest-even
rounding and gradual underflow; decision [0034](decisions/0034-binary64-values-and-execution-profile.md) records
the profile and alternatives. Source and checked literals must be finite, while explicit bit inputs preserve
nonfinite payloads. Float tests use fixed bit oracles and altered rounding/FTZ/DAZ probes. Editor and skill
follow F64 syntax. Additional adversarial review also found and closed unchecked Function-literal admission;
its failing/fixed tests supersede earlier artifact-safety evidence. Fixed program/planned-change acceptance,
final whole-repository checks and version 1.0 remain pending. The running evidence record retains all attempts.

### Language 1.0 implementation authorized

- The user selected the bounded four-feature milestone, requested implementation through completion and
  explicitly made it the goal. Recorded decision 0033 and frozen public acceptance programs/changes before
  production edits. Preserve the previous compiler/source baseline. Version 1.0 is not yet achieved.
  [Release decision](decisions/0033-bounded-language-1-release.md),
  [acceptance contract](spec/language-v1-acceptance.md),
  [baseline](benchmarks/results/2026-11-04-language-v1-baseline.json).

### Finite future backlog and definition of enough

- At the user's request, documented four proposed later language families: generic data declarations,
  definition-time generic checking, non-capturing function references and one F64 numeric family.
  Each has a bounded purpose and acceptance target; none reopens the current Core 0.11 evaluation freeze.
- Defined completion through those four families and three fixed public programs, including frozen changes,
  resource/failure tests, both native backends, tooling and reproducible evidence. Comparative agent success
  is measured separately, not assumed from language conformance or used to keep adding constructs.
- Kept pure libraries, controlled host/FFI access, operational policy and semantic workspace in separate
  application-specific milestones. Marked other feature families optional. Proposal only; no implementation,
  new experiment or dependency. [Future scope and gates](docs/future-language-scope.md).

### Corrected the stopping rule for language development

- The user clarified that the concern was a finish line for feature growth. Prior recommendations had
  continued extending the feature list without enforcing a fixed checkpoint. Set current Core 0.11 and
  compiler 0.11.1 as the language evaluation checkpoint and defer the generic-data continuation proposal.
- Restrict proactive continuation to existing-specification fixes, review, documentation, reproducibility
  and study design. Record unsupported capabilities as findings. A later feature stage needs a user-selected
  bounded milestone; no model trial or ecosystem expansion starts automatically. This is a scope rule,
  not a claim of universal completeness or mechanical enforcement. Historical snapshots remain unchanged.
  [Decision 0032](decisions/0032-stop-feature-growth-at-the-evaluation-checkpoint.md).

### Proposed continuation after the LLVM checkpoint

- Recommend generic records and variants next, driven by the I64-specific sequence Cursor and recursive
  generic-helper call limit. Proposed acceptance is reusable iterative collections and typed optional/error
  outcomes across modules/types, including a large internally generated case and maintained editor/skill.
- Keep declaration-time generic checking and behavior-parameterized functions as separate later decisions.
  Define a workload checkpoint before expanding again, then assess construction and code comprehension
  independently in any later controlled pilot. No feature implementation, study or new measured claim.
  [Proposed milestone](docs/next-language-milestone.md). Frozen paper evidence remains unchanged.

### Paper evidence preservation

- Added a local claim-to-evidence index and hashed register for the future paper, linking the prior execution,
  verification, agent-repair, representation-calibration, recursive-data, generic-function and LLVM records.
  Keep external literature findings separate and preserve original versions/denominators and negative results.
- Recorded C/Clang as the better current default on measured compile/runtime/size tradeoffs. LLVM's future
  control/optimization benefits remain unmeasured. No inference of agent construction, comprehension or token
  gains follows from the native comparison, and unknown effects are not zero.
- Preserved a local compressed bundle of reports and recent raw evidence, source/compiler baselines and
  measured artifacts, with a file manifest and SHA-256 receipt. Older private agent attempts remain at their
  originally reported locations; no off-device backup, commit or publication is implied.
  [Paper evidence](research/local-evidence.md), [preservation receipt](research/verification-local-evidence.json).

### Direct LLVM native backend

- Implemented the user's request to do direct LLVM work now, superseding the earlier deferral recommendation.
  Compiler 0.11.1 adds `compile/run --backend llvm`, `--backend c` and `--emit llvm`; C remains default.
  Core/artifact 0.11, native adapter 0.6 and extension 0.13 remain compatible and unchanged.
- Application arithmetic, calls, branching, short circuiting, contracts, matching and iteration emit LLVM.
  Reuse the C runtime/typed layout primitives through a private pointer ABI, compose with llvm-link-18 and
  optimize with Clang 18. Entry-block slots avoid per-iteration stack growth. No interpreter, new Cargo
  dependency, tool installation, new authority or public FFI.
- Expanded shared fixtures and independent generated oracles to compare optimized/sanitized LLVM with C.
  All 163 Rust, 30 Python and 114 editor tests pass, plus real VS Code host, LLVM verifier, sanitizer probes,
  strict Clippy and skill examples on both routes. Tool/selector errors never fall back or publish output.
- Preserved the prior compiler/source baseline and failed runs. Corrected ineffective sanitizer probes after
  inspecting optimization, and removed only regenerable incremental cache after a full filesystem caused
  host linker failures. Hardware counters were unavailable under host permissions.
- Two rounds retain six builds and eleven timed runs per backend/workload, independent expected outputs
  and zero benchmark failures. No runtime win demonstrated; final LLVM compile medians are 18–31% slower
  here, with equal or eight-byte-larger binaries. The LLVM generic example is 22,896 bytes. No agent
  construction, comprehension or token-efficiency measurement, commit, publication or profile install.
  [Decision 0031](decisions/0031-direct-llvm-native-backend.md), [contract](spec/native-llvm-v0.1.md),
  [verification and corrections](benchmarks/results/2026-11-04-llvm-backend-verification.md).

### LLVM backend clarification

- Confirmed from the native emitter and primary LLVM/Clang documentation that Lobo already uses LLVM
  indirectly through C and Clang `-O3`. Recommend retaining this path during language evolution; direct
  LLVM IR is a possible future prototype for a demonstrated need, with no measured advantage yet.
  Documentation-only follow-up; no compiler, dependency, language or extension version change.
  [Assessment](benchmarks/results/2026-11-04-llvm-backend-assessment.md).

### Explicit generic function specialization

- Preserved the Core 0.10 compiler, test runner and source/spec/test/editor/skill snapshot before editing.
  Added `function name<T>` and `name::<Type>(...)` for first-order reusable functions; Core/compiler/artifact
  advance to 0.11. Native adapter 0.6 and machine-code execution remain unchanged.
- Specialize through a bounded work list, deduplicate recursive requests, preserve definition dependencies
  in explicit generated scopes and admit caller-owned concrete types without exposing private names.
  Existing concrete checker, contracts, action bounds and native/reference resource semantics still apply.
- Check local names/types in requested bodies only. Unrequested templates are parsed and declaration
  references resolve; their bodies are not universally checked.
  No generic records/variants, inference, traits, function values or new dependencies. Recompile old artifacts.
- Extension 0.13 adds generic highlighting, type-parameter/instantiated-result navigation and compiler 0.11
  support. Update the agent skill and keep all public development fixtures separate from paper holdouts.
- [Decision 0030](decisions/0030-explicit-generic-function-specialization.md),
  [Core v0.11](spec/core-v0.11.md), [workload](spec/generic-reuse-workload.md).
- Acceptance: 159 Rust, 30 Python and 114 editor tests, strict Clippy, native/sanitizer checks, real VS Code
  host, packaged VSIX and executable skill examples pass. The direct generic example is 22,888 bytes.
  [Verification/corrections](benchmarks/results/2026-11-04-core011-verification.md),
  [baseline](benchmarks/results/2026-11-04-core010-language-baseline.json),
  [native smoke](benchmarks/results/2026-11-04-generic-reuse-native.json),
  [foreground test session](benchmarks/results/2026-11-04-generic-reuse-test-session.json).
  Contract edits invalidate instantiated dependents, while unrelated instantiations retain passing evidence.
  No model trial, agent productivity/comprehension result, commit, publication or user-profile install.

### List-guarded recursive values and native AST workload

- Preserved the Core 0.9 compiler/test-runner and source/spec/test/editor/skill snapshot before
  implementation. Core/compiler/artifact 0.10 permits nominal recursion through existing List storage;
  direct inline cycles remain errors. Native adapter 0.6 specifies recursive shapes and depth limits.
- Separated C layout definitions from structural-helper declarations/bodies. Added bounded input/output
  traversal and structural-comparison guards, plus iterative reference reclamation across nested values.
  Allocation cells and invocation-owned native cleanup remain unchanged. No dependency or Box syntax.
- Implemented the public native AST evaluator/rewriter, with frozen independent values, preserved order,
  missing-case/overflow and resource checks. Native smoke yields 14 and 23; its 22,888-byte binary is
  identified in the result, without a performance claim. General generics/function values remain deferred.
- Extension 0.12 recognizes compiler 0.10 and tests recursive type/case navigation, diagnostics and grammar;
  real VS Code checks pass and VSIX is rebuilt. Maintained the Lobo skill and native examples.
- TDD failures, fixture mistakes, editor version rejection and pre-execution oracle correction are retained
  in the [verification report](benchmarks/results/2026-11-04-core010-verification.md). Final evidence:
  151 Rust, 30 Python, 108 editor tests; native/reference/sanitizer/leak gates, real extension host,
  strict Clippy, formatting, release build, skill validation and documentation checks pass.
  [Decision 0029](decisions/0029-list-guarded-recursive-values.md), [Core v0.10](spec/core-v0.10.md),
  [native data](benchmarks/results/2026-11-04-recursive-ast-native.json).
  No agent-quality claim, new agent trial, user-profile installation, commit, publication or deployment.

## Unreleased — 2026-11-03

### Language readiness for comparison

- Audited Core 0.9 specifications, the implemented type model, language-milestone evidence and actual
  tree/shared-helper examples. Lobo supports focused state/failure and small-program comprehension
  comparisons; the current evidence cannot support a broad mature-language productivity claim.
- Recorded recursive named data, user generics, function values, source/recursion limits and deferred
  proof/resource/workspace facilities as possible confounds. Recommended separate development
  workloads and selective evolution before the broad study, retaining the existing baseline and
  held-out evaluation. Proposed priorities are not implemented features or new milestone acceptance.
- [Assessment and evidence](research/comparison-readiness.md). Documentation-only; no additional
  agent, model trial, compiler/editor change, dependency, installation or publication. JSON/JSONL,
  local Markdown links and whitespace checked; prior executable/test results remain unchanged.

### Cross-language calibration and code comprehension

- Added comprehension as a separate required research outcome under all three hypotheses: behavior,
  witnesses, dependencies, fault localization and change impact. Kept source-only and tool-assisted
  conditions distinct, and human comprehension separate from agent scores. Added CRUXEval,
  CRUXEVAL-X and RepoQA to the bibliography, now 36 records; preserved the 33-record snapshot.
- Implemented five native Lobo/Rust and Python fixture arms, an independent transition oracle,
  fifteen mutation probes and an eleven-item deterministic comprehension grader. All 750 baseline
  observations agree. Lobo/Rust variant arms reject missing cases and unknown constructors; the
  wrong boundary rule compiles in every arm and fails execution checks. No overall language winner.
- TDD: initial missing-module failure; a later probe test caught an ambiguous Python mutation
  target. Unique context fixes it; all six focused tests pass. Sources, compiler hashes, diagnostics,
  commands and limits are retained in the [calibration report](benchmarks/results/2026-11-03-language-comparison-fixtures.md)
  and [data](benchmarks/results/2026-11-03-language-comparison-fixtures.json).
- No additional agent was launched pending the explicit delegation request. Model comprehension,
  H1 construction benefits, H2 and H3 comparisons remain unknown; no human study was performed.
  [Current comparison](research/lobo-comparison.md), [comprehension framework](research/code-comprehension.md).
  Verification: all 30 Python tests, 145 Rust tests including native/sanitizers, source/result hashes,
  JSON/bibliography consistency and local links pass. [Verification record](research/verification-comprehension.json).
  No language/compiler/editor change, new dependency, installation or publication.

### Three candidate research hypotheses

- Formulated falsifiable proposals for checked representations, task-aligned abstractions and
  semantic context/feedback. Each states an intervention, primary outcome, contrary evidence and
  scope limits. Recommended checked representations as the initial lead because it directly tests
  construction quality and can use existing native Lobo records/variants.
- Retained the other mechanisms as alternatives that may coexist. Preference does not count as
  support; existing-language success is an informative possible result. Added controls for failure
  counting, requirement information, task difficulty and survivor bias in change measurements.
- [Hypotheses and rationale](research/hypotheses.md), [paper plan](research/paper-plan.md).
  No experiment, language change, model spend or publication. Verification: local Markdown-link,
  JSON/JSONL and whitespace checks. Prior compiler test evidence is unchanged.

### Construction quality as the research outcome

- Extended the prior-work collection from 26 to 33 papers/artifacts. Added RustBelt, VERINA,
  DafnyBench, AutoCodeBench, the code-quality reproduction study, Vera and VeraBench. Added eight
  structured findings with source IDs, reported results, conditions and limitations. Preserved the
  initial review's source, bibliography, verification and framing artifacts as a snapshot.
- Reframed the proposed paper around requirements satisfaction, prevented/escaped defects and correct
  requirement changes at a fixed budget. Token use is a cost dimension. Added existing-language
  baselines and within-language construct comparisons. No construct benefit has been measured in Lobo.
- Pinned Vera READMEs by commit/hash. Recorded saturation, grading asymmetry, incomparable headline
  scope, weak specifications and unavailable raw attempts rather than adopting causal typing claims.
  Formal language guarantees, agent observations and hypotheses remain distinct. No new model run.
  Clarified initial metadata accounting: 25 network-fetched records and one curated institutional
  record, rather than implying that all 26 entries were fetched independently.
- Evidence and checks: [construction-quality framework](research/construction-quality.md),
  [findings register](research/findings.json), [paper plan](research/paper-plan.md),
  [follow-up verification](research/verification-construction-quality.json).
  Documentation-only follow-up; language/compiler/editor/dependency versions unchanged. Nothing published.

### Academic related work and paper preparation

- Recorded a targeted scoping review of 26 works on agent-oriented languages, token/language costs,
  semantic context, tools, skills, evaluation and safety. Added an annotated review, version-pinned
  primary metadata/BibTeX and a standard-library metadata collector. Access failures, review depth,
  unreviewed technical-note status and version-dependent numerical claims remain explicit.
- Proposed a paper framing and controlled Lobo/Rust/Python agent study. Existing 60 scheduled
  real-agent attempts (59 accepted, one interruption) all use Lobo and cannot establish a language
  advantage. Execution speed, source compactness and whole-trajectory agent cost stay separate.
  Our skill study's negative/inconclusive findings are retained. No new agent experiment was run.
- Sources, commands, verification and limitations: [research record](research/README.md),
  [related work](research/related-work.md), [paper plan](research/paper-plan.md),
  [result register](benchmarks/results/README.md). All 26 metadata retrievals succeeded; JSON,
  bibliography/version consistency, local links and whitespace were checked. `cargo test --locked`
  passed all 145 tests; [verification record](research/verification.json). Compiler, language,
  extension, dependencies and distribution are unchanged. Nothing was published or submitted.

### Lobo programming skill and paired evaluation

- Created the repository [lobo-programming skill](.agents/skills/lobo-programming/SKILL.md),
  targeting Core/compiler 0.9, with focused language/tool references and native executable examples.
  Added a maintenance rule alongside language/tool changes. No plugin or global install was added.
- Completed a frozen twenty-attempt comparison against equivalent guide knowledge on two new tasks.
  All attempts passed 1,220 fresh native final cases, and all ten skill agents read the entry and
  both references. No compiler diagnostics, timeouts or infrastructure failures occurred. No accuracy
  or consistent time/token gain was demonstrated; median total time was higher with the skill.
- Preserved three skill-arm parent-instruction probes as separate workflow deviations, without
  changing acceptance or excluding trials. Stored all raw evidence privately outside build cleanup.
  Checkpointed blocks avoid silently replacing started attempts. TDD caught a record-type fixture tag.
- Verification: skill frontmatter validator, native example checks, 24 Python tests, 145 Rust tests;
  [decision 0028](decisions/0028-lobo-skill-onboarding-comparison.md),
  [findings](benchmarks/results/2026-11-03-skill-evaluation.md),
  [data](benchmarks/results/2026-11-03-skill-evaluation.json),
  [audit](benchmarks/results/2026-11-03-skill-evaluation-audit.json).
  Compiler/language/extension versions and dependencies are unchanged. No publication or deployment.

### Result recording and agent language knowledge

- Made the user's requirement to document all results explicit in `AGENTS.md`: retain failed,
  negative, interrupted and inconclusive outcomes; keep missing values unknown; preserve evidence
  and label proposals separately. Added a [result register](benchmarks/results/README.md) with
  current findings and raw-artifact retention limits.
- Documented [a Lobo skill and optional plugin proposal](decisions/0027-agent-language-knowledge-package.md).
  Existing trials already supplied language guides. A dedicated package may improve onboarding,
  but has not been built or evaluated. No pretraining or productivity claim follows.
- Documentation-only change. Verification: JSON/JSONL and Markdown-link checks, plus whitespace
  review. No language, compiler, extension, runtime, dependency or public distribution changed.

### Repeated library-repair study

- Added two multi-defect, five-source repair tasks with five repetitions per task and three arms:
  concise full-run text, full JSON and affected JSON. Preserved the earlier pilot/harness as history.
  All 30 scheduled records remain: 29 accepted native repairs and one SIGTERM controller interruption,
  without replacement or invented time/usage. Independent final grading executed 4,500 native cases.
- Affected mode ran 84 rather than 128 recorded verify cases per completed repair and reused 44 passes.
  All completed agents used detail, but repaired in one candidate; timing/token results remain
  workload-dependent and inconclusive. Two global-context read deviations are flagged separately.
- TDD/canaries corrected concurrent shared-mailbox overwrites using per-request files. Export preserves
  raw evidence and uniformly corrects a printed-source timeout counter; unknown interrupted latency
  is not zero. A specific recovery controller verifies frozen identities before continuing the schedule.
- Verification: 16 Python harness/report tests and 145 Rust tests including native/reference/sanitizers.
  [Decision 0026](decisions/0026-repeated-library-repair-study.md),
  [findings](benchmarks/results/2026-11-03-library-repair-study.md),
  [all records](benchmarks/results/2026-11-03-library-repair-study.json).
  TLA+/state-machine verification is recorded as a complementary proposal; no model was checked.
  No language/compiler/editor version, dependency, publication or deployment changed.

### Real-agent testing pilot

- Added a rerunnable, bounded Codex CLI repair comparison: five seeded bugs, ten fresh conversations,
  alternating conventional full-run text and structured affected feedback on the same native checks.
  Recorded actual input/cached/output/reasoning usage, timings, source/plan/tool identities and failures.
- All ten repairs passed independent forced-full grading (490 native case executions); no pilot trial
  was dropped or retried. Structured mode executed 80 visible cases versus 100 and reused 20 passes.
  Timing/token comparison is inconclusive; structured feedback returned more bytes and check time was
  nearly identical. This is a small feedback pilot, not a third-party-framework or general speed claim.
- Corrected an excluded canary transport failure: workspace-write blocks local Unix sockets, so the
  experiment uses a bounded task-local file mailbox without sandbox bypass. Fixed template-brace
  substitution before the pilot. Seven harness tests cover native oracles, actual client transport,
  reuse/full behavior, schedule, usage, fixture generation and source admission.
- Decision/history: [0025](decisions/0025-real-agent-testing-pilot.md);
  [analysis and verification](benchmarks/results/2026-11-03-agent-repair-pilot.md).
  No Core/compiler/artifact/extension version or dependency changed. No publication or deployment.

## Unreleased — 2026-10-18

### Agent-first native verification prototype

- Added `lobo-test`, a native foreground JSONL session with fixed requirements/checks, explicit candidate/base
  snapshots, conservative static-closure reuse, compact differences, exact-source explain/replay and a forced-full gate.
  No new Lobo syntax, interpreter dependency, daemon, MCP server or runtime authority is introduced.
- Added ordinary Lobo law functions, an executable intentional-regression session and a six-scenario scripted
  full/affected comparison. All 384 case pairs agree; a leaf edit executes 16 instead of 64 checks. This is
  not a measured AI-agent token/repair improvement or proof of selector soundness.
- TDD fixed invocation-type admission, infrastructure execution counts, strict unit-action fields, temporary
  permissions, observation-limit reporting and protocol error verbosity. Preserved `cargo run` behavior via
  `default-run = "lobo"`. Core/compiler/artifact 0.9 and extension 0.11.0 remain; no dependencies changed.
- Verification: 145 Rust tests per debug/release profile, 104 editor tests, native/reference/sanitizer law
  fixtures, real VS Code host marker/exit 0, release builds, formatting/Clippy/JSON/link/whitespace gates.
  No normal editor install, commit, publication or deployment; legacy benchmark results/comparison worktrees are unchanged.
- Runner release is 993,336 bytes. [Protocol](spec/agent-testing-v0.1.md),
  [decision and exact evidence](decisions/0024-agent-first-verification-prototype.md),
  [comparison report](benchmarks/results/2026-10-18-agent-testing-prototype.json).
  Durable/protected acceptance, counterexample shrinking and real-agent workflow trials remain unimplemented.

### Explicit editor projects and shared workloads — language-only audit

- Extension 0.11.0 adds explicit `lobo.sourceFiles` projects, cross-file stable nominal navigation,
  no-IO bundle compiler checks, exact dependency diagnostics, unsaved overlays, stale-result cancellation,
  and explicit-file watcher invalidation. Trust gates dependency IO; no workspace scan or package resolver.
- TDD found/fixed FIFO-open blocking, newer-host-only Unicode validation, old bundle-success acceptance,
  and overbroad document invalidation. Bounded reads reject invalid paths, nonregular files and malformed UTF-8.
- Moved the original collection reverse/cursor to `examples/sequences.lobo`; tree/module-example public
  reverse IDs delegate to it. One linked algorithm/cursor remains, without increased test allowances or
  changes to frozen numerical/workload results. Removed unused tree/module internal cursor definitions.
- Verification: 129 Rust tests in each debug/release profile, 103 editor tests, reloaded/native/reference/
  ASan-UBSan/leak fixtures, actual VS Code host remote/unsaved/external-dependency checks, format/Clippy/docs.
  A host bootstrap SIGPIPE log is retained in the evidence; the run completed all assertions and exited 0.
- Rebuilt the VSIX and local native collection/tree examples (18,784/31,080 bytes). No install, commit,
  publication, deployment, runtime dependency, foreign library, service, or benchmark mutation.
- The [full language-only requirement audit](docs/language-completeness.md#language-only-requirement-audit--2026-10-18)
  now has local acceptance evidence. This is not universal replacement, Turing completeness, ecosystem or
  application/workspace v1.0 readiness. New ecosystem scope/target still requires separate selection.
- Decision, artifact/source identities and residual limits: [0023](decisions/0023-explicit-editor-source-snapshots.md).

### Closed module linker — Core/compiler 0.9 (composition slice incomplete)

- Added selected imports by stable declaration ID, module-local namespaces, one nominal identity across
  imports, explicit ownership scopes in checked IR, and loader revalidation of import/owner/visibility rules.
  The pure compiler API accepts bounded source snapshots; no import path search or IO is performed there.
- Added explicit compiler `--source FILE` inputs and a separate no-IO `check --bundle-stdin` agent envelope.
  Native/reference execution uses the existing linked declaration graph. New module examples share one
  reverse implementation/cursor type; collection/tree consolidation remains pending.
- Preserved source provenance and original alias spans. TDD found and fixed declaration-range regressions,
  unknown-versus-invisible type diagnostic ordering, and a compiler panic from using resolved ID length for
  a noncallable import's source range. Source labels are boxed to keep error propagation below Clippy's large-error threshold.
- Extension 0.10.0 maintains import highlighting/local alias targets and compiler compatibility. Actual
  cross-file navigation/dependency-aware squiggles are explicitly incomplete; the composition gate and
  overall goal remain active. No package registry, download, external runtime authority, daemon, or deployment.
- Compatibility: compiler/Core/artifact 0.9; recompile old artifacts. Native executable adapter 0.5 is unchanged.
  Rules/status/evidence: [Core v0.9](spec/core-v0.9.md), [decision 0022](decisions/0022-closed-module-composition.md).
- Local verification: 128 Rust tests per debug/release profile, 87 editor tests, optimized/reference/
  ASan-UBSan/leak fixtures, actual VS Code host checks for local features, formatting/Clippy/JSON/link gates.
  Rebuilt compiler/VSIX; linked native example is 18,784 bytes. Cross-file editor checks remain outstanding.

### Immutable state-driven iteration — Core 0.8 / compiler 0.8.0

- Added contextual `while(state: T = initial; condition) { next }`: initialize once, test/carry immutable
  state, and return on false without predetermined counts or per-iteration call frames. Every condition
  probe, including the final false one, is metered. Heap, actions, contracts, and first-fault ordering remain explicit.
- Added parser/checker/resolution/IR/reference/native lowering and maintained extension 0.9.0 highlighting,
  state scope/navigation, compiler compatibility, and unsaved-condition diagnostics. Converted parser
  reduction/tree search; added native GCD and early-match list search in `state_iteration.lobo`.
- TDD exposed debug-compiler stack overflow on nested loops. Replaced the precedence-level call tower with
  precedence climbing and isolated loop parsing, retaining nesting limits and canonical operator semantics.
  A further mixed comparison-chain regression preserved syntax rejection rather than a type-error fallback.
- Compatibility: emit only Core/artifact 0.8; recompile old checked artifacts. Existing source semantics,
  numerical/collection/parser/tree oracles, ordinary `while(value)` calls, and old native executables remain.
  Native adapter 0.5/CLI are unchanged. No assignment, new authority, dependency, package ecosystem, or daemon.
- Definitions, failure/budget gates, and verification: [Core v0.8](spec/core-v0.8.md),
  [decision 0021](decisions/0021-immutable-state-driven-iteration.md). Cross-file reuse and the final coverage
  audit remain outstanding; the overall language-first goal stays active.
- Final verification: 112 Rust tests in each debug/release profile, 83 editor tests, optimized/reference/
  ASan-UBSan/leak fixtures, actual VS Code host checks, formatting/Clippy/JSON/link gates. Rebuilt compiler
  and VSIX; standalone state-iteration example is 18,776 bytes. Artifact evidence is in decision 0021.

### Search-tree workload expressed in existing Lobo

- Implemented frozen workload C in [tree.lobo](examples/tree.lobo): dynamically constructed parent-indexed
  nodes, duplicate handling, in-order/reverse traversal, unique count, and insertion-order-dependent height.
  No compiler feature, host sort/tree algorithm, dependency, runtime, or external authority was added.
- Added reference/optimized/sanitized fixtures for frozen cases, actual parent edges, extreme/negative keys,
  duplicate-preserving variations, 40-level chains, and structured fuel/heap exhaustion. Maintained grammar,
  definition navigation, and actual VS Code host fixtures on the full example.
- All three workload gates are now executable. The [coverage assessment](docs/language-completeness.md#coverage-assessment-after-the-three-workloads)
  records cross-file helper duplication and state-driven termination as concrete remaining pressures; the
  overall goal is active. Compiler/Core/artifact 0.7 and extension runtime/version 0.8.0 are unchanged.
- Decision and verification: [0020](decisions/0020-tree-workload-in-existing-lobo.md).
- Final verification: 100 Rust tests in debug and release, 79 editor tests, native ASan/UBSan/leak checks,
  actual VS Code host navigation, formatting/Clippy/JSON/link checks. The host's incidental SIGPIPE log is
  retained in the decision. The standalone stripped native tree executable is 31,080 bytes.

## Unreleased — 2026-10-17

### Arithmetic parser expressed in existing Lobo

- Implemented frozen workload A in [parser.lobo](examples/parser.lobo): streaming byte scanning, operator
  precedence/parentheses, explicit immutable stacks, and typed syntax/integer/arithmetic failures with offsets.
  No compiler feature, host-language parser, workload intrinsic, dependency, or authority was added.
- Added checked/reference/optimized/sanitized acceptance tests, independent generated expression variations,
  arbitrary-octet classification, deep nesting, and structured resource exhaustion. Added the full workload
  to editor grammar/navigation fixtures and actual VS Code definition checks.
- Compatibility: compiler/Core/artifact 0.7 and extension 0.8.0 are unchanged. The tree workload and broader
  program composition remain pending; the overall language-first goal is active. Scope and verification:
  [workload specification](spec/language-milestone-workloads.md), [decision 0019](decisions/0019-parser-workload-in-existing-lobo.md).
- Final verification: 94 Rust tests in debug and release, 76 editor tests, ASan/UBSan/leak checks, actual
  VS Code host navigation, formatting/Clippy/JSON/link checks. A failing error-order regression was fixed;
  the host's incidental SIGPIPE log is recorded in the decision. The standalone native parser is 43,384 bytes.

### Immutable bytes and UTF-8 text — Core 0.7 / compiler 0.7.0

- Added distinct Bytes/Text values, JSON-escaped Text literals, explicit conversions, indexing/length/slicing,
  concatenation, byte push, and bulk List<I64>-to-Bytes construction. Text indices count Unicode scalars;
  bytes count octets. Invalid text is never silently normalized or replaced.
- Shared immutable views/static literals preserve lifetime without copying; new buffers use the existing
  invocation arena. Copy/validation/comparison fuel and heap-cell spending are explicit. Input byte limits
  and JSON/hex/escape-aware output preflight cover large single values as well as nested data.
- Added independent UTF-8 corpus, exact budget/output-boundary, native admission, source/artifact rejection,
  cleanup/leak, action-purity, and source-range regressions. Native remains default; no host parsing shortcut.
- Extension 0.8.0 tracks text/byte syntax, escapes, navigation, and compiler diagnostics on unsaved text.
- Compatibility: Core/compiler/artifact 0.7, native adapter 0.5; Bytes/Text are now reserved. Recompile old
  artifacts. Existing numeric/boolean/list/record/variant oracles are preserved. No dependency, external
  authority, FFI, package ecosystem, or workspace service is added; parser/tree/composition work remains.
- Rules and verification: [Core v0.7](spec/core-v0.7.md), [decision 0018](decisions/0018-text-and-byte-values.md),
  [text example](examples/text.lobo). The full language-first goal remains active.

### Growable persistent lists — Core 0.6 / compiler 0.6.0

- Added immutable `List<T>` values with constant-time tail sharing, empty/prepend/head/tail/length operations,
  native invocation-scoped allocation, explicit cumulative heap-cell budgets, and cleanup on success/fault.
- Metered list equality; added bounded input/output value traversal and an exact 8 MiB result-payload byte
  preflight. Metadata and shared nested values cannot bypass these limits or yield partial successful JSON.
- Added typed list interchange/input, artifact revalidation, and exact heap/fuel/serialization regressions.
  Contract resource faults and entry-time resource subjects now agree between native and reference paths.
- Froze parser/collection/tree acceptance inputs before implementation; collection workload B and its odd
  predicate variation now run entirely in Lobo, including first-invalid typed errors. Parser/tree remain pending.
- Extension 0.7.0 tracks nested List types, built-ins, element/field definition navigation, and diagnostics.
- Compatibility: compiler/Core/artifact advance to 0.6, native adapter to 0.4. `List` is reserved; recompile old
  artifacts. Reference CLI now rejects repeated fuel options to match native behavior. No new dependency,
  recursive named type, general text/bytes, or external authority. The language-first goal stays active.
- Semantics and evidence: [Core v0.6](spec/core-v0.6.md), [decision 0017](decisions/0017-persistent-growable-lists.md),
  [acceptance workloads](spec/language-milestone-workloads.md).

### Tagged alternatives and explicit handling — Core 0.5 / compiler 0.5.0

- Added immutable nominal variants with optional typed payloads and exhaustive expression matches. Typed
  success/failure returns now compose with records, equality, functions, contracts, and folds. Safe division
  distinguishes ordinary failures without exception control flow; only the selected arm executes.
- Native adapter 0.3 supports compound shapes containing alternatives; artifact 0.5 carries stable variant
  references and rechecks cases, arm scopes, actions, and mixed layout limits. Recompile older artifacts.
- Shared generated helpers bound output expansion by type declarations, not by all possible nested case paths.
  A reproduced compiler stack overflow from a long field chain now has frontend bounds and a regression test.
- Repeated release-gate host failures exposed transient `ExecutableFileBusy` launches. A deterministic writer
  regression now covers a bounded retry of failed launches only; program failures are never retried.
- Extension 0.6.0 maintains highlighting, type/case/binding navigation, and unsaved diagnostics. No heap,
  recursive/growable data, ambient authority, or ecosystem integration is introduced; the full goal stays active.
- Rules and verification: [Core v0.5](spec/core-v0.5.md), [decision 0016](decisions/0016-tagged-alternative-values.md).

### Immutable record values — Core 0.4 / compiler 0.4.0

- Added nominal records, nested acyclic layouts, constructors and selection, value equality/copying,
  record parameters/returns/contracts, and structured fold accumulators. Iterative GCD now uses an ordinary
  two-field record instead of requiring a new loop construct.
- Native adapter 0.2 supports typed record arguments/results; checked artifacts 0.4 carry record definitions
  and stable references. Source/artifact validation enforces layout and copy bounds. Recompile old artifacts.
- Extension 0.5.0 adds record type/field definition navigation, highlighting, and matching diagnostics.
- No heap, growable data, variants, host authority, or ecosystem integration is introduced. The broader
  language goal remains active. Rules and evidence: [Core v0.4](spec/core-v0.4.md),
  [decision 0015](decisions/0015-immutable-record-values.md).

### Current goal: language expressiveness before the ecosystem

- Accepted the user's language-first goal: a small composable general-purpose core, adding constructs only
  when demonstrated programming patterns justify them. Ecosystem integrations and workspace services follow
  later; existing native compilation, TDD, editor maintenance, and authority boundaries remain required.
- Defined language-only acceptance evidence separately from Turing completeness and application readiness.
  Proposed first slice: structured values and memory semantics, beginning with bounded record values.
- Updated agent guidance and roadmap sequencing while preserving earlier milestone/decision history.
  This is a planning change; no compiler feature or version changed.
- Goal and rationale: [language completeness](docs/language-completeness.md),
  [decision 0014](decisions/0014-language-first-development.md).

### Follow-up: checked action declarations

- Implemented Core v0.3 action clauses, conservative per-call bounds (including recursion), pure contracts,
  canonical/revalidated action metadata, and `lobo check` summaries keyed by stable function ID.
- Compiler 0.3.0 emits checked artifacts 0.3. Recompile old source; do not retag artifacts. Native adapter
  0.1 and proposed LTP 0.2 are unchanged. No host resource APIs, capabilities, quotas, or sandbox are added.
- Extension 0.4.0 adds clause highlighting, navigation through declarations, and matching compiler diagnostics.
- Specification and evidence: [Core v0.3](spec/core-v0.3.md),
  [decision 0013](decisions/0013-checked-action-bounds.md). This supersedes only the earlier unsupported-clause
  boundary below; earlier entries retain their original scope and evidence.

### Earlier: terminology and history ledger

- Adopted **actions** instead of effects in the current language design, documentation, illustrative source,
  tool descriptors, and verification examples. Actions describe behavior, capabilities supply authority, and
  limits constrain use. Resource actions remain proposed, not implemented Core syntax.
- Advanced the proposed protocol to **LTP 0.2** because renaming the descriptor field and verification check is
  breaking. Schema IDs and envelope examples move together; old clients/descriptors need explicit migration.
  Compiler 0.2.1, Core v0.2, checked artifacts 0.2, native adapter 0.1, and extension 0.3.0 are unchanged.
- Established this ledger and maintenance rules in `AGENTS.md` and the compiler workflow.
- Decision and verification: [0012 — actions terminology](decisions/0012-actions-terminology.md).

## Earlier recorded milestones

### 2026-10-13 — editor navigation and diagnostics

- Local definition navigation added in extension 0.2.0:
  [0010](decisions/0010-vscode-local-definition-navigation.md).
- Compiler source ranges and stdin checking, consumed by extension 0.3.0 diagnostics:
  [0011](decisions/0011-compiler-source-locations-and-editor-diagnostics.md).
- Initial declarative syntax highlighting:
  [0009](decisions/0009-declarative-vscode-syntax-highlighting.md).
- These records contain the original test evidence and limitations; they do not imply a complete language server.

### 2026-10-12 — counted native iteration

- Core v0.2 counted folds with immutable carried state, native lowering, and versioned checked artifacts:
  [0008](decisions/0008-counted-folds-with-immutable-state.md).

### 2026-10-10 — foundations and native execution

- Canonical semantic graph accepted for prototyping, not yet implemented:
  [0001](decisions/0001-canonical-semantic-graph.md).
- Human-auditable projections required: [0002](decisions/0002-human-auditable-projections.md).
- Existing execution targets preferred over a new VM:
  [0003](decisions/0003-existing-execution-target-first.md), subsequently updated by the native-first decision.
- Rust compiler and small executable Core:
  [0004](decisions/0004-rust-reference-compiler-and-core-first.md).
- Interim checked artifact, not final canonical graph interchange:
  [0005](decisions/0005-checked-artifact-for-first-executable-core.md).
- Native machine code became the default, retaining the evaluator for reference testing:
  [0006](decisions/0006-native-machine-code-is-the-default.md).
- Independent differential and sanitizer gates:
  [0007](decisions/0007-native-differential-and-sanitizer-gates.md).
