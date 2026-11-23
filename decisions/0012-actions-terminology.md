# 0012: Call resource interactions actions

- Status: accepted terminology; runtime and language support remain proposed
- Date: 2026-10-17

Follow-up: [decision 0013](0013-checked-action-bounds.md) supersedes the unsupported-clause/version boundary
below with static Core v0.3 action checking. The terminology, historical evidence, and protocol migration
decision here remain unchanged; this record describes the earlier terminology-only change.

## Context and alternatives

The user prefers a concrete term over "effects". Use **actions** for declared possible resource interactions,
including reads, writes, time, randomness, and operations performed through callees. "Permissions" confuses
behavior with authority; "operations" is already used for semantic workspace mutations. This decision replaces
the earlier terminology in current documents without rewriting the rationale of older decision records.

## Decision and boundaries

- Proposed source uses `actions { ... }`; typed-hole sketches use `allowed_actions`.
- Actions describe possible behavior; capabilities supply authority; host-enforced limits constrain use.
- In the proposed model, no declared actions means pure, including transitive calls. Resource handles are
  explicit parameters. This is not a normative implementation of inference, ownership, quotas, or isolation.
- Core remains `core/0.2-draft`, compiler 0.2.1, checked artifacts `lobo-checked/0.2`, and extension 0.3.0.
  `actions` remains an ordinary identifier. No compiler keyword or new execution facility is introduced.
- Tool descriptors require `actions`, not `effects`; verification examples request the `actions` check.
  Following the schema versioning rule, proposed LTP advances from 0.1 to 0.2 and schema IDs change together.
  Old descriptors/clients must migrate explicitly; there is no legacy alias. The daemon is not implemented.
  Tool-specific API versions remain independent and unchanged.
- Maintain the root [change ledger](../CHANGELOG.md) alongside durable decisions. Preserve historical entries;
  later decisions explicitly supersede earlier ones. Plans are not recorded as shipped features.

## Verification and limits

[Protocol regressions](../tests/protocol_actions.rs) check required/allowed descriptor fields, schema versions,
JSONL and documentation examples, and the unchanged Core identifier/rejection boundary. These focused schema
structure assertions are not a general JSON Schema validator or evidence of host capability enforcement.

TDD first observed two failures for the old descriptor field and schema version, with the Core compatibility
case already passing. A subsequent harness failure exposed an existing blank JSONL line; matching the existing
JSONL checker's blank-line handling corrected the test without weakening its field/version assertions.

On 2026-10-17, `cargo test --locked` passed all 34 integration tests, including the three new protocol/Core
regressions and the native/reference/sanitizer suites. Rust formatting, Clippy with warnings denied, and the
JSON/JSONL/local-link and diff whitespace checks passed. All 46 extension tests passed, including explicit
checks that `actions` retains ordinary identifier/call coloring and definition navigation. A fresh-reader
review correctly distinguished actions, capabilities, limits, proposals, version migration, and historical
entries; its missing-verification-entry finding was resolved by this evidence record. No release build,
installation, commit, publication, or benchmark-worktree edit was made.

The extension's advertised support remains Core-only; no grammar/runtime change or VSIX rebuild is required
for proposed syntax. Earlier transfer/component sketches remain non-normative and still need explicit resource
binding, transaction snapshot, failure, deadline, and idempotency definitions before implementation.
