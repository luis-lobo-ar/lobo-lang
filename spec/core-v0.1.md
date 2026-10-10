# Lobo Core v0.1

Status: proposed normative specification for the first executable subset, implemented by the Rust reference.
Version: `core/0.1-draft`.

The compiler emits [native executables](native-v0.1.md) by default. [Checked artifacts](checked-artifact-v0.1.md)
and the reference evaluator support inspection and testing. Resource limits are implementation limits, not
promises that all Core programs terminate.

This document defines a deliberately small language that two implementations can compare. The wider
[language design](../docs/language-design.md) describes later features. A compiler claiming Core conformance
must implement every rule here; support for wider illustrative syntax is not implied.

## Scope

Core contains named pure functions, `I64`, `Bool`, immutable local bindings, conditionals, calls, and runtime
preconditions and postconditions. Programs contain one module. Inputs arrive as values passed by an explicit
host invocation; evaluation produces a value or a classified failure.

Records, enums, collections, generics, host effects, persistence, state machines, concurrency, and static
contract proofs need additional specifications. Core alone is insufficient to implement its own compiler.

## Source and tokens

Source is valid UTF-8. Identifiers use ASCII letters, digits, and underscore, start with a letter or underscore,
and are case-sensitive. Spaces, tabs, carriage returns, and line feeds separate tokens and otherwise have no
meaning. `//` comments run to the next line ending or end of input. Block comments are unsupported.

Keywords are `module`, `function`, `let`, `if`, `else`, `requires`, `ensures`, `true`, `false`, `I64`, and `Bool`.
They cannot be identifiers. `result` is reserved for postconditions and cannot name a declaration or binding.
Punctuation tokens use the longest matching token. Decimal integer tokens contain one or more digits with no
separators or suffixes. Leading zeroes are accepted and do not change the base.

Stable IDs are quoted strings containing one to 128 ASCII characters drawn from letters, digits, `.`, `_`,
`:`, and `-`; their first character must be a letter or digit. Escapes are unsupported in ID strings.
Module names and callable names are simple identifiers in Core.

## Grammar

EBNF notation: `{ X }` means repetition, `[ X ]` means optional, and `|` means alternatives. Terminals are
quoted. Whitespace and comments occur between tokens.

```ebnf
program       = "module", identifier, identity, ";", function, { function } ;
identity      = "@", "id", "(", id_string, ")" ;
function      = "function", identifier, identity,
                "(", [ parameters ], ")", "->", type,
                { precondition }, { postcondition }, block ;
parameters    = parameter, { ",", parameter } ;
parameter     = identifier, ":", type ;
type          = "I64" | "Bool" ;
precondition  = "requires", "{", expression, "}" ;
postcondition = "ensures", "{", expression, "}" ;
block         = "{", { binding }, expression, "}" ;
binding       = "let", identifier, [ ":", type ], "=", expression, ";" ;
expression    = disjunction ;
disjunction   = conjunction, { "||", conjunction } ;
conjunction   = equality, { "&&", equality } ;
equality      = comparison, [ ("==" | "!="), comparison ] ;
comparison    = sum, [ ("<" | "<=" | ">" | ">="), sum ] ;
sum           = product, { ("+" | "-"), product } ;
product       = unary, { ("*" | "/" | "%"), unary } ;
unary         = ("!" | "-"), unary | primary ;
primary       = integer | "true" | "false" | identifier
              | identifier, "(", [ arguments ], ")"
              | "(", expression, ")"
              | "if", expression, block, "else", block ;
arguments     = expression, { ",", expression } ;
```

Repeated binary operators associate to the left. The grammar defines precedence, from `||` at the lowest
precedence through unary operators at the highest. Equality and ordered comparison cannot chain at their own
precedence level. Parentheses can express nested comparisons whose types are valid. Each block has a final
value expression; semicolons terminate bindings only. There is no `return` statement.

The token `id` in an identity annotation is context-specific and is not a global keyword.

## Declarations, identity, and resolution

The module and all functions must carry explicit IDs. All declaration IDs in a program must be unique, including
the module ID. Function names must also be unique. All functions are visible throughout the module, including
before their declaration. Calls may be recursive or mutually recursive.

Parameters must have distinct names. A binding sees parameters, earlier bindings in the same block, enclosing
block bindings, and module functions. Its own name is unavailable in its initializer. A binding cannot shadow
any visible binding, parameter, or function name; sibling branches may independently use the same local name.
Function names cannot be used as values or rebound as parameters.

Unresolved references are compile errors, including inside an unreachable branch. Branch-local bindings are not
visible outside their branch. Preconditions see parameters and functions; postconditions additionally see
`result`. Function body bindings are not visible in contracts.

Stable IDs identify declarations independently of source names. Graph imports must resolve references to IDs;
source imports resolve names first and retain the resulting declaration identity. The complete graph interchange
format is a separate pending specification. A Core compiler must not advertise graph import conformance before
that format and its validation rules exist.

## Types

`I64` values are integers in `[-9223372036854775808, 9223372036854775807]`. `Bool` values are exactly `true` and
`false`. There are no implicit conversions, truthy values, floating-point values, nulls, or subtypes.

| Construct | Input types | Result type |
| --- | --- | --- |
| Integer literal | literal in range | `I64` |
| Boolean literal | boolean | `Bool` |
| Unary `-` | `I64` | `I64` |
| `+`, `-`, `*`, `/`, `%` | two `I64` | `I64` |
| `<`, `<=`, `>`, `>=` | two `I64` | `Bool` |
| `==`, `!=` | two values of the same type | `Bool` |
| `!` | `Bool` | `Bool` |
| `&&`, `||` | two `Bool` | `Bool` |
| Conditional | `Bool` condition; equal branch types | branch type |
| Call | exact parameter types and arity | declared output type |

