# Lobo language 1.7

Status implemented by compiler 1.7.0. This version inherits [1.6](core-v1.6.md) and adds the closed
[native interface v0.1](native-interface-v0.1.md). Source emits `core/1.7`; checked framing remains
`lobo-checked/1.0`. Admission revalidates core/1.0 through core/1.7. Earlier tags cannot admit resource
nodes, native operations or an ABI activation marker. Existing programs need no migration.

Modern source recognizes `Native::operation(arguments)` and `Native::Capability`, `Native::Listener`,
`Native::Stream`, plus six native result types. This is a compiler catalog, not a discoverable package or
arbitrary symbol call. Contextual Ok/Err cases use existing inference rules. The old source grammar retains
its prior `Native::Case` user-variant meaning; it gains no native-call syntax.

Native syntax activates `program.native_abi: 1` in checked code. Each operation stores `NativeOp`, the stable
binding ID, ABI 1 and typed arguments. Source locations remain nonsemantic. The checker validates the exact
catalog signature, actions and six built-in result layouts, including synthetic scope ownership. Native-enabled
programs reserve `lobo.native` identities. Programs without this marker retain preexisting user IDs with that
prefix, even when they resemble new result IDs. A prefix alone never activates authority or grants visibility.

Resource values may be passed through functions and data structures. They cannot be made from literals,
external arguments, printed results or reference evaluator values. Native outputs containing resources fail
with `invocation.native.resource_value`. Only the four pure ABI conformance operations run in the evaluator;
OS operations fail with `invocation.native.required`. No fallback is attempted. OS calls require declared
actions and runtime grants independently. Impure function values and contract expressions remain prohibited.

The compiler links the selected C provider into both native routes: default C/Clang and direct LLVM. Native
programs target 64-bit Linux. Pure programs do not link this OS provider or accept its grant flags. Native
support is trusted in-process code, not a sandbox or an arbitrary external-library SDK. The extension 1.7
mirrors built-in signatures/types for highlighting, local/payload navigation and original-range diagnostics.
Built-ins have no fabricated source definition. See [decision 0048](../decisions/0048-typed-native-os-boundary.md)
and [engineering evidence](../benchmarks/results/2026-11-22-native-boundary.md).

## Compiler 1.7.1 correction

The patch release preserves this language and ABI. It fixes cleanup of a connect socket when fuel fails
before registry publication. The [finite state models and C replays](../verification/native/README.md)
check the selected lifecycle contract. Earlier 1.7.0 results remain historical evidence.

## Compiler 1.7.2 packaging support

The language remains unchanged. C export includes a conditional Windows clock/random snapshot provider;
the [application build](../examples/native/windows/README.md) produces a standalone x64 executable. Full
Windows networking/compiler support is outside that profile.

## Compiler 1.7.3 native REST sample

No syntax or ABI change. C export includes an explicitly selected Windows TCP provider for the
[native REST application](../examples/native/rest/README.md). Application HTTP behavior is written in Lobo.
The existing snapshot remains the default Windows build profile. The native compiler host remains Linux.
