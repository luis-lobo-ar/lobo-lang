# Lobo language 1.11 details

Declarations use stable ASCII IDs. A project supplies every source explicitly; imports name
declarations, never paths. Global IDs are unique; source names are local to each module.
Ordinary source can omit the inline table when its tracked `.lobo.ids.json` map is supplied by the file
loader or explicit bundle 0.2. The identity rules below apply to either representation. Authoring commands
allocate once. See [the tool workflow](tools.md) for init, sync, rename and recovery.

Binary arithmetic bindings use `operator +(Pair, Pair) -> Pair = add;` at module level. The local pure
non-generic function `add` must match both operand types and the output exactly. The defining module must
own at least one operand's non-generic nominal type. Import the target to enable the binding in another
module. Supported symbols are `+ - * / %`, with fixed precedence and ordered operands. Binding types
exclude generic nominal applications. A closed bundle allows 64 bindings and one target per symbol/type pair.
`Text + Text` is built-in concatenation. Preserve parentheses when replacing nested `Text::concat` calls
to preserve allocation and fuel behavior. Other operators keep their existing definitions.

```lobo
module sample;
use shared { helper };
record Pair { left: I64, right: I64 }
variant Result { Found(Pair), Empty }
fn make(n: I64) -> Result {
  let pair = Pair { left: n, right: helper(n) };
  Result::Found(pair)
}
identities {
  module: "sample", shared.helper: "shared.helper",
  Pair: "sample.pair", Result: "sample.result", make: "sample.make"
}
```

`I64`, `Bool`, `Text`, `Bytes`, records, variants and `List<T>` are supported. Recursive nominal
types are allowed only when every cycle passes through a List. Direct inline cycles are rejected.
This supports nested typed child values without indices or a new pointer constructor. Generic functions
are supported explicitly. Generic records/variants and function values with declared action bounds are also supported.
Constructor syntax is `Pair { left: 1, right: 2 }`, `Result::Found(pair)` and `Result::Empty`.
Field access is `pair.left`. Match all variant cases:

```lobo
match answer { Found(pair) => pair.left, Empty => 0 }
```

Declare a generic function as `fn identity<T>(x: T) -> T { x }` with an `identity` entry in the identities table.
The explicit spelling is `identity::<Text>("hello")`, including inside recursive definitions.
Modern calls can omit arguments when call inputs determine every type parameter. One to eight type parameters
may abstract over any existing concrete value type. See [the inference rules](../../../../spec/core-v1.7.md).
Names are local to the function and cannot shadow declarations/imports or be shadowed by bindings.
Imported templates keep their defining dependencies and may accept caller-owned record types.
Every definition, including unused bodies and contracts, is checked parametrically. Unconstrained type
parameters support homogeneous equality, passing values and list operations; arithmetic/field/call operations
need explicit concrete types or signatures. Concrete instances repeat layout/type/action checks.
Ordinary named callback signatures can determine generic arguments. Expected Fn signatures permit bare named
functions and fully determined generic callbacks. Explicit forms remain useful for underconstrained calls.
A bundle permits 256 function and 128 nominal specializations, with other admission limits still applying.
Keep ordinary concrete wrapper IDs for external invocation; template IDs are not entries.

