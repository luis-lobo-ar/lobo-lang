# 0040 Language complexity review

Date 2026-11-14. Status proposal following the user's question about streamlining complexity.
This records a review and recommended order. Implementation scope requires selection of a bounded milestone.

Same-day follow-up. The user selected the compact MVC authoring prototype. [Decision 0041](0041-compact-mvc-authoring-prototype.md)
records its implemented projection scope and limits. The remaining recommendations below remain proposals.

## Findings in the current examples

The MVC source repeats stable-ID annotations and individual declaration-ID imports. Its Json view wraps
a Text value, an object field and the object value explicitly. The JSON library handles those representations
with existing records, variants, lists and loops. More library construction power therefore already exists
within language 1.1. The application's visible bookkeeping is a concrete simplification target.

The language documentation also imposes context work. `docs/language-design.md` still describes Core 0.11
as the current authority and lists generic result types among future designs, while the implemented 1.1
specification inherits generic data from 1.0. The current normative specification also inherits earlier
versions. An agent must reconcile these sources to find an answer. Historical specifications remain valuable
as evidence, while one complete current guide could provide an authoritative entry point.

These are source/documentation observations. They supply no measured construction, comprehension or token
advantage for the proposals below.

## Recommended simplifications

1. Have authoring tools assign stable identities once and preserve them across renames. Treat identity as
   persistent metadata. Deriving it again from a renamed function or edited body would change program identity.
   The current compiler still requires explicit source IDs. Managed identity is a proposed authoring workflow.
2. Group explicitly selected imports by module. Resolve each selected member to its canonical stable ID using
   the supplied source bundle. Define aliases and conflicts explicitly. Preserve the existing operator rule
   that importing a target enables its binding. Grouped import syntax is a proposal.
3. Consider local generic-argument inference where declared parameter signatures and supplied argument types
   determine one substitution. Keep definition-time rigid checking, exact types, specialization bounds and an
   explicit syntax for ambiguous calls. This introduces a rule and therefore needs its own bounded comparison.
4. Generate ordinary JSON encoder functions from an explicitly selected wire schema. A generated `encode_user`
   could contain today's checked builder calls, leaving the view with one named call. Freeze field names,
   number handling and exposure policy in that schema. The generator is future tooling. The language already
   supports calling such a normal function.
5. Consolidate the implemented language into one current guide and synchronize the skill/editor entry points.
   Preserve versioned specifications and result records as history. Separate currently implemented syntax from
   older sketches when an agent retrieves a language reference.

## Semantics and stopping point

The user's same-day clarification emphasizes syntax conventions and the growing resemblance to Java.
This changes the immediate presentation priority toward a compact source-style prototype on the MVC example.
The proposed conventions use `fn`, module-qualified library calls, record values, expression-returning blocks,
compact data construction and tool-managed identity metadata. Match-arm expression notation is another
candidate. These spellings are illustrative future syntax and require a separately selected implementation.

For example, `json { "name": user.name }` describes a proposed typed construction form replacing today's
`Json::Object(field(object(), "name", Json::String(user.name)))`. The prototype must define the admitted
value types, escaping, duplicate names, evaluation order, numeric handling and charged operations. Mapping
this form to existing checked constructors can preserve a small execution core, but the source rule still
has a grammar/tooling cost. It must be justified by the same construction and comprehension comparison.

Prefer one canonical spelling and consistent formatting for ordinary functions, imports and data. Retain
explicit types at function boundaries and the existing expression-returning behavior. Compare concrete
examples before selecting keywords or adding another layer of general abstraction. The immediate deliverable
recommended by this review is a source-style proposal, with implementation decisions still pending.

Keep exact types, explicit actions/contracts, closed source composition, native compilation and resource
accounting as the constraints on each simplification. Existing operator bindings already lower to ordinary
calls. Further convenience syntax should also have an inspectable canonical operation.

The recommended first pass is finite. Consolidate the current guide, provide identity-preserving creation
tools, and prototype grouped selected imports. Finish when the MVC/JSON examples become easier to author,
rename/import checks preserve identity and scope, required compiler/editor/skill checks pass, and a bounded
construction/comprehension comparison has recorded its outcomes. Generic inference and encoder generation
are later candidates rather than automatic additions to that pass.

Compare the same public task requirements and planned changes before and after. Retain accepted behavior,
repair attempts, context/tool retrievals, total model/tool tokens and independent comprehension answers.
Source brevity alone cannot select the design. Equal or worse results are valid findings. No such comparison
was run for this review. Documentation validation uses `./scripts/check.sh`; output is retained at
`target/language-complexity-review-documents.log`.
