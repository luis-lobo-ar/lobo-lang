# Language 1.0 acceptance contract

Frozen development requirements, 2026-11-04, before implementation. Public fixtures, not evaluation holdouts.
The user-authorized scope is [decision 0033](../decisions/0033-bounded-language-1-release.md).

## Features

- Generic records and variants compose with generic functions, Lists, guarded recursive data, modules,
  contracts and exhaustive matches. Concrete IDs preserve nominal identity and are independent of aliases
  and unrelated instantiations. Reject unknown/incorrect arguments, malformed layouts and unbounded growth.
- Check generic bodies' local names/types under declared requirements before a use site exists. Check all
  branches and pure contracts, then recheck concrete instances and admission. Unconstrained parameters allow
  value passing, homogeneous equality and List operations, not arithmetic or arbitrary field/case access.
  Requirement mechanisms are limited to what these fixtures need; no unrestricted trait system.
- Named non-capturing references use checked function signatures. Initial function-value types are pure;
  reject references to actionful functions rather than losing their action bounds. Static actionful calls
  remain supported. Callbacks can pass/return values but do not capture an environment or grant host access.
- F64 supports finite fractional literals, + - * /, negation, comparisons, explicit I64 conversion, finite/NaN
  classification and lossless bit interchange. Use binary64 round-to-nearest, ties-to-even, no fast-math.
  Arithmetic non-finites follow IEEE behavior; canonicalize NaN results. Equality treats NaN as unequal and
  signed zeros as equal; interchange preserves signed zero. Invalid float-to-I64 conversion is a fault.

Constructor/reference/type spelling and diagnostic codes receive a normative specification before their
implementation slice. Type/name checking does not prove runtime contracts, termination or absence of exhaustion.

## A: reusable data pipeline

Library modules define Cursor<T>, Option<T>, Result<T,E>, iterative reverse/map/filter and optional find.
Application logic uses shared algorithms for I64, Text and a caller-owned record. Preserve source order.

Fixed small cases: reverse [1,2,3] gives [3,2,1]; reverse ["é","lobo"] gives ["lobo","é"]. Map I64 values
through double and filter by distinct named even/odd predicates. Find has present, absent and empty cases.
Record values retain field/type identity through mapping, reversal, lookup and exhaustive failure handling.

Generate [0..9999] internally using iteration. Keep even inputs, double them and reverse: count 5000,
first 19996, last 0, sum 49990000. Planned change: use odd inputs instead; count 5000, first 19998, last 2,
sum 50000000. There is no 10,000-frame recursion or host algorithm. Freeze fuel/heap allowances at
2,000,000 each for the large acceptance invocation; smaller exhaustion probes use independent fixed bounds.

## B: multi-module document validation/query/transformation

Model: Document has id:I64, name:Text and items:List<Item>. Item has key:I64, score:I64 and enabled:Bool.
Validate positive document/item IDs and a nonempty Unicode-scalar name. Return typed errors for invalid
document ID, empty name and invalid item with its zero-based index. Empty items are valid.
Select enabled items in original order, double scores using checked arithmetic, prefix the name with
"checked:", and report selected count and total. Arithmetic overflow remains the defined runtime fault.

Case: id=7, name="é", items (1,3,true),(2,4,false),(3,-1,true) produces name="checked:é", selected keys
[1,3], scores [6,-2], count 2 and total 4. Include disabled-only/empty lists, invalid IDs and overflow.

Planned schema change: Item gains category:Text. Require nonempty categories and select items that are both
enabled and category="keep". For categories "keep","keep","skip" on the case above, selected keys [1],
scores [6], count 1, total 6. Update producers/consumers and handle the new typed invalid-category case.
Keep baseline and changed sources as distinct public fixtures, with dependency/change comprehension probes.
Each invocation permits 2,000,000 fuel/heap cells, at most 100 items and 4096 UTF-8 input bytes.

## C: fractional numerical validation/aggregation

Return typed Empty/NonFinite(index)/Negative(index) errors as appropriate and a Stats record on success.
Baseline accepts finite inputs; Negative is a declared but unused policy case. Compute count, sum, mean and
population variance in Lobo, rejecting a non-finite intermediate as a typed numerical failure.

[1.5,2.25,-0.75] gives count 3, sum 3.0, mean 1.0, variance 1.625. Empty gives Empty. A NaN/infinite input
gives NonFinite at its first index. Include singleton, signed-zero, exact fractional, conversion boundary
and deliberately non-finite-intermediate cases. Any nonexact expected result gets an explicit tolerance
before execution, never an oracle copied from candidate output.

Planned change: reject negative inputs with their first index. The baseline case then returns Negative(2);
[1.5,2.25,0.0] succeeds with count 3, sum 3.75, mean 1.25 and variance 0.875. Each acceptance invocation
permits 2,000,000 fuel/heap cells and at most 100 input elements. The exact binary fractions above need
bit-exact results. Conversions include truncation of 1.75 to 1 and rejection of NaN, infinities and values
outside I64's representable conversion interval.

## Compiler and release gates

Before implementation, set finite admission: 1 MiB source, 8192 tokens, 16 source modules; at most 256
function and 128 nominal specializations, 1024 lowered scopes, 16384 metadata slots and 262144 expansion
nodes. Existing logical input/output depth eight, inline width 64/height eight, call depth 32 and comparison
depth 64 remain unless an explicit specification change is recorded before changing a measured fixture.
These are compiler/host budgets, not a claim that arbitrary programs or inputs are unbounded.

All existing compatible conformance tests and new independent acceptance cases pass. Test rejection,
artifact reload/forgery, fault order, contract phase, imports, recursion, exact resource bounds and callbacks.
Verify evaluator, C, LLVM, active ASan/UBSan and representative release binaries. Maintain syntax/navigation,
real VS Code host tests, package, skill validator/examples, release specification and reproducible source/
artifact/log identities. Preserve every failed/interrupted run and baseline. No added service or syntax
family becomes another release gate. Final language/compiler/extension versions are assigned at acceptance.
