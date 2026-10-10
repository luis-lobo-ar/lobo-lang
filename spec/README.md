# Specifications

This directory holds language rules, conformance fixtures, and machine-readable formats.

- [Core v0.1](core-v0.1.md) defines the first executable language subset.
- [Checked artifact v0.1](checked-artifact-v0.1.md) defines the first compiler output and evaluation input.
- [Native backend v0.1](native-v0.1.md) defines default executable generation and its invocation interface.
- [Core conformance corpus](conformance/core-cases.json) specifies accepted, rejected, and executed cases.
- [Fixture schema](schemas/conformance-case.schema.json) defines the corpus framing.

The current `schemas` define the outer envelopes for LTP 0.1 requests and responses and the discovery descriptor
for tools. Tool-specific `arguments` and `result` schemas remain deferred until the workspace prototype chooses
its exact operations. The canonical graph interchange format is also pending; the compiler's checked artifact
has a separately versioned format.

Versioning rules:

- breaking changes create a new protocol version and schema ID;
- schemas use strict outer objects to catch misspelled or accidental fields;
- flexible payload positions are validated again using the selected tool's schema;
- examples are illustrative until an implementation test suite declares them fixtures.

