---
name: lobo-programming
description: Write, repair and test .lobo programs using Lobo language 1.11, explicit source bundles and native tools. Use for Lobo application code; compiler implementation in Rust has a separate repository workflow.
---

# Develop Lobo programs

This skill targets Lobo language 1.11 and compiler 1.11.2. Read the project's requirements
and tool contract first. Treat the compiler and current [specification index](../../../spec/README.md)
as the language authority. A newer or older compiler requires checking its matching specification.

Use modern `module name;`, `fn`, grouped `use` imports and arrow match arms. Track the adjacent
`.lobo.ids.json` identity file with the source, or retain an existing inline identities table.
The compiler parses this source directly. No marker, Node.js command or source projection is required.
Use `lobo identities init FILE` to allocate new IDs and `sync` after adding declarations. Supply explicit
`--source` dependencies. After renaming source declarations/references, use `identities rename FILE OLD NEW`
to retain the old ID under the new binding. These commands write metadata only. Checks never allocate IDs.
Move source and sidecar together. Restore lost metadata from version control instead of allocating replacement IDs.
The `json { ... }` shorthand accepts Text fields. Select `Json` and any functions called in source;
do not select `object`/`field` just for literals. Compiler 1.6+ binds those builders by stable ID from explicitly
supplied JSON/HTTP library sources. Checked imports expose the dependencies; hidden aliases are not callable.
The [small JSON example](references/json-example.lobo) demonstrates this without an application solution.
The older `function` and inline-ID spelling remains deprecated read compatibility for existing code.
Read [the 1.11 rules](../../../spec/core-v1.11.md) and [native commands](../../../docs/compact-syntax-prototype.md).
The [small modern example](references/compact-example.lobo) is independent of the MVC comparison workload.

Language 1.0 includes generic records/variants, checks all generic definitions parametrically, and supports
pure non-capturing `Fn(T) -> U` values via `&function` or `&generic::<Type>`. Unconstrained `T` has no
arithmetic/field/call operations; use explicit concrete types or signatures. F64 adds homogeneous binary64
arithmetic/comparisons and explicit `F64::from_i64` / checked `F64::to_i64`; JSON uses exact lowercase hex bits.
Language 1.1 adds `Text + Text` and module-level bindings such as `operator +(Vec2, Vec2) -> Vec2 = add;`.
The named target is local, pure, non-generic and matches the exact signature. The module must own an operand
type. Importing the named target enables its binding. Available symbols are binary `+ - * / %`.
Language 1.4 infers omitted generic function arguments from call arguments and monomorphic callback
signatures. Write `identity(value)` and `respond_json(find_user(id), render_user)` when determined.
Named callbacks need no `&` when an exact Fn signature is expected. Generic callbacks can specialize from
a complete expected Fn signature. Unresolved outer types need explicit arguments; `apply(identity,42)`
may require `apply::<I64,I64>(identity,42)`. Language 1.5 infers constructor arguments from members and expected
nominal types. Generic function calls still do not infer type arguments from an expected result. Constructors
can use that context. Bare unannotated function bindings still need `&`. Exact callback action bounds and rigid generic checks remain.
When an expected variant identifies the owner, use `Found(value)` or `Missing` without repeating its name.
For example, a function returning `Lookup<User>` can return `Found(user)` or `Missing`. Qualified constructors
such as `Lookup::Found(user)` can infer arguments from payloads; generic records infer from fields or expected
types. Without an identifying context, a bare case stays unresolved. Explicit aliases and locals take precedence.
Do not search variants by case spelling or infer a function signature. The small skill examples preserve IDs.
Unconstrained T still has no arithmetic. See [the language specification](../../../spec/core-v1.7.md).

Use [language details](references/language.md) when syntax, types, iteration or buffer semantics
are unfamiliar. Use [tool details](references/tools.md) for native invocation, typed arguments,
source bundles or verification-session handling. [Small executable examples](references/examples.lobo)
demonstrate syntax without implementing an application algorithm.

