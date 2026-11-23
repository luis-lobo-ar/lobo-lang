# 0052: Explicit result propagation

Date: 2026-11-22
Status: Implemented in compiler/extension 1.8.0 and language core/1.8 after the user selected implementation.

The snapshot program repeats Err(error) => Err(error) three times. Each next operation depends on the
previous success. Use the prefix keyword try to expose this sequence without nested matches.

Implemented syntax:

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

On success, the expression yields the payload. On failure, it returns the error through the enclosing
function's result. Evaluate the operand exactly once. No later statements execute after a propagated error.
The return success type can differ from the operand success type, as I64 and Snapshot do in this example.
Error types must agree. Do not silently add conversions, exceptions, permission grants or error recovery.

The implemented rule is an explicit structural result protocol over nominal types. Operand and function
output must each have exactly two cases, Ok(T) and Err(E), with both payloads present. Error types must match
exactly; success types and result owners can differ. This supports ordinary user-defined Result<T,E>,
IoResult<T> and native ABI result variants without privileged library identities. Shape, output and exact
error typing are all checked, including parametric definitions and reloaded artifacts. Case names alone
are insufficient. This resolves and supersedes the earlier narrow stable-ID-only candidate.

The implementation preserves action-bound checks, capability attenuation, evaluation order, source
ranges, native/reference equivalence and function boundaries. Propagation from nested expressions, fold and
while children exits the current function body. Postconditions still run and frame metadata restores through
a shared function epilogue. Runtime faults remain faults. Direct use in contracts is rejected. A failure after acquiring a socket must not bypass required explicit cleanup.
Invocation-exit cleanup alone does not establish correct recovery within a running server. Do not migrate
transport error handling mechanically.

Keep action declarations visible as checked operation bounds. Runtime grants remain a separate authority
mechanism. Replacing the numeric permission mask 3 with named permissions is a separate API improvement,
not required to solve error-forwarding repetition and not specified here.

Reviewed alternatives were existing exhaustive matches, callback-based result chaining, postfix ? and prefix try.
Matches remain useful for recovery and error mapping. Chaining would require nested callbacks or additional
binding machinery for the dependent operations. Prefix try directly represents the repeated propagation pattern.
The user selected try over the initially suggested ? spelling. Preserve that selection in implementation;
do not introduce both spellings. The semantics are explicit result propagation through the current function.
This keyword does not introduce a catch block or recovery behavior.

The first conformance tests failed on unrecognized prefix syntax. The implemented Unary op try is checked
and evaluated by the shared compiler/evaluator, emitted as C control flow or LLVM branches, and gated to
core/1.8 on artifact loading. Existing expression traversal and dependency analysis already visit Unary
operands. Success values retain their inferred types in compiler and editor navigation.

The contextual prefix form is try followed by a name, keyword, number or string token. Legacy try(value)
calls and identifiers remain ordinary syntax. Parenthesize the containing expression as (try fetch()).
The maintained snapshot uses try with unchanged sidecar IDs. REST cleanup code is intentionally unchanged.

The conformance, Core 1.7 compatibility and inferred-import groups cover the five execution modes,
loops/expressions/short-circuiting, caller boundaries, postconditions, faults, actions, reloading, single native
evaluation and explicit caller cleanup. Editor tests include highlighting, payload navigation, Core 1.8 JSON
and real-host unsaved errors/repair. A failing cross-module regression corrected an extra visibility check: an inferred success payload must
not demand an unused import when the equivalent match does not. Both compiler and navigation now cover it.
See [the full rule](../spec/core-v1.8.md).

This implementation has no measured effect on agent construction, comprehension, tokens or runtime performance.
Fewer repeated match arms are a source-level observation, not evidence of improved software construction.

## Existing language precedent

Reviewed 2026-11-22: [Rust's Result documentation](https://doc.rust-lang.org/core/result/index.html)
and [operator reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html) define postfix ?
for propagation. On Result success it yields the payload; on failure it returns through the enclosing
function, with Rust's applicable conversion rules. Lobo's implemented rule leaves implicit error conversions out of scope. This is an existing syntax convention, not a new Lobo invention or evidence
of an agent-efficiency benefit. Swift's [optional chaining](https://docs.swift.org/latest/documentation/the-swift-programming-language/optionalchaining/)
uses the question mark for optional values and has different semantics.

[Zig's error propagation](https://ziglang.org/documentation/master/#Errors) uses the word try for the same
broad pattern: yield the success value or return the error from the current function. It provides a keyword
alternative now selected by the Lobo user. The spelling is implemented in compiler 1.8.0. Neither precedent demonstrates a measured agent-development advantage for Lobo.
