# 0069 — Focused Lobo development skill

Date: 2026-11-24. Status: implemented and locally checked.

The user requested a Lobo development skill after reviewing the compact-reference results. Add a separate
`lobo-development` entrypoint rather than modify the existing programming skill or frozen study materials.
The new skill targets compiler 1.11.2 / Core 1.11, routes to task-relevant existing cards/specifications,
and distinguishes development from source-only comprehension. It preserves authority boundaries,
independent behavioral tests, persistent IDs, explicit source bundles, actions and resource cleanup.

Keep the entrypoint smaller than the older programming skill and load references as needed. Reuse the
checked cards without copying their rules into another maintained packet. Links resolve from the physical
skill directory. Store the skill in the repository and install a local symlink under the user's skills
root; it depends on this accompanying checkout. A standalone distributable package is not part of this task.
Keep automatic invocation enabled by default; also support explicit `$lobo-development` invocation.

Validate skill structure, every reference, UI metadata and the installed target. Exercise the documented
native workflow on an isolated two-source program: create identity sidecars, detect an intentional wrong
result, fix the implementation, check known outputs and preserve IDs. Preserve commands, sources and
observations. Run relevant research/harness and repository checks. For this skill-only addition, reuse
the identical compiler's prior 312-pass receipt; do not claim a new full compiler/editor run.

The earlier card study measures selected comprehension outcomes, not this skill or development efficiency.
These deterministic checks establish that referenced materials and native operations work locally. They
are not an agent compliance, implicit-discovery, token or construction experiment. No further model calls,
product changes, old-fixture edits or statistical gain claim is authorized by this packaging task.

See [the verification receipt](../benchmarks/results/2026-11-24-development-skill.json).