Lobo returns the final expression of a block. Bindings and collections are immutable. Use
`fold` or `while` with carried state instead of assignment. Do not invent a `return` statement,
mutable arrays, exception catching or package discovery. Make resource interactions
explicit; an action label does not grant authority or implement a host call.

Keep requirements and expected outcomes independent of candidate output. Reproduce a failure,
make a focused change, then run the relevant checks. Inspect compiler diagnostics before adding
workarounds for a syntax/type error. Use a bounded counterexample or retained observation when
the summary omits the input; avoid repeatedly loading the entire project for a single failure.

If the project supplies a test client, use its declared commands and fixed criteria. Otherwise
use the native compiler/CLI as documented. A transport exit code is not test acceptance. Pending,
resource exhaustion and missing tools do not pass. Preserve explicit snapshot bases and budgets;
replay confirms an observation, not a repair. Finish with fresh full verification when the task
requires an acceptance gate, and record results, failures and remaining limits with source identity.

Do not relax a frozen test plan or silently restart a full session. Skill instructions supply
language knowledge, not permission to read other projects, install dependencies or access services.

Language 1.7 adds a closed `Native::` catalog for 64-bit Linux. Read
[the native contract](../../../spec/native-interface-v0.1.md) and [OS helpers](../../../libraries/native/README.md).
Acquire `Native::capability()` directly in the executable entry, pass it to helpers and restrict it before
delegation. Actions do not grant authority; launch flags default to deny. Compile then invoke the executable
with explicit grants; the `run` helper does not forward grant options. OS calls cannot run through `eval`.
Resources cannot be literals, external inputs or printed outputs. The
[small native example](references/native-example.lobo) covers owned byte copying and an entry clock request.
No arbitrary FFI, external network, TLS, HTTP server or OAuth support is implied.

Compiler 1.7.1 fixes pending connection cleanup on faults without changing language or ABI. When changing
the compiler's native provider rather than application code, use the repository's lobo-native-verification
skill and source-linked model gate. Do not infer security or production readiness from a bounded model pass.

A standalone Windows snapshot distribution uses the same Lobo source with a bounded clock/random provider.
See [Windows demo](../../../examples/native/windows/README.md). Do not infer full Windows compiler or network
support from that artifact. Linux formal-model checks do not cover the Windows provider.

For a standalone local REST API, use [the native REST example](../../../examples/native/rest/README.md).
Supply its controller, transport, parser and JSON/HTTP sources as one closed bundle. The app uses a pure
Fn(Request)->Response callback, with networking and clock actions declared by the transport. Compile-time
LOBO_WINDOWS_TCP selects the Winsock cross-build provider. This is not a Rust HTTP host or a Windows compiler.

Use `let value = try producer();` for typed result propagation. Operand and function output must have exactly
Ok(value)/Err(error) cases and matching error types. The enclosing function's success type may differ.
Errors still run postconditions. Keep direct try out of contracts and preserve action declarations.
`try(value)` is an ordinary call, not propagation. Do not use try to bypass explicit close calls; handle the
result, close resources, then return it. See the 1.8 specification for bounds and source compatibility.

[The pure propagation example](references/try-example.lobo) exercises successful and failing results without
native permissions. Compile and run entry skill.propagation.increment with I64:41 or I64:0.

Compiler/extension 1.8.1 retain Core 1.8. Maintained sources use the available inferred spellings, Text +,
concise match arms and try for exact forwarding. Keep explicit arguments/owners where context is insufficient.
Native error-mapping and transport retry matches remain intentional. The REST entry closes the listener
before returning the handled serve result; the loopback example retains explicit stream cleanup.
See [the migration decision](../../../decisions/0053-maintained-core-1.8-source-modernization.md).

Core/compiler 1.9 adds the [closed SQLite interface](../../../spec/sqlite-interface-v0.1.md) and
[SQL library](../../../libraries/sql/README.md). The [small SQLite example](references/sqlite-example.lobo)
selects a literal result without an application solution. Supply libraries/sql/sql.lobo, compile to native,
then launch with an absolute --sqlite-file and an explicit role. Use schema only to create/setup, write for
existing-file mutations and read otherwise. SQL authority is separate from clock/socket capabilities.

