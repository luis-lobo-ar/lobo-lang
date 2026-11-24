# Develop Lobo with the focused skill

Use [lobo-development](../.agents/skills/lobo-development/SKILL.md) for application features and repairs:

```text
$lobo-development Implement [requirement] in [project], using TDD and native verification.
```

It retrieves only relevant checked language rules, preserves IDs and cleanup, and uses compiler diagnostics
and behavioral tests to guide changes. The existing programming skill below remains the broader reference.
The new skill's links and native red/green workflow are checked; improved agent development performance
has not yet been measured. See [the verification receipt](../benchmarks/results/2026-11-24-development-skill.json).

The skill is maintained in `.agents/skills/lobo-development`. The local installation at
`/home/luis/.codex/skills/lobo-development` links to that directory. Keep the accompanying Lobo checkout
available: references resolve relative to the skill's physical location. The skill uses the application's
supplied compiler; this checkout's existing `target/release/lobo` is available when `lobo` is not on PATH.
The filesystem installation is checked; separate UI-discovery and agent-invocation experiments were not run.

The existing reading guide remains a comprehension-only artifact. The development skill instead authorizes
the use of edits/tools within the task's already approved scope and retrieves individual relevant cards.
No source-only experiment instruction is pasted into its development workflow.

## Subsequent comparison, 2026-11-24

The creation-time checks above did not measure development benefit. The later
[matched pilot](../research/development-skill-study-2026-11-24.md) now records 12 fresh attempts:
control passes 6/6 tasks, skill 5/6 with one timeout. Five known token pairs use a median 41.2% more
with the skill; the timeout's usage is unknown. No development advantage is demonstrated. The tested
skill text remains unchanged; proposed revisions require a new selected scope and comparison.