A binding without an annotation takes its initializer's type. An annotation must match exactly. A function's
final expression must match its declared output type. Every contract expression must have type `Bool`.
All declarations, branches, and contracts are checked whether or not a host invocation will reach them.

## Integer literals and arithmetic

A bare integer token must fit the non-negative `I64` range. An integer token immediately beneath unary minus
may additionally have magnitude `9223372036854775808`; together they denote the minimum `I64` value. Parentheses
break that special case: `-(9223372036854775808)` is a literal-range compile error. This rule is based on parsed
structure, so whitespace between minus and digits is immaterial.

Addition, subtraction, multiplication, and negation use checked arithmetic. An out-of-range result produces
`runtime.integer.overflow`; it never wraps and never has undefined behavior.

Division truncates toward zero. Remainder has the dividend's sign and satisfies `a = (a / b) * b + (a % b)` as a
mathematical integer equation when division is defined. Division and remainder by zero produce
`runtime.integer.zero_divisor`. Both minimum `I64 / -1` and minimum `I64 % -1` produce
`runtime.integer.overflow`, making this boundary uniform across backends.

Constant evaluation must preserve runtime faults. A constant overflowing expression remains a well-typed
program that fails if evaluated; it is not a mandatory compile error. Literal range errors are compile errors.

## Evaluation

Evaluation is sequential. Bindings evaluate in source order, then the block's final expression evaluates.
Ordinary binary operands and call arguments evaluate left to right. A fault stops the current invocation;
later operands, bindings, and calls are not evaluated.

`false && e` returns `false` without evaluating `e`; `true || e` returns `true` without evaluating `e`.
Other boolean combinations evaluate the right operand. A conditional evaluates its condition followed by
exactly one branch. Type checking still checks the unevaluated branch or operand.

Function arguments are values. Bindings are immutable. No operation in Core can read or write external state,
use a clock, sample randomness, or spawn a task. Backend imports implementing such behavior are prohibited.

## Contracts

The runner checks each precondition in declaration order before entering the body. A false precondition
produces `runtime.contract.precondition`. It then evaluates the body and, if that succeeds, checks each
postcondition in declaration order with `result` bound to the output value. A false postcondition produces
`runtime.contract.postcondition`. Nested calls follow the same procedure.

An arithmetic fault or nested contract failure during a contract preserves its underlying fault code and
records that the fault occurred during a precondition or postcondition. Postconditions are not evaluated after
a body fault. Core contract checks cannot be disabled by an optimization or build mode.

Passing particular invocations is test evidence only. Core defines no proof that a contract holds for all inputs
and does not infer that a precondition is satisfied at a call site.

## Host invocation and outcomes

An invocation identifies a compiled artifact, a function by stable ID, arguments, and host resource limits.
I64 values in JSON interchange are decimal strings, avoiding loss of integer precision. Bool values are JSON
booleans. Types are explicit, for example `{"type":"I64","value":"42"}`.

Host arguments with the wrong types, wrong count, or out-of-range values are invocation errors before function
evaluation. They do not constitute compiler failure or a language runtime fault.

The outcome categories are:

- `compile_error`: the program violates a static rule;
- `value`: invocation returns a value after contracts pass;
- `runtime_fault`: evaluation hits a specified arithmetic or contract failure;
- `invocation_error`: host supplied an invalid entrypoint or arguments;
- `resource_exhausted`: host fuel, deadline, stack, or memory limit prevents completion;
- `internal_error`: implementation defect, never an acceptable conformance outcome.

Recursion is permitted and termination is not guaranteed. Resource accounting is host-specific in v0.1; comparing
exact fuel use or the order of exhaustion across implementations is outside language conformance. Differential
tests use sufficiently small, terminating programs and report exhaustion separately from semantic disagreement.

## Diagnostics

The following stable primary codes are required when the corresponding problem is the only error in a fixture:

| Code | Condition |
| --- | --- |
| `compile.syntax.invalid` | Source cannot be tokenized or parsed |
| `compile.identity.duplicate` | Declaration ID is repeated |
| `compile.name.duplicate` | Function or parameter name is repeated, or binding shadows a visible name |
| `compile.name.unresolved` | Referenced name is unavailable |
| `compile.name.reserved` | Reserved word is used as a name |
| `compile.literal.range` | Integer literal is out of range |
| `compile.type.mismatch` | Operand, annotation, contract, branch, argument, or output type is wrong |
| `compile.call.arity` | Call has the wrong argument count |
| `runtime.integer.overflow` | Checked arithmetic exceeds `I64` |
| `runtime.integer.zero_divisor` | Division or remainder divisor is zero |
| `runtime.contract.precondition` | Precondition evaluates to false |
| `runtime.contract.postcondition` | Postcondition evaluates to false |

Human wording and source spans can differ between implementations. Multi-error reporting order is not prescribed
in v0.1. A diagnostic should identify the affected declaration ID and phase where they are available.

## Minimal example

```lobo
module arithmetic @id("example.arithmetic");

function increment @id("example.increment")(x: I64) -> I64
  requires { x < 9223372036854775807 }
  ensures { result == x + 1 }
{
  let next: I64 = x + 1;
  next
}
```

Invoking `example.increment` with `41` returns `42`. Invoking it with the maximum `I64` fails its precondition
before evaluating `x + 1`. Its postcondition is checked at runtime; it has not been statically proven.

## Conformance boundary

Conformance applies to the rules above and the associated fixtures. It does not imply graph interchange,
daemon behavior, effects, resource-limit equivalence, static verification, or self-hosting. A release declares
its supported language version and separate backend and protocol versions.

