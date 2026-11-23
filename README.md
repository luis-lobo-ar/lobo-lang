# Lobo

Repository: [luis-lobo-ar/lobo-lang](https://github.com/luis-lobo-ar/lobo-lang).

Lobo is an experimental programming language and development environment designed for software agents.
Its primary abstraction is not a source file. It is a versioned semantic workspace containing program
structure, contracts, actions, evidence, history, and explicit uncertainty.

Lobo explores a specific hypothesis:

> Agents will produce more reliable software when they can query and change a program through typed,
> transactional operations instead of reconstructing intent from files and terminal output.

This repository contains a Rust native compiler for Lobo language 1.8. It checks source and emits
optimized machine-code executables with runtime contracts. A reference evaluator supports correctness testing.
The [language 1.8 specification](spec/core-v1.8.md) adds explicit `try` error propagation and inherits inferred generic calls and constructors,
contextual callbacks and case names, automatic JSON builders, typed native OS calls, `fn`, grouped imports, arrow match arms and tracked identity sidecars.
Authoring tools manage IDs separately from ordinary source. The older spelling remains deprecated read
compatibility for existing libraries and fixtures. Both spellings use the same checker and native backends.
The [frozen language 1.0 specification](spec/core-v1.0.md), [1.0 local release workflow](docs/language-1.0.md) and
[release receipt](benchmarks/results/2026-11-04-language-v1-release.json) define its scope and final verification status.
The language is experimental; the wider workspace, WASM backend, and self-hosting are future work.
Closed module linking is available in the compiler/CLI and explicit-project editor tooling. The collection
and tree workloads share one Lobo sequence helper; this remains an experimental language, not a production platform.

## Try the current language

For typed operator bindings and `Text + Text`, see [the executable operator example](examples/operators.lobo).

The [MVC example](examples/mvc.compact.lobo) combines an in-memory model, a typed JSON view and an HTTP response
controller. [JSON helpers](libraries/json/README.md) supply immutable values and serialization. The
[generic MVC example](examples/mvc.generic.lobo) shares missing-to-404 handling through a reusable adapter:

```lobo
fn show_user(id: I64) -> Response {
  respond_json(find_user(id), render_user)
}
```

The compiler infers User from the lookup and uses the view's expected function signature. Explicit type
arguments remain available when argument-based inference is insufficient.
The model's declared `Lookup<User>` also lets it return `Found(user)` or `Missing` without repeating the owner or type arguments.

The compiler accepts that file directly, without a marker or projection command. See
[the native syntax guide](docs/compact-syntax-prototype.md). Extension 1.9.0 highlights and navigates the source
and sends original text for diagnostics. The [older MVC spelling](examples/mvc.lobo) remains a comparison fixture.

The [native examples](examples/native/README.md) compile to one executable with typed capabilities, clock,
randomness and bounded loopback TCP. Launch grants default to deny. See [OS helpers](libraries/native/README.md)
and the [native contract](spec/native-interface-v0.1.md). A [native REST demo](examples/native/rest/README.md) serves a bounded read-only JSON API. TLS and OAuth remain future work.
Compiler 1.7.1 fixes fault cleanup before handle publication. [Two TLA+ models and C witness tests](verification/native/README.md) cover the selected finite lifecycle properties.
A [standalone Windows x64 snapshot](examples/native/windows/README.md) uses the same Lobo source with a
restricted Windows clock/random provider. A separate explicit Windows TCP build profile supports the native REST demo.
A native Windows compiler distribution remains unsupported.
The older [web-server sample](examples/web-server/README.md) uses a Rust HTTP host and stays as a historical
working sample; that host is not required for native OS calls.

Maintained libraries and executable examples use modern 1.2 source with their original stable IDs.
The frozen `examples/v1` corpus, legacy MVC comparison and illustrative transfer sketch retain their historical
spelling. See [the migration record](decisions/0043-maintained-source-migration.md) for scope and verification.
Code fragments below omit the surrounding module and identity metadata. The MVC keeps its unchanged IDs in
`examples/mvc.compact.lobo.ids.json`. Other maintained sources can still use inline identity tables.

With Rust, Cargo, and Clang installed, run from this directory:

```sh
cargo run -- check examples/core.lobo
cargo run -- compile examples/core.lobo -o core
./core --entry example.increment --arg I64:41
./core --entry example.absolute --arg I64:-7
```

The increment invocation returns `42`; the absolute-value invocation returns `7`. The executable runs directly
without Lobo or Clang. It emits JSON on stdout and exits nonzero on errors. `compile` requires a new output path
and refuses to overwrite existing files. Arguments use explicit types and functions are invoked by stable ID.
Use `--fuel` to set the native execution budget. `lobo run` compiles to a temporary binary and executes it;
`lobo eval` explicitly selects the reference evaluator. See the [native backend specification](spec/native-v0.1.md).

Verify with `cargo test --locked` and `./scripts/check.sh`. The transfer example describes later language
features and is not accepted by the Core compiler.

Compiler 0.11.1 also has a direct LLVM backend. With Clang/LLVM 18 installed:

```sh
cargo run -- compile examples/generics.lobo --backend llvm -o generics-llvm
./generics-llvm --entry example.generics.text --arg Text:c3a9
cargo run -- compile examples/generics.lobo --emit llvm -o generics.ll
```

Application functions emit LLVM IR directly; the native runtime and typed layout primitives remain C.
The C backend remains default and available as `--backend c`. Both emit native executables. See
[the LLVM backend contract](spec/native-llvm-v0.1.md) for tools, validation and remaining limits.

For recursive GCD and factorial, see the executable [algorithms example](examples/algorithms.lobo) and
[native development guide](docs/native-development.md). The test suite also requires Clang's AddressSanitizer
and UndefinedBehaviorSanitizer runtimes; release executables do not include them.

Core v0.2 adds counted native iteration with immutable carried state:

```lobo
fold (i in n, total: I64 = 0) { total + i + 1 }
```

See the executable [iteration example](examples/iteration.lobo) and [fold specification](spec/core-v0.2.md).
The compiler now emits `lobo-checked/1.0` and revalidates language 1.0 through 1.9 artifacts.
Older `.loboc` formats must be recompiled from source, not retagged.
Previously generated executables still run independently. `fold` and `in` remain usable as ordinary names.

Core v0.3 adds checked action bounds:

```lobo
fn increment(n: I64) -> I64
actions { demo.observe }
{ n + 1 }
```

Every caller must declare its callee's bounds, including through recursive calls. Contracts remain pure.
Labels are conservative metadata, not host operations or permission grants: this function still only computes
a number. See the [executable actions example](examples/actions.lobo), [Core v0.3](spec/core-v0.3.md), and
[decision 0013](decisions/0013-checked-action-bounds.md). Resource handles, host APIs, quotas, and isolation
are not implemented. `lobo check` returns declared sets by stable function ID in its `actions` JSON field.

## SQLite storage

Core/compiler 1.9 adds a native SQLite adapter and [SQL library](libraries/sql/README.md) with bound values,
typed owned rows, explicit transactions, structured errors and per-invocation grants/limits. SQLite is linked
into the executable. Database operations need neither a Rust server nor an installed SQLite SDK at runtime.
Use [the persistent users model](examples/native/sqlite/README.md) or
[the standalone persistent REST API](examples/native/sqlite-rest/README.md). The earlier read-only fixture
and frozen research evidence remain unchanged. See [Core 1.9](spec/core-v1.9.md) for source capacity and
compatibility, and [the native contract](spec/sqlite-interface-v0.1.md) for the closed policy and limits.

## Typed error propagation

Language 1.8 supports `try producer()` to continue with an Ok payload or return the Err payload through the
current function's result. Error types must match exactly. Postconditions still run. Keep explicit handling
where recovery or resource cleanup is needed. See [the rule](spec/core-v1.8.md) and the
[snapshot program](examples/native/snapshot.lobo). `try(value)` remains an ordinary function call.

Maintained libraries, examples and skill samples now use these constructs where the existing checker can
infer them. Explicit error conversions and cleanup remain visible. See the
[source migration](decisions/0053-maintained-core-1.8-source-modernization.md) for scope and verification.

## Record values

Core v0.4 adds immutable nominal records, nested acyclic values, constructors, field selection, equality,
and record parameters/returns/fold accumulators. See [the executable records example](examples/records.lobo)
and [Core v0.4](spec/core-v0.4.md). Native and reference modes support record arguments; for example:

```sh
cargo run -- run examples/records.lobo --entry example.records.swap --arg Record:example.records.pair:I64:1,I64:2
cargo run -- run examples/records.lobo --entry example.records.gcd --arg I64:1071 --arg I64:462
```

Record fields are flattened in declaration order for CLI input; output is typed JSON. Layouts and copying
are bounded; Core v0.6 supports growable lists. Core v0.10 permits nominal recursion through lists;
direct inline cycles remain unsupported.
This advances, but does not complete, the language goal.

## Alternative values and typed failures

Core v0.5 adds nominal tagged alternatives and exhaustive expression matching for ordinary typed failures:

```lobo
variant Answer { Ok(I64), Empty }
fn fallback(a: Answer) -> I64 {
  match a { Ok(n) => n, Empty => 0 }
}
```

See the executable [safe-division example](examples/variants.lobo) and [Core v0.5](spec/core-v0.5.md).
Variants compose with records, functions, equality, contracts, and folds. Only the selected match arm runs;
all cases must be handled and are statically checked. Runtime faults/exhaustion remain diagnostics, not caught values.

```sh
cargo run -- run examples/variants.lobo --entry example.variants.divide --arg I64:20 --arg I64:0
cargo run -- run examples/variants.lobo --entry example.variants.fallback --arg Variant:example.variants.quotient:Case:Ok,I64:5
```

## Growable lists

Core v0.6 adds immutable growable `List<T>` values. `List::empty<T>()`, `prepend`, `head`, `tail`, and `length`
compose with records, variants, contracts, and folds. Tails are shared, not copied. Allocation has an explicit
cumulative allowance (`--heap-cells`, default 1,000,000); argument/result size limits bound JSON expansion.
Native nodes are reclaimed when the invocation terminates, including structured faults.

The executable [collection workload](examples/collections.lobo) validates input, filters in original order,
aggregates values, and returns typed failures entirely in Lobo:

```sh
cargo run -- run examples/collections.lobo --source examples/sequences.lobo --entry example.collections.select --arg List:Count:3,I64:5,I64:2,I64:8 --arg Bool:true --heap-cells 1000
```

This returns selected values `[2,8]`, count 2, sum 10. See [Core v0.6](spec/core-v0.6.md) for lifetime, budgets,
comparison metering, input encoding, and limits. Recursive named types remain pending.

## Text and bytes

Core v0.7 adds immutable `Bytes` and validated UTF-8 `Text`, quoted Text literals with JSON escapes, explicit
conversions, lengths, indexing, shared slices, concatenation, and bulk byte construction from `List<I64>`.
Text indices count Unicode scalar values; byte indices count octets. There is no implicit normalization or
replacement of invalid encodings. Local buffer allocation/traversal uses the existing heap/fuel budgets.

The executable [text example](examples/text.lobo) frames Unicode text and returns invalid bytes as an ordinary
typed failure. CLI atoms use hex to carry arbitrary bytes without shell/NUL/comma ambiguity:

```sh
cargo run -- run examples/text.lobo --entry example.text.frame --arg Text:f09f90ba
cargo run -- run examples/text.lobo --entry example.text.decode --arg Bytes:ff
```

These return `[🐺]` and an `Invalid` variant carrying the original `ff` byte, respectively. See
[Core v0.7](spec/core-v0.7.md) for conversion, encoding, literal, input/output, and exact cost rules.

The executable [arithmetic parser](examples/parser.lobo) uses ordinary Lobo records, alternatives, lists,
and folds to scan and evaluate byte input with precedence and parentheses. Invalid input/overflow returns
a typed error with its byte offset; resource exhaustion remains a diagnostic. For `12 + 3 * 4`:

```sh
cargo run -- run examples/parser.lobo --entry example.parser.evaluate --arg Bytes:3132202b2033202a2034
```

This returns `Ok(24)`. The [frozen workloads](spec/language-milestone-workloads.md) and
[parser decision](decisions/0019-parser-workload-in-existing-lobo.md) record the tested cases and remaining gaps.

The executable [tree example](examples/tree.lobo) constructs a binary search tree in a growable node table,
ignores duplicates, and traverses it in either order without recursive named types or a host sorting shortcut:

```sh
cargo run -- run examples/tree.lobo --source examples/sequences.lobo --entry example.tree.summarize --arg List:Count:4,I64:3,I64:1,I64:3,I64:2 --arg Bool:false
```

This returns keys `[1,2,3]`, count 3, height 3. `Bool:true` reverses traversal without changing count/height.
All three acceptance workloads are executable. Collection/tree callers share the sequence module without
changing public entry IDs; see [the coverage assessment](docs/language-completeness.md#coverage-assessment-after-the-three-workloads).

## State-driven iteration

Core v0.8 adds an immutable state-carrying loop with no predetermined count:

```lobo
while (n: I64 = 0; n < 5) { n + 1 }
```

It initializes once, tests the current state, and returns the state on false; a true test carries the body's
result into the next test. Each probe consumes fuel, and iteration does not grow the call stack. There is
no assignment or implicit permission. See [Core v0.8](spec/core-v0.8.md) and executable
[GCD/early list search](examples/state_iteration.lobo). Parser reduction and tree search now use this mechanism.

## Explicit module composition (experimental)

Explicit composition imports a function, record, or variant by stable declaration ID from a supplied
source set. Modern source uses `use sequences { reverse };` with a table entry
`sequences.reverse: "example.modules.reverse"`. Imports never search paths or download code.
Collection/tree callers and the [module examples](examples/modules/reverse.lobo) share one
[reverse helper and cursor type](examples/sequences.lobo); existing module-example entries delegate through a compatibility wrapper:

```sh
cargo run -- run examples/modules/reverse.lobo --source examples/modules/sequences.lobo --source examples/sequences.lobo --entry example.modules.reversed --arg List:Count:3,I64:1,I64:2,I64:3
```

This returns `[3,2,1]`. [The other caller](examples/modules/mirror.lobo) reverses twice using the same
implementation. Agents can submit a bounded source snapshot to `check --bundle-stdin`, without source IO,
Clang, or execution. See [Core v0.9](spec/core-v0.9.md) for scopes, nominal identity, budgets, bundle format,
and compiler bounds. Extension 0.11.0 uses explicit `lobo.sourceFiles` workspace-relative lists for remote
definition targets and dependency-aware squiggles, including unsaved text; no workspace scanning is performed.

## Recursive values

Core v0.10 allows recursive records and variants when every cycle passes through a `List`:

```lobo
variant Node { Num(I64), Sum(List<Node>), Product(List<Node>) }
```

The [AST example](examples/ast.lobo) evaluates nested sums/products and rewrites every literal while
preserving child order. All computation runs in Lobo; no host AST evaluator supplies its implementation.

```sh
cargo run -- run examples/ast.lobo --entry example.ast.shifted_value --arg Variant:example.ast.node:Case:Num,I64:7 --arg I64:1
```

This returns `8`. Existing lists provide heap indirection, and direct inline cycles remain rejected.
Input/output nesting stays bounded to eight; internal values can be deeper, subject to allocation,
fuel, call and structural-comparison limits. See [Core v0.10](spec/core-v0.10.md) and
[the frozen development workload](spec/recursive-ast-workload.md). Function values remain future work.
The Core 0.9 comparison baseline is preserved separately.

## Generic functions

Core v0.11 specializes explicit generic functions to concrete native code:

```lobo
fn identity<T>(value: T) -> T { value }
fn number(value: I64) -> I64 { identity::<I64>(value) }
```

The [executable example](examples/generics.lobo) reuses functions for numbers, text and records.
Imported generic helpers may also use caller-owned record types. Definitions are checked parametrically,
including unused bodies. Concrete instances retain contract and action checks. Modern function calls can infer type arguments. Constructors can infer arguments from their members and expected nominal types.
Generic data and pure function references are supported. See the 1.4 specification for bounded argument-based inference.
See [Core v0.11](spec/core-v0.11.md) and [the public reuse workload](spec/generic-reuse-workload.md).

```sh
cargo run -- run examples/generics.lobo --entry example.generics.text --arg Text:c3a9
```

## Agent programming skill

The repository includes [lobo-programming](.agents/skills/lobo-programming/SKILL.md), targeting
language 1.9/compiler 1.9.0 with language/tool references and executable examples. In Codex, explicitly
invoke it with `$lobo-programming` when writing or repairing Lobo programs. The skill supplies
knowledge; compiler checks, independent tests and host authority remain separate.

The [twenty-attempt comparison](benchmarks/results/2026-11-03-skill-evaluation.md) accepted every
attempt in both skill and equivalent-guide arms. Skill invocation worked, but no accuracy or
consistent efficiency advantage emerged. A plugin and implicit-discovery evaluation remain future work.

## VS Code language support

The [Lobo extension](editors/vscode/README.md) recognizes `.lobo` files and highlights language 1.8
declarations, contracts, types, literals, comments, operators, and counted folds. It also supports line
commenting and bracket/quote pairing, plus Ctrl-click/F12 definition navigation for functions, parameters,
local bindings, fold bindings, record fields, variant cases, and match payload bindings. Highlighting/navigation need no compiler; compiler-produced squiggles
require Lobo 0.2.1+. No runtime npm dependencies are required.

Build and install locally:

```sh
cargo test --locked
cd editors/vscode
npm ci --ignore-scripts
npm run package
code --install-extension ../../dist/lobo-syntax-1.9.0.vsix
```

Navigation follows Core's scope rules within the current file or an explicit source set and reads unsaved edits.
Cross-file reads require workspace trust; bundle diagnostics require compiler 0.9+. This is lightweight
editor support, not a language server or validator. See the extension README for development, tests, and
limitations. Node.js 22+ is needed only to build/test the extension.

Compiler diagnostics include optional exact source ranges: UTF-8 byte offsets and zero-based UTF-16
line/character positions. `lobo check --stdin` checks unsaved source without Clang, program execution, or
temporary files. See [source diagnostics](spec/source-diagnostics-v0.1.md). The repository's VS Code workspace
setting uses `target/release/lobo`; build the release compiler and open the repository as a trusted folder
for squiggles. Compiler checks are disabled in Restricted Mode, and can be turned off independently.

## Optimized release

```sh
cargo build --locked --release --bin lobo
./target/release/lobo run examples/core.lobo --entry example.increment --arg I64:41
stat -c '%n %s bytes' target/release/lobo
```

The release profile optimizes for small executable size, enables full link-time optimization, uses one codegen
unit, strips symbols, and omits debug information. Internal Rust panics abort the process; ordinary Lobo errors
still return structured diagnostics. Runtime contracts and checked arithmetic remain enabled. The native
executable targets the build host; the current Linux build depends on the system's C runtime.

## Agent-first testing prototype

`lobo-test` is a native foreground tool for agents testing Lobo programs. It pins requirements/expected
outcomes, verifies explicit candidate snapshots, reuses eligible in-session passing evidence, and returns
compact differences with exact-source explain/replay. Ordinary Lobo functions can express test laws;
no test keyword, interpreter dependency, daemon or external library is introduced.

```sh
cargo build --locked --release --bin lobo-test
./target/release/lobo-test --session --clang /usr/bin/clang < examples/testing/agent-session.jsonl
```

The example intentionally introduces a failing increment and replays it; inspect response statuses, not
the transport's exit code. `scope:all` remains the distinct full-run gate. Reuse relies on a host-declared
immutable environment identity; an empty identity disables it. This is an experimental interaction, not
proof of arbitrary correctness, protected external approval, or measured AI-agent productivity.
The later [ten-trial real-agent pilot](benchmarks/results/2026-11-03-agent-repair-pilot.md) validates
repair interaction and finite independent grading; its productivity comparison remains inconclusive.
The [30-attempt repeated library study](benchmarks/results/2026-11-03-library-repair-study.md)
exercises detail/replay and separates feedback from selection. It retains 29 accepted repairs
and one controller interruption; execution savings do not establish general agent productivity.
See the [protocol](spec/agent-testing-v0.1.md), [decision/evidence](decisions/0024-agent-first-verification-prototype.md),
and [scripted full-versus-affected comparison](benchmarks/results/2026-10-18-agent-testing-prototype.json).

## Design principles

1. **Semantics before syntax.** The canonical program is a typed graph; text is one projection.
2. **Intent is executable.** Contracts, examples, policies, and resource budgets are program artifacts.
3. **Actions are explicit.** Functions declare possible resource interactions; capabilities supply authority,
   and the host enforces scopes and limits. Declaring an action does not grant permission.
4. **Every mutation is transactional.** A change names its base revision and either commits with evidence or
   leaves the workspace unchanged.
5. **Context is queried.** The environment returns bounded, attributable context packages instead of asking an
   agent to read an entire repository.
6. **Evidence travels with change.** Verification results are tied to exact revisions, environments, and
   assumptions.
7. **Humans retain oversight.** Machine-native storage must have stable, readable, auditable projections.
8. **The host enforces authority.** A model may request a tool call; the host validates and executes it.

## The system

```text
agents, editors, CI, human review
                 │
          Lobo Tool Protocol
                 │
               lobod
      ┌──────────┼───────────┐
 semantic DB   verifier   sandboxed runner
      └──────────┼───────────┘
       content-addressed store
                 │
       native / optional WASM backends
```

`lobod` is the proposed workspace daemon. It owns revisions, semantic queries, transactions, validation,
execution, and evidence. All clients use the same protocol; the CLI is an adapter, not a privileged interface.

## Repository map

- [Vision](VISION.md) — the problem, thesis, boundaries, and measures of success.
- [Architecture](docs/architecture.md) — components, artifacts, trust boundaries, and lifecycle.
- [Language design](docs/language-design.md) — the initial semantic model and illustrative syntax.
- [Core v0.11 specification](spec/core-v0.11.md) — explicit generic functions and bounded native specialization.
- [Core v0.10 specification](spec/core-v0.10.md) — inherited list-guarded recursive values, allocation and bounded traversal.
- [Core v0.9 specification](spec/core-v0.9.md) — inherited module linking, scope metadata and editor source snapshots.
- [Checked artifact format](spec/core-v0.9.md#checked-artifact) — flat declaration graph, ownership scopes, inherited value/loop IR.
- [Language acceptance workloads](spec/language-milestone-workloads.md) — frozen inputs/outcomes and implementation status.
- [Native backend](spec/native-v0.1.md) — machine-code generation, executable CLI, semantics, and limits.
- [Native development](docs/native-development.md) — executable algorithms, generated corpus, and sanitizer gates.
- [Initial native measurements](docs/native-performance.md) — executable sizes and scoped Rust comparison.
- [Repeatable comparisons](benchmarks/README.md) — native, interpreted, and Rust measurements.
- [Result register](benchmarks/results/README.md) — all recorded experiments, findings and evidence limits.
- [Worktree workflow](docs/worktree-comparisons.md) — baseline/candidate checkouts and source attribution.
- [Compiler and backends](docs/compiler-and-backends.md) — Rust implementation, checked IR, WASM, and LLVM.
- [Compiler development](docs/compiler-development-workflow.md) — specification and implementation loop.
- [Testing and TDD](docs/testing-and-tdd.md) — conformance, differential testing, fuzzing, and release evidence.
- [Agent-first tests](spec/agent-testing-v0.1.md) — native testing session, fixed criteria, bounded reports and reuse limits.
- [Self-hosting](docs/self-hosting.md) — prerequisites and reproducible bootstrap stages.
- [Development workflow](docs/development-workflow.md) — the agent change loop and multi-agent coordination.
- [Tool protocol](docs/tool-protocol.md) — discovery, requests, responses, safety, and transport.
- [State and context](docs/state-and-context.md) — explicit state categories and context packages.
- [Roadmap](docs/roadmap.md) — staged implementation with exit criteria.
- [v1.0 target](docs/v1.0-target.md) — the agent workspace outcomes the minimal Core version serves.
- [Transfer example](examples/transfer.lobo) — a non-normative sketch of Lobo source.
- [Agent session](examples/agent-session.jsonl) — an illustrative protocol exchange.
- [Protocol schemas](spec/schemas) — early machine-readable envelopes.
- [Architecture decisions](decisions) — durable design choices and their consequences.
- [Change ledger](CHANGELOG.md) — dated changes, decision history, compatibility notes, and verification links.
- [Research record](research/README.md) — annotated academic related work, versioned bibliography and a proposed paper study.

## Proposed vocabulary

| Term | Meaning |
| --- | --- |
| Program graph | Canonical typed representation of a Lobo program |
| Projection | A deterministic view of graph content, such as readable source |
| Revision | Immutable content-addressed workspace state |
| Transaction | Isolated sequence of operations against one base revision |
| Obligation | A property that must be decided, tested, or proven |
| Evidence | A revision-bound result supporting or refuting a claim |
| Context package | Bounded, attributable information assembled for a goal |
| Capability | Explicit authority to observe or affect a resource |
| Action | Declared possible resource interaction, not an authorization grant |
| Limit | Host-enforced constraint on execution or resource use |

`actions { ... }` is supported at function signatures in Core v0.3. `actions` remains usable as an ordinary
identifier elsewhere. Host resource access remains future work; see [the implementation decision](decisions/0013-checked-action-bounds.md).

## Language-first milestone

Build a small general-purpose language core capable of expressing representative self-contained programs,
without moving their computational logic into another language. Prioritize structured/growable data, explicit
failures, reusable algorithms, and program composition. Add constructs when demonstrated patterns justify
them, not to reproduce another language's feature list. Native compilation, TDD, and editor support remain
required. The defined language-only milestone has local acceptance evidence in the
[requirement audit](docs/language-completeness.md#language-only-requirement-audit--2026-10-18).
This is not universal language replacement, production readiness, or a Turing-completeness claim.

See [the language-completeness goal](docs/language-completeness.md) and
[decision 0014](decisions/0014-language-first-development.md). Ecosystem integrations and the agent workspace
remain deferred until a separate next target is selected; their wider requirements remain recorded.

## Later semantic-workspace goal

After the language-first milestone, the semantic workspace should demonstrate one end-to-end change:

1. Load a small canonical program graph.
2. Ask for goal-specific context.
3. Begin a transaction at an explicit revision.
4. Add a semantic operation without rewriting a source file.
5. Return affected obligations.
6. Verify the changed closure.
7. Commit a new revision with an evidence manifest.
8. Render a deterministic human-readable projection.

See the [roadmap](docs/roadmap.md) for the acceptance criteria and deliberate non-goals.

## Naming

The project directory is `lobo-lang`; the language, CLI, and source extension are `Lobo`, `lobo`, and `.lobo`.
This is explicit enough for discovery while keeping the eventual command short. The daemon name `lobod` follows
the usual service convention.

