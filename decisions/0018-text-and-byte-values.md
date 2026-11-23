# 0018: Separate immutable byte and UTF-8 text values

- Status: implemented in compiler 0.7.0 / extension 0.8.0 (immutable buffer slice)
- Date: 2026-10-17
- Risk mode: standard with encoding/memory/failure-oriented adversarial gates; no deployment.

## Outcome and alternatives

The frozen parser workload needs byte input and offsets; general application data also needs text without
scalar-list encoding or ASCII-only identity strings. Follow [Core v0.7](../spec/core-v0.7.md).

Local alternative analysis (not independent prototypes):

- Treating every string as List<I64> permits invalid octets and has linear indexing/large per-octet nodes.
- One unvalidated byte-string type makes Unicode validity/indexing implicit and lets malformed text escape.
- Separate Bytes/Text values with shared immutable views make conversions and units explicit; choose this.

Use flat immutable buffers rather than applying the list-node representation to every sequence. Concat/push
copy and spend explicitly; a bulk List<I64>-to-Bytes operation avoids mandating repeated append copies for
builders. Unicode scalar operations intentionally do not claim grapheme/locale/property handling. A codepoint
integer can be inspected without introducing a Char type yet. Syntax expands only for these observed needs.

## Encoding and lifetime contract

Strict UTF-8 validation uses the byte ranges in [RFC 3629 section 4](https://www.rfc-editor.org/rfc/rfc3629#section-4),
with Rust's UTF-8 validator as the reference and a native decoder for independent differential evidence.
Reject overlong encodings, surrogates, stray continuations, truncations, and out-of-range codepoints. Preserve
NUL and U+FEFF as data. Do not silently normalize or replace invalid bytes. Source strings use JSON escapes;
stable IDs retain their narrower unescaped ASCII grammar.

Views retain immutable backing storage. Native buffers join the existing invocation arena; literals use static
storage. No pointer into temporary C stack data is retained. Heap/input/output/fuel limits cover byte counts,
not only value-node counts, and JSON preflight must agree with actual escaped output. Validate input before
allocation, free arena and adapter storage on success/fault, and never grant external authority from conversion.

## Gates and scope

TDD source/encoding/operation tests precede production code. Verify known outcomes, independent UTF-8 cases,
source/artifact bounds, native/reference budget/phase parity, ASan/UBSan/leaks, and maintained VS Code support
including real host checks and packaging. Keep prior tests/oracles, the ledger, and wider language goal.
No dependency, package ecosystem, FFI, host library, daemon, normal-profile installation, or publication.

## Implementation and verification evidence

Core 0.7 is implemented in the reference evaluator, checked-artifact validation, and native C lowering.
`examples/text.lobo` is executable: Unicode framing returns `"[🐺]"`, while invalid input octets can be
guarded and returned as an ordinary tagged failure. The compiler remains native-first.

Final local working-tree gates:

- `cargo test --locked --quiet` and `cargo test --locked --release --quiet`: 89 tests each
  (88 integration tests and one native-launch unit test). Existing numerical/collection oracles remain unchanged.
- Fifteen text/byte integration tests include a seeded 1,295-case UTF-8 corpus checked against Rust's validator,
  the evaluator, optimized native code, and ASan/UBSan native code with leak detection. Eleven scalar boundary
  values additionally cover preserved BOM/NUL and the upper Unicode range.
- Exact fuel/heap and failure-phase checks cover views, conversions, equality, left-to-right argument evaluation,
  action bounds, and pure contracts. Exact 8 MiB serialized output succeeds; one byte over and JSON-escape
  expansion fail before output framing. Oversized input fails before allocation/execution, including a native
  in-process argv probe that avoids the operating system's smaller command-line limit.
- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `./scripts/check.sh`, and
  `git diff --check` pass. The shared native/sanitizer fixture was extracted to `tests/support` without
  changing existing list-test outcomes.
- `npm --prefix editors/vscode test`: 73 tests pass. Real VS Code host checks exercise escaped Unicode
  strings, definition navigation, and unsaved type-error diagnostics using a temporary profile. The rebuilt
  VSIX excludes development dependencies/tests and has not been installed into the normal editor profile.

Release artifacts from this working tree (not a claim of publication or a committed release):

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `target/release/lobo` | 989,264 | `84794ea7b8b22047fe4d11cc07b0f8c2224c242fad048a9e0975e8042bdff923` |
| `dist/lobo-syntax-0.8.0.vsix` | 19,217 | `9c9f6fae00753615de82b8380f4bbd651d400fb8af359abbe4b7775432bba419` |

## Remaining boundaries

This completes the immutable text/byte slice, not the language-first goal. The frozen parser and tree workloads
and broader program composition remain pending. Unicode normalization, grapheme segmentation, locale handling,
regex, recursive named types, and ecosystem integrations are not implied. Native backing allocations are
cumulative within an invocation and reclaimed at its end; shared slices do not introduce a long-lived garbage
collector. Logical limits and sanitizer evidence are neither an operating-system sandbox nor a proof that all
process-memory exhaustion can be caught.
