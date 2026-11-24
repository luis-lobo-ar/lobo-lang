# Agent reading reference for Lobo 1.11.2

This additive reference covers selected rules in Core 1.11. The compiler, extension, libraries and
programming skill remain frozen. The [normative specification](../../../spec/core-v1.11.md) and its
inherited specifications define the language. This reference is a reading index, not a complete manual.

The [concept data](concepts.json) separates language rules, libraries, runtime limits, authoring tools and
experiment input encodings. Each concept states its meaning, common mistakes and source references with
hashes. Relationships describe dependencies and distinctions. They do not add compiler behavior or a
semantic workspace service. The [editable concept map](language-relationships.drawio) uses the same data.

For bounded context retrieval, run these commands from the repository root:

```sh
python3 research/language_reference.py check
python3 research/language_reference.py lookup try --neighbors 3
python3 research/language_reference.py lookup bytes --neighbors 2
python3 research/language_reference.py draw
```

Lookup returns the selected concept and up to eight immediate neighbors. It reports omitted neighbors.
It does not recursively include the whole graph. `check` rejects source drift or invalid relationships.
Review a changed rule against the current specification before creating a new reference version.
Do not refresh hashes merely to silence a failed check.

An agent needs the following information for a concrete task:

- The exact language/compiler version and supported features.
- The relevant syntax and evaluation/type rules, including boundaries and faults.
- The source dependencies, stable identities and complete called signatures.
- The input/output schema, separate from source-language types.
- Runnable examples, precise diagnostics and a way to verify claims.
- The task's authority, resource limits and acceptance criteria.

The graph helps locate those facts. An edge alone cannot explain byte counts, constructor inference or
resource cleanup. A precise rule and a checked example provide that detail. Whether this reference
improves LLM comprehension remains unmeasured. It is excluded from all decision-0066 trial prompts.

The [reading examples](reading-examples.lobo) demonstrate language rules without study answers. Their
adjacent identities are tracked. Build with the frozen compiler or the matching `lobo` installation:

```sh
lobo compile docs/agent-reference/1.11.2/reading-examples.lobo -o /tmp/lobo-reading-examples
/tmp/lobo-reading-examples --entry agentref.slice_count
```

The `slice_count` entry returns `I64:3`. The example's slice starts at byte 1 and takes three bytes.
The trial input-schema treatment is recorded in the versioned research driver and registration, not
injected through this reference or the existing programming skill.
