# Lobo language 1.8: explicit result propagation

Compiler and extension 1.8.0 implement Core 1.8. It inherits [Core 1.7](core-v1.7.md), including the native ABI,
and adds the prefix expression `try operand`. No postfix question-mark spelling is supported.

```lobo
fn main() -> IoResult<Snapshot>
actions { native.authorize, clock.read, random.read }
{
  let cap = try os.restrict_capability(Native::capability(), 3);
  let now = try os.monotonic_ms(cap);
  let bytes = try os.random_bytes(cap, 16);
  Ok(Snapshot { monotonic_ms: now, random_bytes: bytes })
}
```

## Type rule

The operand's type must be a nominal variant with exactly two cases, `Ok(T)` and `Err(E)`, each with a payload.
The enclosing function must return a variant of the same two-case shape, `Ok(U)` and `Err(E)`. Case order does
not matter. T and U may differ. Error payload types must be exactly equal after normal generic substitution.
Nominal error records or variants retain identity. No implicit error conversion occurs.

This explicit shape rule is the built-in propagation protocol. It works with user-defined `Result<T,E>` or
`IoResult<T>` declarations and native ABI result variants. It is not restricted to a library name or source
alias. Case names alone are insufficient: both payloads, exactly two cases, the output shape and error type
must satisfy the rule. Payload-free optional types and variants with additional cases are rejected.

The expression type of `try operand` is T. Normal inference must determine the operand variant type. This
operator does not infer a missing result family or its generic error parameter from an unqualified constructor.
Supply a qualified constructor or an explicitly typed producer when normal inference cannot determine it.
An inferred success payload does not require a new explicit type import, matching existing match binding
behavior. Imports are still required when source explicitly names that type.
Generic definitions are checked parametrically, including definitions that are not instantiated.

## Execution

Evaluate the operand once. For `Ok(value)`, yield value and continue the enclosing expression. For `Err(error)`,
construct the enclosing function's error case and finish that function body immediately. Skip later bindings,
operands, arguments, fields and loop iterations on that path. A caller of that function continues normally
with its returned result and may handle it explicitly.

The operator binds as a unary expression. `try fetch() + 1` propagates the result of fetch before addition.
Normal short-circuit rules apply: an unevaluated operand cannot propagate. `try` works in branches, function
arguments, loop counts, initial states, loop conditions and loop bodies. Propagation exits the function,
not merely the nearest block, match arm, fold or while loop.

Preconditions run before the body. Postconditions run on both normal and propagated results. Call depth,
diagnostic subject and contract phase restore through the same function epilogue. Runtime faults and resource
exhaustion remain faults. The operator handles a typed Err value; it does not catch faults.

Direct use in requires or ensures expressions is rejected with `compile.try.contract`. A pure helper function
may use try internally and return its normal result to a contract that handles that result explicitly.
Action bounds and runtime permission checks remain unchanged. A try expression does not grant authority.

## Contextual spelling and compatibility

The prefix spelling starts with `try` followed by a name, keyword, number or string token. Its operand is
parsed with the existing unary rules. Use `try fetch()`, `try value` or `try Result::<I64,Text>::Ok(42)`.
Parentheses around a larger containing expression work normally, such as `(try fetch()) + 1`.

For compatibility, `try(value)` remains an ordinary function call, as does `try (value)`. It does not spell
propagation. Group a larger operand in a typed helper if needed. Existing declarations, fields and variables
named try remain ordinary identifiers outside the prefix form. Comments and whitespace between prefix and
operand are accepted by the parser. This is consistent with the language's contextual fold/while keywords.

Checked framing remains `lobo-checked/1.0`. The expression is a Unary node with op `try`; operands are ordinary
checked expressions. Core 1.0 through 1.7 artifacts retain their existing rules and are revalidated. A try
node in an older language artifact is rejected with `invocation.artifact.version`. Native operations remain
valid in Core 1.7 and 1.8; this milestone changes neither ABI 1 nor native resource layout.

## Resource ownership

Try has no implicit per-function resource cleanup. Keep explicit cleanup around handled failures:

```lobo
let outcome = work(stream);
let closed = close_stream(stream);
outcome
```

If work uses try and returns an error, the caller still executes close_stream. Propagating directly past that
close would change cleanup behavior. Invocation-exit native cleanup remains a final bound; it does not replace
cleanup within a long-running server. Existing REST transport handling is retained unchanged.

## Diagnostics and tools

| Code | Meaning |
| --- | --- |
| compile.try.operand | Operand lacks the required result shape |
| compile.try.output | Enclosing function lacks the required result shape |
| compile.try.error | Error payload types differ |
| compile.try.contract | Direct propagation in a contract |

Diagnostics identify the try expression in the original source. The extension highlights the contextual
operator, navigates its operand and inferred success payload, and accepts core/1.8 compiler results.
Run `cargo test --locked --test try_propagation` for conformance and native cleanup checks.
