# Native backend v0.1

Status: implemented on Linux x86_64 using Clang 18. Other targets are not yet advertised as supported.
Language: `core/0.1-draft`. Compiler release: `0.1.1`.

## Compilation

```text
Lobo source → resolution and type checking → checked Core → C lowering → Clang/LLVM → native executable
```

The C representation is a code-generation input. Generated executables contain native instructions for each
Core function, plus a small invocation and error-reporting adapter. They contain no evaluator or program tree.
The compiler uses Clang with C11, `-O3`, no debug information, section garbage collection, and stripped symbols.
The executable uses the host's system C library. Clang is needed for building, not executing, the output.

The first version targets the build host. Explicit target triples, cross-compilation, dynamic libraries,
link-time optimization of application code, and a public foreign-function ABI need later definitions.

## CLI

```sh
lobo compile examples/core.lobo -o core
./core --entry example.increment --arg I64:41
lobo run examples/core.lobo --entry example.absolute --arg I64:-7
```

`compile` creates a new executable and refuses an existing output path. It builds in a private temporary
directory before creating the requested file. The `.loboc` extension is reserved for `--emit checked`.
`run` compiles to a temporary executable, executes it, relays its structured outcome, and removes the temporary
files. For repeated execution, compile once and invoke the binary directly.

Inspection/reference modes are explicit:

```sh
lobo compile examples/core.lobo -o core.c --emit c
lobo compile examples/core.lobo -o core.loboc --emit checked
lobo eval examples/core.lobo --entry example.increment --arg I64:41
```

All generated functions can be selected through `--entry STABLE_ID`; there is no implicit entrypoint name.
Arguments use `--arg I64:DECIMAL` or `--arg Bool:true|false`. Wrong types, arity, IDs, or range are invocation
errors before entrypoint execution. JSON outcomes use the Core interchange rule, with I64 values as strings.
Runtime failures exit with status 1; successful values exit with status 0. Diagnostics include stable codes,
phase, and the affected function ID where available. Diagnostic prose is not a conformance requirement.

## Semantics-preserving lowering

- Locals and function parameters are typed C values, with generated identifiers independent of source names.
- Calls resolve through stable function identity at compile time.
- Operand and argument evaluation is emitted as an ordered sequence of temporaries.
- Conditional and short-circuit operands are emitted inside actual control-flow branches.
- Addition, subtraction, multiplication, and negation use checked overflow builtins.
- Division and remainder explicitly check zero and minimum-I64/-1 before the operation.
- Preconditions run before the body and postconditions run after successful body evaluation.
- Failures terminate the invocation immediately with the specified category and code.

The optimizer may remove checks it proves redundant, but it must preserve all specified observable failures.
Passing tests alone never authorizes removing checks. All potentially overflowing arithmetic in contracts is
checked too. The program uses signed 64-bit integers and cannot inherit C signed-overflow undefined behavior.

## Host limits

The native host defaults to 100,000 fuel; `--fuel N` sets a different budget. Each entered function and evaluated
expression consumes one fuel unit, including contract expressions. Short-circuited expressions consume no fuel.
The host caps calls at depth 32, including calls from contracts. Limits produce structured `resource_exhausted`
outcomes. Exact resource accounting across backends is outside the Core language conformance guarantee.

This runtime is sequential and uses process-local state for fuel, depth, and diagnostic context. It does not
provide an OS sandbox, effectful host APIs, or a library ABI for concurrent callers.

## Verification

Native tests compare values and failure codes against the independent fixture expectations and reference
evaluator. They verify ELF magic, direct execution, CLI convenience execution, output permissions, and refusal
to overwrite an existing executable. Boundary tests cover overflow in every arithmetic family, zero divisors,
short-circuiting, conditional selection, recursion, and invalid host arguments.

