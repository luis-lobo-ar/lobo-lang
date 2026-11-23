#!/usr/bin/env python3
"""Build, verify and retain a local language release; never publish or overwrite one."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs():
    paths = set()
    for name in ["src", "tests", "spec", "decisions", "docs", "examples", "libraries", "scripts",
                 ".agents/skills/lobo-programming", "editors/vscode/src", "editors/vscode/test",
                 "editors/vscode/syntaxes", "benchmarks/language-comparison"]:
        paths.update(path for path in (ROOT / name).rglob("*")
                     if path.is_file() and "__pycache__" not in path.parts)
    for name in ["Cargo.toml", "Cargo.lock", "AGENTS.md", "README.md", "VISION.md", "CHANGELOG.md",
                 "editors/vscode/package.json", "editors/vscode/package-lock.json",
                 "editors/vscode/README.md", "editors/vscode/language-configuration.json"]:
        paths.add(ROOT / name)
    paths.update((ROOT / "benchmarks").glob("*.py"))
    return {str(path.relative_to(ROOT)): digest(path) for path in sorted(paths)}


def number(value):
    return {"type": "I64", "value": str(value)}


def floating(bits):
    return {"type": "F64", "value": bits}


def record(identity, fields):
    return {"type": "Record", "value": {"id": identity, "fields": fields}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="new release directory")
    parser.add_argument("--vscode", type=Path, required=True, help="installed VS Code executable")
    parser.add_argument("--skill-validator", type=Path, required=True)
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    if version != "1.0.0" or platform.system() != "Linux" or platform.machine() != "x86_64":
        parser.error("this release contract requires version 1.0.0 on Linux x86_64")
    if not args.vscode.is_file() or not args.skill_validator.is_file():
        parser.error("the installed editor and skill validator must exist; tools are not downloaded")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    logs = output / "logs"
    logs.mkdir()
    manifest = {"format": "lobo-local-release/1.0", "version": version, "status": "running",
                "platform": platform.platform(), "inputs": inputs(), "commands": [], "acceptance": [],
                "scope": "bounded language release; no publication or agent-outcome claim"}
    manifest["external_validation_tools"] = [{"path": str(path.resolve()), "sha256": digest(path)}
                                             for path in [args.vscode, args.skill_validator]]

    def save():
        (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    def run(label, command, data=None, timeout=300):
        command = list(map(str, command))
        print(json.dumps({"step": label, "status": "running"}), flush=True)
        result = subprocess.run(command, cwd=ROOT, input=data, text=True,
                                capture_output=True, timeout=timeout,
                                env={**os.environ, "CARGO_INCREMENTAL": "0"})
        path = logs / f"{len(manifest['commands']):03d}-{label}.log"
        path.write_text(json.dumps(command) + "\n" + result.stdout + "\nSTDERR:\n" + result.stderr)
        manifest["commands"].append({"step": label, "command": command, "returncode": result.returncode,
                                     "log": str(path.relative_to(output)), "sha256": digest(path)})
        save()
        if result.returncode:
            raise RuntimeError(f"{label} failed; see {path}")
        print(json.dumps({"step": label, "status": "passed"}), flush=True)
        return result.stdout

    try:
        for command in [["rustc", "-Vv"], ["cargo", "--version"], ["clang", "--version"],
                        ["clang-18", "--version"], ["llvm-link-18", "--version"], ["opt-18", "--version"],
                        ["python3", "--version"], ["node", "--version"], ["npm", "--version"]]:
            run(f"tool-{command[0]}", command)
        run("release-build", ["cargo", "build", "--locked", "--release", "--bins"], timeout=600)
        rust = run("rust-tests", ["cargo", "test", "--locked"], timeout=600)
        manifest["rust_tests"] = sum(map(int, re.findall(r"test result: ok\. (\d+) passed", rust)))
        run("release-float-tests", ["cargo", "test", "--locked", "--release", "--test", "floats"], timeout=600)
        run("format", ["cargo", "fmt", "--all", "--check"])
        run("clippy", ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"])
        run("python-tests", ["python3", "-m", "unittest", "discover", "-s", "benchmarks", "-p", "test_*.py"])
        editor = run("editor-tests", ["npm", "--prefix", "editors/vscode", "test"])
        manifest["editor_tests"] = int(re.search(r"# pass (\d+)", editor)[1])
        host = run("editor-host", ["env", f"LOBO_VSCODE_EXECUTABLE={args.vscode.resolve()}",
                                   "xvfb-run", "-a", "npm", "--prefix", "editors/vscode", "run", "test:host"])
        if "LOBO_HOST_TESTS_PASSED:" not in host:
            raise RuntimeError("editor host exited without its completion marker")
        run("documents", ["bash", "scripts/check.sh"])
        run("skill-validation", ["python3", args.skill_validator.resolve(), ".agents/skills/lobo-programming"])
        run("editor-package", ["npm", "--prefix", "editors/vscode", "run", "package"])
        for name in ["lobo", "lobo-test"]:
            shutil.copy2(ROOT / "target/release" / name, output / name)
            if (output / name).read_bytes()[:4] != b"\x7fELF":
                raise RuntimeError(f"{name} is not an ELF executable")
        compiler = output / "lobo"
        help_ = json.loads(run("compiler-discovery", [compiler, "--help"]))
        if help_.get("language") != "core/1.0":
            raise RuntimeError("release compiler reports the wrong language")
        discovery = json.loads(run("test-tool-discovery", [output / "lobo-test", "--session", "--clang", shutil.which("clang")],
                                  '{"format":"lobo-test/0.1","action":{"op":"describe"}}\n'))
        if discovery.get("status") != "ok" or discovery.get("language") != "core/1.0" or discovery.get("compiler") != version:
            raise RuntimeError("test tool did not report the matching compiler/language contract")
        shutil.copy2(ROOT / "dist/lobo-syntax-1.0.0.vsix", output / "lobo-syntax-1.0.0.vsix")
        samples = [("skill-boxed", ".agents/skills/lobo-programming/references/examples.lobo", "skill.examples.boxed", ["Text:c3a9"], "é"),
                   ("skill-fraction", ".agents/skills/lobo-programming/references/examples.lobo", "skill.examples.fraction", ["I64:2"], "0x3fe8000000000000"),
                   ("generic-data", "examples/generic_data.lobo", "example.generic_data.main", [], "lobo")]
        for name, source, entry, arguments, expected in samples:
            for backend in ["c", "llvm"]:
                command = [compiler, "run", source, "--backend", backend, "--entry", entry]
                for argument in arguments:
                    command += ["--arg", argument]
                value = json.loads(run(f"{name}-{backend}", command))
                if value.get("status") != "ok" or value.get("value", {}).get("value") != expected:
                    raise RuntimeError(f"{name}/{backend} returned an incorrect result")
        programs = output / "programs"
        programs.mkdir()
        for family in ["collections", "document", "numerical"]:
            for changed in [False, True]:
                suffix = "_changed" if changed else ""
                root = family + suffix if family != "document" else "document"
                dependencies = ["sequences"]
                if family == "document":
                    dependencies += [f"document_{part}{suffix}" for part in ["types", "validate", "transform"]]
                source_args = [f"examples/v1/{root}.lobo"]
                for dependency in dependencies:
                    source_args += ["--source", f"examples/v1/{dependency}.lobo"]
                name = family + suffix
                checked = programs / f"{name}.loboc"
                run(f"{name}-checked", [compiler, "compile", *source_args, "--emit", "checked", "-o", checked])
                entry = {"collections": "v1.collections.run", "document": "v1.doc.run", "numerical": "v1.numerical.run"}[family]
                invoke = ["--entry", entry, "--fuel", "2000000", "--heap-cells", "2000000"]
                if family == "document":
                    items = "Count:3,I64:1,I64:3,Bool:true,Text:6b656570,I64:2,I64:4,Bool:false,Text:6b656570,I64:3,I64:-1,Bool:true,Text:736b6970" if changed else "Count:3,I64:1,I64:3,Bool:true,I64:2,I64:4,Bool:false,I64:3,I64:-1,Bool:true"
                    invoke += ["--arg", f"Record:v1.doc.document:I64:7,Text:c3a9,{items}"]
                if family == "numerical":
                    invoke += ["--arg", "List:Count:3,F64:1.5,F64:2.25," + ("F64:0.0" if changed else "F64:-0.75")]
                reference = json.loads(run(f"{name}-eval", [compiler, "eval", checked, *invoke]))
                if reference.get("status") != "ok":
                    raise RuntimeError(f"{name} evaluator failed")
                if family == "collections":
                    expected = record("v1.collections.report", {"count": number(5000), "first": number(19998 if changed else 19996), "last": number(2 if changed else 0), "sum": number(50000000 if changed else 49990000)})
                    if reference["value"] != expected:
                        raise RuntimeError(f"{name} violated the frozen integer oracle")
                elif family == "numerical":
                    expected = record("v1.numerical.stats", {"count": number(3), "sum": floating("0x400e000000000000" if changed else "0x4008000000000000"), "mean": floating("0x3ff4000000000000" if changed else "0x3ff0000000000000"), "variance": floating("0x3fec000000000000" if changed else "0x3ffa000000000000")})
                    if reference["value"]["value"]["case"] != "Ok" or reference["value"]["value"]["payload"] != expected:
                        raise RuntimeError(f"{name} violated the frozen float oracle")
                else:
                    result = reference["value"]["value"]
                    report = result["payload"]["value"]["fields"]
                    document = report["document"]["value"]["fields"]
                    items = document["items"]["value"]["elements"]
                    observed = {"case": result["case"], "count": report["count"], "total": report["total"],
                                "id": document["id"], "name": document["name"],
                                "keys": [item["value"]["fields"]["key"] for item in items],
                                "scores": [item["value"]["fields"]["score"] for item in items]}
                    expected = {"case": "Ok", "count": number(1 if changed else 2), "total": number(6 if changed else 4),
                                "id": number(7), "name": {"type": "Text", "value": "checked:é"},
                                "keys": list(map(number, [1] if changed else [1, 3])),
                                "scores": list(map(number, [6] if changed else [6, -2]))}
                    if observed != expected:
                        raise RuntimeError(f"{name} violated the frozen document oracle")
                for backend in ["c", "llvm"]:
                    binary = programs / f"{name}-{backend}"
                    run(f"{name}-{backend}-compile", [compiler, "compile", *source_args, "--backend", backend, "-o", binary])
                    if binary.read_bytes()[:4] != b"\x7fELF":
                        raise RuntimeError(f"{binary} is not a machine-code ELF executable")
                    actual = json.loads(run(f"{name}-{backend}-execute", [binary, *invoke]))
                    if actual != reference:
                        raise RuntimeError(f"{name}/{backend} differs from evaluator")
                    manifest["acceptance"].append({"program": name, "backend": backend, "binary": str(binary.relative_to(output)), "bytes": binary.stat().st_size, "sha256": digest(binary), "result": actual})
        if inputs() != manifest["inputs"]:
            raise RuntimeError("release inputs changed during verification; no release accepted")
        with tarfile.open(output / "source.tar.gz", "w:gz") as archive:
            for path in manifest["inputs"]:
                archive.add(ROOT / path, arcname=path, recursive=False)
        manifest["artifacts"] = [{"path": str(path.relative_to(output)), "bytes": path.stat().st_size, "sha256": digest(path)} for path in sorted(output.rglob("*")) if path.is_file() and path.name != "manifest.json"]
        manifest["status"] = "passed"
        save()
        print(json.dumps({"status": "passed", "manifest": str(output / "manifest.json"), "rust_tests": manifest["rust_tests"], "editor_tests": manifest["editor_tests"]}), flush=True)
    except Exception as error:
        manifest["status"] = "failed"
        manifest["error"] = str(error)
        save()
        raise


if __name__ == "__main__":
    main()
