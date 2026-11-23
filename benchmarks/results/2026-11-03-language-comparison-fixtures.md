# First language-comparison calibration

Date: **2026-11-03**. These are deterministic compiler/behavior observations, not coding-agent results.
The [data](2026-11-03-language-comparison-fixtures.json) retains commands, source/tool hashes,
all 750 input/output observations and fifteen mutation builds with diagnostics.

| Arm | Baseline cases passed | Missing case | Invalid constructor/tag | Wrong inclusive bound |
| --- | --- | --- | --- | --- |
| Native Lobo variants | 150/150 | Rejected before execution | Rejected before execution | Compiles; wrong result detected |
| Native Lobo integer tags | 150/150 | Compiles; wrong result detected | Compiles; wrong result detected | Compiles; wrong result detected |
| Native Rust enums | 150/150 | Rejected before execution | Rejected before execution | Compiles; wrong result detected |
| Native Rust integer tags | 150/150 | Compiles; wrong result detected | Compiles; wrong result detected | Compiles; wrong result detected |
| Python integer tags | 150/150 | Byte-compiles; wrong result detected | Byte-compiles; wrong result detected | Byte-compiles; wrong result detected |

Lobo and Rust have comparable observed protection on these particular variant probes. Both still
accept the wrong boundary rule. No overall quality, comprehension, token or speed ranking follows.
There is no Python variant arm; external static checking or stronger Python runtime models were
not tested. Native Lobo is compiled, not interpreted. Rust uses optimized compilation; this run
does not compare compilation or execution time.

Commands:

```sh
python3 -m unittest discover -s benchmarks -p test_language_comparison.py
python3 benchmarks/language-comparison.py --output benchmarks/results/2026-11-03-language-comparison-fixtures.json
```

Six tests pass. Initial TDD evidence was a missing new module; a subsequent probe test failed on
a non-unique Python mutation target. The mutation now uses unique branch context. No expected
behavior was weakened. The five baseline programs are checked against a separate transition-table
oracle. Three mutations per arm provide wrong-behavior canaries and static-check observations.

The comprehension grader's eleven reference items pass, and negative/partial answers fail the
relevant checks. This validates the calibration machinery, not model comprehension. No additional
agent was launched. Human comprehension was not measured. A separate explicit request to launch
additional agents is pending under the session's agent-delegation rule.

Broader verification passed all 30 Python tests and 145 Rust tests, including native/sanitizer
checks. A focused follow-up strengthens alternative-counterexample and dependency-order coverage.
Source/result hashes, bibliography identities, JSON and local links pass;
[verification record](../../research/verification-comprehension.json).

Limits: one tiny state-workflow family, single-file sources, bounded input domain, three handpicked
mutations and public calibration answers. Rust's guarded `step` catch-all differs from Lobo's explicit
matching; align that policy before a controlled state-extension study. Temporary build paths in
the data no longer exist; fixture sources and the runner recreate them. Results are not overwritten.
OCaml, Dafny, Lean and TypeScript compiler commands were unavailable; no toolchain was installed.

For H1, these results establish selected compiler checks, not improved agent construction. H2
abstraction benefits and H3 context benefits remain unmeasured between languages. Future trials
must include both construction and [code comprehension](../../research/code-comprehension.md).
