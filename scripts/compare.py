#!/usr/bin/env python3
"""Build and measure all three execution modes from the same checkout."""

import argparse
import json
import os
from pathlib import Path
import statistics
import subprocess
import time


def output(command, root):
    return subprocess.check_output(command, cwd=root, text=True).strip()


def expected_checksum(workload, count):
    cycle, remainder = divmod(count, 2048)
    values = [x + 1 if workload == "increment" else abs(x) for x in range(-1024, 1024)]
    return str((cycle * sum(values) + sum(values[:remainder])) % (1 << 64))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--samples", type=int, default=9)
    parser.add_argument("--warmup", type=int, default=2)
    parser.add_argument("--native-iterations", type=int, default=10_000_000)
    parser.add_argument("--interpreted-iterations", type=int, default=100_000)
    parser.add_argument("--cpu", type=int)
    options = parser.parse_args()
    if options.samples < 1 or options.warmup < 0 or not all(
        1 <= n <= 1_000_000_000 for n in (options.native_iterations, options.interpreted_iterations)
    ):
        parser.error("invalid sample counts or iteration budget")

    root = Path(__file__).resolve().parent.parent
    # Pin this runner and its children only; do not change the invoking shell's affinity.
    cpu = None
    if hasattr(os, "sched_getaffinity"):
        available = os.sched_getaffinity(0)
        cpu = min(available) if options.cpu is None else options.cpu
        if cpu not in available:
            parser.error("requested CPU is outside the available affinity set")
        os.sched_setaffinity(0, {cpu})
    elif options.cpu is not None:
        parser.error("CPU affinity is unavailable on this platform")

    revision = output(["git", "rev-parse", "HEAD"], root)
    dirty = bool(output(["git", "status", "--porcelain", "--untracked-files=normal"], root))
    builds = output([
        "cargo", "bench", "--locked", "--bench", "compare", "--no-run", "--message-format=json"
    ], root)
    executables = []
    for line in builds.splitlines():
        item = json.loads(line)
        if item.get("reason") == "compiler-artifact" and item["target"]["name"] == "compare" and item.get("executable"):
            executables.append(item["executable"])
    if len(executables) != 1:
        raise RuntimeError("expected exactly one comparison executable")
    benchmark = executables[0]
    directory = root / "target" / "comparisons" / str(time.time_ns())
    directory.mkdir(parents=True)
    subprocess.run([benchmark, "emit-c", str(directory / "core.c")], cwd=root, check=True)
    native = directory / "native-bench"
    subprocess.run([
        "clang", "-std=c11", "-O3", "-g0", "-I", str(directory),
        str(root / "benchmarks" / "native.c"), "-o", str(native)
    ], cwd=root, check=True)

    modes = ("native", "rust", "interpreted")
    report = {
        "format": "lobo-execution-comparison/0.1",
        "source_revision": revision,
        "source_dirty": dirty,
        "cpu_affinity": cpu,
        "cpu_frequency_locked": False,
        "rustc": output(["rustc", "--version"], root),
        "clang": output(["clang", "--version"], root).splitlines()[0],
        "samples": options.samples,
        "warmup_samples": options.warmup,
        "scope": "in-process iterations, including loop and checksum; excluding build, startup, parsing, and output",
        "interpreted_scope": "Artifact.run includes per-call semantic revalidation, argument allocation, and evaluation",
        "workloads": {},
    }
    for workload in ("increment", "absolute"):
        samples = {mode: [] for mode in modes}
        counts = {
            "native": options.native_iterations,
            "rust": options.native_iterations,
            "interpreted": options.interpreted_iterations,
        }
        for repetition in range(options.warmup + options.samples):
            offset = repetition % len(modes)
            for mode in modes[offset:] + modes[:offset]:
                command = [str(native), workload, str(counts[mode])] if mode == "native" else [
                    benchmark, mode, workload, str(counts[mode])
                ]
                result = json.loads(output(command, root))
                if result["iterations"] != counts[mode] or result["checksum"] != expected_checksum(workload, counts[mode]):
                    raise RuntimeError(f"incorrect output for {mode}/{workload}")
                if repetition >= options.warmup:
                    samples[mode].append(result["ns_per_call"])
        report["workloads"][workload] = {
            mode: {
                "iterations_per_sample": counts[mode],
                "median_ns": statistics.median(samples[mode]),
                "min_ns": min(samples[mode]),
                "max_ns": max(samples[mode]),
                "samples_ns": samples[mode],
                "checksum": expected_checksum(workload, counts[mode]),
            } for mode in modes
        }
    destination = directory / "report.json"
    with destination.open("x") as handle:
        json.dump(report, handle, indent=2)
        handle.write("\n")
    report["report_path"] = str(destination)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
