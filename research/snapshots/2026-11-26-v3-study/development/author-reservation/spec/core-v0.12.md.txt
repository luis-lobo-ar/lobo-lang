# Core 0.12 development rules

Historical development version, superseded by [language 1.0](core-v1.0.md). The rules below remain its
semantic reference; their development/version labels are historical, not current package status.

Status: implemented development subset on the path to the bounded language 1.0 release.
Inherit [Core 0.11](core-v0.11.md), except for the rules explicitly superseded here.
Compiler `0.12.0-dev`, language `core/0.12-draft`, checked format `lobo-checked/0.12`.
Old checked artifacts require recompilation. The fixed program acceptance and final release remain pending.

## Generic nominal types

Declarations use `record Cursor<T> @id("cursor") { rest: List<T> }` or
`variant Option<T> @id("option") { None, Some(T) }`. A declaration has at most eight distinct
type parameters. Parameters cannot shadow visible declarations. Use `Cursor<I64>` in types,
`Cursor::<I64> { rest: xs }` and `Option::<I64>::Some(7)` in constructors. Every application
supplies exactly the declared number of arguments; inference and partial application are absent.

Type arguments are value types, including nested generic nominals, lists and function signatures.
Applications retain nominal identity: a template's stable identity and ordered type arguments determine
the instance, independent of source aliases or unrelated requests. Functions and data have distinct
identity domains. Compiler-generated `lobo.generic.` declaration/scope identities remain reserved.
Inline layout cycles remain invalid; recursion through List remains valid after specialization.

## Definition-time generic checking

Every generic function and nominal definition is checked, including unused declarations, every branch,
local annotation and contract. Each formal parameter is a distinct rigid symbolic value type. This is
parametric checking, not sampling a collection of concrete types. A parameter supports passing/copying,
homogeneous equality, and uniform list operations. It does not imply I64 arithmetic, a record shape,
a variant shape or a callable signature. Such operations require an explicit concrete type/signature.
No numeric/trait constraint syntax is introduced by this subset. Contracts are type-checked, not proved.

Checking uses a separate bounded expansion with rigid parameters; concrete instantiation repeats layout,
scope, action, type and resource checks. Recursive type growth can exceed admission and is rejected even
for an unused definition. This is a resource boundary, not a termination proof. Symbolic types never appear
in accepted checked artifacts or native code. Earlier instance-only checking in Core 0.11 is superseded.

## Pure non-capturing function values

`Fn(I64, Text) -> Bool` is an exact checked signature with an empty action bound. `Fn() -> I64` is valid.
`&name` refers to a named ordinary function; `&name::<I64>` explicitly specializes a generic function.
References can be passed, returned, stored in records/lists/variants, selected at runtime and compared
by function identity. They capture no environment. References to actionful functions are rejected with
`compile.action.reference`; ordinary static actionful calls retain their existing bound checks.

Invoke a value with `callback(argument)` or `record.callback(argument)`; arbitrary postfix calls are valid.
The callee is evaluated before arguments, and arguments left to right. Arity and every input/output type
must agree. Calls enter the same contract/fuel/call-depth machinery as static calls. Pure references can
be invoked in contracts. `Fn` is reserved. Lambda/capture syntax and action-polymorphic callbacks are absent.

CLI atoms are `Function:stable.id`, including inside compound values. Input validation requires that ID
to name a pure function in the closed artifact with the exact expected signature. Typed JSON values use
`{"type":"Function","value":{"id":"stable.id","signature":{"parameters":["I64"],"output":"I64"}}}`.
Programmatic input is revalidated recursively, including signature/identity/purity. Each reference occupies
one logical inline cell, allocates no heap and consumes one input/output node. Signature metadata is included
in the existing output byte budget. Normal JSON-string escaping applies. References are represented by a
closed function index natively; C and direct LLVM dispatch to compiled function bodies.

## F64

F64 is IEEE binary64 under round-to-nearest ties-even, with gradual underflow and default masked floating
exceptions. The existing Linux x86-64 platform is the release target. Entry probes reject unsupported
rounding/subnormal modes with `invocation.float.environment`; they do not modify the host environment or
isolate arbitrary exception-trap configurations. See decision 0034. No fast-math or fused operation is used.

Source literals have decimal digits, an optional fractional part with digits after `.`, and an optional
`e`/`E` exponent with optional sign and at least one digit. A fraction or exponent makes a literal F64;
otherwise it is I64. Unary `-` preserves signed zero. Source/checked literals must be finite, including
subnormal or rounded-to-zero values. Arithmetic `+ - * /` and unary negation are homogeneous F64 operations;
there is no implicit I64 promotion or F64 remainder. Division by zero and overflow follow IEEE results.
Every arithmetic NaN result is canonical `0x7ff8000000000000`.

Comparisons use IEEE ordered `< <= > >=` and equality. A NaN is unequal to every value including itself;
all ordered comparisons involving NaN are false; signed zeros compare equal. Structural equality of lists,
records and variants applies these rules recursively. Tool/JSON value identity can still distinguish bits.

`F64::from_i64` rounds to binary64. `F64::to_i64` truncates toward zero after rejecting nonfinite values or
values outside `[-2^63,2^63)` with `runtime.float.conversion`. `F64::from_bits(I64)` and `F64::to_bits(F64)`
reinterpret all 64 bits using two's-complement I64 representation; they preserve zero sign and NaN payloads.
`F64::is_finite` and `F64::is_nan` return Bool. Each operation has one explicitly typed input.

Typed JSON is `{"type":"F64","value":"0x3ff8000000000000"}`: exactly sixteen lowercase hex digits encode
all bits in numerical most-significant-digit order. CLI atoms accept `F64:1.5`, `F64:-0.0`, `F64:1e-3`, or
`F64:0x7ff0000000000000`. Decimal atoms are finite decimal numbers, allowing leading sign or `.5`/`1.`;
nonfinite spellings, decimal overflow, hex arithmetic notation and noncanonical bit strings are rejected.
Inputs, output, bit conversions and plain copying preserve bits; arithmetic normalizes NaNs. Rust's public
`Value::F64(u64)` stores bits. A value occupies one inline cell, consumes one I/O node, and allocates no heap.
Existing evaluation, contract, list-allocation and output-byte rules apply unchanged.

## Admission and artifact changes

Retain source 1 MiB / 8,192 tokens / 16 modules and existing logical value limits. An expansion admits at
most 256 function instances and 128 nominal instances, with 262,144 visited expression nodes. Definition
checking and concrete lowering each have these budgets, including requests arising from ordinary functions.
Checked metadata admits 1,024 scopes and 16,384 declaration/import slots. Total concrete nominal declarations
are bounded at 160, preserving the 32-source-nominal bound plus up to 128 instances; inline width/height remain 64/8. These are deterministic admission limits, not performance
claims. Exceeding a limit fails closed. No new dependency or custom runtime was added.

Checked programs contain concrete records, variants and functions only. Source type applications, nonempty
template parameters, constructor/call/reference type arguments and symbolic parameters are rejected on load.
Function signatures and pure references are checked again. Native backends may rely on those validated facts.
