# Pure HTTP helper module

Status: implemented and verified following the user's request for helpers around the native web sample.
Source/API: [HTTP helpers](../../libraries/http/README.md). Decision: [0036](../../decisions/0036-pure-http-helper-module.md).

The module supplies typed text and serialized-JSON responses, standard 404/405 replies, GET/HEAD checks,
tagged UTF-8 prefix results and JSON string quoting. Response retains its stable `example.web.response`
identity after moving modules. The sample and every compile caller now explicitly supply the library.
No compiler, language, host ABI or dependency changes. Editor runtime/grammar are unchanged; an actual-library
navigation regression and the workspace's explicit source list now cover the module.

## Verification and corrections

- Three behavioral conformance tests failed against valid stubs before implementation (`http-helpers-red.log`).
  They then passed after implementation (`http-helpers-green.log`) across evaluator, C, direct LLVM, C
  ASan/UBSan and LLVM sanitizer builds, after checked-artifact reload.
- Response tests check exact status/content type/body and unsupported-status precondition failure. Method
  and prefix cases include empty/exact/nonmatching/Unicode values. JSON tests independently parse the output
  with serde_json and check round trips for every U+0000–U+001F control, quotes/backslashes, UTF-8, 512-byte
  input and injection-shaped text. Quoting uses list construction plus one reverse/conversion rather than
  repeated growing text copies. `json_document` deliberately accepts an already-serialized document; it is
  not a JSON parser or arbitrary-record serializer.
- Six original HTTP/sample tests pass without weakening criteria (`http-helpers-web-tests.log`), including
  the 2 KiB maximum greeting under the original 10,000 fuel/heap limits. C and LLVM handler cases still pass.
- All 188 Rust tests pass (`http-helpers-full-rust.log`). All 31 Python tests pass (`http-helpers-full-python.log`).
  All 123 editor tests pass (`http-helpers-full-editor.log`), including navigation from app calls/cases to the
  actual helper source. The real VS Code host passes (`http-helpers-editor-host.log`); no new VSIX is needed
  because only its tests/configuration changed. Clippy with warnings denied and formatting pass.
- The existing seven-source editor workspace plus both new sources is accepted as a nine-file closed
  bundle by the frozen compiler (`http-helpers-workspace-bundle.json`); source-list configuration was updated.
- A new release-input test failed because the source snapshot builder omitted the new `libraries/` directory
  (`http-helpers-release-inputs-red.log`). Included it in the existing whitelist and the test passes
  (`http-helpers-release-inputs-green.log`, repeated in `http-helpers-release-inputs-final.log`). The test's
  assertion message was shortened to avoid dumping all source hashes on failure; its criterion is unchanged.
  One attempted edit failed context validation and changed no files before a corrected patch was applied.
- The README's complete JSON-composition example ran natively with quotes, newline, backslash, Unicode and
  injection-shaped input. Its JSON body independently parses to exactly one `name` value
  (`http-helpers-documented-example.json`). Skill validation and Markdown/JSON/JSONL checks pass.
- The local demo was stopped for rebuilding and restarted at 127.0.0.1:8080. Live health/greeting requests
  pass (`http-helpers-live-check.json`). All 86 frozen language 1.0 artifact hashes remain valid; compiler
  src/Cargo and editor implementation/grammar still match the frozen release. Previous sample artifacts and
  receipts remain historical, with their preserved copies unchanged.

Conformance uses 200,000 fuel / 100,000 heap for escaping workloads. This does not grant those budgets to
the live host or promise that every arbitrary input fits them. No network API, package resolver, new status
family or application framework was added. The sample host remains loopback-only, sequential and restricted
to 200/404/405 responses and its existing output limits.

This is public engineering/conformance evidence. No speedup, token-efficiency, software quality or
comprehension advantage for independent agents was measured. The [receipt](2026-11-05-http-helpers.json)
pins source, binaries and raw successful/failed results; original language 1.0 manifests are not rewritten.
