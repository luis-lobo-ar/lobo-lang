# Language 1.0 public acceptance programs

These are development fixtures for the [frozen acceptance contract](../../spec/language-v1-acceptance.md).
They are excluded from claims about unseen tasks or agent construction/comprehension advantage.
Every baseline and planned change is separate source; shared helpers are explicit source modules.

| Program | Root / dependencies | Stable entry | Planned change |
| --- | --- | --- | --- |
| Collections | `collections.lobo`, `sequences.lobo` | `v1.collections.run` | `collections_changed.lobo` selects odd values |
| Documents | `document.lobo`, `document_types.lobo`, `document_validate.lobo`, `document_transform.lobo`, `sequences.lobo` | `v1.doc.run` | Replace the three document dependency files with their `_changed` counterparts; Item gains category validation/filtering |
| Numerical | `numerical.lobo`, `sequences.lobo` | `v1.numerical.run` | `numerical_changed.lobo` rejects negative inputs |

`sequences.lobo` implements generic iterative reverse/map/filter/find with Cursor, Option and Result.
The collection entry builds 10,000 values inside Lobo, filters/maps/reverses them and summarizes the actual
result. Baseline count/first/last/sum are 5000/19996/0/49990000; the changed values are 5000/19998/2/50000000.
Additional entries exercise I64, UTF-8 Text, caller-owned records and missing typed results across modules.

Document validation checks the whole input, including disabled items, before transforming it. The changed
schema reports BadCategory for an empty category. Selection preserves order and doubles scores with checked
I64 arithmetic. Overflow remains an arithmetic fault; typed input-validation failures use Result/DocError.

Numerical validation checks input finiteness first. Aggregation computes sum, arithmetic mean and population
variance in input order, using a second pass for squared differences. A nonfinite sum/intermediate variance
becomes ArithmeticNonFinite. The changed rule reports the first invalid negative value. Singleton signed
zero is valid; summation starts at positive zero. The fixed cases have exact binary fraction oracles, with no
tolerance fitted to the implementation.

Run the collection baseline using a built compiler:

```sh
target/release/lobo run examples/v1/collections.lobo --source examples/v1/sequences.lobo --entry v1.collections.run --fuel 2000000 --heap-cells 2000000
```

Run fractional statistics:

```sh
target/release/lobo run examples/v1/numerical.lobo --source examples/v1/sequences.lobo --entry v1.numerical.run --arg 'List:Count:3,F64:1.5,F64:2.25,F64:-0.75' --fuel 2000000 --heap-cells 2000000
```

Add `--backend llvm` for direct LLVM; C/Clang remains default. `compile ... -o NEW_PATH` produces a standalone
optimized machine-code executable, while `eval` selects the reference implementation. Native executions do
not require Lobo or Clang at runtime. Outputs use typed JSON; F64 values carry exact bits.

The executable assertions are [v1_acceptance.rs](../../tests/v1_acceptance.rs). Their test support checks C,
direct LLVM, both sanitized native routes and the evaluator after checked-artifact reload. Every large
acceptance invocation uses the frozen 2,000,000 fuel and heap ceilings. Passing establishes these scoped
language/workload outcomes; it does not establish universal correctness, production readiness or a speedup.
