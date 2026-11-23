#!/usr/bin/env python3
"""Export compact study evidence and correct a source-printing timeout-counter false positive."""
import argparse
import copy
import hashlib
import importlib.util
import json
import pathlib
import re


def audit_events(events, task_root):
    paths, outside_changes = set(), []
    for event in events:
        if event.get("type") != "item.completed":
            continue
        item = event.get("item", {})
        if item.get("type") == "command_execution":
            for path in re.findall(r"/(?:home|root|tmp|etc|workspace|mnt)/[^\s'\";<>]+", item["command"]):
                if not pathlib.Path(path).is_relative_to(task_root):
                    paths.add(path)
        if item.get("type") == "file_change":
            outside_changes.extend(c for c in item["changes"] if pathlib.Path(c["path"]).parent != task_root)
    return {"external_path_references": sorted(paths), "outside_file_changes": outside_changes,
            "limit": "Heuristic path references plus manual command review; not containment or a shell parser."}


def transport_deadlines(events):
    return sum(e.get("type") == "item.completed" and e.get("item", {}).get("type") == "command_execution"
               and e["item"].get("exit_code") not in [0, None]
               and "Check adapter deadline" in e["item"].get("aggregated_output", "").splitlines()
               for e in events)


def compact_event(event):
    result = copy.deepcopy(event)
    if event["args"]:
        # Exact detail/list sources and observations remain in the private raw report/events.
        payload = result.get("result", {})
        for field in ["bundle", "tests"]:
            if field in payload:
                data = json.dumps(payload.pop(field), sort_keys=True).encode()
                payload[field+"_sha256"] = hashlib.sha256(data).hexdigest()
    return result


def reanalyze(study, rows):
    # Preserve all correctness denominators; missing latency is not zero or a known deadline.
    numeric = [dict(r, elapsed_seconds=r["elapsed_seconds"] if r["elapsed_seconds"] is not None else 0)
               for r in rows]
    analysis = study.analyze(numeric)
    for key, group in analysis["by_task_arm"].items():
        task, arm = key.split("/")
        subset = [r for r in rows if (r["task"], r["arm"]) == (task, arm)]
        group["unknown_latency_trials"] = sum(r["elapsed_seconds"] is None for r in subset)
        group["elapsed_seconds_all"] = study.spread([r["elapsed_seconds"] for r in subset if r["elapsed_seconds"] is not None])
        group["elapsed_seconds_accepted"] = study.spread([r["elapsed_seconds"] for r in subset
                                                         if r["accepted"] and r["elapsed_seconds"] is not None])
    for key, pair in analysis["paired"].items():
        task, comparison = key.split("/")
        after, before = comparison.split("-minus-")
        deltas = []
        for repeat in range(5):
            a = next((r for r in rows if (r["repeat"], r["task"], r["arm"]) == (repeat, task, before)), None)
            b = next((r for r in rows if (r["repeat"], r["task"], r["arm"]) == (repeat, task, after)), None)
            if a and b and a["elapsed_seconds"] is not None and b["elapsed_seconds"] is not None:
                deltas.append(b["elapsed_seconds"]-a["elapsed_seconds"])
        pair["all_elapsed_deltas_seconds"] = deltas
        pair["known_latency_pairs"] = len(deltas)
    return analysis


def export(source, destination):
    report = json.loads(source.read_text())
    if report["canary"] or len(report["trials"]) != 30:
        raise ValueError("Export requires all 30 registered study trials, excluding canaries")
    artifacts = pathlib.Path(report["artifact_directory"])
    harness = pathlib.Path(__file__).with_name("agent-repair-study.py")
    if hashlib.sha256(harness.read_bytes()).hexdigest() != report["identities"]["harness.py"]:
        raise ValueError("Study harness drift: analyze using the matching source")
    spec = importlib.util.spec_from_file_location("study", harness)
    study = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(study)
    expected = study.schedule()
    if [(r["repeat"], r["task"], r["arm"]) for r in report["trials"]] != expected:
        raise ValueError("Unexpected trial schedule")
    corrections = []
    frozen_inputs = json.loads((artifacts/"inputs.json").read_text())
    for i, row in enumerate(report["trials"]):
        directory = artifacts/f"{i:02d}-{row['repeat']}-{row['task']}-{row['arm']}"
        events = [json.loads(line) for line in (directory/"events.jsonl").read_text().splitlines()]
        original = row.get("transport_errors")
        corrected = transport_deadlines(events)
        if original != corrected:
            corrections.append({"trial": i, "original": original, "corrected": corrected})
        row["transport_errors_original"] = original
        row["transport_errors"] = corrected
        task_root = pathlib.Path(row["command"][row["command"].index("-C")+1])
        row["trace_audit"] = audit_events(events, task_root)
        verifies = [e for e in row["events"] if e["args"] == []]
        initial = {k: hashlib.sha256(v.encode()).hexdigest() for k, v in frozen_inputs[row["task"]]["sources"].items()}
        row["trace_verified_interface"] = (len(verifies) >= 2 and verifies[0].get("source_sha256") == initial
                                           and verifies[-1].get("source_sha256") == row.get("final_source_sha256")
                                           and verifies[-1].get("result", {}).get("status") == "checks_satisfied")
        row["workflow_adherent"] = (row.get("fixture_integrity", False) and not row.get("unexpected_files")
                                    and not row["trace_audit"]["external_path_references"]
                                    and not row["trace_audit"]["outside_file_changes"]
                                    and row["trace_verified_interface"])
        row["events"] = [compact_event(e) for e in row["events"]]
        row["raw_events_sha256"] = hashlib.sha256((directory/"events.jsonl").read_bytes()).hexdigest()
    original = report.pop("analysis", None)
    report["analysis_original"] = original if original is not None else {
        "prefix_only": True, "trials": 19, "analysis": report.pop("analysis_last_complete_prefix")}
    report["analysis"] = reanalyze(study, report["trials"])
    report["export"] = {
        "raw_report_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "reporter_sha256": hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
        "compaction": "Large list/source bundles replaced by hashes; exact data retained in private artifacts.",
        "counter_correction": "Count actual nonzero command outputs containing the standalone deadline line, not successful prints of client source.",
        "corrected_trials": corrections}
    destination.write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps({"trials": len(report["trials"]), "accepted": sum(t["accepted"] for t in report["trials"]),
                      "counter_corrections": corrections, "analysis": report["analysis"]}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=pathlib.Path)
    parser.add_argument("destination", type=pathlib.Path)
    args = parser.parse_args()
    export(args.source, args.destination)


if __name__ == "__main__":
    main()
