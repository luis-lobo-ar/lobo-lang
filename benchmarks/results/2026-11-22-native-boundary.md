# Typed native OS boundary, 2026-11-22

Compiler/language 1.7 and extension 1.7 implement the first closed native OS slice: typed ABI calls, opaque
capabilities/listeners/streams, explicit launch grants, monotonic clock, Linux randomness and bounded IPv4
loopback TCP. Both backends link the C provider into a native executable. A separate Rust host is not required.
The compiler remains Rust. The [native contract](../../spec/native-interface-v0.1.md),
[decision 0048](../../decisions/0048-typed-native-os-boundary.md) and
[machine-readable receipt](2026-11-22-native-boundary.json) define scope, inputs and evidence.

This milestone does not implement arbitrary third-party FFI, filesystem access, TLS, HTTP serving, OAuth,
static asset embedding, concurrency or the requested Google-auth/Angular application. The historical
[native-interface review](../../research/native-interop-2026-11-22.md) remains a proposal at its original
checkpoint. Completing this bounded slice does not authorize automatic ecosystem expansion.

## Test-first development and borrowed material

Thirty new Rust test functions cover the new boundary. The full suite passes 259 tests, with no ignored
cases. The editor suite passes 161 tests. The isolated installed VS Code host verifies native local/field/
payload navigation, no fabricated built-in definition and unsaved type-error repair. ASan/UBSan run both
native routes and the standalone provider harness. Tool absence fails the native suite rather than skipping.

The [upstream manifest](../../tests/upstream/native/manifest.json) retains 16 unchanged source/license files
from five pinned projects: Node.js 22.21.0, CPython 3.14.0, libuv 1.51.0, Go 1.25.0 and Rust 1.90.0. Eight
adaptation groups map sources to Lobo tests. [Attribution and exclusions](../../tests/upstream/native/README.md)
explain exact I64 versus coercion/wrapping, invocation-owned buffers versus GC, synchronous operations versus
callbacks and excluded address families. The adapted scalar/payload corpus is consumed by the Rust tests.
We did not execute the upstream suites or inherit a claim of their runtime maturity.

The provider harness adds deterministic syscall fault injection, descriptor accounting, a 50,000-step handle
model and all 256 four-bit parent/child authority combinations. Each harness case runs optimized and under
sanitizers. It checks denial before OS calls, wrong/stale/forged handles, generation retirement, cleanup after
faults, forced short IO, partial/interrupted randomness, absolute deadlines, clock failure, refused connections,
bind conflicts and interrupted connect completion. This is finite model-based testing, not formal verification.

## Failures found and corrected

All attempt logs, including failed fixtures and verification scripts, are retained under
`benchmarks/private-evidence/native-boundary-20261122/logs`. The receipt hashes each log. Early experiments
were performed while the package still reported 1.6; the version was raised to 1.7 with artifact admission
rules once the new contract was selected. The before compiler and source archive remain available locally.

| Attempt | Observation | Correction / interpretation |
| --- | --- | --- |
| ABI red | Three tests failed before Native syntax existed | Added dedicated checked NativeOp and closed signatures |
| First ABI build | Missing native-list traversal prevented Rust compilation | Added NativeOp traversal; all initial ABI routes passed |
| Resource/TCP red | First failures included unsupported match subjects, else-if spelling and unresolved bare Err | Corrected the parser's call-like native match subject and invalid fixture assumptions; these red runs alone do not prove OS behavior coverage |
| Deadline injection | Repeated interruption exhausted fuel instead of reaching Timeout | Check absolute deadline before each poll and after interruption |
| Authority red | A nested helper could obtain the root | Entry-depth guard applies even to cached root retrieval |
| Delegation red | Restriction binding absent | Added subset-only masks; tested all 256 parent/child pairs |
| Header red | No standalone ABI declaration contract | Added C11 header/layout checks and incompatible-signature rejection |
| Plain-program red | Pure executable accepted native grant flags | Select provider only with an explicit ABI activation marker |
| Unused-type mutation | Older artifact tag admitted resource types in unused expression metadata | Traverse annotations, list elements, loop state and type arguments on admission |
| Namespace mutation | Native-enabled artifact could impersonate catalog IDs | Pin complete built-in layouts, scope and reserved IDs |
| Legacy namespace red | Initial reserved-ID rule rejected legitimate earlier IDs | Explicit program ABI marker; old native-looking IDs retain their semantics |
| Library red/first | Missing source, then a parameter shadowed a local function | Implemented typed IoResult library; renamed parameter without changing function IDs |
| Editor/grammar red | Missing native signatures, match elaboration and nested type scopes | Added mirrored catalog inference/highlighting and original-range checks |
| Legacy editor red | Built-in recognition intercepted a legacy Native variant | Restrict recognition to modern inference mode |
| Buffer fault injection | Nonempty null buffer crashed the provider harness | Classify it as invocation.native.buffer before copying |
| Interrupted connect | A synthetic EINTR without initiated connection published a stream | Require matching connected peer before handle publication |
| Format check | Newly expanded payload array needed formatting | Applied cargo fmt; corrected check passed |
| VSIX packaging | New relative documentation link had no published base URL | Use an explicit repository-file reference in the packaged README |
| Smoke script | Mistook Text output for hex; native output itself was correct | Corrected independent expected-output decoding; 39 commands then passed |
| Integrity scripts | Old live research index differed; archive suffix lookup selected two Cargo.lock files | Document live-index exclusion and use exact archive paths |
| Dependency preservation | Broad version replacement accidentally altered mime metadata | Restored original dependency entries; root-version-only lock comparison now passes |

