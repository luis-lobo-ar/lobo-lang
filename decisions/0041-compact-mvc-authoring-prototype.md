# 0041 Compact MVC authoring prototype

Date 2026-11-14. Status implemented as an opt-in authoring prototype.

Same-day follow-up. The user selected direct language adoption. [Decision 0042](0042-native-compact-syntax-and-legacy-deprecation.md)
supersedes the projection requirement and records native 1.2 syntax with deprecated legacy read support.

## Selected scope

The user selected proposal 0040's simpler MVC syntax and requested synchronized tooling and documentation.
This decision selects a source projection experiment. The Rust compiler and language remain version 1.1.
The prototype uses existing checked operations and native compilation.

The selected conventions use `fn`, grouped selected imports, qualified member references, expression match
arms, and Text-valued JSON object construction. Stable identities occupy a trailing table in the same source
snapshot. This separates application code from repeated annotations without hiding identity in ambient state.

The existing MVC file remains the canonical comparison fixture. A separate compact file has the same IDs.
A bundle supplies one of those two files because supplying both repeats declarations.

## Implementation choice

A shared JavaScript projector lowers source tokens and preserves original spans. The CLI and VS Code extension
use that same projector. The existing structural resolver consumes projected tokens with original positions.
Compiler diagnostics first validate against generated source and then map back to the authoring snapshot.
The CLI retains both original inputs and generated source when an output directory is requested.

Directly extending the Rust grammar would make these conventions part of the language contract immediately.
A text-only mockup would leave execution and editor behavior untested. The opt-in projection allows executable
comparison before selecting a language migration. Node.js is an authoring-tool requirement. Generated native
executables require no JavaScript runtime. There are no new package dependencies or application runtimes.

## Identity and import rules

The identities table explicitly binds the module, each local declaration and each selected import.
Projection rejects missing, repeated and stale keys. Renaming a source binding requires moving its table key
and preserving the ID value. A regression checks that renamed bindings retain their external entry ID.
Automatic identity assignment, rename refactoring and external metadata storage are future tooling.

Selected imports retain explicit stable declaration IDs. Qualified spelling resolves only within the selected
group and lowers to its selected alias. Group names are reserved throughout compact source, and member names
must be unique across groups. Dependencies arrive through explicit source arguments or configured editor files.
The projector performs no discovery or downloads.

## JSON boundary

`json { "name": user.name }` admits Text-valued fields in this prototype. It lowers to the exact existing
`Json::Object(field(object(), "name", Json::String(user.name)))` construction. Empty objects are accepted.
Selected imports must bind Json, object and field to their existing helper identities when used.
The projector rejects duplicate decoded literal keys. The compiler enforces Text values and helper signatures.
Field order follows source order. Nested builder calls retain existing argument evaluation and resource costs.
The serializer owns escaping. Numeric values retain their existing explicit constructors.

This finite scope avoids adding implicit conversion, record reflection, a parser, generic inference or schema
generation. The prototype is deliberately narrower than the full 1.1 source grammar.

## Acceptance and limits

Acceptance requires identical serialized checked MVC programs, native 200 and 404 behavior, budget exhaustion,
original-source local and dependency navigation, diagnostic mapping, highlighting, real editor-host checks,
fresh-directory CLI output and current documentation. The shared projector ships inside the extension.

The [prototype reference](../spec/compact-source-v0.1.md) defines admitted syntax and bounds.
The [usage guide](../docs/compact-syntax-prototype.md) names the required projection step.
The [result record](../benchmarks/results/2026-11-14-compact-syntax.md) records successes and failed attempts.
Construction quality, comprehension and token efficiency remain unmeasured. This task does not authorize
automatic grammar adoption, formatter development or additional language features.
