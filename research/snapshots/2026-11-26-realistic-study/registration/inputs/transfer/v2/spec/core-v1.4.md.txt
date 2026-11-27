# Lobo language 1.4

Status implemented by compiler 1.4.0. Modern source supports argument-based generic function inference and
named function values where a function signature is expected. All other rules inherit [language 1.3](core-v1.3.md).
Checked format stays `lobo-checked/1.0`. Source compilation emits `core/1.4`. Artifact admission revalidates
core/1.0 through core/1.4. Inference never runs on loaded checked artifacts.

## Generic function calls

```lobo
fn identity<T>(value: T) -> T { value }
fn answer() -> I64 { identity(42) }
```

Omitting all function type arguments requests inference from that call's arguments. The declared signature
provides constraints. Matching traverses lists, nominal type applications and function signatures structurally.
Nominal identities must match exactly. Each formal parameter receives one consistent type. There is no numeric
promotion, overload search, default type or implicit conversion. All parameters must be determined.
Explicit `identity::<I64>(42)` remains supported and produces the same specialization identity and checked code.
Partial explicit type-argument lists remain invalid.

Nested calls, immutable bindings, constructors, fields, match payloads and loop state can supply argument types.
Ordinary named callback signatures also contribute constraints, independent of argument order. Function
signatures and nominal declarations remain explicit. Generic nominal constructors still require type arguments.
Inference does not use a call's expected return type. A zero-argument `make<T>()` needs explicit type arguments,
even inside a function with a concrete return type. Unresolved parameters produce `compile.generic.inference`.
Conflicting types produce `compile.type.mismatch`. Invalid arity retains `compile.call.arity`.

## Contextual named function values

```lobo
fn respond_json<T>(lookup: Lookup<T>, render: Fn(T) -> Json) -> Response {
  match lookup {
    Missing => http.not_found(),
    Found(value) => json.json_response(200, render(value))
  }
}

fn show_user(id: I64) -> Response {
  respond_json(find_user(id), render_user)
}
```

This fragment assumes the declarations and selected imports in the executable
[generic MVC example](../examples/mvc.generic.lobo). `find_user` returns `Lookup<User>` and `render_user`
has signature `Fn(User) -> Json`. The call infers T as User and elaborates the callback to a named reference.

Named function values are accepted in static-call arguments, function-value invocation arguments, explicitly
typed local bindings and function results when an exact `Fn` signature is known. Branches inherit that expected
signature. Function names resolve through the current module's explicit aliases. This adds no closures,
captures or action-polymorphic callbacks. Purity remains mandatory and actionful references still fail with
`compile.action.reference`. Non-function names cannot become function references.

A generic callback can itself specialize from a complete expected function signature. For example,
`apply::<I64,I64>(identity, 42)` is valid for `apply<T,U>(f: Fn(T)->U, x:T)->U`. This bounded implementation
does not jointly solve outer unknowns from a generic callback's partially known signature. `apply(identity,42)`
therefore needs the explicit outer arguments. With a monomorphic callback such as `increment`, `apply(increment,42)`
infers both parameters. There is no inference from an uncalled generic name alone.

Explicit `&function` and `&generic::<Type>` remain available. An unannotated `let f = function_name` still
requires explicit reference syntax. Record fields and variant payloads retain explicit reference syntax in
this step. Bare names outside the supported expected-function contexts remain ordinary value names.

## Checking, compatibility and bounds

The source pass fills explicit call/reference arguments before existing parametric definition checking,
specialization, action checking and concrete validation. Rigid parameters in generic bodies remain distinct.
Unused definitions and every branch/contract are still checked. Arguments and callbacks retain existing runtime
evaluation order, fuel, allocation, contract and call-depth behavior. Native backends receive the existing
concrete calls and pure references. The checked representation and runtime require no new construct.

Inference applies to modern `module name;` source. Deprecated annotation-based source keeps its frozen
explicit-call grammar and existing read semantics. Mixed bundles may import a legacy generic function into a
modern caller and infer at that modern call site. Source spellings do not change stable declaration IDs.

The pass admits at most 262,144 expression visits and retains the existing expression nesting bound of 32.
Substituted types are checked before copying, with nesting at most eight and at most 8,192 visited type nodes
per substitution. Exhaustion is `resource.frontend.limit`. Existing source, template and specialization bounds
also apply. These limits prevent deeply nested inferred function signatures from expanding without bounds.

Extension 1.4.0 resolves inferred results and contextual callback names using supplied source signatures and
nominal identities. It retains original source positions and native compiler diagnostics. Structural editor
inference is not the type authority. No grammar token is added. The programming skill and executable examples
use the new calls while retaining explicit examples where inference is insufficient.

See [decision 0045](../decisions/0045-local-call-inference.md) for scope, alternatives and evidence.
