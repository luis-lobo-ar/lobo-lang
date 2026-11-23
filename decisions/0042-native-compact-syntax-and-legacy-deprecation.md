# 0042 Native syntax and legacy deprecation

Date 2026-11-14. Status implemented in language and compiler 1.2.0.

## Selected scope

The user requested adopting the compact MVC conventions as actual language syntax and asked whether old
syntax should be deprecated. The selected policy makes the new spelling the documented default and retains
the old spelling as deprecated read compatibility. There is no separate legacy command or execution mode.

Modern source starts with `module name;`. The Rust parser reads `fn`, grouped `use` imports, qualified
references, arrow match arms, Text-valued JSON objects and a trailing identities table directly.
No marker, Node.js tool or generated source is required by compiler commands.
Modern declarations retain the existing generic functions, generic data, callbacks, contracts and actions.

## Compiler architecture

Both source spellings produce the same existing program representation and use one resolver, checker,
specializer and native backend implementation. JSON object syntax constructs ordinary checked helper calls
and nominal values. Persistent identities come from explicit source metadata. Resource accounting and host
authority follow the existing semantics.

Native metadata parsing, qualified references and JSON construction live in `src/compact.rs`.
Diagnostics retain original token spans without generated-source mapping.
Checked format stays `lobo-checked/1.0`. Loaders revalidate core/1.0, core/1.1 and core/1.2 artifacts.
Frozen source compilers and research receipts remain historical inputs.

The editor submits original source to the compiler. Its read-only structural resolver still normalizes tokens
for navigation. That internal representation is independent of application compilation and does not grant
execution authority. Highlighting no longer requires the prototype marker. Extension version is 1.2.0.

## Deprecation policy

Old source beginning with `module name @id(...)` remains accepted automatically. Existing libraries and
regression fixtures can therefore participate in mixed-source bundles while maintained code migrates.
The old spelling is frozen and discouraged for new code. Current examples and skill guidance teach modern
source. Old and new declaration styles are not mixed within one source file.

Deprecation is documented. This release does not introduce compiler warning output or a removal deadline.
Removal requires an inventory of maintained callers, migration with stable identities, replacement coverage
for compatibility-dependent tests and a separate decision. Historical experiments retain their exact compilers.
The old spelling has no separate checker, runtime or backend to maintain.

## Boundaries and acceptance

The [1.2 specification](../spec/core-v1.2.md) defines source selection, metadata, groups and expression rules.
JSON shorthand remains Text-valued. Numeric construction, automatic IDs and rename refactoring are separate
work. Import groups precede declarations, member aliases are unique, and dependencies remain explicit.

Acceptance includes stock CLI checks with an empty PATH, exact checked MVC equality, native responses,
budgets, Unicode/CRLF source locations, generic/callback/block-arm coverage, old-artifact admission,
the full conformance suite, real editor-host checks and synchronized documentation and skill examples.
The [result record](../benchmarks/results/2026-11-14-native-syntax.md) retains outcomes and failures.
Agent construction, comprehension and token-efficiency outcomes remain unmeasured.
