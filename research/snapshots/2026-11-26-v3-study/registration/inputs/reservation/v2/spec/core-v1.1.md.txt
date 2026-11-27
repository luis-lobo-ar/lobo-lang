# Lobo language 1.1

Compiler version `1.1.0`. Source language `core/1.1`. Checked format `lobo-checked/1.0`.
Inherit [language 1.0](core-v1.0.md) with the additions below. The user selected this bounded milestone
after the operator discussion. [Decision 0038](../decisions/0038-explicit-typed-operator-bindings.md) records
the scope and rationale. [The executable example](../examples/operators.lobo) demonstrates the syntax.

## Typed binary operator bindings

At module level, a binding names an existing function with the exact signature.

```lobo
operator +(Vec2, Vec2) -> Vec2 = add;
operator *(Vec2, I64) -> Vec2 = scale;
```

Eligible symbols are binary `+`, `-`, `*`, `/` and `%`. Existing precedence and left associativity apply.
A binding may appear before or after its target function. Operand order matters. Left and right types
may differ, and the output may differ from both. The target must be a local non-generic function with
an empty action set and exactly two parameters matching the binding's ordered operand types and output.
Requires and ensures clauses remain part of the target function's checked call behavior.

At least one operand must be a non-generic record or variant declared in the binding's module. Binding
types must be concrete, with generic nominal applications excluded in this milestone. Primitive, List,
Fn and non-generic nominal types can occupy the other operand or output positions. A bundle admits at most
64 bindings. Each symbol and ordered operand-type pair has exactly one binding across the closed bundle.
Duplicate bindings fail even if they name the same target or occur in modules whose call scopes differ.

A binding is available when its target function is declared or explicitly imported in the calling module.
Importing a type alone supplies its identity and shape. Importing the target also supplies its operator
binding. Resolution uses exact canonical types after module aliases have resolved. Missing or reversed
signatures fail type checking. The compiler performs neither conversion ranking nor implicit operand swaps.
The identifier `operator` remains contextual at declaration boundaries.

The checked program contains a `Call` to the target's stable ID. The source binding supplies no second
body. Operands evaluate exactly once, left to right, using ordinary argument evaluation. Fuel, heap, stack,
overflow, precondition and postcondition behavior match the equivalent explicit call tree. The symbol
asserts no associativity, commutativity or other algebraic law.

## Text concatenation

`Text + Text` lowers to the existing `Text::concat` intrinsic with identical typed arguments and cost.
Mixed Text/numeric operands fail type checking. Parentheses preserve the intended call tree.

```lobo
"Hello, " + (name + "!")
```

This expression has the same call tree as `Text::concat("Hello, ", Text::concat(name, "!"))`.
An unparenthesized three-term chain groups left to right and can charge different intermediate allocations.

## Generic definitions and existing operators

Every generic definition is checked with rigid parameters before concrete specialization. Arithmetic on
unconstrained `T` fails even if all observed calls use a type with a binding. Concrete nominal arithmetic
inside a generic body can use available bindings. Generic operation requirements remain a future design.
Numeric arithmetic, unary operations, structural equality, comparisons, short-circuiting and control syntax
inherit 1.0 rules. User bindings cover the five binary arithmetic symbols listed above.

## Artifacts, diagnostics and tooling

Source operator metadata is consumed before checked-artifact emission. Artifacts use the existing Call and
BufferOp kinds. Loaded artifacts are revalidated for scope, exact signatures, contracts and actions.
The 1.1 compiler accepts canonical `core/1.0` and `core/1.1` artifacts in checked format 1.0. New compilation
labels source semantics `core/1.1`. Artifact input containing operator declarations fails strict decoding.
Existing standalone native programs retain their embedded behavior.

Declaration diagnostics use `compile.operator.symbol`, `.ownership`, `.signature`, `.purity` and `.duplicate`.
An unresolved target uses `compile.name.unresolved`. A missing expression signature uses
`compile.type.mismatch`. Each diagnostic includes source/range information where available.
The VS Code extension highlights bindings and resolves binding targets and overloaded use-site symbols to
the named function. Built-in numeric and Text operators have their signature in this specification and
their canonical meaning in checked output. Editor navigation is a structural projection of compiler rules.