Declare `record Box<T> { value: T }` and `variant Option<T> { None, Some(T) }` with stable metadata entries.
Use `Box<Text>` / `Option<Text>` as types and `Box::<Text> { value: "hi" }` / `Option::<Text>::Some("hi")`
as explicit constructors. Constructor arguments can be inferred in 1.5; nominal identity still uses the template ID and arguments.
`Box { value: 42 }` infers Box<I64>; `Option::Some(42)` infers Option<I64>. A function returning Option<I64>
can return `Some(42)` or `None`. Typed annotations, call parameters and constructor members also supply context.
An unannotated `let absent = None` has no owner and fails. Local bindings and explicit aliases take precedence.
Use `Fn(T) -> U` for a pure callback and `&identity::<Text>` or `&function_name` to obtain a named reference.
Call it with `callback(value)` or `record.callback(value)`. For an action-bound value use
`Fn(T) actions { demo.read } -> U`. The clause precedes the arrow to distinguish it from actions on a
containing function. References capture no environment. Parameter/output types and declared action sets
match exactly, including in generic inference. Invocation requires the callback's actions in its caller;
contracts cannot invoke actionful callbacks. Merely obtaining a reference performs no actions. Bounds,
target identity and exact signatures are rechecked on artifact load and typed input. Old pure signatures
retain their wire spelling. No implicit widening, closures or action polymorphism. See the
[callback example](callbacks.lobo) and [Core 1.11](../../../../spec/core-v1.11.md).
F64 literals include `1.5`, `2e-3`, `-0.0`; plain `1` is I64. Use homogeneous `+ - * /`, comparisons and
explicit `F64::from_i64` / `F64::to_i64` conversions. Float-to-integer conversion faults on nonfinite or
out-of-range values before truncation toward zero. `F64::is_finite`, `is_nan`, `from_bits` and `to_bits`
handle classification and exact interchange. NaNs compare unequal and arithmetic NaNs are canonical;
signed zeros compare equal. All bits survive copying and explicit interchange. Decimal source literals
must be finite. JSON encodes F64 as `{"type":"F64","value":"0x3ff8000000000000"}`. CLI accepts finite decimal
`F64:1.5` or exact bits `F64:0x7ff0000000000000`. See Core 1.0 for the required floating execution profile.
Generic equality is type-valid but need not be reflexive: a `result == input` contract fails for NaN.
Type checking a generic contract does not prove its truth for all values.

Use `let name = expression;` for an immutable binding. Names cannot shadow visible bindings or
functions. Every block returns its last expression, with no trailing semicolon on that expression.
`if condition { value } else { value }` requires both branches with the same type.
`&&` and `||` short-circuit. Equality and comparisons use `== != < <= > >=`.
I64 arithmetic `+ - * / %` is checked, with range -9223372036854775808 through 9223372036854775807.
Division truncates toward zero; remainder has the dividend's sign. Faults cannot be caught as values.
Use explicit guards and user-defined variants for ordinary application failures.

Lists are persistent values. `List::empty<I64>()`, `List::prepend(value, list)`, `List::head(list)`,
`List::tail(list)` and `List::length(list)` are the operations. Head and tail need a nonempty list.
Prepending reverses processing order unless the algorithm restores it. List elements can be records.
Input/output values retain a logical nesting limit of eight. Internal values may be deeper; structural
equality permits 64 active compound comparisons before a classified stack-exhaustion diagnostic.
Existing fuel, allocation and function-call limits still apply.

Counted iteration carries one typed value, which may be a record:

```lobo
fold (i in n, state: I64 = 0) { state + i }
```

Indices run from 0 through n-1. A negative count faults. A zero count returns the initial value.
The body's type must match the state type. Initialization is outside the new bindings' scope.
An immutable state-driven loop has this shape:

```lobo
while (state: I64 = 1; state < bound) { state + 1 }
```

It checks the current state before each body and returns the final state. There is no assignment,
`break` or `continue`. Use a carried Bool/variant to represent an early stop. Fuel still bounds work.

Text literals use JSON string escapes and contain validated UTF-8. `Text::length(t)` counts Unicode
scalars, not bytes or grapheme clusters. `Text::at(t, i)` returns a scalar integer.
`Text::slice(t, start, count)` uses scalar indices. `Text::concat(a, b)` concatenates text.
`Text::bytes(t)` exposes UTF-8 bytes. Bytes have `empty`, `length`, `at`, `slice`, `push`, `concat`
and `from_list`, all called as `Bytes::operation(...)`. `Text::valid(bytes)` checks UTF-8 and
`Text::from_bytes(bytes)` converts validated bytes or faults. Empty slices at the end are valid.
Check nonnegative bounds using subtraction after validating start; avoid overflowing start+count.

