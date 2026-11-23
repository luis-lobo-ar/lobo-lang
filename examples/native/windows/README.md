# Standalone Windows snapshot

This builds the existing [Lobo snapshot](../snapshot.lobo) as one Windows 10/11 x64 console executable.
Application code, IDs and typed result are unchanged. The C backend emits the application; LLVM-MinGW links
it with a Windows clock/random provider and a small console launcher. No Rust host, interpreter, WSL or Lobo
installation is required to run the resulting executable. Imported runtime DLLs are Windows system components.

Double-click `LoboSnapshot.exe`. The launcher supplies the documented clock/random grants, prints the typed
JSON snapshot and waits for Enter when its input is an interactive console. `LoboSnapshot.exe --no-pause`
uses the same demo policy without waiting. Explicit native CLI arguments retain normal default-deny behavior:

```powershell
.\LoboSnapshot.exe --entry native.snapshot.main --native-clock true --native-random true
.\LoboSnapshot.exe --entry native.snapshot.main
```

The second invocation returns Err(Denied). The artifact is an unsigned local build, tested under Wine;
actual Windows execution has not been performed in this session. Its presence does not imply general
Windows compiler/network support. The full native provider remains Linux-only.

## Build from the Linux compiler host

Use the current compiler and an explicitly provided installed LLVM-MinGW toolchain. The build/test scripts
never download tools. The task used LLVM-MinGW 20261117 UCRT, SHA-256
`e7a15a49a2ea95e9f3d8a9d0cfabe5052a4b10fb4777cc54de5a8c27b64bb701` for the upstream Linux x86_64 archive.
The portable toolchain was unpacked under the task's target directory without changing system packages/PATH.

```sh
python3 scripts/build-windows-snapshot.py --toolchain PATH_TO_LLVM_MINGW --out NEW_OUTPUT_DIRECTORY
xvfb-run -a python3 scripts/test-windows-snapshot.py --toolchain PATH_TO_LLVM_MINGW --exe NEW_OUTPUT_DIRECTORY/LoboSnapshot.exe --wine-prefix PRIVATE_WINE_PREFIX --out NEW_TEST_DIRECTORY
```

The build checks PE x64 headers and the import table, and supplies a ZIP with executable, source, IDs, build
receipt and toolchain notices. Only Kernel32, BCrypt and UCRT API-set/system libraries are allowed imports.
Wine tests use an isolated prefix. Do not run them against a user's normal Wine profile.

## Provider boundary

[src/native_windows_snapshot.c](../../../src/native_windows_snapshot.c) implements the ABI's pure scalar/byte
operations plus capability creation/restriction, monotonic milliseconds and random bytes. Root authority is
entry-only; derived rights remain subsets; byte/call/handle quotas and malformed/duplicate option rejection
match this profile's contract. Capabilities are opaque process-local tokens and live until invocation end.
No OS handles are opened: GetTickCount64 is a direct system clock query and BCryptGenRandom uses the system
provider with a NULL algorithm handle. Invocation arena cleanup still frees returned buffers.

The Windows timer uses GetTickCount64's system-timer resolution; it is not a calendar timestamp. Random bytes
come from BCryptGenRandom, with no fallback PRNG. Unsupported network operations fail with
`invocation.native.operation_unsupported`; network grant flags are rejected. This profile does not implement
listeners, streams, socket generations, partial network IO or a Windows HTTP server.

The existing Linux provider prefix is byte-identical. Its TLA+ models and C witnesses were rechecked after
reviewing conditional inclusion; those formal results do not establish correctness of this new Windows code.
Windows-specific executable/provider tests cover normal/default-deny behavior, 64 parent/child masks, invalid
tokens/counts, exhausted quotas/fuel/heap, injected RNG failure, clock overflow, nested root access and fail-closed
unsupported calls. Wine compatibility tests are distinct from testing on actual Windows hardware.

Primary references: [LLVM-MinGW release](https://github.com/mstorsjo/llvm-mingw/releases/tag/20261117),
[GetTickCount64](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount64),
[BCryptGenRandom](https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptgenrandom), and
[Windows UCRT deployment](https://learn.microsoft.com/en-us/cpp/windows/universal-crt-deployment).
