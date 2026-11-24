---
name: lobo-development
description: Implement, repair and test Lobo application code using task-relevant language rules, TDD and native compiler feedback. Use for .lobo features and fixes; compiler implementation and research experiments have separate workflows.
---

# Develop with Lobo

Target compiler 1.11.2 / Core 1.11. Read project instructions and the requested behavior first.
This is development mode: use editing and compiler/test tools within the authorized task scope.
For a comprehension-only request, keep the work read-only. This skill grants no additional permissions.

Resolve these links from this file's physical directory, including when installed through a symlink.
They refer to the accompanying Lobo checkout; application files belong to the user's project.
Use its supplied compiler. In the Lobo checkout, an existing `target/release/lobo` is usable when
`lobo` is absent from PATH. Check compiler compatibility before applying these versioned rules.

## Retrieve the needed rules

Inspect the affected declarations, callers, source bundle and existing tests. Load relevant rows below;
leave unrelated topics unloaded. The cards supply selected rules, not the full language. Confirm omitted
API signatures in library source or the matching [specifications](../../../spec/README.md).

| Task | Read when needed |
| --- | --- |
| Basic syntax, immutable state, iteration | [Language details](../lobo-programming/references/language.md) |
| Bytes, Text, ranges, encoding | [Buffer card](../../../docs/agent-reference/compact-pilot-2026-11-24/ranges.txt) |
| Records, variants, updates, generic inference | [Nominal card](../../../docs/agent-reference/compact-pilot-2026-11-24/nominal.txt) |
| Result propagation and contextual try | [Propagation card](../../../docs/agent-reference/compact-pilot-2026-11-24/propagation.txt) |
| Preconditions, postconditions, static admission | [Contract card](../../../docs/agent-reference/compact-pilot-2026-11-24/contracts.txt) |
| Callback signatures and action bounds | [Core 1.11](../../../spec/core-v1.11.md) |
| CLI, typed inputs, metadata, test-session protocol | [Tool details](../lobo-programming/references/tools.md) |
| JSON / HTTP / OS / SQLite APIs | Relevant [JSON](../../../libraries/json/README.md), [HTTP](../../../libraries/http/README.md), [native](../../../libraries/native/README.md) or [SQL](../../../libraries/sql/README.md) guide and source |

Blocks return their final expression; values are immutable. Use carried-state fold/while rather than
inventing assignment or return statements. Declarations have explicit boundary types. Actions declare
possible operations; runtime grants provide authority. Prefix try propagates errors without cleanup.
Preserve captured outcomes and explicit resource teardown. A successful check does not prove contracts
true or establish runtime correctness.

## Implement and verify

Derive a behavioral test from the requirement, run it to expose the missing behavior, then make the
smallest change that passes. Keep expected outcomes independent of candidate output. Preserve frozen
criteria, stable IDs and source/sidecar pairs. Use `identities init` for new declarations' files,
`identities sync` for additions and `identities rename` for renames that must retain IDs.

Use the project's test client and its acceptance contract when supplied. Otherwise check the complete
explicit source bundle, compile to a fresh output path and execute the binary against known answers.
Use one `--source` per dependency; there is no package discovery. For example, with the project's
compiler available as `lobo`:

```sh
lobo check program.lobo --source helpers.lobo
lobo compile program.lobo --source helpers.lobo -o fresh-output
./fresh-output --entry app.entry --arg I64:7
```

Replace the example files/entry with the actual project values. Read both exit status and typed JSON.
Compile refuses existing output paths. OS/SQLite programs require explicit launch grants; consult the
relevant guide. Eval is a reference comparison tool, not native execution, and cannot perform OS/DB IO.

For a rejected construct, inspect its diagnostic and relevant rule before trying another spelling.
Use a small isolated compiler probe when needed. For a behavior failure, retain the input and observation
and repair the responsible code. Do not weaken a requirement or regenerate expected answers to pass.
Run affected regressions and the project's required final gates. Preserve failures and missing checks;
report changed behavior, commands, observed results and remaining uncertainty.

The cards improved selected source-only comprehension cases. This development workflow has no measured
agent productivity or token advantage yet; see [the evidence](../../../research/compact-pilot-2026-11-24.md).
