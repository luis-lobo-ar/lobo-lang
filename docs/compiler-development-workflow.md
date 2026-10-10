# Developing the Lobo compiler

Status: workflow definition in use for the first Rust Core implementation.

## Unit of work

Each change states a rule or defect, scope, acceptance examples, and evidence required. Keep changes small enough
that an agent can explain exactly which program behavior changed and why. Language evolution, implementation
correction, and optimization are distinct intentions, even when they touch the same code.

## Specification and TDD loop

```text
rule + examples → conformance fixture → observed failure → implementation
                                                         │
accept evidence ← full required checks ← focused checks ──┘
```

1. Define the rule with a stable section and specify boundaries and errors.
2. Add valid, invalid, and boundary examples with expected outcomes independent of implementation internals.
3. Confirm that new tests fail because the behavior is missing or wrong. Record unsupported harness behavior
   separately while the harness is being built.
4. Implement the smallest complete behavior across parsing, resolution, checking, evaluation, and code generation
   as applicable. Unsupported backends must report their support honestly.
5. Run focused conformance and regression cases. Inspect failure evidence, then repair the implementation or
   explicitly revise the specification and expectations when the design itself is wrong.
6. Refactor without changing observable behavior; rerun the checks affected by the refactor.
7. Run required integration checks and record results against the exact revision and environment.

For example, implementing checked addition requires normal addition, both overflow directions, precedence,
operand rejection, and behavior when the expression occurs in an unevaluated branch. One test for `1 + 2` does
not establish the arithmetic rule.

## Agent context

An agent needs the relevant specification sections, diagnostic definitions, existing fixtures, affected compiler
stages, implementation entrypoints, and current failure output. It also needs the advertised language/backend
versions so it does not mistake illustrative future syntax for supported behavior.

Record unresolved design assumptions in the task. Do not resolve ambiguity by copying a host-language default
such as wrapping arithmetic, truthiness, or exception handling into Lobo.

## Oracle review

When implementation and tests disagree, identify the governing rule first. Expected-value snapshots cannot be
updated solely because the current compiler emits a different result. Review changes to assertions and golden
outputs separately from the implementation change, with a rationale tied to the specification.

## Integration evidence

The eventual runner should report structured results with:

- compiler source revision and artifact identity;
- language, fixture, backend, and toolchain versions;
- executed case IDs, seeds, and input artifacts;
- pass, fail, unsupported, skipped, and infrastructure-failure counts;
- mismatch details and reduced reproducers;
- timing and declared resource budgets;
- evidence links and unresolved limitations.

`cargo test --locked` runs the current compiler tests; `scripts/check.sh` validates documentation artifacts.
The future runner should expose focused selection and full required suites through both a CLI and the agent
protocol. A failed test is data for the agent; the host owns execution and acceptance policy.

## First implementation sequence

1. Define the checked IR and graph mapping for Core.
2. Build a fixture loader that distinguishes syntax errors, type errors, values, and runtime faults.
3. Implement source parsing, name resolution, and type checking in Rust.
4. Implement the reference evaluator with required runtime contracts and checked arithmetic.
5. Run the Core corpus end to end, including rejection cases.
6. Define the native invocation adapter and implement C/Clang lowering.
7. Compare evaluator and native outcomes and retain every disagreement as a minimized regression.
8. Build workspace operations around the working semantic representation.

See [testing and TDD](testing-and-tdd.md) for acceptance gates, and
[self-hosting](self-hosting.md) for the later bootstrap extension.

