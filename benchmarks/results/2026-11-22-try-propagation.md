# Try result propagation, 2026-11-22

Compiler and extension 1.8.0 implement [Core 1.8](../../spec/core-v1.8.md) with the selected prefix `try` syntax.
The [snapshot](../../examples/native/snapshot.lobo) now propagates typed failures through three sequential
bindings and keeps its original identity sidecar. [Decision 0052](../../decisions/0052-error-propagation-proposal.md)
records the selected type rule and supersedes the initial narrow candidate. See the
[structured receipt](2026-11-22-try-propagation.json).

## Contract and implementation

Operand and function output each require exactly two payload-bearing cases, Ok and Err. Error payload types
must match exactly. Success types and nominal result owners may differ. This explicit protocol supports
ordinary generic result declarations, the OS library and native ABI results without hard-coded library IDs.
It adds no implicit error conversion, permission grant, recovery block or automatic per-function cleanup.

The operand runs once. Its Ok payload continues the expression; its Err payload finishes the current function
body. Calls to that function still return an ordinary result to their callers. Short-circuiting and evaluation
order remain in force. Propagation from branch, argument, record, fold or while children exits the function.
Postconditions check both successful and propagated results. Runtime faults remain faults. Direct contract
propagation is rejected. The contextual try(value) call spelling remains ordinary source compatibility.

The checked expression uses the existing Unary form with op try. Resolver, generic substitution, action and
dependency traversal already visit unary operands. The checker validates the result shapes and enclosing
output type. Inference retains the success payload. The evaluator uses a separate return-control value from
runtime diagnostics. The C emitter branches to a common result/postcondition/frame epilogue. LLVM branches
through the same conceptual epilogue with a populated output slot. A source continuation rewrite was rejected
because nested loop exits and postconditions require real function-return semantics.

Checked framing remains lobo-checked/1.0. New try nodes require core/1.8. Existing core/1.7 native artifacts
remain accepted without try. Earlier language versions retain their admission rules. Built-in ABI 1, native
provider C files and runtime C cleanup files are unchanged. Existing Linux finite models do not prove this
new evaluator or generated control flow; their original claims remain scoped to their mapped functions.

The implementation uses the standard engineering workflow with additional fault, permission and cleanup
checks for the control-flow boundary. No external implementation source or new runtime dependency was added.
Rust's postfix question mark and Zig's try were reviewed as precedents in decision 0052. The user chose try.

## Test-first and negative results

The first three conformance groups failed because prefix try was unrecognized, with original-source parse
ranges. They pass after implementation. The editor's new fixtures initially failed highlighting, payload
navigation and Core 1.8 compiler-result admission. Its initial suite had 18 failures and 146 passes; that
includes existing compiler fixtures affected by the new language tag. The corrected suite passes all 166.

Implementation checks also caught a missing enclosing-output field in the operator checker's construction
and a missing propagation module import in the C emitter. Clippy rejected a match that should use matches;
the correction passes. VSIX packaging rejected a relative specification URL without a published repository
base URL. The extension README now names the repository path without inventing a public URL, and packaging
passes. The first full Rust run stopped at an old core/1.7 output-version expectation in the actions suite.
The actions and fold version expectations were updated to the selected core/1.8 contract; their behavior
assertions were retained. The corrected full suite passes. A later adversarial import case exposed an extra visibility check on
inferred success payloads: the equivalent match worked without naming the payload type, but try did not.
The failing regression was retained, the extra restriction removed, and both compiler and bundle navigation
now cover this case. A documentation-link check also ran before the receipt existed and failed on those
pending links; the final check runs after sealing. These failures remain retained attempts.

## Verification

- Nine try conformance groups pass. Pure result cases run through the evaluator, C, direct LLVM and both
  sanitizer builds. They cover success/error payloads, different success and result-owner types, propagation
  through expressions and loop children, short-circuiting, skipped later work, function values and repeated
  calls without leaked call depth. Postcondition failure still occurs on a propagated Err. Arithmetic faults
  remain runtime faults. Unused generic bodies, actions, malformed result shapes, mismatched error types,
  contracts and checked-version forgery are rejected.
- A one-call native clock budget verifies single operand evaluation. A native timeout propagates from a
  worker to its caller, which explicitly closes and reopens a listener within a two-handle budget. The result
  remains the original timeout error. These cases run on C/LLVM and their sanitizer builds.
- The maintained snapshot passes grants/default denial and exact three-call/two-call quota cases across
  the four native builds. Its source changes while its tracked IDs remain byte-identical.
- The final full repository Rust suite passes 272 tests. A further focused run covers the expanded loop-condition,
  loop-initializer and loop-count cases. Both runs pass; they are overlapping engineering checks.
- All 166 Node editor tests pass. The isolated real VS Code host verifies try field and producer navigation,
  no fake keyword definition, original-source unsaved errors and repair, plus existing editor behavior.
  The 1.8.0 VSIX is packaged. The programming skill and a new pure example are synchronized; the example's
  successful and failing inputs run through both native backends.
- Clippy and formatting pass. Existing Linux model checks pass both finite baselines, eleven intended
  counterexamples, 22 concrete C witness builds and 22 C mutation builds. Covered source fingerprints are
  unchanged. This rerun does not extend those formal claims to try.
- The updated Windows snapshot PE is 48,640 bytes. The exact packaged executable passes 14 application and
  10 provider cases under Wine, with 64 permission pairs. Actual Windows execution of this new 1.8.0 binary
  is unknown. The earlier user-reported Windows 10 run applies to the preserved 1.7.2 snapshot only.

The unchanged REST app is also rebuilt with compiler 1.8.0 and passes its 35-scenario real TCP regression
under Wine. A direct opt-18 verifier run accepts the emitted LLVM for the pure try skill example.

No existing REST cleanup code was mechanically rewritten. Invocation cleanup does not replace explicit
resource handling within a running server. User-facing parser, resource and deadline limits remain in force.
The source compatibility rule intentionally retains ordinary try(value) calls; propagation uses try producer().
There is no postfix spelling or catch syntax in this milestone.

## Evidence and limits

Prior/current source archives, the 1.7.3 compiler baseline, 1.8.0 compiler/VSIX, Windows PE/package, complete
failed and corrected logs, model results and command drivers are retained locally under
benchmarks/private-evidence/try-propagation-20261122. Existing paper evidence, the frozen 1.0 release,
maintained IDs and frozen source fixtures are checked for preservation. Dependency locks change only the
root version. No tool download or global installation occurred.

These are public development and regression cases. Repeated modes are not independent subjects. This work
measures neither agent software construction, comprehension, token efficiency nor runtime performance.
The reduction in repeated match arms is an authoring change, not evidence of an agent-productivity gain.
The files are local working changes and artifacts, without a claimed Git commit, publication or off-device backup.
