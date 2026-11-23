# Typed JSON helper verification

Recorded 2026-11-14 for the user's request to build a JSON helper after the MVC example.
[Decision 0039](../../decisions/0039-typed-json-source-helpers.md) records the scope and chosen policies.
[The library API](../../libraries/json/README.md) documents source inputs and executable usage.
[The receipt](2026-11-14-json-helpers.json) pins commands, inputs, outputs and preservation.

## Implemented behavior

The pure source module provides Json null/Boolean/I64/decimal-text/Text/array/object values. `object`, `field`,
`array` and `append` build immutable documents in insertion order. `stringify` produces compact JSON.
The serializer and field builder reject duplicate object names. Decimal text follows the RFC 8259 number
grammar and preserves lexical precision. I64 formatting covers both bounds. Keys and string values reuse
the existing HTTP quoting function. `json_response` serializes a typed value using the existing response policy.

The MVC view now returns Json instead of Text. The controller passes this value to `json_response`. This
changes the view's example signature while preserving the demonstrated controller's 200/404 body/status
behavior. The original MVC source is retained as `mvc-before.lobo` in the private evidence directory.
Explicit workspace sources include the library and MVC example. A navigation fixture checks builder calls,
the Json Object constructor and the response adapter through supplied module identities.

## Checks and observations

The initial compile-valid scaffold failed three behavior tests. Its serializer returned an empty string
instead of null, and its number validator rejected zero. `target/json-helpers-red.log` retains these failures.
The first complete implementation passed all three tests. A fourth test covers the migrated MVC source.
The focused final run additionally uses the sample host's 10,000 fuel and heap limits for MVC controller calls.

A final integrity assertion initially read the documentation log while its command was still running.
After waiting for command completion, the documentation check and the repeated integrity verification passed.
This was a verification sequencing failure. The receipt retains it separately from the scaffold's behavior failures.

All four focused tests run after checked-artifact reload through evaluator, C, LLVM, C ASan/UBSan and LLVM
ASan/UBSan using the existing Builds fixture. Expected values are independent Rust values and serde_json
decoding. Tests cover strings/keys with controls, quotes, backslashes, Unicode and injection-shaped text,
null/Boolean/empty containers, integer boundaries, array/object order, persistent object bases, number grammar,
duplicate names from both builders and direct construction, response policy and resource errors.
Precision/range-extending decimal tokens are compared as exact text because a receiver may impose numeric limits.

Nested arrays at depths 0, 1, 40 and 1,000 serialize to independently constructed expected strings. This checks
the explicit work-list traversal under the stated one-million fuel/heap test budgets. It supplies a bounded
conformance observation. Object duplicate validation can perform quadratic key comparisons, and resource/output
limits can stop larger inputs. Execution speed, comparative allocation efficiency and agent outcomes remain
unmeasured.

Full verification passed 200 Rust tests, 131 editor tests and 31 Python tests. Clippy with warnings denied,
format checks, skill validation, explicit-workspace bundle checking and the actual VS Code host passed.
The compiler and extension runtime/grammar implementations retain their existing 1.1 behavior. Source-library
and navigation/configuration maintenance use those implemented constructs.

Ten CLI commands build/reload the checked MVC bundle, compile C and LLVM ELF programs, invoke found/missing
users at 10,000/10,000 budgets and inspect the typed view. The C binary is 72,136 bytes and the LLVM binary is
68,040 bytes for this complete entry bundle. The receipt retains output hashes and exact results.
All 86 frozen language 1.0 release artifact hashes were reverified.

## Reproduction and retained evidence

Run `cargo test --locked --test json_helpers` for focused behavior. Run `cargo test --locked`,
`npm --prefix editors/vscode test`, and `python3 -m unittest discover -s benchmarks -p 'test_*.py'` for the
recorded broader suites. The compiler/tool binaries required by those checks are existing local artifacts.
The receipt lists the actual commands and final logs for lint, formatting, skill and editor-host validation.
Documentation checks use `./scripts/check.sh`.

The private directory `benchmarks/private-evidence/json-helpers-20261114` retains the source snapshot, test
scaffold failure reproducers, logs, native/checked MVC programs, source/compiler identities and original MVC
source. The initial failure did not require relaxing an oracle or resource grant. The later focused MVC
check tightened its grant to the existing host limits. This is local engineering evidence for a future paper.
Software-construction quality, comprehension and token effects need separate controlled measurements.

The helper supports construction and serialization. External JSON parsing and F64-to-decimal conversion
remain future work. Numeric grammar and duplicate-key policy are based on the cited standard and an explicit
library decision. [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.html)
