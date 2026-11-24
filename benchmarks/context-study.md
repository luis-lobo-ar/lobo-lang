# Source-only context experiment

This versioned experiment implements decision [0066](../decisions/0066-isolate-comprehension-context-treatments.md).
The original matched-study drivers, guides, questions, outcomes and product 1.11.2 remain frozen.

The new builder in [context.py](matched-study/context.py) accepts an explicit arm:

| Arm | Task/guide copies | Common input documentation |
| --- | ---: | --- |
| original | 2 | absent |
| dedup | 1 | absent |
| schema | 2 | once |
| both | 1 | once |

`prompt(bundle(family, language), 'both', family)` constructs the combined treatment for a future caller.
Controls remain available for reproducibility. This does not change historical calls or silently replace
their prompts. Unrelated authoring/check instructions remain constant in every arm. A later reading-guide
cleanup would need its own experiment.

The [input documentation](results/2026-11-23-context-input-documentation.json) contains the exact strings
appended to schema/both prompts. Each family uses identical documentation across languages. It explains
the wire types, event mapping, job IDs and raw-byte encoding. The language guides are unchanged.

Each attempt starts an ephemeral, read-only, tool-disabled Codex CLI session. Trials are serial.
The 72 assignments use Luna/high then Luna/low. Each setting has nine matched language/family blocks,
with four arms per block. Four balanced arm orders keep positions within one observation of equal totals.
Each program form has six fresh predictions, a witness, direct calls, impact, a cued policy location and
a counterexample. Programs use a new ledger amount cap, highest-ID scheduler, seeded checksum and renamed
helpers. Lobo declaration identity values remain stable; metadata keys follow helper renames.

Fresh means newly authored for this pilot, before any of its participant calls. These questions reuse the
old family and item templates. They were authored after inspecting past failures, not by an independent
blinded author. They are not a confirmatory unseen sample. Their absolute scores cannot identify the
effect of changes relative to the older study. Treatment contrasts use matched questions inside this pilot.
The impact convention includes the changed predicate explicitly in every arm. Policy location is cued,
so passing it does not establish uncued fault localization.

The registration pins the driver, tests, prompts, compiler and validation before model calls. The driver
refuses drift. Old-prompt parity and isolated treatment tests precede reference compilation and mutation
checks. Every fresh reference and policy mutant runs in all three languages against an independent oracle.
Build failures do not count as killed mutants. Keep all red, intermediate and green development outcomes.

The wall limit is 120 seconds per attempt. Tools are disabled. There is no hard token cap. The runner
retains commands, raw events, stderr, last answers, usage, timing, load, inputs and grades. It never retries
or replaces a failed attempt. An availability check precedes each reasoning cohort and is excluded from
scores but included in participant cost accounting. Three consecutive infrastructure failures stop the
cohort; record remaining assignments separately.

The [reporter](matched-study/context_report.py) replays every grade and checks input/prompt hashes before
publishing source bundles. It reports every attempted assignment, including failures. Input plus output
tokens include cached input and reasoning output as subsets. Never add those subsets again. Missing
usage is unknown. Billing and parent-agent development usage are unmeasured. Wall time includes remote
service latency and cache effects, not just reasoning. Do not interpret it as runtime performance.

The new [language concept reference](../docs/agent-reference/1.11.2/README.md) is excluded from all trial
prompts. Its usefulness is a separate hypothesis. This pilot changes no language, library, editor or runtime.

To verify the new analysis without model calls:

```sh
python3 -m unittest discover -s benchmarks/matched-study -p 'test_context*.py'
python3 -m unittest discover -s research -p test_language_reference.py
python3 research/language_reference.py check
```

The second command requires the locally retained frozen compiler. The public registration and reference
validation preserve its hash and exact build commands. `context.py register` and `context.py run` are
explicit model-study operations, not part of these tests. A reproduction needs a new output directory and
registration, an available matching model alias, and the verified local compiler freeze. Aliases do not
identify immutable model weights. Do not rerun a published trial in place or change its registered code.
