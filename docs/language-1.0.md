# Local Lobo language 1.0 release

The four-feature language milestone is implemented: generic data, definition-time checking, pure named
function values and F64. Its three public programs and planned changes pass the fixed acceptance cases.
The [release receipt](../benchmarks/results/2026-11-04-language-v1-release.json) is the authority for final
package/check status. A receipt with status `passed` is required; this document alone is not verification.

The specification is [Core 1.0](../spec/core-v1.0.md). Source files and sample commands are in
[the acceptance program directory](../examples/v1/README.md). The broader agent-workspace/product target
remains separate. No further language features are required by this release contract.

## Artifacts and workflow

The local release directory is `dist/lobo-1.0.0-linux-x86_64`. It contains the optimized native compiler,
JSONL test tool, VS Code VSIX, six checked programs, twelve standalone C/LLVM machine-code executables,
source snapshot, complete final command logs and a SHA-256 manifest. Sizes are exact bytes in the manifest.
There is no publication, Git tag or change to the user's installed editor profile.

Rebuild and reverify into a new directory using existing local tools:

```sh
python3 scripts/release.py --output dist/lobo-1.0.0-linux-x86_64-new --vscode /usr/share/code/code --skill-validator /home/luis/.codex/skills/.system/skill-creator/scripts/quick_validate.py
```

Supply your installed editor and skill-validator paths. The builder never downloads tools or overwrites an
existing release directory. It builds the matching release compiler before Python/skill checks, then runs
Rust conformance/native/sanitizer tests, optimized float-profile tests, formatting, Clippy, Python, editor and
real host checks, documentation validation, skill validation and VSIX packaging. It executes all six fixed
programs with the release evaluator and both native backends, checks independent fixed oracles, and rejects
source drift during verification. Each step reports JSON and retains its raw log; failed attempts remain failed.

Install the packaged extension from any working directory, replacing the path with your checkout:

```sh
code --install-extension /home/luis/dev/lobo-ar/lobo-lang/dist/lobo-1.0.0-linux-x86_64/lobo-syntax-1.0.0.vsix --force
```

Use the packaged compiler by absolute path or place it on PATH. `--help` returns machine-readable commands.
`compile ... -o NEW_PATH` produces a standalone optimized executable; `run` compiles/runs native code;
`eval` explicitly chooses the reference evaluator. Closed imports always require explicit supplied sources.

The source snapshot pins compiler/workload/tooling inputs. Historical research archives and private study
packages remain in the repository's evidence locations and are not copied into the local language release.
Final reports preserve baselines, failed attempts and limits. Public development acceptance is conformance
evidence and does not measure independent agents or establish token/comprehension/construction gains.
