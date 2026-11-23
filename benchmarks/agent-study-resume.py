#!/usr/bin/env python3
"""Recover an interrupted study without replacing its interrupted model attempt."""
import argparse
import hashlib
import importlib.util
import json
import pathlib
import shutil
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("--interrupted-root", required=True, type=pathlib.Path)
    args = parser.parse_args()
    harness = pathlib.Path(__file__).with_name("agent-repair-study.py")
    report = json.loads(args.report.read_text())
    if hashlib.sha256(harness.read_bytes()).hexdigest() != report["identities"]["harness.py"]:
        raise ValueError("Frozen trial harness changed")
    spec = importlib.util.spec_from_file_location("study", harness)
    study = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(study)
    for label, path in [("pilot.py", pathlib.Path(study.pilot.__file__))]:
        if hashlib.sha256(path.read_bytes()).hexdigest() != report["identities"][label]:
            raise ValueError("Frozen dependency changed")
    if study.pilot.digest(study.pilot.RUNNER.read_bytes()) != report["runner_sha256"]:
        raise ValueError("Native runner changed")
    artifacts = pathlib.Path(report["artifact_directory"])
    frozen = json.loads((artifacts/"inputs.json").read_text())
    current = {n: {"root": t["root"], "sources": t["sources"], "visible_cases": study.cases(n),
                   "acceptance_cases": study.cases(n, heldout=True)} for n, t in study.TASKS.items()}
    if current != frozen:
        raise ValueError("Frozen fixtures changed")
    if len(report["trials"]) != 19 or report.get("recovery"):
        raise ValueError("This recovery is specifically for the observed interruption after 19 trials")
    root = args.interrupted_root
    if root.parent != pathlib.Path("/tmp") or not root.name.startswith("lobo-study-") or root.is_symlink():
        raise ValueError("Unexpected owned temporary directory")
    index = 19
    repeat, name, arm = study.schedule()[index]
    destination = artifacts/f"{index:02d}-{repeat}-{name}-{arm}"
    if not destination.is_dir() or not (destination/"events.jsonl").exists():
        raise ValueError("Missing interrupted attempt evidence")
    events = [json.loads(line) for line in (destination/"events.jsonl").read_text().splitlines()]
    row = {"repeat": repeat, "task": name, "arm": arm, "accepted": False, "timeout": False,
           "elapsed_seconds": None, "latency_censored": True,
           "infrastructure_error": "Controller terminated by SIGTERM (exit 143) mid-trial; cause unknown; no retry.",
           "host_observation_log_available": False, "usage": study.pilot.read_usage(events),
           "events": [], "command": study.pilot.command(root)}
    sources = study.read_sources(root, study.TASKS[name])
    for file, source in sources.items():
        (destination/file).write_text(source)
    row["final_source_sha256"] = {f: study.pilot.digest(s.encode()) for f, s in sources.items()}
    with study.Session(study.TASKS[name], study.cases(name, heldout=True)) as judge:
        row["grade"] = judge.verify(sources, "all")
    row["fixture_integrity"] = ((root/"TASK.txt").read_text() == study.guide(name)
                                and (root/"check.py").read_text() == study.CLIENT.format(address=str(root/".check-io")))
    row["unexpected_files"] = []
    (destination/"report.json").write_text(json.dumps(row, indent=2))
    (artifacts/"results.before-recovery.json").write_bytes(args.report.read_bytes())
    (artifacts/"recovery.py").write_bytes(pathlib.Path(__file__).read_bytes())
    report["recovery"] = {"interrupted_index": index, "no_replacement": True,
                           "controller_sha256": study.pilot.digest(pathlib.Path(__file__).read_bytes()),
                           "elapsed_unknown": True}
    report["analysis_last_complete_prefix"] = report.pop("analysis")
    report["trials"].append(row)
    args.report.write_text(json.dumps(report, indent=2))
    shutil.rmtree(root)  # Exact session-owned synthetic directory, copied above.
    for i, (repeat, name, arm) in enumerate(study.schedule()[20:], 20):
        print(f"Continuing {i+1}/30 repeat={repeat} {name} {arm}", flush=True)
        start = time.monotonic()
        try:
            row = study.trial(i, repeat, name, arm, artifacts)
        except Exception as exc:
            row = {"repeat": repeat, "task": name, "arm": arm, "accepted": False, "timeout": False,
                   "usage": None, "events": [], "infrastructure_error": str(exc)[:1024],
                   "elapsed_seconds": time.monotonic()-start}
        report["trials"].append(row)
        args.report.write_text(json.dumps(report, indent=2))
        print(f"Finished accepted={row['accepted']} seconds={row['elapsed_seconds']:.2f}", flush=True)


if __name__ == "__main__":
    main()