The first legacy editor red script aborted while inserting its fixture and then ran only the prior three
tests. It is retained as a failed patch attempt, not counted as a failing four-test regression. The subsequent
actual legacy red failed on navigation, and the corrected four-test file passed. Two attempted Node native
buffer URLs were unavailable; repository-tree discovery found the actual pinned node-api test. No unavailable
source is counted as coverage. Earlier logs are observations at their own source states, not extra independent
samples or a clean linear pass on the first attempt.

## Final verification

Run from the repository root with installed Rust/Cargo, Clang and LLVM 18 tools, Node 22+, Python, jq,
Xvfb and VS Code. No runtime dependency was added. Cargo and npm locks differ only in root package versions.

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt -- --check
python3 scripts/check-native-upstream.py
npm --prefix editors/vscode test
LOBO_VSCODE_EXECUTABLE=/usr/share/code/code xvfb-run -a npm --prefix editors/vscode run test:host
npm --prefix editors/vscode run package
python3 -m unittest discover -s benchmarks -p 'test_*.py'
node --test scripts/test-migrate-syntax.mjs
cargo build --locked --release --bins
./scripts/check.sh
```

Observed gates: 259 Rust, 161 editor, 31 Python benchmark-tool, four source-codemod and six existing HTTP
acceptance tests pass. Counts describe test functions rather than independent research subjects. The old
Rust-host HTTP sample is rebuilt with compiler 1.7 to check compatibility; passing it does not implement a
new Lobo HTTP server. The skill validator passes. The retained smoke driver performs 39 successful compile/
invocation commands, checks 12 ELF executables across both backends and confirms both maintained MVC checked
programs equal those from the preserved 1.6 compiler, excluding only the artifact language tag.

Before/after source, optimized tools, extension, smoke binaries/results, harness/verification drivers, logs,
upstream provenance and integrity data are retained locally. The receipt records their paths and hashes.
The source-migration manifest, 57 selected files from this task's baseline, 291 historical survey files,
21 earlier paper entries and 86 frozen 1.0 release artifacts verify unchanged. Live spec/research indexes,
the earlier repair-driver path rebind and generated caches are explicit historical-survey exclusions.
All retained MVC and earlier skill identities are preserved. No historical manifest was recaptured.

## Limits and paper claims

C and LLVM share the C provider and OS, so backend parity is not an independent implementation oracle for OS
behavior. Sanitizers, fault injection and a finite fixed-seed model do not prove memory safety or security.
Native provider code has process authority; capability checks do not sandbox malicious native code. The API
is synchronous and bounded per operation, with no aggregate wall-clock deadline, resource revocation or
read-only stream attenuation. Quotas reserve requested bytes even on partial IO/timeouts; operations are not
transactions. Resource aliases become invalid on close and capabilities live until process exit.

No runtime performance comparison, sustained fuzz campaign, independent security audit or agent construction,
comprehension or token-efficiency experiment was run. Those effects remain unknown. This is public development
engineering evidence, not unseen confirmatory evaluation. The ignored local evidence directory is not a Git
commit, publication or off-device backup. The project remains experimental; “rock solid” is a testing objective,
not a conclusion established by this result.
