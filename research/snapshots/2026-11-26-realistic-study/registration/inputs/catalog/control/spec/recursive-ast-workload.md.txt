# Recursive AST development workload

Frozen behavior before implementation, **2026-11-04**. Public development evidence, not a confirmatory
agent task. See [decision 0029](../decisions/0029-list-guarded-recursive-values.md).

The AST has integer literals, n-ary sums and n-ary products. Children are ordered typed Node values,
not external integer references. Empty sum evaluates to zero; empty product evaluates to one.
Evaluation uses checked I64 arithmetic. Shifting replaces every literal `n` with `n + delta`, preserving
operation kind, child order and empty nodes. Ordinary runtime overflow and fuel/memory limits apply.

Frozen cases:

| Input AST | Value | Shift by one, then evaluate |
| --- | ---: | ---: |
| Num(7) | 7 | 8 |
| Sum([]) | 0 | 0 |
| Product([]) | 1 | 1 |
| Sum([Num(2), Product([Num(3), Num(4)]), Sum([])]) | 14 | 23 |
| Product([Num(-2), Sum([Num(1), Num(4)])]) | -10 | -7 |

Acceptance must execute evaluation and transformation entirely in native Lobo and the reference
evaluator, with reloaded checked artifacts and ASan/UBSan/leak checks. Cross-module consumers share
the same nominal Node identity. Tests include changed literals, empty branches, negative values,
overflow, malformed recursive input, excessive output depth and exhausted budgets. A Product case
provides a schema-extension pressure beyond the original Num/Sum shape; missing cases remain errors.

The host supplies test input and independently computes expected results. It must not evaluate or
transform the Lobo AST on the program's behalf. No new compiler intrinsic may special-case this workload.
