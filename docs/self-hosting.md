# Self-hosting and bootstrap

Status: future milestone. Core v0.1 cannot yet express a compiler.

Self-hosting means the compiler is written in the language it compiles. Agents still propose source changes;
the specification, verification process, and project policies govern acceptance.

## Entry requirements

Before writing the compiler in Lobo, the language must support collections, strings or bytes, records and enums,
iteration or practical recursion, explicit recoverable errors, modules, and defined host input/output.
Memory behavior, graph interchange, build dependencies, and artifact serialization must also be specified.

Add these features through conformance-tested specifications. Reaching self-hosting is not justification for
adding undocumented compiler-only behavior or unrestricted host authority.

## Bootstrap stages

Let `S` be one fixed revision of the compiler written in Lobo. Hold its dependencies, target, configuration,
standard library, and build environment fixed across stages:

```text
C0: compiler implemented in Rust
  │ compiles S
  ▼
C1: compiler implemented in Lobo, built by C0
  │ compiles the same S
  ▼
C2: compiler implemented in Lobo, built by C1
  │ compiles the same S
  ▼
C3: compiler implemented in Lobo, built by C2
```

Run the conformance corpus using C0, C1, C2, and C3. Compare acceptance, single-error diagnostic codes, runtime
values, and faults. Compare emitted checked representations where the format requires identical output.
Differences must be explained against the specification; agreement is not proof that all compilers are right.

For deterministic compiler artifacts, require C2 and C3 to match byte for byte. Strip or avoid timestamps and
absolute build paths using a documented reproducible-build policy. Do not normalize away instructions,
constants, or other behavior to obtain a match. If artifact comparison is not yet reproducible, report bootstrap
as incomplete even when all behavioral tests pass.

C0 is not byte-compared with C1: they are different compiler implementations. C1 versus C2 can provide extra
evidence, but the stated fixed-point requirement is C2 versus C3.

## What bootstrap establishes

Self-compilation exercises a substantial real program and shows that the language can express its own compiler.
A fixed point establishes reproducibility across the tested stages. Neither demonstrates universal compiler
correctness nor eliminates the need to trust the bootstrap compiler and build environment.

Keep the Rust implementation as an independent reference through the transition. A shared optimizer, copied
algorithm, or shared incorrect test oracle reduces that independence; record those relationships.

## Agent development loop

For a compiler change:

1. Link the change to a language rule, bug, or measured optimization goal.
2. Add independent examples or a reduced regression before editing the affected compiler path.
3. Implement the change in an isolated revision and run focused checks.
4. Run the shared conformance corpus and required cross-implementation checks.
5. Rebuild the affected bootstrap stages with pinned inputs.
6. Inspect semantic and capability changes, test-oracle changes, and reproducibility evidence.
7. Accept only the identified revision whose required checks passed.

A language feature may need staged support: the existing compiler first learns the feature before the next
compiler source starts using it. Every compiler-source revision declares the minimum bootstrap version that
can build it. Avoid a circular dependency in which the only compiler supporting a feature requires that feature
to compile itself.

## Failure and recovery

Preserve the last verified compiler artifact and its build manifest. If a stage fails, use the previous verified
compiler to rebuild or diagnose the candidate. A broken candidate never overwrites the only usable bootstrap
artifact. Compiler binaries, source revisions, dependencies, stage relationships, and test evidence are all
identified separately.

