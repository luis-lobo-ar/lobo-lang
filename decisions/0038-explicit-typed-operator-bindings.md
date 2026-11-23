# 0038 Explicit typed operator bindings

Date 2026-11-08. Status accepted for implementation after the user's "Go for it" response to proposal 0037.

The selected milestone adds binary arithmetic bindings and Text concatenation in language/compiler 1.1.
It extends the completed 1.0 contract through a bounded, user-selected feature. Frozen release and research
inputs remain historical evidence. See [the earlier proposal](0037-symbolic-shortcuts-proposal.md).

The chosen declaration is `operator +(Vec2, Vec2) -> Vec2 = vector_add;`. The target is an ordinary local,
non-generic pure function with exactly the declared signature and a stable ID. Eligible binary symbols are
`+`, `-`, `*`, `/` and `%`. Existing precedence and left associativity apply. At least one operand is a
non-generic nominal type declared by the binding's module. Binding types are concrete and exclude generic
type applications. Different left/right/output types are permitted. A bundle admits at most 64 bindings.
Each symbol/ordered-operand signature has one binding across the closed bundle. Importing the target
function makes its binding available in the importing module. Imports retain explicit source composition.

Built-in `Text + Text` resolves to `Text::concat`. User bindings resolve to ordinary canonical calls before
artifact construction. Arguments evaluate once, left to right. Contracts, recursion bounds, fuel and heap
charges are those of the corresponding explicit operation. Arithmetic symbols imply no algebraic laws.
Numeric arithmetic, structural equality and short-circuiting retain their existing definitions.
Generic definitions are still checked parametrically. A concrete binding supplies no operation on rigid T.

An alternative is C++-style operator bodies with a separate overload candidate system. The alias form reuses
existing function bodies, stable identities, contracts and call validation. It also gives agent tools a
canonical target to inspect. There is no measured agent advantage yet. The current implementation uses
existing typed checking during source lowering and leaves native backends consuming canonical calls.

Checked format remains `lobo-checked/1.0` because this feature emits existing expression kinds. Compiler 1.1
accepts checked artifacts labeled core/1.0 and core/1.1, revalidating both. New source compilation reports
core/1.1. Operator declarations are source metadata and cannot be supplied inside a checked artifact.

Acceptance requires failing-first tests for declaration admission, exact resolution, scope and ownership,
parametric rejection, canonical artifact reload and tampering, Text sugar, contract failures, evaluation
order and resource parity. Execute representative cases through evaluator, C, LLVM and both sanitizer routes.
Update editor highlighting, target/symbol navigation, actual extension-host checks, packaged VSIX and the
programming skill. Retain verification outcomes, failed attempts and limits in a dated result record.

Implementation is recorded in [the operator result](../benchmarks/results/2026-11-08-operators.md) and its
[verification receipt](../benchmarks/results/2026-11-08-operators.json). The bounded milestone includes
compiler 1.1.0, native example outputs and the locally packaged extension 1.1.0.
