#!/usr/bin/env python3
"""Compare C and direct LLVM native routes; no model calls or oracle from candidate output."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import resource
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
COMPILER = ROOT / "target/release/lobo"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def summary(values):
    return {"median_ms": statistics.median(values), "min_ms": min(values),
            "max_ms": max(values), "samples": len(values)}


def cases():
    count = 2048
    parser_input = "+".join(["1"] * count).encode().hex()
    return [
        {"name": "sum", "source": "examples/iteration.lobo", "entry": "example.sum",
         "arguments": ["I64:5000000"], "expected": {"type": "I64", "value": "12500002500000"}},
        {"name": "parser", "source": "examples/parser.lobo", "entry": "example.parser.evaluate",
         "arguments": [f"Bytes:{parser_input}"], "expected": {"type": "Variant", "value": {
             "id": "example.parser.answer", "case": "Ok", "payload": {"type": "I64", "value": str(count)}}}},
        {"name": "ast", "source": "examples/ast.lobo", "entry": "example.ast.shifted_value",
         "arguments": ["Variant:example.ast.node:Case:Sum,Count:3,Case:Num,I64:2,Case:Product,Count:2,Case:Num,I64:3,Case:Num,I64:4,Case:Sum,Count:0", "I64:1"],
         "expected": {"type": "I64", "value": "23"}},
        {"name": "generic", "source": "examples/generics.lobo", "entry": "example.generics.text",
         "arguments": ["Text:c3a9"], "expected": {"type": "Text", "value": "é"}},
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="New evidence directory")
    parser.add_argument("--compile-repeats", type=int, default=6)
    parser.add_argument("--run-repeats", type=int, default=11)
    options = parser.parse_args()
    if options.compile_repeats < 5 or options.run_repeats < 5:
        parser.error("A backend comparison needs at least five observations per side")
    options.out.mkdir(mode=0o700)
    out = options.out.resolve()
    cpu = min(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {cpu})
    environment = {
        "platform": platform.platform(), "cpu_affinity": [cpu], "cpu_count": os.cpu_count(),
        "load_before": os.getloadavg(), "compiler_sha256": sha(COMPILER), "script_sha256": sha(__file__),
        "tools": {name: subprocess.check_output([name, "--version"], text=True).splitlines()[0]
                  for name in ["clang", "clang-18", "llvm-link-18", "opt-18"]},
    }
    runs = []
    jobs = cases()
    for job in jobs:
        job["source_sha256"] = sha(ROOT / job["source"])
    (out / "plan.json").write_text(json.dumps({"environment": environment, "cases": jobs,
        "compile_repeats": options.compile_repeats, "run_repeats": options.run_repeats}, indent=2) + "\n")

    def observe(command, phase, job, backend, trial, expected=None, timed=True):
        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        start = time.perf_counter_ns()
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        elapsed = (time.perf_counter_ns() - start) / 1e6
        after = resource.getrusage(resource.RUSAGE_CHILDREN)
        try:
            response = json.loads(result.stdout)
        except ValueError:
            response = None
        valid = (result.returncode == 0 and not result.stderr and response is not None
                 and response.get("status") == "ok")
        if expected is not None:
            valid = valid and response.get("value") == expected
        record = {"phase": phase, "case": job["name"], "backend": backend, "trial": trial,
                  "command": command, "wall_ms": elapsed, "timed": timed, "valid": valid,
                  "child_user_ms": (after.ru_utime - before.ru_utime) * 1000,
                  "child_system_ms": (after.ru_stime - before.ru_stime) * 1000,
                  "exit_code": result.returncode, "response": response, "stderr": result.stderr}
        runs.append(record)
        with (out / "observations.jsonl").open("a") as stream:
            stream.write(json.dumps(record) + "\n")
        if not valid:
            raise RuntimeError(f"Invalid {phase} outcome; preserved at {out}")
        return record

    artifacts = {}
    for job in jobs:
        for trial in range(options.compile_repeats):
            order = ["c", "llvm"] if trial % 2 == 0 else ["llvm", "c"]
            for backend in order:
                binary = out / f"{job['name']}-{backend}-{trial}"
                record = observe([str(COMPILER), "compile", job["source"], "--backend", backend,
                                  "-o", str(binary)], "compile", job, backend, trial)
                record.update({"binary": str(binary), "bytes": binary.stat().st_size, "sha256": sha(binary)})
                artifacts[job["name"], backend] = binary
        # Warm both routes, then measure whole native invocations, including startup, parsing and JSON.
        for trial in range(-1, options.run_repeats):
            order = ["c", "llvm"] if trial % 2 == 0 else ["llvm", "c"]
            for backend in order:
                command = [str(artifacts[job["name"], backend]), "--entry", job["entry"],
                           "--fuel", "200000000", "--heap-cells", "5000000"]
                for argument in job["arguments"]:
                    command += ["--arg", argument]
                observe(command, "run", job, backend, trial, job["expected"], timed=trial >= 0)
    report = {"environment": environment, "cases": jobs, "observations": runs,
              "failures": sum(not r["valid"] for r in runs), "summaries": [],
              "limits": ["Public development workloads, not agent/comprehension measures.",
                         "Warm local toolchain/filesystem; no cold-cache claim. Other host workloads remain.",
                         "Native invocation timing includes process startup, input parsing and JSON output.",
                         "Single-CPU affinity; same optimized compiler, flags, budgets and oracle per pair.",
                         "Runtime medians near the startup/noise floor cannot establish computation speed."]}
    for job in jobs:
        for phase in ["compile", "run"]:
            row = {"case": job["name"], "phase": phase}
            for backend in ["c", "llvm"]:
                selected = [r for r in runs if r["case"] == job["name"] and r["phase"] == phase
                            and r["backend"] == backend and r["timed"]]
                row[backend] = summary([r["wall_ms"] for r in selected])
            report["summaries"].append(row)
    report["environment"]["load_after"] = os.getloadavg()
    (out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"evidence": str(out), "failures": report["failures"], "summaries": report["summaries"]}, indent=2))


if __name__ == "__main__":
    main()
