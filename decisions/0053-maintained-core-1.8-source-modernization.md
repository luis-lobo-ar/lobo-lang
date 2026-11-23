# 0053 — Modernize maintained source within Core 1.8

Date: 2026-11-22. Status: implemented; verification results are linked below.

The user requested replacing maintained code with the newly implemented constructs. This is a source
migration, not authorization for more grammar, broader inference or a different runtime. Compiler and
extension 1.8.1 retain language core/1.8, checked format 1.0 and native ABI 1.

## Selected conventions

Use inferred generic arguments and constructor cases where existing context identifies them. Use named
callbacks when a complete expected Fn signature is available. Use Text + Text with the original evaluation
association, concise expression match arms, and JSON literals without unused object/field selections.
Use prefix try for exact error forwarding. Preserve matches that translate errors, recover, retry or protect
explicit cleanup. Inline identity tables and qualified/explicit spellings remain supported.

The REST entry uses try to restrict authority and open the listener, then handles the serve result before
closing the listener. A close error retains its previous precedence. Loopback setup uses try with the same
I64 error type across distinct result owners. Roundtrip still reads into a result, closes both streams and
the listener, then returns that result. Try does not introduce function-scope automatic cleanup.

Do not remove explicit arguments from underconstrained calls such as apply::<Text,Text>(identity,value).
Do not remove JsonTask constructor qualifiers when the List builtin supplies no expected owner. Native
library wrappers retain I64-to-IoError matches. These are current inference/type limits, not missing migration.

## Reviewable migration

`scripts/modernize-source.cjs` proposes lexical rewrites and compiles each in an isolated explicit bundle
before acceptance. It compares the complete checked program; only scope import alias names and duplicate
imports of the same stable ID are normalized. Bodies, types, contracts, actions, declaration IDs and the
resolved dependency-ID set must agree. Text concatenation association is preserved, including resource
accounting. Layout changes preserve the complete token sequence; comments and strings remain data.

Automatic rewrites are restricted to maintained examples, libraries and skill examples. Historical specs,
frozen benchmark/research fixtures, version-1 acceptance programs and the legacy MVC comparison remain
unchanged. The older 23-bundle/3-embedded migration checkpoint is verified, never recaptured. Removing two
unused MVC builder-import aliases removes their sidecar keys; no declaration or target identity changes.
Manual try changes are independently exercised with native success, failure and resource tests. Exact source
expression fuel counts are not claimed equal for these changes.

The historical compact projection test now owns frozen pre-migration inputs; current source uses the native
parser. Current MVC equivalence tests compare every checked field except import aliases/duplicates, with
the same resolved ID set. Managed metadata tests load the tracked sidecar instead of duplicating stale keys.
VS Code host tests navigate inferred source forms and require their searched use/target strings to exist.

## Stop and evidence

A repeat audit must find zero accepted rewrites. Native behavior, sanitizers, the editor host and the previous
migration checkpoint must pass. Preserve initial failures and rejected proposals. No runtime speedup,
agent token saving, construction-quality improvement or comprehension improvement follows from this refactor.
See [the report](../benchmarks/results/2026-11-22-modernize-source.md) and
[receipt](../benchmarks/results/2026-11-22-modernize-source.json). No language feature is added.
