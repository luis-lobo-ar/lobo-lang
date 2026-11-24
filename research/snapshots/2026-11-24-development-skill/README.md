# Lobo development skill verification

This snapshot preserves the skill, UI metadata, sources, identity sidecars, commands and observations
at creation. `verify.py --compiler COMPILER --skill SKILL.md --out NEW_DIRECTORY` repeats the isolated
native smoke check. The intentional fee defect produces208 rather than207; the fixed implementation
passes five known-answer cases. This deterministic check is not an LLM-agent experiment.

An initial research regression catches the root README hash changing. The usage instructions move to a
separate guide, the README bytes are restored, and all15 research tests then pass. The first failing run
remains in observations. Source and metadata are preserved; native binaries remain in the private location
recorded by the receipt. `SKILL.md.txt` and `usage-guide.md.txt` retain original relative links as plain text.

See [the user guide](../../../docs/lobo-development-skill.md),
[decision0069](../../../decisions/0069-focused-lobo-development-skill.md) and
[the receipt](../../../benchmarks/results/2026-11-24-development-skill.json).
