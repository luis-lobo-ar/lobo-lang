# 0049 Verify native state lifecycles and own unpublished descriptors

Date 2026-11-22. Status implemented, with finite verification limits. Risk mode high assurance.
The user authorized both TLA+ models, deliberately broken variants, concrete C counterexample tests and
complete documentation. This reopens verification and necessary fixes within the existing native contract;
it does not authorize new language syntax, broader networking or an OAuth implementation.

## Choice and implementation

Keep two small source-linked models rather than a monolithic compiler/OS model. Authority covers entry-only
root acquisition, subset permissions, kind/generation checks, quotas, retirement and cleanup. Operations
covers pre-publication ownership, peer validation, requested-byte reservation, short IO, stable per-call
deadlines, interruption, EOF and cleanup. Model actual synchronous behavior; do not invent thread interleavings.
Use installed TLC 1.7.4, one worker, fixed seed/fingerprint and an offline hash check. No dependency is fetched.

Checking only a correct abstract model would leave implementation drift unchecked. Instead, retain mappings,
covered source hashes, expected counterexamples, hand-translated C witnesses and isolated mutations of the
real provider. The one-command runner requires baseline exploration to finish, all intended actions to execute,
and every selected model/C mutation to fail for the expected reason. Compiler failure/crash is not a killed C
mutation. A negative real-TLC test exposed removable Python assertions in the initial checker under -O;
explicit checks and an optimized-Python self-test now guard acceptance. Ordinary Cargo tests replay concrete witnesses without making Java/TLC a dependency of every build.

The operation model exposed this current implementation path: socket acquired, fuel fault during connection
wait, invocation cleanup, unpublished descriptor still open. A failing C regression reproduced the leak before
kernel process teardown. Add a single pending fd owner to the synchronous provider. Error paths discard it;
publication transfers ownership into the registry; normal/fault cleanup closes either owner without budget.
The source contains no fault point between acquisition/ownership assignment or publication/ownership transfer.
Revisit that atomicity claim before callbacks, threads or a signal-handler policy are introduced.

Compiler and extension patch version 1.7.1 retain language core/1.7, checked format 1.0 and native ABI 1.
No stable identity or source migration is required. The extension syntax/navigation code is unchanged and
is revalidated with the native compiler. Preserve earlier 1.7 results and proposal as historical checkpoints.

## Alternatives and limits

A 50,000-step sampled C model alone does not explore the entire selected finite TLA+ state graph. TLA+ alone
checks a model, not C pointers, ABI layout, kernel behavior or a refinement relation. A full refinement or
unbounded proof would be a separate project. The chosen combination checks specific finite design properties
and proves that representative implementation regressions are detected. No cutoff theorem connects these
small configurations to every 64-slot/56-bit execution. Progress depends on stated syscall/fairness/driver
assumptions. The authority model checks safety; it does not prove arbitrary application termination.

The initial IO model incorrectly omitted closes on explicit error-return paths; source review corrected that
false model counterexample. Parser/tool-option failures, preliminary configurations and the subsequent real
fuel-fault counterexample remain in the evidence. The [report](../benchmarks/results/2026-11-22-native-formal-verification.md)
and [model contract](../verification/native/README.md) state actual bounds, corrections and C translations.
