# Lobo language 1.5

Status implemented by compiler 1.5.0. Constructor inference extends the bounded source elaboration in
[language 1.4](core-v1.4.md). Other language, identity and runtime rules are inherited. Source compilation
emits `core/1.5`; checked format remains `lobo-checked/1.0`. Artifact admission revalidates core/1.0 through
core/1.5. Inference never runs on loaded checked artifacts.

## Generic constructor arguments

```lobo
record Box<T> { value: T }
variant Lookup<T> { Missing, Found(T) }
record User { name: Text }

fn find_user(id: I64) -> Lookup<User> {
  if id == 1 { Found(User { name: "Luis" }) } else { Missing }
}
```

Generic record and qualified variant constructors can omit type arguments. Record field types and variant
payload types constrain the declaration's formal parameters. A matching expected nominal type also supplies
constraints. Every parameter must receive one consistent type. Explicit constructors remain available and
produce the same nominal specialization identities and checked code.

`Box { value: 42 }` infers `Box<I64>`. `Lookup::Found(42)` infers `Lookup<I64>`. A payload-free
`Lookup::Missing` needs an expected `Lookup<T>` that determines T. No default type is chosen. Formal types
remain rigid in generic definitions. Conflicts fail with `compile.type.mismatch`. Unresolved qualified
constructors retain `compile.generic.arguments` from specialization admission.

## Contextual variant cases

When an expected variant type identifies the declaration, its case name can stand alone. In a function
returning `Lookup<User>`, `Found(user)` and `Missing` mean the corresponding Lookup constructors. This does
not import cases into the module's value namespace or search other variants by spelling.

Expected types flow through function results, annotated local bindings, call/invocation arguments, record
fields, variant payloads, match arms, conditional branches and typed loop state/initializers. A generic call
can defer an unknown constructor argument until other arguments determine the parameter type. Generic record
fields can similarly obtain constraints from sibling fields. Expression types remain checked by the existing
parametric and concrete checkers.

For unqualified cases, local bindings and explicit declarations/import aliases take precedence. A bare case
without an identifying expected variant is unresolved, even if only one supplied variant has that name.
Two variants may share case names; the expected nominal identity chooses the owner. Unknown cases do not
silently select a different variant. Wrong payload types or arities fail. Payload-free cases use `Missing`;
the zero-argument call `Missing()` is also accepted in an identifying context.

Annotations remain explicit at declaration boundaries. This change does not infer function signatures,
generic function calls from expected return types, or an unqualified case owner from its payload alone.
An unannotated `let value = Missing` is therefore invalid. Use an annotation or a qualified explicit constructor.
The bounded generic-callback restrictions from 1.4 remain.

## Checking and tooling

The existing source pass elaborates these forms into explicit record/variant constructors before generic
definition checking and specialization. Stable IDs are retained. No checked-expression kind, runtime operation,
implicit conversion, closure or dependency is added. Expected constructor member types can also provide exact
Fn contexts for named callbacks. Function-reference purity remains mandatory.

Existing expression/type/expansion budgets apply, including pre-copy substitution bounds. Source ranges remain
attached to the original expression. Deprecated annotation-based source keeps its explicit constructor rules.
Modern callers may use explicitly supplied legacy data declarations. Loaded checked artifacts require concrete
constructors and do not accept unresolved case names as source sugar.

Extension 1.5.0 follows expected nominal identities for case navigation and infers generic record fields and
variant payloads. Exact source positions, trusted snapshots and native diagnostics remain in use. No grammar
token is added. Editor type construction retains nesting/node/byte bounds and a 262,144-query resolution
budget. Successful type queries are memoized within a snapshot; recursive unresolved queries fail closed.
The [generic MVC](../examples/mvc.generic.lobo) and programming skill demonstrate these forms.
See [decision 0046](../decisions/0046-constructor-inference.md) for scope and verification.
