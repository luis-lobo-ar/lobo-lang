# 0007: Gate native evolution with independent differential and sanitizer tests

- Status: accepted for development
- Date: 2026-10-10

## Context

Native machine code is the required execution path. Existing hand-written Core fixtures and the reference
evaluator establish a baseline, but agreement between two implementations can hide shared semantic mistakes.
C lowering must additionally preserve faults without introducing undefined behavior or invalid memory access.
The roadmap calls for generated bounded programs and sanitizer checks before further language expansion.

## Decision

Keep native compilation as the default and retain explicit reference evaluation as a test oracle. Add a
deterministic, finite native corpus with independent mathematical I64 expectations, fixed seeds, explicit
boundary/ordering probes, and retained failing sources. Compare normal Clang `-O3` output and `-O1` output
instrumented with AddressSanitizer and UndefinedBehaviorSanitizer. Deliberate invalid C probes must demonstrate
that the instrumentation detects faults. Neither backend agreement nor sanitizers replace specification fixtures.

Require the existing Clang toolchain's sanitizer runtimes for the development suite; do not silently skip or
automatically install them. This is a development-only tool prerequisite, not a production dependency, custom
Lobo runtime, or new Cargo dependency. Disable leak detection explicitly and disclose that limitation because
the CLI terminates classified failure paths without manually freeing its process-lifetime argument storage.

## Consequences and limits

- `cargo test --locked` now includes generated native and sanitizer checks.
- Classified language faults remain structured JSON rather than C traps or undefined behavior.
- Finite seed/case budgets keep the routine suite bounded; unrestricted fuzzing and automated shrinking remain later work.
- Native GCD/factorial examples demonstrate recursive Core programs without changing grammar or language version.
- Additional syntax must be defined and verified across all supported compiler stages before advertising support.
- Test binaries are instrumented; shipped native binaries remain optimized and uninstrumented.
