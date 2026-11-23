# 0061: Replace application capability masks with typed permissions

Date: 2026-11-23. Status: proposed after the user's question about the literal 5. No implementation selected.

The native ABI uses clock=1, random=2, listen=4 and connect=8. The HTTP sample's 5 selects clock and listen
rights from its existing root. It does not include randomness or outbound connect, and it cannot grant
rights absent from that root. Native grants and restriction checks remain the authority boundary.

The prior HTTP refactor preserved this raw ABI parameter in application code. That is a remaining domain
model/DX gap. Keep a distinction between the opaque Native::Capability authority token and the permissions
it carries. Model named permissions with the existing payload-free variant construct:

```lobo
variant Permission { ClockRead, RandomRead, Listen, Connect }
```

Recommend a typed permission collection/set at the application-facing restriction boundary. The OS library
converts it to the unchanged ABI mask internally, with duplicate requests idempotent and no unknown bits.
A named HTTP server permission profile can return ClockRead+Listen without repeating numeric encodings.
This proposal does not choose a new list/set literal, bitwise operator, action inference or capability token
representation. Existing variants, records/lists and library helpers can represent the domain. Do not imply
that unimplemented collection syntax already compiles. Preserve existing raw ABI tests and frozen fixtures.

Before an authorized implementation, use TDD for each permission, combinations, duplicates, empty requests,
missing grants, attenuation/no amplification, existing raw-mask compatibility and C/LLVM/sanitizer execution.
Keep editor navigation, named types, error/result compatibility, docs, skill, identities and evidence aligned.
The integer mask remains appropriate at the raw ABI and deliberately low-level test boundary. No capability
security guarantee beyond the inherited grant/handle contract follows merely from naming the permissions.

Evidence: spec/native-interface-v0.1.md, libraries/native/native.lobo, libraries/native/README.md,
examples/native/rest/app.lobo, examples/native/snapshot.lobo and src/native_api.c. Source/specification review
only; no implementation, new runtime tests or agent-performance measurements were made.
