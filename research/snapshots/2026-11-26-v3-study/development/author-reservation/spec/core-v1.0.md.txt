# Lobo language 1.0

Compiler version: `1.0.0`. Source language: `core/1.0`. Checked format: `lobo-checked/1.0`.
This freezes the user-selected bounded language core in decision 0033. Verification and local artifact status
are recorded in the [release receipt](../benchmarks/results/2026-11-04-language-v1-release.json).

## Normative rules

Inherit [Core 0.12](core-v0.12.md) and its explicitly referenced predecessor semantics. This document
supersedes their development/version labels and any conflicting admission limits. Core 0.12 supersedes
Core 0.11's instance-only generic checking. The stable core includes:

- I64 checked arithmetic, Bool, UTF-8 Text, Bytes and binary64 F64.
- Immutable records, tagged variants with exhaustive match, persistent Lists and List-guarded nominal recursion.
- Functions, recursion, immutable let, if, counted fold and immutable-state while.
- Explicit generic functions/records/variants, checked parametrically before use and concretely after specialization.
- Named non-capturing function references, exact Fn signatures and an empty action bound for function values.
- Runtime requires/ensures, static action bounds and explicit closed source-module composition by stable identities.
- Native machine-code compilation through C/Clang or direct application LLVM, with a reference evaluator.

The [frozen acceptance contract](language-v1-acceptance.md) and [public programs](../examples/v1/README.md)
define this release's demonstrated construction patterns and planned changes. No additional language family
joins 1.0 automatically. Standard libraries for external services, FFI, capturing closures, concurrency,
macros, self-hosting and the agent workspace are separate future milestones.

## Types, contracts and representation

Every generic body, branch, annotation and contract must be valid for distinct rigid type parameters.
Passing/copying, homogeneous equality and uniform list operations require no extra bound. Arithmetic,
field/match access and invocation require explicit concrete types or signatures. No trait/constraint syntax
is added. Contract typing does not establish truth, termination or absence of runtime faults. In particular,
generic equality is not universally reflexive because F64 NaNs compare unequal.

Generic type application is `Cursor<I64>`; constructors are `Cursor::<I64> { ... }` and
`Option::<I64>::Some(value)`. Generic functions use `identity::<Text>(value)`. Function values use
`Fn(T) -> U` and `&name` / `&generic::<Type>`, with direct or postfix invocation. Actionful named functions
can be called statically under checked action bounds; obtaining an actionful Fn reference is rejected.
References capture no environment. Nominal IDs derive from stable template identity and ordered arguments.

Checked artifacts contain concrete declarations and calls only, plus checked pure references/invocations.
Reject nonempty template/constructor/call/reference type arguments, source type applications, symbolic types,
unknown/out-of-scope identities, bad layouts/signatures/actions, constructor-only literal kinds and invalid
wire encodings. Finite F64 literals use exact bit strings in the artifact; nonfinite values require bit
construction or typed inputs. Every loaded artifact and external invocation is revalidated.

F64 follows the arithmetic, conversion, NaN, signed-zero, lossless wire and execution-profile rules in
Core 0.12/decision 0034. Supported release platform is Linux x86-64 with the default masked exceptions,
nearest-even rounding and gradual underflow. Unsupported rounding/subnormal settings fail explicitly.
Native code uses no interpreter, fast-math, decimal coercion or new host-library dependency.

## Fixed admission bounds

| Boundary | Limit |
| --- | --- |
| Source | 1 MiB, 8,192 tokens, 16 explicit modules, 32 source nominal declarations |
| Generic parameters/arguments | 8 per declaration/application |
| Function-value signature arity | 16 |
| Function / nominal instances | 256 / 128 per expansion pass |
| Expansion expression nodes | 262,144 |
| Checked scopes / declaration-import slots | 1,024 / 16,384 |
| Concrete nominal declarations | 160 |
| Inline value width / height | 64 cells / 8 |
| Type nesting / logical input-output depth | 8 / 8 |
| Call / structural comparison depth | 32 / 64 |

Definition checking and concrete expansion use separate bounded passes; an ordinary signature can be
needed by the symbolic pass, while its body is checked in the concrete pass. Other inherited expression,
record-field/case, value-count, byte, fuel, heap and native adapter limits remain in force. These are
deterministic implementation admission boundaries, not claims that all programs within them terminate.

## Compatibility

Recompile all pre-1.0 checked artifacts. Existing standalone native executables remain independent.
The JSONL test protocol stays `lobo-test/0.1` and reports the actual compiler/language identities. The VS Code
extension and programming skill target language 1.0. The earlier Core 0.11 evaluation checkpoint, private
source/compiler snapshots and result denominators stay preserved; 1.0 conformance does not imply an agent
token-efficiency, construction, comprehension or performance advantage.
