# Lobo language 1.11

Status: implemented and verified locally. Compiler/extension 1.11.0, source core/1.11.
Inherit [Core 1.10](core-v1.10.md) except for callback purity below.

## Callback actions

A function-value type can declare a sorted, bounded action set:

```lobo
Fn(Context, Request) actions { db.execute, db.read, db.close } -> Response
```

The source action list is canonicalized. Place it before the arrow so a containing function's
post-output action clause cannot be mistaken for a callback clause. An omitted or empty clause means
pure. Existing function declaration syntax and old pure callback source keep their meanings.
Named references capture no environment. Pass context as an ordinary argument. Contextual named and
generic callback inference work as before, including the callback action set. No action variables,
closure captures, automatic action inference or new native authority are introduced.

Callback parameters, output and declared action set match exactly. There is no implicit action-bound
widening or higher-order variance. A function may declare an upper bound larger than its body's needs,
as before; that declared bound is part of its function-value signature. Obtaining a reference does not
perform the function's actions. Invoking it requires every action in its signature to be declared by
the enclosing function, including unreachable expressions. Contracts cannot invoke actionful values.
Callbacks continue to execute the target's requires/ensures and ordinary fuel/depth/native quotas.

Checked Function signatures add an optional actions array, absent for pure signatures. Nonempty arrays
must be sorted, unique qualified IDs with the inherited 64-action and 128-byte-ID limits. Core 1.0 through 1.10
artifacts cannot contain action-bearing callback signatures or references to actionful targets. Revalidate
loaded and programmatic function arguments by exact target identity and complete signature; native C and
LLVM dispatch, parsing and output use those same signatures. Checked format and native ABI stay unchanged.

## HTTP extraction

Extend the existing HTTP library, passing application context explicitly. Preserve the existing pure
serve API and stable IDs. Add a context-handler entry with the concrete database action bound needed by
the SQLite application. This is a bounded database-handler API, not arbitrary action polymorphism. Shared
transport helpers own deadlines, partial IO, framing and socket cleanup. Application code retains capability
restriction, schema/bootstrap policy and database lifetime. Capture serving results before closing resources;
close failures retain precedence. Preserve fatal-quota versus recoverable-connection behavior, HTTP limits,
HEAD responses and the invocation arena model. No HTTP, TLS, async or database feature expansion is implied.

Verification and preserved failures are recorded in the [milestone report](../benchmarks/results/2026-11-23-action-callbacks.md).
