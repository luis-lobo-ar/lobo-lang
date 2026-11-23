# Lobo vision

## Premise

Most programming languages optimize for human authors: readable names, convenient punctuation, flexible source
layout, and abstractions that fit human working memory. Software agents inherit those interfaces, plus shells,
logs, file trees, and version-control diffs. They spend a large part of their effort reconstructing facts that a
development environment could represent directly.

Lobo asks what changes when the primary author is an agent but humans remain responsible for intent, authority,
and outcomes.

## Thesis

The scarce resource is not an agent's ability to emit tokens. It is dependable situational understanding.
Lobo should therefore optimize for:

- unambiguous program structure;
- bounded retrieval of relevant context;
- explicit state, actions, permissions, and uncertainty;
- small semantic mutations with precise impact analysis;
- fast counterexamples and machine-readable diagnostics;
- replayable evidence for every accepted change;
- human inspection at the level of behavior, policy, and risk.

A compact binary format may eventually improve storage or transmission, but bytecode alone does not solve these
problems. Execution format and development interface are separate concerns.

## Product boundary

Current development priority: establish a small general-purpose language core before expanding the ecosystem
or workspace services. This changes sequencing, not the long-term thesis; see
[the language-completeness goal](docs/language-completeness.md) and
[decision 0014](decisions/0014-language-first-development.md).

Lobo is the combination of:

```text
language semantics
+ semantic workspace
+ tool protocol
+ context engine
+ verifier
+ execution environment
```

The syntax is a projection of this system, not the system itself.

## Human relationship

Humans do not need to read every generated implementation. They must still be able to determine:

- what a system is intended to do;
- what it can affect;
- which guarantees were checked and under which assumptions;
- how behavior changed;
- what remains uncertain;
- who or what authorized a consequential action.

Lobo therefore permits machine-native representations but rejects unaccountable opacity. Every canonical artifact
must support deterministic, human-auditable projections.

## Success measures

Lobo succeeds only if controlled experiments show improvements over file-and-shell workflows. Candidate measures
include:

- accepted changes per unit of agent cost;
- first-pass verification rate;
- escaped defect rate;
- context tokens needed per accepted change;
- unrelated code touched per change;
- time to localize a seeded fault;
- percentage of tool failures classified without log scraping;
- reviewer time needed to understand behavior and risk;
- merge conflicts caused by representation rather than semantics.

The comparison workload must include feature work, fault repair, refactoring, dependency changes, and adversarial
permission requests. A novel syntax without measurable workflow gains is not success.

## Non-goals for the first prototype

- Designing a new CPU instruction set or virtual machine.
- Replacing established package ecosystems.
- Proving arbitrary application correctness.
- Automatically granting production or external-write authority.
- Hiding behavior from human operators.
- Optimizing token density before correctness and inspectability.
- Supporting unrestricted metaprogramming or implicit ambient resource access.

## Open research questions

1. What is the smallest semantic graph that improves agent work without becoming an IDE database in disguise?
2. Which contracts should be statically proven, dynamically checked, or treated as policy gates?
3. How should evidence expire when code, dependencies, toolchains, or environments change?
4. Can independent semantic operations merge safely without central ownership?
5. How can context selection expose omissions rather than merely returning a confident summary?
6. Which human projection best supports review: source, behavioral diff, graph, narrative, or a combination?
7. When is a custom runtime justified by measured limitations of WASM or native backends?

