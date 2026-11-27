# Lobo Core v0.5: tagged alternatives and exhaustive handling

Historical version, extended by the current language in the [specification index](README.md). Version-specific resource/interchange rules
below describe 0.5, not the current loader.

Status: implemented experimental subset. Version `core/0.5-draft`, inheriting [Core v0.4](core-v0.4.md).
This is a language-only extension; it grants no resource authority.

## Values and handling

```lobo
variant Quotient @id("example.quotient") { Ok(I64), ZeroDivisor }
function divide @id("example.divide")(a: I64, b: I64) -> Quotient {
  if b == 0 { Quotient::ZeroDivisor } else { Quotient::Ok(a / b) }
}
function fallback @id("example.fallback")(q: Quotient) -> I64 {
  match (q) { Ok(n) { n }, ZeroDivisor { 0 } }
}
```

A variant declares 1–16 distinct cases, each with zero or one payload type. Payloads may be scalars,
records, or variants; records may contain variants. Use a record for multi-field payloads. Forward type
references are allowed. Type/function/record/variant names and stable IDs share their existing namespaces.
Case names are local to their variant, follow normal nonreserved-name rules, and have no separate ID.
`variant` is contextual at module declaration position. `match` is contextual when followed by a balanced
parenthesized expression and `{`; ordinary functions/bindings named `match` remain legal elsewhere.
An ordinary `match(...)` call immediately followed by a block (for example as an `if` condition) must be
parenthesized as `if (match(...)) { ... }` to avoid this new contextual form. This is a source compatibility
change for that previously valid prefix, not global reservation of `match`.

Construct with `Type::Case(expression)` or `Type::Case` for a payload-free case. Wrong presence/type of a
payload is rejected; constructors do not implicitly convert values. Match evaluates its subject once and
executes only the selected arm. Each declared case must occur exactly once, in any order. No defaults,
guards, partial matches, or nested patterns exist. Payload cases bind one fresh name in their arm's block;
payload-free cases bind none. No shadowing of visible bindings/declarations is allowed. Bindings do not
escape an arm. Every arm is checked, including unreachable arms; all arm result types must match exactly.
Use nested matches for nested alternatives. Match may appear wherever an expression may appear.

Values are immutable, nominal, owned copies with no observable identity. Equality compares the case first,
then only its active payload; distinct nominal types cannot compare. Functions, records, contracts, locals,
conditionals, and folds accept variant values. Expected failures are ordinary variants, not exceptions:
arithmetic faults, failed contracts, and exhaustion retain their existing diagnostic behavior and are not
implicitly caught by match. No `try`, propagation shorthand, new loop, or implicit host operations are added.

## Layout and diagnostics

All inline compound layouts remain acyclic, including mixed record/variant cycles. Native variants use a
tagged struct/union and read only the active union member. Empty cases need no payload storage. Maximum
compound declarations: 32 total; depth: eight compound nodes; storage width: 64 cells. Record width is the
sum of field widths; variant width is one tag plus the maximum payload width (empty payload width zero).
Existing per-function 2,048-cell admission accounting extends to both compound kinds. These bounds are
admission rules, not exact stack guarantees or sandboxing. Heap/recursive/growable data remains future work.

Unknown nominal types use `compile.type.unknown`; unknown cases or wrong payload presence use
`compile.variant.case`; nonvariant/duplicate/missing/unknown-case/wrong-binding matches use
`compile.match.invalid`; payload/arm/nominal type mismatches use `compile.type.mismatch`. Mixed cycles retain
`compile.record.cycle` for compatibility. Limits use `resource.frontend.limit`. Source failures carry ranges
where available; artifact reload rechecks all rules without retaining source metadata.

## Invocation and interchange

Compiler/artifact version 0.5; native adapter 0.3. The checked format `lobo-checked/0.5` adds mandatory
`program.variants`: declarations `{name,id,cases:[{name,payload:Type|null}]}`. Variant type references use
`{"Variant":"stable-id"}`. A constructor expression is `{kind:"Variant",variant:"stable-id",case:"Name",
payload:Expr|null}`. A match expression is `{kind:"Match",subject:Expr,arms:[{case:"Name",binding:string|null,
body:Block}]}`. Constructor types resolve to stable IDs; arm cases refer to the checked subject's type.
Compound values cannot be injected as Literal nodes. Prior checked artifacts must be recompiled, not retagged.

Runtime JSON: `{"type":"Variant","value":{"id":"stable-id","case":"Name","payload":Value|null}}`.
CLI root input: `Variant:<full-stable-id>:Case:Name[,payload tokens...]`. Payload tokens are recursively
flattened in record field order, with `Case:Name` selecting each nested variant, followed by only its payload.
Record root inputs retain `Record:<full-stable-id>:` and now allow nested case tokens. The parameter's full
stable ID determines the prefix, including colons in IDs. Exactly the selected shape must be supplied, with
no extra or empty tokens. Scalar spellings and typed JSON output remain unchanged. All invocation shape
checks precede function execution/fuel consumption; caller-supplied runtime values are recursively validated.

Source nominal classification and reference resolution enforce the same 32-expression depth bound as the
checker, before descending into unchecked bodies. Postfix field chains are also bounded during parsing.
Generated input/output helpers are shared by declared type, not expanded over every possible case path.

## Acceptance cases

Before implementation: safe division distinguishes zero divisor and I64-min/-1 overflow as separate empty
cases rather than deliberately faulting; ordinary success carries I64. Matching maps success to its value,
zero to 0, overflow to -1. Also exercise nested record/variant payloads, mixed-layout cycles, forward types,
nominal equality, fold state, contracts/actions in every arm, nonselected faults, malformed source/artifacts/
arguments, exact fuel boundaries, and optimized/reference/ASan/UBSan agreement. Editor highlighting and
definition navigation cover declaration/type/case references and arm-local payload binding scopes, including
unsaved diagnostics in a real extension host. These cases do not complete the full language-first goal.
