# Lobo

Lobo is an experimental programming language and development environment designed for software agents.
Its primary abstraction is not a source file. It is a versioned semantic workspace containing program
structure, contracts, effects, evidence, history, and explicit uncertainty.

Lobo explores a specific hypothesis:

> Agents will produce more reliable software when they can query and change a program through typed,
> transactional operations instead of reconstructing intent from files and terminal output.

This repository contains a Rust native compiler for the proposed Core v0.1 subset. It checks source and emits
optimized machine-code executables with runtime contracts. A reference evaluator supports correctness testing.
The language is experimental; the wider workspace, WASM backend, and self-hosting are future work.

## Try the first version

With Rust, Cargo, and Clang installed, run from this directory:

```sh
cargo run -- check examples/core.lobo
cargo run -- compile examples/core.lobo -o core
./core --entry example.increment --arg I64:41
./core --entry example.absolute --arg I64:-7
```

The increment invocation returns `42`; the absolute-value invocation returns `7`. The executable runs directly
without Lobo or Clang. It emits JSON on stdout and exits nonzero on errors. `compile` requires a new output path
and refuses to overwrite existing files. Arguments use explicit types and functions are invoked by stable ID.
Use `--fuel` to set the native execution budget. `lobo run` compiles to a temporary binary and executes it;
`lobo eval` explicitly selects the reference evaluator. See the [native backend specification](spec/native-v0.1.md).

Verify with `cargo test --locked` and `./scripts/check.sh`. The transfer example describes later language
features and is not accepted by the Core compiler.

## Optimized release

```sh
cargo build --locked --release --bin lobo
./target/release/lobo run examples/core.lobo --entry example.increment --arg I64:41
stat -c '%n %s bytes' target/release/lobo
```

The release profile optimizes for small executable size, enables full link-time optimization, uses one codegen
unit, strips symbols, and omits debug information. Internal Rust panics abort the process; ordinary Lobo errors
still return structured diagnostics. Runtime contracts and checked arithmetic remain enabled. The native
executable targets the build host; the current Linux build depends on the system's C runtime.

## Design principles

1. **Semantics before syntax.** The canonical program is a typed graph; text is one projection.
2. **Intent is executable.** Contracts, examples, policies, and resource budgets are program artifacts.
3. **Effects are explicit.** Network, filesystem, time, randomness, process execution, and mutation require
   declared capabilities.
4. **Every mutation is transactional.** A change names its base revision and either commits with evidence or
   leaves the workspace unchanged.
5. **Context is queried.** The environment returns bounded, attributable context packages instead of asking an
   agent to read an entire repository.
6. **Evidence travels with change.** Verification results are tied to exact revisions, environments, and
   assumptions.
7. **Humans retain oversight.** Machine-native storage must have stable, readable, auditable projections.
8. **The host enforces authority.** A model may request a tool call; the host validates and executes it.

## The system

```text
agents, editors, CI, human review
                 │
          Lobo Tool Protocol
                 │
               lobod
      ┌──────────┼───────────┐
 semantic DB   verifier   sandboxed runner
      └──────────┼───────────┘
       content-addressed store
                 │
       native / optional WASM backends
```

`lobod` is the proposed workspace daemon. It owns revisions, semantic queries, transactions, validation,
execution, and evidence. All clients use the same protocol; the CLI is an adapter, not a privileged interface.

## Repository map

- [Vision](VISION.md) — the problem, thesis, boundaries, and measures of success.
- [Architecture](docs/architecture.md) — components, artifacts, trust boundaries, and lifecycle.
- [Language design](docs/language-design.md) — the initial semantic model and illustrative syntax.
- [Core v0.1 specification](spec/core-v0.1.md) — proposed grammar, types, evaluation, contracts, and diagnostics.
- [Checked artifact format](spec/checked-artifact-v0.1.md) — initial compiler output and stable call references.
- [Native backend](spec/native-v0.1.md) — machine-code generation, executable CLI, semantics, and limits.
- [Initial native measurements](docs/native-performance.md) — executable sizes and scoped Rust comparison.
- [Repeatable comparisons](benchmarks/README.md) — native, interpreted, and Rust measurements.
- [Worktree workflow](docs/worktree-comparisons.md) — baseline/candidate checkouts and source attribution.
- [Compiler and backends](docs/compiler-and-backends.md) — Rust implementation, checked IR, WASM, and LLVM.
- [Compiler development](docs/compiler-development-workflow.md) — specification and implementation loop.
- [Testing and TDD](docs/testing-and-tdd.md) — conformance, differential testing, fuzzing, and release evidence.
- [Self-hosting](docs/self-hosting.md) — prerequisites and reproducible bootstrap stages.
- [Development workflow](docs/development-workflow.md) — the agent change loop and multi-agent coordination.
- [Tool protocol](docs/tool-protocol.md) — discovery, requests, responses, safety, and transport.
- [State and context](docs/state-and-context.md) — explicit state categories and context packages.
- [Roadmap](docs/roadmap.md) — staged implementation with exit criteria.
- [v1.0 target](docs/v1.0-target.md) — the agent workspace outcomes the minimal Core version serves.
- [Transfer example](examples/transfer.lobo) — a non-normative sketch of Lobo source.
- [Agent session](examples/agent-session.jsonl) — an illustrative protocol exchange.
- [Protocol schemas](spec/schemas) — early machine-readable envelopes.
- [Architecture decisions](decisions) — durable design choices and their consequences.

## Proposed vocabulary

| Term | Meaning |
| --- | --- |
| Program graph | Canonical typed representation of a Lobo program |
| Projection | A deterministic view of graph content, such as readable source |
| Revision | Immutable content-addressed workspace state |
| Transaction | Isolated sequence of operations against one base revision |
| Obligation | A property that must be decided, tested, or proven |
| Evidence | A revision-bound result supporting or refuting a claim |
| Context package | Bounded, attributable information assembled for a goal |
| Capability | Explicit authority to observe or affect a resource |

## Near-term goal

After the Core prototype, the semantic workspace should demonstrate one end-to-end change:

1. Load a small canonical program graph.
2. Ask for goal-specific context.
3. Begin a transaction at an explicit revision.
4. Add a semantic operation without rewriting a source file.
5. Return affected obligations.
6. Verify the changed closure.
7. Commit a new revision with an evidence manifest.
8. Render a deterministic human-readable projection.

See the [roadmap](docs/roadmap.md) for the acceptance criteria and deliberate non-goals.

## Naming

The project directory is `lobo-lang`; the language, CLI, and source extension are `Lobo`, `lobo`, and `.lobo`.
This is explicit enough for discovery while keeping the eventual command short. The daemon name `lobod` follows
the usual service convention.

