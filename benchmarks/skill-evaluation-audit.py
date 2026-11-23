#!/usr/bin/env python3
"""Separate trace audit; preserve the frozen experiment and its primary outcomes."""
import argparse
import hashlib
import json
import pathlib
import re


def inspect(events, task_root):
    commands = [e["item"]["command"] for e in events if e.get("type") == "item.completed"
                and e.get("item", {}).get("type") == "command_execution"]
    probes = [c for c in commands if re.search(r"\.\./|(?:^|\s)\.\.(?:[;\s'\"/]|$)|/tmp/AGENTS\.md|/AGENTS\.md", c)]
    changes = [change for e in events if e.get("type") == "item.completed"
               and e.get("item", {}).get("type") == "file_change" for change in e["item"]["changes"]]
    outside = [c for c in changes if pathlib.Path(c["path"]).parent != task_root
               or pathlib.Path(c["path"]).name not in ["program.lobo", "helpers.lobo"]]
    return {"parent_scope_probes": probes, "outside_file_changes": outside,
            "lobo_reference_commands": [c for c in commands if ".agents/skills/lobo-programming/references/" in c],
            "mcp_calls": sum(e.get("type") == "item.completed" and e.get("item", {}).get("type") == "mcp_tool_call" for e in events)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("destination", type=pathlib.Path)
    args = parser.parse_args()
    report = json.loads(args.report.read_text())
    root = pathlib.Path(report["artifact_directory"])
    original = json.loads((root/"results.json").read_text())
    rows = []
    for i, row in enumerate(report["trials"]):
        file = root/f"{i:02d}-{row['repeat']}-{row['task']}-{row['arm']}"/"events.jsonl"
        events = [json.loads(line) for line in file.read_text().splitlines()]
        task_root = pathlib.Path(row["command"][row["command"].index("-C")+1])
        audit = inspect(events, task_root)
        verifies = [e for e in row["events"] if e["args"] == []]
        initial = {n: hashlib.sha256(t.encode()).hexdigest() for n, t in original["tasks"][row["task"]]["sources"].items()}
        client = (len(verifies) >= 2 and verifies[0].get("source_sha256") == initial
                  and verifies[-1].get("source_sha256") == row["final_source_sha256"]
                  and verifies[-1]["result"]["status"] == "checks_satisfied")
        audit.update({"index": i, "task": row["task"], "arm": row["arm"], "client_workflow_verified": client,
                      "raw_events_sha256": hashlib.sha256(file.read_bytes()).hexdigest(),
                      "workflow_adherent": client and row["fixture_integrity"] and not row["unexpected_files"]
                                           and not audit["parent_scope_probes"] and not audit["outside_file_changes"]
                                           and not audit["mcp_calls"]})
        rows.append(audit)
    output = {"format": "lobo-skill-evaluation-audit/0.1",
              "report_sha256": hashlib.sha256(args.report.read_bytes()).hexdigest(),
              "auditor_sha256": hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
              "limitations": "Heuristic path/parent probes plus command review, not a shell parser or containment proof. No trial excluded or primary acceptance changed.",
              "trials": rows}
    args.destination.write_text(json.dumps(output, indent=2)+"\n")
    print(json.dumps({"adherent": sum(r["workflow_adherent"] for r in rows),
                      "scope_probe_indices": [r["index"] for r in rows if r["parent_scope_probes"]]}))


if __name__ == "__main__":
    main()
