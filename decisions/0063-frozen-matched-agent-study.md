# 0063 — Freeze 1.11.2 for matched agent experiments

Date: 2026-11-23. Status: completed exploratory study, including the recorded infrastructure amendment.

The user requested a frozen-version comparison with Rust and Python for construction,
comprehension and requirement changes, followed by performance benchmarks. Freeze compiler/extension
1.11.2, Core 1.11 at `75a551fc0aec1fbc94f1fddb2a0f6a59ddb3f122`, annotated tag
`research/lobo-1.11.2`. Archive the source and compiler binaries; retain their hashes.
No language, native-provider, editor or skill behavior changes are selected.

Use the same configured model (`gpt-6.1-sol`, high reasoning) for all languages. Run 54 fresh,
isolated conversations: three task families, three languages, three phases, two repetitions.
Interleave languages with a deterministic balanced schedule. Each attempt has 120 seconds,
16 command-execution tool items and six visible check submissions. Actual tokens are outcomes;
the CLI does not provide a hard token cap. Cache hits remain part of input tokens.

Construction starts from deliberately wrong, compile-valid stubs. Change trials start from a
common correct base, independently of construction success. Comprehension includes the full
base source and uses no tools or execution. Its eleven objective items include prediction,
call relationships, semantic change impact, witnesses and counterexamples.

Use matched contracts, immutable DTOs/adapters, independent deterministic oracles and the same
visible/held-out observations across languages. Validate all eighteen canonical programs and
kill eighteen deliberate boundary defects before registration. Transport canaries are excluded.
Pin tasks, guides and harness before the first scheduled attempt. Preserve every scheduled
attempt and unknown usage; no failed-trial replacement. If three consecutive infrastructure
failures occur, stop and report remaining assignments as not attempted until an explicit,
versioned protocol amendment. Compile/type/test failures from candidate code are ordinary outcomes.

Prefer a working-budget pilot over a token-cap experiment: it measures software construction
under equal tool/time opportunity and records token cost separately. A token-cap study, multiple
models and larger representative programs remain future experiments. Familiarity, reference
idioms and unequal instruction length remain confounders. This is not a causal language ranking.

Benchmark canonical programs as a complete cohort; any accepted agent solutions are a separate,
success-conditioned cohort. Use optimized native builds, output validation, interleaved repeats,
full-process single-operation and 100-operation measurements. Preserve timing errors and limits.
Keep timing, agent cost and software correctness separate. See the protocol in
`benchmarks/matched-study/README.md`. Append all results to the research ledger and paper index.

## Infrastructure amendment 1

The original cohort was interrupted after five completed attempts and one partial attempt. Empty,
transient CLI-protected mountpoints caused false scope errors in the Lobo construction attempt's visible
checks. Its final candidate passed, but its feedback opportunity was impaired. Preserve the original
cohort, mark its remaining assignments not attempted, and use a separately registered 54-assignment
cohort for the matched comparison. Only the empty-mountpoint audit changes; no tasks, guides, oracles,
model settings, budgets or schedule changes. Excluded live transport canaries now exercise this audit.

## Completed observations

All 54 amended-cohort attempts pass the declared checks, with a comprehension ceiling. Median paired
Lobo construction input-plus-output token cost is 1.23 times Rust and 1.81 times Python. Runtime
measurements retain 270 successful builds, 162 correct warmups and 1134 correct timed process runs.
Nine hardware-counter denials limit native CPU attribution. Earlier evidence and the original interrupted
cohort remain separate. See research/matched-agent-study-2026-11-23.md and paper entry local-038.
No product change or automatic next language milestone is selected.
