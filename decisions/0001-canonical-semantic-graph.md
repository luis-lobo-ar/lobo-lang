# 0001: Use a semantic graph as the canonical program

- Status: accepted for prototyping
- Date: 2026-10-10

## Context

File-oriented source makes stable identity, semantic mutation, impact analysis, and multi-agent merging indirect.
Making text canonical would force the agent-facing workflow to recover structure after every edit.

## Decision

The canonical Lobo program is a versioned typed graph. Textual source is a deterministic projection that can also
be parsed as an interchange and human-authoring format. Semantic nodes use stable logical IDs; revisions and blobs
use content identity.

## Consequences

- Tool calls can target program entities independently of layout or renaming.
- Diffs and merges can describe behavior rather than lines.
- The projection/parser round trip becomes a critical compatibility contract.
- Graph schema evolution and migrations require explicit design.
- Human review tools must not depend on direct inspection of storage internals.

## Revisit when

The semantic workspace prototype shows that the graph adds substantial complexity without improving context retrieval, mutation
precision, merge behavior, or review outcomes relative to a conventional AST plus files.

