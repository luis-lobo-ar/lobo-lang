# 0046 Infer constructor arguments and contextual case owners

Date 2026-11-22. Status implemented. Risk mode standard.

The user selected constructor inference after seeing explicit `Lookup::<User>::Found` and `Missing` forms.
The requested outcome includes TDD, extension maintenance, documentation and the decision ledger.

## Selected behavior and alternatives

Retaining only type-argument omission would still require repeated nominal qualifiers in a typed MVC model.
Globally importing or searching case names would introduce ambiguity when variants share names.
Select expected-type-directed case names and local constructor constraints instead. `Found(user)` and
`Missing` use the variant named by an existing expected type. Qualified constructors remain available.
This is one implementation with self-review and differential tests, not independent candidate implementations.

Extend the existing source elaborator to constrain record fields, variant payloads and matching expected
nominal applications. Defer unresolved members until sibling fields or call arguments supply their types.
Keep the existing checker and specialization pipeline authoritative. Preserve all identity values and emit
the same concrete checked program as explicit source. No new runtime, dependency or artifact format is added.

The [1.5 contract](../spec/core-v1.5.md) defines accepted contexts and limits. Local bindings and explicit
aliases win over contextual case interpretation. Unknown owners stay unresolved; there is no case-name search
across declarations. Payload-free generic cases require context. Signatures and return-context inference for
generic function calls remain outside scope. Frozen legacy source remains explicit.

## Failure matrix and acceptance

| Condition | Required result |
| --- | --- |
| Fields/payload/context determine every constructor parameter | Same checked program and nominal IDs as explicit source |
| A case name appears in multiple variants | Expected nominal identity selects the owner |
| No expected owner for a bare case | Unresolved value; no global search |
| Unresolved qualified constructor parameters | Explicit specialization required |
| Conflicting payload/field types or bad arity | Compile rejection with original source range |
| Local or imported name matches a case | Existing lexical binding takes precedence |
| Named callback in a constructor member | Exact Fn signature and existing purity checks |
| Unresolved source sugar in loaded artifact | Rejection; artifact loading performs no inference |

Compiler/editor regressions run before implementation. Native and sanitizer checks compare contextual and
explicit forms. Coverage includes typed locals, nested records, calls, loop state, generic bodies, imports,
duplicate case names, ambiguous owners, payload faults, callback purity and artifact admission. Real VS Code
checks exercise definitions, an incompatible payload edit, and repair.

The generic MVC changes only constructor spelling; its sidecar IDs are unchanged. The simpler MVC, legacy
comparison, migration manifest, old specifications and research fixtures remain frozen. Skill constructors
adopt inferred arguments while preserving prior IDs and checked behavior. Compiler/language and extension
advance to 1.5.0/core/1.5. Stop at this selected scope.

The [result record](../benchmarks/results/2026-11-22-constructor-inference.md) retains all outcomes, including
test-fixture corrections. No agent construction-quality, comprehension, token-efficiency or runtime-speed
comparison accompanies this engineering result.
