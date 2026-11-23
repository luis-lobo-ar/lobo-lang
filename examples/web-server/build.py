"""Build the maintained native web example with the Lobo 1.7 compiler."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", choices=["c", "llvm"], default="c")
    parser.add_argument("--compiler", type=Path)
    parser.add_argument("--out-dir", type=Path, default=ROOT / "dist/web-server")
    args = parser.parse_args()
    compiler = args.compiler.resolve() if args.compiler else ROOT / "target/release/lobo"
    if args.compiler is None:
        subprocess.run(["cargo", "build", "--locked", "--release", "--bin", "lobo"], cwd=ROOT, check=True)
    help_ = json.loads(subprocess.check_output([str(compiler), "--help"], text=True))
    if help_.get("language") != "core/1.7":
        raise SystemExit("The maintained source requires a Lobo 1.7 compiler")
    subprocess.run(["cargo", "build", "--offline", "--locked", "--release", "--manifest-path",
                    str(ROOT / "examples/web-server/Cargo.toml"), "--target-dir", str(ROOT / "target/web-server")], cwd=ROOT, check=True)
    destination = args.out_dir.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".build-", dir=destination) as scratch:
        scratch = Path(scratch)
        subprocess.run([str(compiler), "compile", str(ROOT / "examples/web-server/app.lobo"),
                        "--source", str(ROOT / "libraries/http/http.lobo"),
                        "--backend", args.backend, "-o", str(scratch / "handler")], check=True)
        shutil.copy2(ROOT / "target/web-server/release/lobo-web-example", scratch / "server")
        for name in ["server", "handler"]:
            if (scratch / name).read_bytes()[:4] != b"\x7fELF":
                raise SystemExit(f"{name} is not a native ELF executable")
        for name in ["server", "handler"]:
            (scratch / name).replace(destination / name)
    print(json.dumps({"status": "built", "backend": args.backend, "compiler": str(compiler), "language": help_["language"], "server": str(destination / "server"),
                      "handler": str(destination / "handler"), "sizes_bytes": {name: (destination / name).stat().st_size for name in ["server", "handler"]}}))


if __name__ == "__main__":
    main()