Functions may have pure `requires { condition }` and `ensures { condition }` clauses before the
body. `result` names the return value in a postcondition. Contracts are runtime checks, not a
replacement for independent tests. `actions { label.name }` declares conservative bounds;
callers include their callees' action labels. The closed native provider separately enforces runtime grants. Actions do not grant permissions.

Current source rules are in [language 1.8](../../../../spec/core-v1.8.md). Old inline-ID source is deprecated read
compatibility. Generic definitions, actions and resource rules apply equally to both spellings.

Inherited details: [base syntax](../../../../spec/core-v0.1.md),
[folds](../../../../spec/core-v0.2.md), [records](../../../../spec/core-v0.4.md),
[variants](../../../../spec/core-v0.5.md), [lists](../../../../spec/core-v0.6.md),
[text/bytes](../../../../spec/core-v0.7.md), [while](../../../../spec/core-v0.8.md),
[closed modules](../../../../spec/core-v0.9.md), [list-guarded recursion](../../../../spec/core-v0.10.md),
[generic functions](../../../../spec/core-v0.11.md).

Result propagation uses `try producer()`. Both result types need exactly two payload-bearing Ok/Err cases.
The success types can differ; error types must match exactly. The operator exits the function body on Err
and still checks postconditions. It works through loop children and nested expressions. Direct contract use
is rejected. Keep explicit cleanup after a handled result. See [the pure example](try-example.lobo).

Core 1.9 adds a closed SQLite catalog, not SQL grammar. Native::SqliteCapability/Database/Statement are
opaque and native-only. The library supplies SqlResult<T>, typed Cell/Row values and bound parameters.
See [the interface](../../../../spec/sqlite-interface-v0.1.md). Explicit close/finalize bypass query quotas;
invocation cleanup rolls back active transactions. SQL execution has a separate progress/memory budget.
The core source bundle limits are now 16,384 aggregate tokens and 64 source nominals, with existing
per-file/size/depth/module/specialization bounds retained.


For owned SQLite rows, prefer the pure SQL helpers `expect_columns`, `integer_at`, `decimal_at`, `text_at`
and `bytes_at` with `try` instead of repeating cell-variant matches in controllers. `cell_at` preserves NULL
when the application needs a nullable policy. These functions reject wrong classes and out-of-range indices
as `SqlError::Decode` with backend code zero. They add no authority or implicit conversion. See
[the SQL library](../../../../libraries/sql/README.md). A constructor inside `List::prepend` can still need
an explicit owner such as `Cell::String("")` when expected-type inference cannot determine its owner.


Core 1.10 record updates use three dots: `State { ...state, flag: true }`. One same-type base comes first;
its value is computed once and overrides run in written order. No mutation or arbitrary object spread.
Standalone `try operation();` discards success and propagates Err. It can appear between let bindings;
a final block expression remains required. Neither convenience introduces automatic cleanup or implicit
error conversion. [The current rules](../../../../spec/core-v1.10.md) define bounds and fuel lowering.

Compiler 1.11.1 is a library/tooling refactor on unchanged Core 1.11. HTTP apps use serve/serve_with for
library-owned listeners. serve_listener retains the old listener-based signature/ID. Exact callback action
sets stay unchanged; pure APIs do not acquire DB bounds. Shared policy and reviewed exclusions are in
[the HTTP library reference](../../../../libraries/http-server/README.md).

Compiler/extension 1.11.2 add typed permission library sets on Core 1.11. Include libraries/native/permissions.lobo
with the OS wrapper or HTTP transport. Use named permission helpers in maintained consumers; raw masks remain
at the explicit ABI adapter and regression boundary. Both HTTP workspace files include the new dependency.
