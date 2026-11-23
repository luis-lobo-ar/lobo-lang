# Standalone Windows snapshot, 2026-11-22

Delivered one Windows 10/11 x64 PE executable built from the unchanged Lobo snapshot source and identity
sidecar. Compiler/extension patch 1.7.2 retain language core/1.7, checked format 1.0 and native ABI 1.
The application is compiler-generated machine code, linked with a bounded Windows clock/random provider
and a small console launcher. There is no Rust host, interpreter, WSL requirement or application rewrite.
See [build/profile instructions](../../examples/native/windows/README.md),
[decision 0050](../../decisions/0050-standalone-windows-snapshot.md) and
[structured receipt](2026-11-22-windows-snapshot.json).

## Artifact and scope

The executable is `dist/windows-snapshot-1.7.2/LoboSnapshot.exe`, 48,640 bytes in this build. The companion
ZIP includes the executable, source/IDs, build receipt, validation summary and upstream notices. Imported
DLLs are Kernel32, BCrypt and Windows UCRT API-set system libraries; import-table validation rejects other
runtime dependencies. No compiler, SDK or additional runtime DLL is bundled as a prerequisite for the user.
This is an unsigned local demo build, not a published/signed release or production security certification.

No arguments runs the snapshot with explicitly supplied clock/random grants and pauses only for a real
console. `--no-pause` uses the same demo policy for automation. Explicit native CLI arguments retain normal
default-deny behavior. The launch policy belongs to this app wrapper; source action labels do not grant
ambient authority. The wrapper contains no clock/RNG/application implementation: it invokes generated main.

The profile supports the common pure ABI helpers, capabilities/restriction, clock and randomness. It does
not implement Windows sockets or a complete native Windows compiler distribution. Network grant options
are rejected and network operations fail with invocation.native.operation_unsupported. Source IDs, helper
library and application body remain byte-identical to the earlier Linux snapshot. The full provider and
ordinary host compiler workflow remain Linux-based; cross-building uses explicit C export and LLVM-MinGW.

## Test-first implementation and corrections

Cross-compiling the old generated C failed before the new provider existed. The failure also exposed stale
Linux pending-fd references in the unsupported-target cleanup stub, introduced by the earlier cleanup fix.
Linux never selected that fallback; no supported Linux behavior changed. The fallback now has no Linux-only
references, and the separately appended Windows provider defines its own limited supported operations.
The existing pure/Linux provider prefix was compared byte-for-byte with the preserved source and is unchanged.

The first standalone build executed successfully under Wine and its first automated suite passed. A manual
run with stdin redirected from NUL nevertheless printed the old isatty-based pause prompt. Console detection
now uses GetConsoleMode, and --no-pause is explicit for automation. This was a console UX correction, not a
failed authority/RNG test. Subsequent suites include both default launch and --no-pause behavior.

The Windows provider uses GetTickCount64 and BCryptGenRandom with NULL algorithm handle and the system RNG
flag. It opens no OS handles. It keeps entry-only/cached root checks, subset attenuation, strict option parsing,
opaque process-local capabilities, byte/call/handle quotas and classified errors. Injected RNG failure returns
OsFailure with no fallback PRNG. A clock value beyond I64 returns OsFailure without narrowing overflow.
This Windows implementation is outside the existing Linux TLA+ model claims.

## Verification

The exact delivered executable passes **14 application cases** under Wine: default launch, no-pause launch,
explicit granted success, missing/partial grants, byte/call/handle exhaustion and malformed/duplicate/unsupported
launch options. The Windows provider harness passes **10 cases**, including all **64 parent/child mask pairs**
for four supported parent masks and all sixteen requested child masks. It checks denial before clock/RNG
calls, invalid tokens/counts, zero/max byte lengths, no widening, nested root rejection, quotas, injected RNG
failure, clock overflow, unsupported network calls and fuel/heap faults.

Wine is an existing compatibility layer used with a fresh task-specific prefix. These are actual PE executions
under Wine 9.0, not runs on Windows hardware. Actual Windows validation remains unknown. Interactive Windows
console behavior is implemented using documented console APIs; noninteractive default and explicit no-pause
paths are exercised by the executable suite. Do not infer all Windows behavior or crypto quality from Wine.

The Linux model mapping was reviewed after conditional provider inclusion. The provider prefix is unchanged;
both finite baseline models, eleven model defects, twenty-two C witnesses and twenty-two isolated C mutations
pass their existing acceptance criteria. Their scope remains the selected Linux functions/configurations,
with no mechanical C refinement or Windows formal-proof claim. The new Windows file is not silently covered
by updating a Linux source hash.

Conventional gates pass 262 Rust tests, 161 editor tests, the real isolated VS Code host, Clippy, formatting,
31 Python benchmark-tool tests and four codemod tests. The compiler/VSIX are rebuilt as 1.7.2. Existing source,
identity and frozen-result preservation checks accompany the receipt. No grammar/navigation behavior changes
were needed. Failed-before build logs, prototypes, Wine results, model output and tool/source identities are
retained locally. All results are engineering validation, not independent research subjects.

## Toolchain and provenance

The authorized Windows artifact needed an isolated portable cross-compiler; no system packages or PATH were
changed. LLVM-MinGW 20261117 UCRT Linux x86_64 was resolved from the upstream release, downloaded explicitly
for this task and checked against the asset's SHA-256:
`e7a15a49a2ea95e9f3d8a9d0cfabe5052a4b10fb4777cc54de5a8c27b64bb701`.
Build/test scripts require an explicit toolchain path and never download one automatically. The package
retains LLVM and MinGW runtime notices. This does not add a runtime dependency to Lobo applications.

Primary sources consulted 2026-11-22:

- [LLVM-MinGW release](https://github.com/mstorsjo/llvm-mingw/releases/tag/20261117): portable cross-toolchain asset and digest.
- [GetTickCount64](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount64): elapsed milliseconds, timer resolution and API requirements.
- [BCryptGenRandom](https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptgenrandom): system-provider flag, NULL handle and failure contract.
- [UCRT deployment](https://learn.microsoft.com/en-us/cpp/windows/universal-crt-deployment): Windows 10/11 inbox runtime component.

The local artifact is not a Git commit, public deployment or off-device backup. No Windows network, full
Windows compiler, macOS port, new language construct, performance benchmark, agent construction/comprehension
or token-efficiency experiment is implied. Those outcomes remain unmeasured or unimplemented as stated.
