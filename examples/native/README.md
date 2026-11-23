# Native programs without a host process

The standalone OS examples use Core 1.8 constructs. Current compiler/extension 1.11.2 also build the HTTP examples with Core 1.11 callbacks. [probe.lobo](probe.lobo) uses the
[Lobo OS library](../../libraries/native/README.md). [loopback.lobo](loopback.lobo) exercises raw typed
operations, short IO loops, write-side shutdown and EOF; the Rust regression suite compiles this same source.
Its bounded test protocol writes before reading, so keep messages at the tested 4096-byte bound. It is not a
concurrent production server. Acquiring handles or sending data can fail; operation budgets are conservative.

```sh
mkdir -p target/native-samples
./target/release/lobo compile examples/native/loopback.lobo -o target/native-samples/loopback
./target/native-samples/loopback --entry native.loopback.roundtrip --arg Bytes:00ff7f80 --native-listen 127.0.0.1:0 --native-connect self --fuel 1000000 --heap-cells 200000
```

Expected result: `Ok` with the same `00ff7f80` bytes. Compile with `--backend llvm` to select the other native
route. The executable contains the approved OS provider and needs no compiler, interpreter or Rust host to
run. Kernel/system libraries still provide OS services. See the
[complete authority and ABI contract](../../spec/native-interface-v0.1.md) before extending the sample.

## One program using restricted OS access

[snapshot.lobo](snapshot.lobo) obtains a root capability at entry, restricts it to clock/random rights,
reads monotonic time and requests 16 random bytes. It returns IoResult<Snapshot>, preserving named errors.
The main function uses `try` to return typed errors without nested matches.
Its stable identities are in [the adjacent sidecar](snapshot.lobo.ids.json).

From the repository root, choose a new executable output path:

```sh
./target/release/lobo compile examples/native/snapshot.lobo --source libraries/native/native.lobo --source libraries/native/permissions.lobo -o /tmp/lobo-snapshot
/tmp/lobo-snapshot --entry native.snapshot.main --native-clock true --native-random true
```

Both grants are required for the requested ClockRead and RandomRead permissions. Without both, the program
returns Err(Denied). The returned clock value is elapsed monotonic milliseconds, not a calendar timestamp.
The executable contains the native support code and needs no Rust host or interpreter at runtime.
[Recorded checks](../../benchmarks/results/2026-11-22-native-snapshot-example.json) cover both native backends,
missing/partial grants and zero byte quota. This example adds no language/runtime behavior.

Windows users can follow the [WSL 2 testing guide](../../docs/windows-testing.md). The full native OS
interface targets 64-bit Linux. A [standalone Windows snapshot](windows/README.md) now provides only clock/random
operations in its default profile. Its user-reported Windows 10 run is recorded separately from Wine checks.

The [native REST app](rest/README.md) now adds Lobo HTTP parsing, JSON routes and bounded socket IO. Its
explicit Windows TCP cross-build profile is separate from the default snapshot profile. No Rust HTTP host is
needed. The user has also [reported a successful Windows 10 snapshot](../../benchmarks/results/2026-11-22-windows10-user-run.json).

For VS Code project navigation and diagnostics on the snapshot, select its explicit source closure in your
workspace settings. Choose the files for the active program rather than combining every repository example:

```json
{
  "lobo.sourceFiles": [
    "examples/native/snapshot.lobo",
    "libraries/native/native.lobo",
    "libraries/native/permissions.lobo"
  ]
}
```

Keep the snapshot's adjacent identity file in place. The compiler and editor use the same three-file closure.