Bind values with no_params/params1/params2 instead of constructing SQL with user strings. query returns
SqlResult<List<Row>>. Cell distinguishes Null, Integer, Decimal, String and Binary. Validate row shapes and
storage classes. SqlError carries its kind and original SQLite backend code. Do not retry writes blindly.
Capture operation results before close/finalize. These teardown calls remain available at exhausted query
budgets; try alone does not clean up a transaction. Core 1.11 callbacks declare their action bounds and
receive resources as explicit context, as in the persistent REST example. DB calls cannot run through eval.

SQLite operations select checked sqlite_abi:1 and require Core1.9. No arbitrary driver/FFI, remote database,
NoSQL interface, SQL portability or native sandbox is implied. Native query work has separate quotas from
Lobo fuel. Source bundles allow 16,384 aggregate tokens and 64 source nominal declarations. Individual
source lexing stays at 8,192 tokens, with 16 sources, 1 MiB and the existing type-depth/instance limits.


Core 1.10 supports `State { ...state, changed: value }`. Use one base first with the same record and
matching type arguments. The base runs once, followed by overrides in source order. Unspecified fields
are retained; duplicate/unknown fields and two dots are errors. Source stays immutable. This is not general
JavaScript object merging. Generic arguments can be inferred from the base.
Use `try operation();` to discard its successful payload without an unused let name. Keep a final value
expression in the block. Errors still propagate with exact payload types, postconditions still run and
cleanup remains explicit. Save an operation's outcome before closing its resources when close must run
after a handled error. See [the independent example](references/conveniences.lobo). Source conveniences
lower to existing checked operations and can change fuel usage; do not claim an optimization.

Core 1.11 supports `Fn(C, T) actions { demo.observe } -> U`. Omitted/empty actions mean pure. Named
references capture no environment; pass C explicitly. Function signatures include the exact declared action
set, and an indirect invocation requires that set in the enclosing function. Contracts cannot invoke an
actionful callback. No closures, action variables or implicit widening are available. Study the small
[executable callback example](references/callbacks.lobo) and [current rules](../../../spec/core-v1.11.md).
The HTTP library provides a context-generic serve_with with the concrete db.close/db.execute/db.read bound;
keep grants, bootstrap policy and database ownership in the application. Preserve captured results before close.

Compiler/extension 1.11.1 complete the maintained HTTP library refactor without changing Core 1.11.
For pure controllers, call `serve(cap, port, count, handle)`; for the concrete DB-bound context handler,
call `serve_with(cap, port, count, db, handle)`. Both own listener lifetime. Keep capability selection,
bootstrap and database ownership in the app. Use serve_listener only when the caller deliberately owns an
existing listener; its ID and signature are the former listener-based serve API. The library has one accept
state machine and thin typed callback adapters. Do not add callback action polymorphism to simplify them.
Check [maintained HTTP roles](../../../scripts/http-refactor-scope.json) when adding an example or library;
the source audit rejects unreviewed files and application-owned HTTP listener scaffolding.

Compiler/extension 1.11.2 add typed library permissions on unchanged Core 1.11.
Supply libraries/native/permissions.lobo with native/native.lobo or HTTP transport. OS Permission has
ClockRead, RandomRead, Listen and Connect. Build sets with none(), one(case) and allow(set, case).
Use access.restrict(root, permissions) for native results or os.restrict_capability for IoResult.
HTTP apps request server_permissions(). SQL defines separate Read/Write/Schema permissions and the same
set helpers; use restrict for SqlResult or restrict_native for native error results. SQL one(Read) replaces
raw read-only masks. Sets do not grant authority, duplicates are idempotent and none() requests zero rights.
Do not pass integers or another module's permission type. Raw restrict_mask helpers retain old IDs only
for explicit ABI compatibility. Keep root acquisition at entry and the explicit closure in editor/build inputs.
