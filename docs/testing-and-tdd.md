# Testing and TDD

Status: Core corpus, compiler behavior, native/reference comparison, checked-artifact round trips, and CLI tests
run through Cargo. WASM differential testing, sustained fuzzing, and bootstrap remain planned.

## What a green result means

“All pass” means every required check for an identified revision and environment completed successfully.
It does not prove that all possible programs behave correctly. A report must state the supported language
subset, suites run, fixture count, skipped cases, tool versions, inputs, and remaining limitations.

The specification is the authority for expected behavior. Agreement between implementations, successful
self-compilation, coverage percentages, and passing generated tests provide complementary evidence.

## Test layers

| Layer | Question | Representative check |
| --- | --- | --- |
| Conformance | Does implementation follow a named language rule? | Overflow faults instead of wrapping |
| Rejection | Does invalid input fail with the right category? | A Bool cannot be added to an I64 |
| Execution | Does valid code produce the specified value or fault? | Conditional evaluates only its selected branch |
| Differential | Do independent execution paths agree? | Evaluator and native binary produce the same outcome |
| Properties | Do general relationships hold for generated cases? | Projection round trip preserves declaration identity |
| Fuzzing | Can unexpected inputs break an assumption? | Random source bytes never crash the compiler |
| Tooling | Does workspace behavior preserve its contract? | Failed commit leaves the branch and graph unchanged |
| Bootstrap | Can the Lobo compiler compile and rebuild itself consistently? | Rebuilt compiler artifacts reach a fixed point |

Compiler projects use multiple suites for success, failure, diagnostics, code generation, and whole programs.
See [Rust compiletest](https://rustc-dev-guide.rust-lang.org/tests/compiletest.html) and the
[LLVM testing guide](https://llvm.org/docs/TestingGuide.html) for established examples.

## TDD's role

TDD is the default for implementing defined language behavior and repairing bugs:

```text
specify observable rule → write failing conformance case → implement → run → refactor → run
```

First establish what the failure means. A test failing because its runner cannot load a file is not evidence
that the compiler violates the intended rule. Once the harness exists, new feature cases should fail in the
expected phase before implementation.

TDD does not choose the language semantics for us. New design choices need written rules and examples before
tests can serve as an oracle. Exploratory implementations may help resolve a design question; their behavior
does not automatically become the specification.

Agents must not rewrite expected results merely to make their implementation pass. An expectation change needs
the revised rule, rationale, compatibility impact, and a test review separate from the implementation claim.
Review the oracle as carefully as the code when the same agent authors both.

## Conformance fixtures

The initial [Core cases](../spec/conformance/core-cases.json) use the
[fixture schema](../spec/schemas/conformance-case.schema.json). Cargo executes these cases against source
compilation and reloaded artifacts. Each case names the rule it covers, its source, and expected phase and result. Cases that
run also include the invoked function ID and typed arguments.

Core fixture families must cover:

- tokenization, precedence, grouping, and invalid syntax;
- duplicate identities, resolution, scope, and forward calls;
- accepted and rejected operand and return types;
- minimum and maximum integers, all checked operators, and literal range;
- left-to-right evaluation and short-circuiting;
- contract success, false contracts, and faults inside contracts;
- small recursive programs and host interruption;
- unsupported syntax rejected instead of silently ignored.

An invalid-program fixture should isolate one intended error. Compare stable diagnostic codes and categories;
human wording and optional source-span snapshots have a separate presentation suite.

## Differential and generated tests

Run the same bounded, terminating program and arguments through the Rust evaluator and generated native code. Later,
also compile it using the Lobo implementation. Compare values, classified faults, and specified effect traces
when effects exist. Different machine code is permitted; semantic outcomes must agree.

Generate well-typed programs to exercise execution. Generate deliberately invalid programs to exercise rejection.
Shrink disagreements into small fixtures and retain the seed, compiler revision, runtime version, and options.
Resource exhaustion is reported separately and never counted as agreement on a value.

Useful properties include:

- equivalent whitespace and comments preserve checked program meaning;
- renaming a local binding consistently preserves behavior;
- unoptimized and optimized execution agree;
- printing and reparsing preserves semantic identity under the projection contract;
- serializing and reloading a specified graph preserves references;
- operation replay with the same idempotency key does not duplicate mutation.

Round-trip tests alone are insufficient: a parser and printer can share the same bug. Include independently
written examples with expected structures. Wasmtime describes generators, oracles, differential checks, and
shrinking in its [fuzzing guide](https://docs.wasmtime.dev/contributing-fuzzing.html).

## CI cadence

| Trigger | Required evidence |
| --- | --- |
| Editing a rule or implementation | Targeted fixtures and relevant unit checks |
| Proposed integration | All Core conformance, regression, native/reference comparison, and format checks |
| Scheduled broader run | Larger generated corpus, fuzzing, platform matrix, and performance checks |
| Release candidate | Full supported suites, pinned reproducible build, and bootstrap when self-hosting exists |

Do not block routine iteration on an unbounded fuzz campaign. Every run has a declared budget. Unsupported,
skipped, or infrastructure-failed checks are visible in reports and cannot be collapsed into “passed.”

## Release acceptance

A release is eligible when:

1. Every mandatory fixture for its advertised subset passes with no unexplained skips.
2. Every known regression has a permanent minimal case.
3. Required differential runs have no unresolved mismatches.
4. Artifact validation succeeds for every advertised backend.
5. Rebuilding identical inputs in the declared environment reproduces the required artifacts.
6. The manifest records evidence, environment, compiler identity, and unresolved limitations.
7. Self-hosting releases additionally satisfy the [bootstrap workflow](self-hosting.md).

Formal proof may later strengthen selected compiler transformations. It requires explicit assumptions and does
not replace tests for parsers, build systems, tool protocols, hosts, and interfaces outside the proof's scope.

