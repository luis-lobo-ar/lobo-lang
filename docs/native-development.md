# Native-first development

Native machine code is Lobo's default execution path, as defined in
[decision 0006](../decisions/0006-native-machine-code-is-the-default.md). The reference evaluator remains an
explicit testing tool. This step strengthens the next roadmap milestone without changing Core syntax or
claiming that two tiny arithmetic benchmarks establish general application performance.

## Executable algorithms

Use the built `lobo` compiler on your PATH. See [the source-build instructions](../README.md#build-the-compiler-from-source).

The [direct LLVM backend](../spec/native-llvm-v0.1.md) is selectable with `--backend llvm` in compiler 0.11.1.
Shared fixtures now compare C, LLVM and sanitized LLVM on each native invocation; the independent generated
oracle corpus runs both native routes too. Sanitizer probes explicitly exercise hand-emitted LLVM stack
accesses and C-runtime UBSan checks. Use `cargo test --locked --test llvm` for focused backend checks.

[The algorithms example](../examples/algorithms.lobo) contains Euclid's GCD and factorial, using existing Core
conditionals, recursion, checked I64 arithmetic, and runtime contracts. It is executable, not illustrative syntax.

```sh
mkdir -p target/native-examples
lobo compile examples/algorithms.lobo -o target/native-examples/algorithms
./target/native-examples/algorithms --entry example.gcd --arg I64:48 --arg I64:18
./target/native-examples/algorithms --entry example.factorial --arg I64:20
```

The results are `6` and `2432902008176640000`. The executable runs without Lobo or Clang. Compilation refuses
to overwrite an existing output path, so reuse an already compiled binary or choose a new path for another build.

GCD rejects negative inputs and `(0, 0)`, and checks that its positive output divides both inputs. That contract
alone is not a proof of greatestness; the algorithm and independent known-answer tests provide additional
evidence. Factorial accepts 0 through 20 inclusive, so its result fits I64; 21 fails its precondition rather than
overflowing inside recursion. Both algorithms obey the host's fuel and depth limits. Some valid large GCD inputs
can exceed the 32-call-depth limit; native execution currently has no tail-call guarantee.

The local 2026-10-10 build at `target/native-examples/algorithms` is a stripped Linux x86_64 ELF executable,
14,600 bytes, built with the release compiler and normal native `-O3` backend. Both invocations above returned
the expected values. It uses system libraries, not the Lobo evaluator. This is build evidence for this example,
not a claim that all future programs have the same size.

## Repeatable native verification

Run the focused suite:

```sh
cargo test --locked --test native_generated -- --nocapture
```

[The suite](../tests/native_generated.rs) constructs ten bounded modules with ten test entrypoints each:

- 24 explicitly specified boundary, ordering, short-circuit, and contract probes;
- 76 depth-three generated, well-typed expressions using a fixed xorshift64* generator;
- seed `10b0c0de20261010`, with deterministic per-case input/flag values;
- forward calls, immutable branch bindings, both Core types, unary/binary operators, and conditionals;
- independent expected values/fault codes computed using mathematical i128 operations and explicit I64 bounds.

Each case must match its independent expectation through source evaluation, checked-artifact reload, optimized
native execution, and sanitizer-instrumented native execution. Expected language faults must still be JSON
`runtime_fault` outcomes with exit status 1, not a sanitizer abort or resource exhaustion. Successful values
must have exit status 0. Native stderr must be empty. The fixed corpus currently yields 75 values and 25 faults.
The two helper functions in each module are not counted as additional generated cases.

Local verification on 2026-10-10: all 13 integration tests passed in both `cargo test --locked` and
`cargo test --locked --release`; formatting, `cargo clippy --locked --all-targets -- -D warnings`, and
`./scripts/check.sh` passed. The toolchain was Rust 1.92.0 and Ubuntu Clang 18.1.3 on Linux x86_64.

The normal native build uses the compiler's production Clang `-O3` path. Instrumented test binaries use
`-O1 -g -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer`, and halt on sanitizer
errors. These flags follow Clang's [AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) and
[UndefinedBehaviorSanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) documentation.
Separate deliberate signed-overflow and heap-out-of-bounds probes confirm that both kinds of instrumentation
are active. Those probe failures are expected evidence, not Lobo conformance failures.
The generated suite disables stack symbolization: assertions check diagnostic kinds, and external
symbolizer/debuginfod lookups can delay intentional failure probes. Instrumentation remains enabled.

Clang's sanitizer runtimes (`compiler-rt`) are a required development prerequisite for this suite; missing
tools are infrastructure failures, never silent skips. No Cargo dependencies were added. Sanitizers are not
included in production executables. Leak detection is explicitly disabled because classified failure paths
exit the process and leave CLI argument storage for the OS to reclaim; the suite does not claim leak coverage.

Temporary sources/binaries are removed after successful tests, and retained with their path printed on a
panic. Reproduce a mismatch using the fixed seed, case ID, printed arguments, and retained `program.c`.
Shrinking is manual for now; preserve any minimized mismatch as an independent conformance regression.

This is a finite differential corpus, not unrestricted fuzzing, a proof of compiler correctness, coverage of
all runtime diagnostics, or platform certification beyond the documented Linux x86_64 environment. The example
adds 16 independent known-answer/contract invocations, each run through both native builds and the evaluator.

## Counted iteration

Counted iteration is implemented in [Core v0.2](../spec/core-v0.2.md), with executable
[sum and iterative factorial](../examples/iteration.lobo) examples. Run `cargo test --locked --test fold` for
source/reloaded/O3/ASan-UBSan fixtures covering zero/negative counts, types/scope, nested folds, contracts,
contextual names, overflow, and finite fuel. A contract-phase regression found by this suite is retained:
native helper faults inside a contract now preserve the enclosing phase, matching the evaluator.

To build the new example, use a fresh output path:

```sh
lobo compile examples/iteration.lobo -o target/native-examples/iteration
./target/native-examples/iteration --entry example.sum --arg I64:5
./target/native-examples/iteration --entry example.factorial.iterative --arg I64:20
```

These return 15 and 2432902008176640000. Larger sums may overflow or exhaust fuel; finite count is not a
guarantee of completion. Folds currently carry one scalar value, so ordinary iterative GCD still needs
multiple carried values. Old `.loboc` artifacts must be recompiled for the explicit 0.2 format/language pair.
Old generated executables remain independent and unaffected. Records/enums and their memory model need separate definitions.

Local verification on 2026-10-12: all 23 integration tests passed in debug and release, with no skips;
formatting, Clippy with warnings denied, and JSON/link checks passed. Rust 1.92.0 and Ubuntu Clang 18.1.3
were used on Linux x86_64. The release compiler is 669,992 bytes. The new iteration example is a stripped,
dynamically linked x86_64 ELF executable of 14,600 bytes; both invocations above returned the expected values.
Its SHA-256 is `2017d28bc167d2021df4f92c6b38f6c3f36a5351affb39a9a4763aa529c15c4c`.

An initial debug attempt, run alongside the release suite, encountered Linux `Text file busy` while launching
a just-built native binary. Subsequent separate full debug and release runs passed with default test parallelism;
the underlying transient cause is not confirmed and no execution retry was added. Intentional sanitizer failure
probes also took 180 seconds with symbolization enabled; disabling unnecessary symbolization reduced the generated
suite to about five seconds while preserving both positive instrumentation checks. Neither event was counted as
a successful initial verification run.

The semantic workspace and agent tooling remain the product goal; native speed is an execution choice, not a
replacement for stable identity, explicit context, capabilities, or revision-bound evidence.

## Language-first parser workload (Core 0.7)

The earlier counted-iteration section records its historical scalar-only slice; records, alternatives,
lists, and text/bytes are now supported. The executable [parser](../examples/parser.lobo) scans Bytes and
evaluates arithmetic entirely in Lobo, using immutable operator/value stacks rather than host parsing or
recursive-descent calls. Run `cargo test --locked --test workloads` for the frozen cases, 64 seeded expression
variations (seed `0x7a310f29`), all 256 single octets, 128-level nesting, arithmetic limits, typed byte offsets,
and fuel/heap failure cases. Every workload case runs through the checked-artifact reference path, optimized
native code, and ASan/UBSan native code with leak detection. These are finite conformance checks, not a proof
of correctness or a performance claim. See [decision 0019](../decisions/0019-parser-workload-in-existing-lobo.md).

## Language-first tree workload (Core 0.7)

The executable [tree](../examples/tree.lobo) builds a binary search tree using parent-indexed immutable nodes,
then traverses it with explicit state. Run `cargo test --locked --test tree` for frozen outputs, actual parent
edges, ascending/reverse traversal, duplicates, I64 minimum/maximum keys, and 40-level ascending/descending
chains. Thirty-two positive affine variations of the frozen 15-node balanced shape use seed `0x6d20a815`;
expected order/count/height follow from the known shape, not a host tree implementation. All cases run via
reloaded checked artifacts, optimized native code, and ASan/UBSan native code with leak detection. Normal
cases use fuel 1,000,000 / heap 100,000; deep chains use fuel 20,000,000 and affine variations 2,000,000.
Separate exhaustion cases require structured diagnostics without partial summaries. No compiler or runtime
change was needed. This is expressiveness evidence, not a performance claim or proof of Turing completeness;
see [decision 0020](../decisions/0020-tree-workload-in-existing-lobo.md).

## State-driven iteration (Core 0.8)

`cargo test --locked --test while` covers typed carried-state loops, exact five-tick false termination,
initialization/condition/body fault ordering, heap sharing and final-condition allocation, nontermination,
contracts/actions, compound layouts, contextual names, ranges, artifacts, and nesting. A compiler-stack
overflow regression led to precedence-climbing parsing without lowering the existing depth allowance;
canonical operator grouping and nonassociative comparison/equality rejection have dedicated checks.
The executable [GCD/search example](../examples/state_iteration.lobo) returns a head match with execution
fuel 100 for both one-element and 1,001-element inputs (input admission/heap still scales with input).
Parser reduction and tree search now use state-driven termination; earlier sections describe their original
Core 0.7 implementations. All frozen workload expectations remain unchanged. See
[Core v0.8](../spec/core-v0.8.md) and [decision 0021](../decisions/0021-immutable-state-driven-iteration.md).

## Generic function specialization (Core 0.11)

`cargo test --locked --test generics` checks the public reuse workload, specialization identity/sharing,
module scope, template checking boundaries, contracts/actions, forged artifacts, admission limits and
native/reference/ASan/UBSan behavior. `examples/generics.lobo` is an independently runnable small example.
Generic templates lower to concrete functions/scopes before the existing checker and native emitter;
there is no interpreter or new runtime type dispatch. See [Core v0.11](../spec/core-v0.11.md) and
[decision 0030](../decisions/0030-explicit-generic-function-specialization.md). Core 0.10 compiler/source
evidence is preserved before this change; acceptance is not a cross-language agent comparison.

## List-guarded recursive data (Core 0.10)

`tests/recursive_values.rs` covers guarded/mutual nominal types, malformed arguments, old-artifact
rejection, forged inline cycles, exact allocation thresholds, logical input/output depth, comparison
depth/fuel and contract phases. It constructs/reclaims 10,000 nested nodes and checks success, partial
allocation failure and rejected deep output in the reference, native and ASan/UBSan/leak paths.
The public `examples/ast.lobo` evaluates and rewrites ordered n-ary syntax trees entirely in Lobo.

List pointers break physical layout dependence. Native emission defines inline layouts first, declares
all structural helpers, then emits bodies. Equality caps active compound depth at 64; input/output
depth stays eight. Reference reclamation walks nested list/compound values iteratively. See
[Core v0.10](../spec/core-v0.10.md), [workload](../spec/recursive-ast-workload.md), and
[decision 0029](../decisions/0029-list-guarded-recursive-values.md). Existing Core 0.9 source/compiler
snapshots are preserved for comparisons. Development examples are not held-out agent benchmarks.

## Closed module linker (Core 0.9)

`cargo test --locked --test modules` covers native/reference/sanitized shared functions/nominal types,
module-local name collisions, deterministic linking, direct-import visibility and forged scope metadata,
actions/contracts/resource subjects, mutual function dependencies, recursive-layout rejection, source
labels/ranges, aggregate budgets, and the agent/explicit-file CLI adapters. The module examples in
`examples/modules` and actual collection/tree workloads delegate to the one implementation/cursor in
`examples/sequences.lobo`, preserving public function IDs. Supply `--source examples/sequences.lobo` to
collection/tree CLI invocations; module examples also require their wrapper module. Existing workload
oracles and resource subjects remain unchanged. Extension 0.11.0 supplies explicit-project remote navigation
and bundle diagnostics, including unsaved overlays and dependency invalidation. See
[Core v0.9](../spec/core-v0.9.md), [decision 0022](../decisions/0022-closed-module-composition.md), and
[editor snapshot evidence](../decisions/0023-explicit-editor-source-snapshots.md).

## Native interface 1.7 verification

The [native interface](../spec/native-interface-v0.1.md) has failing-first ABI/artifact/resource/TCP tests,
C and LLVM native routes, ASan/UBSan, a fault-injected C provider harness, a fixed-seed handle model and
exhaustive four-bit delegation masks. [Upstream attribution](../tests/upstream/native/README.md) distinguishes
adapted cases from complete upstream suites. Run `python3 scripts/check-native-upstream.py` and the full
Cargo suite. OS calls intentionally have no reference evaluator fallback. The two native backends share their
C provider, so their agreement does not independently validate that provider.

The [result record](../benchmarks/results/2026-11-22-native-boundary.md) retains red/green attempts, corrections,
commands and scope limits. Editor native-type/signature/payload navigation, unsaved diagnostics and repair
are covered by Node and the real isolated VS Code host. No IO is executed by compiler diagnostics.

## Native state-model gate

Compiler 1.7.1 adds source-linked finite authority/IO TLA+ models and concrete C witnesses. See
[the model contract](../verification/native/README.md) and apply the local lobo-native-verification skill
when changing covered code. The one-command checker requires a pinned installed TLC jar, completed baseline
exploration, enabled-action coverage and expected rejection of isolated model/C mutations. It refuses stale
source mappings. Ordinary Cargo runs the C witnesses without requiring TLC. The model found a real
fuel-fault cleanup gap before connection-handle publication; its failing-before and corrected C runs are
retained in [the result](../benchmarks/results/2026-11-22-native-formal-verification.md).
These checks establish neither C refinement nor unbounded correctness.


## Optional SQLite extension

Core 1.9 adds a separately marked SQLite extension to the closed native catalog. See
[the extension contract](../spec/sqlite-interface-v0.1.md),
[SQL helpers](../libraries/sql/README.md) and the
[persistent REST example](../examples/native/sqlite-rest/README.md).
The C and direct LLVM backends compile the same pinned amalgamation and C adapter into the executable.
No SQLite DLL, Rust service or interpreter is needed to run it. Programs that do not use SQLite retain
the earlier native layouts and omit the engine. Source compilation still uses the Rust compiler.

Run the SQLite adapter, library and source-correspondence Cargo tests, the separate
[SQLite lifecycle gate](../verification/sqlite/README.md), editor tests and extension-host checks.
The test corpus covers binding, NULL/empty values, ownership, transaction failure/rollback, denial,
limits, stale handles, allocation/copy faults and independent on-disk SQLite oracles. Linux and Wine
executable runs are distinct from actual Windows hardware coverage. Retained upstream Tcl tests and
adapted cases do not imply that the entire upstream SQLite suite ran.
