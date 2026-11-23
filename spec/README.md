# Specifications

The current [language 1.9 specification](core-v1.9.md) adds the closed [SQLite interface](sqlite-interface-v0.1.md) and workload-driven source capacity.
It inherits [Core 1.8](core-v1.8.md) try error propagation.
It inherits [Core 1.7](core-v1.7.md) and the closed [native OS interface](native-interface-v0.1.md).
It inherits [1.6 automatic JSON builders](core-v1.6.md).
It inherits [1.5 constructor inference](core-v1.5.md).
It inherits [1.4 call inference](core-v1.4.md).
It inherits the [1.3 identity rules](core-v1.3.md).
It inherits [language 1.2](core-v1.2.md), which supersedes
the [historical compact projection prototype](compact-source-v0.1.md). The old source spelling remains
deprecated read compatibility, with shared checking and native backends.

This directory holds language rules, conformance fixtures, and machine-readable formats.

- [Core 1.1](core-v1.1.md) adds exact typed binary operator bindings and Text concatenation.

- [Direct LLVM backend v0.1](native-llvm-v0.1.md) defines the optional native route in compiler 0.11.1.

- [Core 1.0](core-v1.0.md) freezes the bounded language contract, including generic data, parametric checking, pure function values and F64.
- [Language 1.0 acceptance](language-v1-acceptance.md) fixes the authorized finish line; public programs/changes pass, with final artifact status in the release receipt.
- [Core v0.12 development](core-v0.12.md) records the development semantics inherited by language 1.0.
- [Core v0.11](core-v0.11.md) defines the frozen evaluation checkpoint and inherited generic function syntax.
- [Generic reuse workload](generic-reuse-workload.md) freezes public development acceptance cases.
- [Core v0.10](core-v0.10.md) defines inherited list-guarded recursive values and bounded structural traversal.
- [Core v0.9](core-v0.9.md) defines inherited closed module linking and explicit-source editor snapshots.
- [Core v0.8](core-v0.8.md) defines inherited state-driven iteration and checked-artifact extension.
- [Core v0.7](core-v0.7.md) defines inherited byte/UTF-8 text values, encoding/cost rules, and checked/native interchange.
- [Core v0.6](core-v0.6.md) defines inherited persistent lists and allocation accounting.
- [Language workload fixtures](language-milestone-workloads.md) freeze the language-first acceptance inputs/outcomes.
- [Core v0.5](core-v0.5.md) defines inherited tagged alternatives and exhaustive handling.
- [Core v0.4](core-v0.4.md) defines inherited record semantics and the historical 0.4 interchange.

- [Core v0.3](core-v0.3.md) defines current action bounds, inheriting folds and scalar Core semantics.
- [Checked artifact v0.3](checked-artifact-v0.3.md) defines inherited mandatory action metadata and historical reload checks.

- [Core v0.1](core-v0.1.md) defines the first executable language subset.
- [Checked artifact v0.1](checked-artifact-v0.1.md) defines the first compiler output and evaluation input.
- [Native backend v0.1](native-v0.1.md) defines default executable generation and its invocation interface.
- [Agent-first testing v0.1](agent-testing-v0.1.md) defines the executable foreground test session; separate from Core and LTP.
- [Core conformance corpus](conformance/core-cases.json) specifies accepted, rejected, and executed cases.
- [Fixture schema](schemas/conformance-case.schema.json) defines the corpus framing.

The current `schemas` define the outer envelopes for proposed LTP 0.2 requests and responses and the discovery descriptor
for tools. Tool-specific `arguments` and `result` schemas remain deferred until the workspace prototype chooses
its exact operations. The canonical graph interchange format is also pending; the compiler's checked artifact
has a separately versioned format.

LTP 0.2 replaces the 0.1 descriptor field and verification check name `effects` with `actions`; old descriptors
must be regenerated, not silently accepted under the new schema. This does not change Core or checked-artifact
versions, and the daemon is still unimplemented. See [decision 0012](../decisions/0012-actions-terminology.md).

Versioning rules:

- breaking changes create a new protocol version and schema ID;
- schemas use strict outer objects to catch misspelled or accidental fields;
- flexible payload positions are validated again using the selected tool's schema;
- examples are illustrative until an implementation test suite declares them fixtures.

