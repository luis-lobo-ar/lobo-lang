# Language comparison calibration

These are small executable fixtures and a comprehension grader, not agent-performance results.
The five arms are native Lobo variants, native Lobo integer-tag records, native Rust enums,
native Rust integer-tag records and Python integer-tag tuples. Python's arm is one encoding,
not a claim that Python cannot provide stronger runtime validation or external static checking.

The common behavior is an amount-bounded state workflow. For amounts outside 0 through 100,
`run` returns -100 before examining the state or action. Valid transitions are Draft/0 to
Approved, Approved/1 to Paid, and Paid/2 to Draft. The output is the destination state number
times 1000 plus the amount. Unknown states and invalid transitions return -900.

## Run the calibration

From the repository root:

```sh
python3 -m unittest discover -s benchmarks -p test_language_comparison.py
python3 benchmarks/language-comparison.py --output /tmp/new-comparison-result.json
```

The script requires the existing release compiler at `target/release/lobo`, Clang, Rust and
Python. It refuses to overwrite a result. Lobo and Rust programs are compiled to machine code;
Python is byte-compiled and then executed. This is a correctness comparison, not a runtime-speed
benchmark. Process startup, compilation time and file size are not used to rank the languages.

Every arm executes the same 150 input triples against an independently structured transition-table
oracle. Three source mutations test missing-case handling, an invalid state constructor/tag and
an incorrect inclusive bound. Compilation rejection and executed wrong behavior are recorded
separately. Full diagnostics, input/output observations and source/tool hashes accompany the result.

The [first calibration result](../results/2026-11-03-language-comparison-fixtures.json) contains
750 agreeing baseline observations. It does not contain any model answers. Compiler rejection of
selected mutations demonstrates those checks on those fixtures, not a universal safety or quality
guarantee. Rust's `step` uses guards and a catch-all arm; Lobo's uses explicit alternatives.
Align missing-case policy before using these sources for a controlled requirement-evolution study.

## Comprehension questions

Give the participant its arm's source and ask for one JSON object with these fields:

1. `outputs`: predict `run` for `[0,7,0]`, `[1,7,1]`, `[2,7,2]`, `[0,100,0]`,
   `[9,101,0]` and `[2,7,0]`, in that order.
2. `witness`: provide any bounded integer triple producing 2007.
3. `direct_calls`: identify the application functions directly called by `run`, ignoring CLI/library
   wrappers. Order does not matter.
4. `impacted`: identify application functions whose return values can change for identical direct
   arguments when the upper amount bound changes from 100 to 99. Ignore reachability alone.
5. `fault_location`: name the function responsible if amount 100 is incorrectly rejected while
   amounts 0 through 99 retain the original behavior.
6. `counterexample`: give a triple with a known state/action that distinguishes the inclusive-bound
   program from its exclusive-bound mutant.

The grader has eleven scored items: six predictions and five other checks. Equivalent valid
witnesses are accepted; booleans are not integers. Partial scores and complete acceptance are
separate. Missing answers, wrong dependency sets and non-distinguishing counterexamples cannot pass.
No language model judges the answer. Its counterexamples are grounded in the behavioral oracle,
which the native/other fixtures are executed against during calibration.

These are public calibration tasks with public expected answers. Future confirmatory tasks must
use fresh held-out programs, names, parameters and independent oracles. Do not treat this small
single-file family as repository-level comprehension or general software-construction evidence.

For source-only comprehension, prohibit executing the target and record any violation separately.
For tool-assisted comprehension, allow the same capabilities in every arm and record executions,
retrieval and total cost. Do not mix those conditions. The current calibration ran no agents in
either condition. The [research protocol](../../research/code-comprehension.md) defines wider outcomes.
