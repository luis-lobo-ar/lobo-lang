# Frozen language-first acceptance workloads

Status: inputs/outcomes frozen before implementation. Workload A is executable in
[parser.lobo](../examples/parser.lobo), tested in `tests/workloads.rs` in reference/optimized/sanitized
modes, including new expression variations, arbitrary octets, deep nesting, and fuel/heap exhaustion.
Workload B is executable in
[collections.lobo](../examples/collections.lobo), tested in `tests/lists.rs` in reference/optimized/sanitized
modes, including the odd-predicate variation and heap exhaustion. Workload C is executable in
[tree.lobo](../examples/tree.lobo), tested in `tests/tree.rs` in the same three modes, including actual
parent-edge inspection, reverse traversal, extreme keys, deep chains, duplicate-preserving variations,
and fuel/heap exhaustion. The overall goal remains in
[language completeness](../docs/language-completeness.md); three successes alone do not imply universal replacement.
Inputs are in-memory values. The harness may encode them and assert outcomes, but no host-language parser,
algorithm, database, or FFI may perform the workload computation. Use native and reference paths, independent
expected values, negative/boundary/exhaustion cases, and editor regressions. Freeze additions before using them.

## A: arithmetic parser and evaluator

Parse an in-memory ASCII byte sequence into structured tokens/state and evaluate nonnegative decimal I64
integers, `+`, `*`, parentheses, and ASCII space. Multiplication binds tighter; both operators associate left.
Reject missing operands/operators, unexpected bytes, unbalanced parentheses, and integer/arithmetic overflow
as distinct typed failures with zero-based offending byte offset. End-of-input errors point to input length.

- `12 + 3 * 4` -> success 24; `(12 + 3) * 4` -> 60; `2*3*4+5+6` -> 35.
- `0` -> 0; spaces around `42` -> 42; empty input -> missing operand at 0.
- `1 +` -> missing operand at 3; `1 2` -> missing operator at 2; `(1+2` -> unbalanced at 4.
- `2 & 3` -> unexpected byte at 2; `9223372036854775808` -> overflow at 18.
- `9223372036854775807+1` -> arithmetic overflow at operator offset 19.
- Variation: supply a new valid expression using the same operators without compiler changes.

Additional parser cases frozen before its implementation:

- `9223372036854775807` succeeds; leading zeroes are accepted; only byte 32 counts as space.
- `2*(3+4*(5+6))` -> 94; 128 nested parenthesis pairs around `7` -> 7 without call-depth exhaustion.
- `2*0+7` -> 7; `3037000500*3037000500` -> arithmetic overflow at 10.
- `()` -> missing operand at 1; `1++2` -> missing operand at 2; `1(2)` -> missing operator at 1.
- `1)` and `)` -> unbalanced at 1 and 0 respectively; `( ` -> missing operand at 2.
- Tab, NUL, and non-ASCII octets -> unexpected byte at the first such byte's offset.

Error names in the executable workload are `MissingOperand`, `MissingOperator`, `UnexpectedByte`,
`Unbalanced`, `IntegerOverflow`, and `ArithmeticOverflow`, each carrying an I64 offset. Its result is
`Ok(I64)` or `Failed(Error)`, both ordinary user-declared alternatives. Integer overflow identifies the
first digit that cannot be accumulated; arithmetic overflow identifies the stored operator being applied.
Operators are reduced at the next operator of equal/lower precedence, a closing parenthesis, or end of
input. Scanning stops logically at the first detected failure; no later byte replaces it. Closing an
unmatched parenthesis reports unbalanced; closing a matched empty/incomplete group reports missing operand.
End of input reports missing operand before looking for remaining unmatched opening parentheses.
Resource exhaustion remains a diagnostic rather than a successful or typed parser result.

## B: validate, filter, and aggregate a collection

For a dynamic sequence of I64 values, validate every value is between 0 and 1,000 inclusive; return the
first invalid value/index as a typed failure. Otherwise preserve the order of even values and return the
filtered sequence, its length, and sum. Do not stop validating when a value is filtered out.

- `[5,2,8,3,0,10]` -> `[2,8,0,10]`, count 4, sum 20.
- `[]` -> `[]`, count 0, sum 0; `[1,3,5]` -> empty; `[1000,1000]` -> both retained, sum 2000.
- `[2,-1,4]` -> invalid index 1/value -1; `[1001,-1]` -> invalid index 0/value 1001.
- Variation: an odd-number predicate and corresponding expected results without compiler changes.

## C: construct and traverse a search tree

Build an integer binary search tree from a dynamic insertion-order sequence. Ignore duplicate keys, accept
negative keys, and perform a data-dependent in-order traversal. Return ordered keys, unique node count, and
height (empty 0, leaf 1). Representation may be recursive values or a growable node table with indices;
neither the harness nor a compiler special case may build/traverse it on the program's behalf.

- `[5,3,7,2,4,6,8]` -> `[2,3,4,5,6,7,8]`, count 7, height 3.
- `[3,1,3,2]` -> `[1,2,3]`, count 3, height 3; `[]` -> empty/count 0/height 0.
- `[-2,-3,-1]` -> `[-3,-2,-1]`, count 3, height 2; `[1,2,3,4]` -> same keys, height 4.
- Variation: reverse-order traversal without compiler changes.

Additional tree cases frozen before its implementation:

- A single zero or repeated zeroes -> `[0]`, count 1, height 1.
- `[0,-9223372036854775808,9223372036854775807]` -> the three keys in ascending order,
  count 3, height 2. Key comparison must not subtract keys or reserve a key as a null sentinel.
- The level-order input `[8,4,12,2,6,10,14,1,3,5,7,9,11,13,15]` -> keys 1 through 15,
  count 15, height 4. Repeating keys must preserve this shape and height.
- Forty strictly ascending keys -> the same keys, count 40, height 40; descending insertion -> the same
  ordered result and height. This must not depend on one language call frame per tree level.
- Reverse traversal has the same count/height as forward traversal, with exactly reversed output keys.

All three must report resource exhaustion as a structured diagnostic, not return partial successful output.
Programs must define their own computation with reusable Lobo functions, not a host intrinsic tailored to a
fixture. Basic text/bytes are supported in [Core v0.7](core-v0.7.md); program organization now uses explicit
closed modules in [Core v0.9](core-v0.9.md). Collection/tree workloads share `examples/sequences.lobo`;
CLI invocations must supply it explicitly with `--source`. Public entries and frozen outputs remain unchanged.
The full language milestone needs a coverage audit rather than assuming these examples establish universal replacement.
No Turing-completeness claim follows from these finite examples.
