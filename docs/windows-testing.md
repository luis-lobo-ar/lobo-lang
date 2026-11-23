# Try the native example on Windows through WSL 2

Compiler 1.7.3 has a Linux native-ABI compiler host. The platform guard in
[src/native.rs](../src/native.rs) rejects native-ABI compilation on other host targets. The
[Windows snapshot](../examples/native/windows/README.md) and [native REST app](../examples/native/rest/README.md)
are separately cross-built standalone Windows executables. Neither needs WSL to run.

Use the WSL recipe below to build Linux programs from source. The recipe has not been executed on Windows
in this session. A user separately [reported a successful snapshot on Windows 10](../benchmarks/results/2026-11-22-windows10-user-run.json).
The REST API's Windows validation currently uses Wine.

## Install WSL

In an Administrator PowerShell window:

```powershell
wsl --install -d Ubuntu
```

Restart if prompted, open Ubuntu and complete its Linux account setup. New installs use WSL 2 by default;
`wsl -l -v` in PowerShell shows each distribution's version.
See [Microsoft's WSL installation instructions](https://learn.microsoft.com/en-us/windows/wsl/install).

## Install the build tools inside Ubuntu

```sh
sudo apt update
sudo apt install -y build-essential clang curl ca-certificates
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Follow rustup's prompts. Install the Linux toolchain inside Ubuntu, as described in
[the official Rust WSL instructions](https://rust-lang.org/tools/install/).
Rust builds the Lobo compiler; it does not host the compiled example at runtime.
The default C backend does not require installing the optional direct LLVM 18 tools or the formal-model tools.

## Build and run the example

Copy the current `lobo-lang` source folder into Ubuntu, for example `~/lobo-lang`. Include Cargo manifests,
source directories, libraries, examples and adjacent `.lobo.ids.json` files. Generated target/dist directories
and node_modules are not needed for this build. Use the current working source; the new example and compiler
changes are not yet a published Windows release. Microsoft recommends the Linux filesystem for Linux tools:
[WSL development environment](https://learn.microsoft.com/en-us/windows/wsl/setup/environment).

Inside Ubuntu, choose a fresh output executable path:

```sh
cd ~/lobo-lang
cargo build --locked --release --bin lobo
mkdir -p target/windows-demo
./target/release/lobo compile examples/native/snapshot.lobo \
  --source libraries/native/native.lobo \
  --source libraries/native/permissions.lobo \
  -o target/windows-demo/snapshot
./target/windows-demo/snapshot --entry native.snapshot.main \
  --native-clock true --native-random true
```

Expected: a typed Ok snapshot containing monotonic milliseconds and 16 random bytes. Omit the launch grants
to exercise the typed Err(Denied) path. Build output is a Linux executable running inside WSL, not a Windows
.exe. The compiler requires an unused output filename; choose another filename when recompiling.

Keep [snapshot.lobo.ids.json](../examples/native/snapshot.lobo.ids.json) alongside
[the program](../examples/native/snapshot.lobo). The explicit helper source is
[libraries/native/native.lobo](../libraries/native/native.lobo). No Rust host or interpreter is needed to
run the compiled program inside Ubuntu. Installing native Windows Rust alone does not port the compiler host.
The separate Windows application builds link the selected Windows provider at cross-compilation time.

## Standalone native REST API

Compiler 1.7.3 also cross-builds a [read-only native REST demo](../examples/native/rest/README.md).
Download its package, extract it, and double-click `LoboRestApi.exe`. Open <http://127.0.0.1:8080/health>.
The Lobo application uses an explicit Windows TCP provider and needs no Rust HTTP host or WSL at runtime.
This adds no Windows compiler distribution. The snapshot and REST application have separate test records.
