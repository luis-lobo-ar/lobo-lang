# 0050 Deliver the snapshot as a standalone Windows executable

Date 2026-11-22. Status implemented as a bounded standalone app profile. Risk mode high assurance for native/authority boundaries.
The user requested a version of the existing program that runs directly on Windows, without installing WSL,
Rust or Lobo. Select Windows 10/11 x64 and the existing clock/random snapshot. No GUI toolkit, network port,
full Windows compiler release, arbitrary FFI or broader language change is selected.

Keep the application in Lobo and compile its checked code through the existing C export. Add an explicitly
bounded Windows clock/random provider using GetTickCount64 and BCryptGenRandom. Preserve root-at-entry,
subset-only capabilities, strict launch grants, quotas, typed errors and ABI layouts. Windows networking must
fail closed; do not claim the Linux socket provider or its TLA+ results cover this profile. Keep Linux behavior
unchanged and recheck the mapped Linux model after reviewing the conditional provider inclusion.

Rejected: asking the user to install WSL again; rewriting the application in Rust/C; handwritten CRT/Windows
SDK declarations; or silently claiming a complete Windows runtime. An isolated portable LLVM-MinGW toolchain
is a necessary build dependency for the explicitly requested Windows artifact. Pin upstream release/digest,
retain provenance/licenses, unpack only under this task's build directory and do not install/change system
packages or PATH. Build scripts require an explicit installed toolchain path and never download one themselves.
This is task-scoped build provisioning, not an automatic tool installation/fallback in ordinary tests.

Produce one PE x64 executable. A tiny console launcher supplies the documented demo entry and clock/random
policy when launched with no arguments, and pauses only for interactive input. Explicit command-line arguments
retain default-deny semantics. Application logic remains the same compiler-generated Lobo code. Verify using
an isolated Wine prefix, inspect PE imports for extra DLL dependencies, and label Wine versus actual Windows
coverage accurately. Preserve failing-before builds, default/granted/denied/quota/fault checks and all results.
Unsigned local delivery is not publication or a production-readiness/security certification.
