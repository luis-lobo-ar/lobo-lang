# Core v0.11: explicit generic functions

Status: implemented experimentally, 2026-11-04. Inherits [Core v0.10](core-v0.10.md).
Versions: compiler 0.11.0, `core/0.11-draft`, `lobo-checked/0.11`. Native adapter 0.6 is unchanged.
Motivation and alternatives: [decision 0030](../decisions/0030-explicit-generic-function-specialization.md).
Public acceptance inputs: [reuse workload](generic-reuse-workload.md).

## Source and checking

```lobo
function identity<T> @id("example.identity")(x: T) -> T
ensures { result == x }
{ x }
function number @id("example.number")(x: I64) -> I64 { identity::<I64>(x) }
```

Function declarations may have one through eight distinct type parameter names between their source
name and identity. These names are function-local, visible in parameter/output annotations, contract
expressions, local annotations, List elements, loop states and recursive/nested call type arguments.
They may not shadow visible declaration/import names or be shadowed by value bindings. They do not
escape the function. Empty parameter/argument lists are syntax errors. Every existing concrete value
type, including nested Lists and recursive records/variants, may be an argument. There is no implicit
type inference. `f::<A, B>(...)` is always a call, preserving ordinary `a < b` expression parsing.

Every call to a generic function supplies exactly its declared number of type arguments, including
recursive calls. Missing/extra arguments and arguments on an ordinary function are
`compile.generic.arguments`. Existing value-argument arity and type diagnostics still apply. Unknown
types fail resolution/checking. Type parameters do not declare constructors, fields, methods, or cases;
construct a value using an ordinary concrete constructor or pass an existing value.

Templates are parsed and declaration references (types, calls, concrete constructors) resolve in their
defining module. Each requested concrete specialization then undergoes the existing local-name, type,
action, contract and resource-admission checks before code generation. An unrequested body's local names
and types are not checked. For example `x + x` is accepted in a template instantiated with
I64 and rejected when instantiated with Bool. This is instantiation-based checking, **not** a guarantee
that an arbitrary template works for every type. Diagnostics for an instantiated body identify the
defining source/function; runtime faults identify the concrete generated function where applicable.

Requires/ensures and action bounds are copied and checked for each concrete instance. A caller still
declares the callee's action labels; contracts still call only pure functions. Specialization adds no
capabilities or host operations. Runtime fuel, stack, heap, comparison and serialization limits remain.

## Modules, identity and lowered artifacts

Templates are imported by their existing stable declaration ID. Their resolved dependencies keep the
definition's meaning; the caller cannot capture a private helper by supplying a same-spelled name.
An imported template may accept a nominal type owned by the caller without the library importing it.
Generated scopes explicitly import the definition's concrete dependencies, supplied nominal types and
other requested instances. Source caller scopes import generated functions, without gaining access to
the library's unrelated private names. Saved artifacts retain these concrete scopes and imports.

A specialization key is the JSON serialization of `(template stable ID, ordered concrete Type arguments)`.
Its ID is `lobo.generic.` followed by the 32 lowercase hexadecimal digits of FNV-1a-128 of the key's UTF-8
bytes (offset 6c62272e07bb014262b821756295c58d, prime 0000000001000000000000000000013b).
This is a deterministic identity convention, not a security/content hash. Cache hits compare the full
key; a collision fails `compile.identity.duplicate`. Repeated requests share one instance, including
recursive requests. IDs do not depend on request order, argument values, source labels, aliases, or
unrelated declarations. Source IDs beginning `lobo.generic.` and source names beginning `__lobo_` are
reserved. Generated scope IDs append `.module`; generated names are internal and not source APIs.

Checked artifacts contain ordinary concrete functions, Types, calls and scopes, with no templates or
type arguments. Nonempty `type_parameters` on a function or `type_arguments` on a call in a forged artifact
fails `compile.generic.unlowered`. Loading always rechecks concrete code, types, visibility, actions and
contracts. Generic template IDs are not executable entries. Concrete wrapper IDs remain the intended
external API; check output may also list generated entries and their action bounds. Recompile older
artifacts from source. Existing standalone native executables still run independently.

## Admission and current boundary

One closed bundle supplies at most sixteen source modules and may request at most 64 distinct
specializations. Lowered artifacts allow at most eighty source/generated scopes. The existing 1024
declaration/import metadata slot budget includes generated scopes. Expansion visits at most 32768
expression nodes, counts ordinary bodies too, and retains the existing expression-depth bound.
Type substitution may not produce more than eight nested List constructors. A library source may contain
only templates, but the linked program must contain at least one ordinary concrete function.
Violations are classified
frontend/module diagnostics before execution; type-growing recursion terminates compilation with an
admission diagnostic instead of recursing indefinitely in the compiler. These are finite implementation
limits, not a completeness theorem.

No generic records/variants, constraints/traits, overloads, inference, higher-kinded types, function
values or closures are added. Concrete specialization can increase compile time and executable size;
neither a speedup nor improved agent construction/comprehension is established by acceptance tests.
