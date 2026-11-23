# Native Lobo REST API

Extract the Windows package and double-click `LoboRestApi.exe`. Open
<http://127.0.0.1:8080/health> in your browser. Use Ctrl+C to stop.

| Request | Response |
| --- | --- |
| GET /health | JSON status and language |
| GET /api/users | Two sample users |
| GET /api/users/1 | Luis |
| GET /api/users/2 | Ada |
| HEAD on these routes | Same status and content length, no body |
| Unknown path | 404 JSON error |
| Other methods without a body | 405 JSON error, Allow: GET, HEAD |

The application and HTTP parser, routing, JSON serialization, read loop and partial write handling are Lobo
code. The compiler links native runtime support and Windows socket calls into one machine-code executable.
No Rust server, interpreter, WSL, Node or Python is required at runtime. Windows system DLLs provide OS services.
This is an unsigned local Windows 10/11 x64 build. Wine execution is recorded separately from actual Windows runs.

The app serves at most 128 connections and polls accept at most 1024 times. Each accept waits at most two
seconds. Request headers and response writes each have a total two-second deadline. Headers are limited to
4096 bytes. Connections close after one response. Request bodies, transfer coding, persistent connections,
HTTP/2, TLS, authentication and persistent storage are outside this read-only local demo.

It binds only loopback. A launcher supplies explicit clock and listen grants for port 8080. Explicit arguments
bypass these defaults and retain default-deny behavior. To select another port in PowerShell:

```powershell
.\LoboRestApi.exe --entry example.rest.main --arg I64:8081 --arg I64:128 --native-clock true --native-listen 127.0.0.1:8081 --native-calls 65536 --native-bytes 1048576 --fuel 100000000 --heap-cells 16000000
```

The invocation arena, fuel, operation and byte budgets bound this demo's lifetime. It is sequential and does
not recover its arena between requests. Restart it after it reaches a limit. Port conflicts return typed errors.

Build on the Linux compiler host with an explicitly installed LLVM-MinGW toolchain:

```sh
python3 scripts/build-native-rest.py --toolchain PATH_TO_LLVM_MINGW --out NEW_OUTPUT_DIRECTORY
```

The builder never downloads tools. The package retains source identity files, build hashes and toolchain notices.
The broader native compiler host remains Linux. This artifact does not distribute a Windows compiler.

Build the same program as a Linux executable:

```sh
target/release/lobo compile examples/native/rest/app.lobo --source examples/native/rest/controller.lobo --source libraries/http-server/server.lobo --source libraries/http-server/transport.lobo --source libraries/http/http.lobo --source libraries/json/json.lobo -o NEW_EXECUTABLE
```

Use the same explicit entry, arguments and grants shown above when launching the Linux executable.

Run the real application over TCP, with a new result directory:

```sh
python3 scripts/test-native-rest.py --exe NEW_EXECUTABLE --out NEW_LINUX_TEST_DIRECTORY
xvfb-run -a python3 scripts/test-native-rest.py --exe PATH_TO_LoboRestApi.exe --wine-prefix PRIVATE_WINE_PREFIX --out NEW_WINE_TEST_DIRECTORY
xvfb-run -a python3 scripts/test-windows-tcp.py --toolchain PATH_TO_LLVM_MINGW --wine-prefix PRIVATE_WINE_PREFIX --out NEW_PROVIDER_TEST_DIRECTORY
```

Add `--default-launch` to the Wine application driver to test the no-argument launcher on port 8080.
Use a task-specific Wine prefix, never your normal Wine profile. See the
[HTTP subset and limits](../../../libraries/http-server/README.md).

The current Core 1.8 source uses try for capability/listener setup. It captures the serve result, closes the
listener, then returns the result; a close error takes precedence as before. Try does not close resources
automatically at each function boundary. Frozen 1.7.3 distributions retain their original source snapshots.
