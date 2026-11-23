# 0037: Symbolic shortcuts — proposal only

Date: 2026-11-05. Status: recommendation in response to the user's question; not accepted for implementation.
Language 1.0 and its fixed acceptance/release remain unchanged. No experiment or compiler work is authorized
by this note. The standing request to retain design findings motivates this record.

Subsequent authorization on 2026-11-08 selected a bounded implementation in
[decision 0038](0038-explicit-typed-operator-bindings.md). The dated discussion below retains the proposal history.

The controller exposes a concrete candidate: Text concatenation currently needs nested `Text::concat` calls.
Consider a bounded post-1.0 prototype of `Text + Text`, resolved statically to the existing concat operation.
It needs no arbitrary-symbol declaration mechanism or general trait system. Existing I64/F64 addition stays
unchanged. Exact operand types determine the meaning; there is no implicit conversion to Text.

Illustrative proposed syntax, unsupported by language 1.0:

```text
"Hello, " + (name + "!")
```

The parentheses preserve the current controller's right-nested call tree. A left-associated chain produces
the same text but can change intermediate allocations and charged fuel, so value associativity is not a
license to reassociate operations. Specify evaluation order, failure timing and resource accounting explicitly.
Symbolic syntax must lower to existing typed canonical operations; it must not hide contracts or authority.

If later workload evidence justifies user-defined bindings, constrain them to a fixed symbol set and fixed
precedence, exact checked signatures, pure named functions, explicit availability and unambiguous resolution.
Inspectors/diagnostics must expose the resolved operation identity; user-function bindings should navigate to
their definition. Built-in intrinsic operators need an inspectable signature, not an invented source location.
Protect existing numeric, structural equality, short-circuiting and control semantics. Do not add custom
precedence, arbitrary punctuation or actionful operator dispatch merely for source brevity.

Primary comparisons reviewed:

- [Rust std::ops](https://doc.rust-lang.org/std/ops/index.html): selected existing operators are trait-backed;
  this API does not create arbitrary new operators, and short-circuit operators are excluded. Its bounded
  operator vocabulary is a useful design reference, without assuming Lobo needs its trait system.
- [Swift Advanced Operators, official source](https://raw.githubusercontent.com/swiftlang/swift-book/main/TSPL.docc/LanguageGuide/AdvancedOperators.md):
  permits custom operators with precedence/associativity. This demonstrates a broader design space, not evidence
  that its tradeoffs improve Lobo agent outcomes. The rendered Swift documentation required JavaScript;
  the official source was read instead. Neither language's implementation proves an agent benefit.

Before adoption, freeze a small construction/comprehension comparison against explicit calls. Measure accepted
behavior, repair attempts, resolved-operation comprehension and total model/tool tokens; source character count
alone is insufficient. Also verify evaluator/C/LLVM behavior, checked-artifact revalidation, diagnostics/editor
support and skill compatibility. No comparative measurement, token benefit or performance gain exists yet.

## 2026-11-06: C++ operator-function follow-up

The user asked whether Lobo should implement C++-style operator overloading. Recommendation: consider its
operator-function model for a later bounded milestone. Keep the current Text-concatenation candidate smaller
than a general user-defined binding system. This follow-up does not authorize implementation or change 1.0.

The [C++ working draft, overloaded operators](https://eel.is/c++draft/over.oper) describes operator functions
and a fixed operator vocabulary. Operator overloads cannot change precedence, grouping or operand count.
Binary operators can invoke member or non-member functions selected through overload resolution.
These facts support the design analogy, not a claim that C++ improves agent outcomes.

Illustrative binding notation only, unsupported by Lobo 1.0 and not a chosen grammar:

```text
operator +(Vec2, Vec2) -> Vec2 = vector_add;
```

Here `vector_add` is an ordinary pure function with its own stable ID and contracts. A typed `a + b` would
resolve to that exact function and retain its checks, evaluation order and resource charges. The declaration
adds a spelling for the call, not a second body or a runtime dispatch mechanism. Tooling should expose both
the binding and the target; definition navigation should reach the target function.

Candidate limits for that later design:

- Begin with selected arithmetic symbols, fixed precedence and exact operand signatures. Reject ambiguous
  bindings rather than rank implicit conversions. Preserve existing numeric and structural equality rules.
- Require explicit binding availability and an ownership rule. A candidate rule permits a user binding only
  in the module defining at least one operand type. Built-in Text support remains compiler-defined.
- Keep `&&`, `||`, assignment and control behavior outside user bindings. Pure targets cannot introduce hidden IO.
- Do not infer algebraic laws from a symbol. A `+` binding alone proves neither associativity nor commutativity.
- Generic bodies still need declared operation requirements before arbitrary `T + T` can type-check.
  An overload declaration alone does not supply a generic constraint system.

The binding form, ownership details, eligible symbols and generic constraints remain open design questions.
No speed, token or comprehension benefit has been measured. This is a documentation-only review of existing
code and the cited draft. Documentation validation command: `./scripts/check.sh`; raw output is retained at
`target/operator-shortcuts-cpp-doc-check.log`. No compiler or editor behavior changed.
