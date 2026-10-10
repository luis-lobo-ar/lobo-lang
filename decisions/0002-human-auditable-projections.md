# 0002: Require human-auditable projections

- Status: accepted
- Date: 2026-10-10

## Context

An agent-native representation may be compact, binary, or optimized for machine traversal. Humans remain
accountable for requirements, authority, risk, incident response, and governance. An opaque canonical format
without dependable projections would make oversight ceremonial.

## Decision

Every canonical program revision must support deterministic human-readable projections. At minimum, the platform
must render source, semantic diff, public contracts, capability changes, unresolved obligations, and verification
evidence. Projections cite the revision and projection version used.

## Consequences

- Machine efficiency cannot remove auditability.
- Projection stability is tested independently from runtime behavior.
- Human review can focus on contracts, effects, and evidence instead of generated implementation details.
- A projection bug is a correctness issue because it can misrepresent approved behavior.

