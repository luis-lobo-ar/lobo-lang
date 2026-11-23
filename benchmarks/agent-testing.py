#!/usr/bin/env python3
"""Scripted full-vs-affected native verification, not an agent-efficiency claim."""
import argparse
import json
import pathlib
import subprocess
import hashlib
import platform


def envelope(action):
    return {"format": "lobo-test/0.1", "action": action}


SOURCE = '''module workload @id("bench.testing");
function helper @id("helper")(n: I64) -> I64 { n + 1 }
function guard @id("guard")(n: I64) -> Bool { true }
function a @id("a")(n: I64) -> I64 { helper(n) }
function b @id("b")(n: I64) -> I64 { helper(n) + 1 }
function c @id("c")(n: I64) -> I64 requires { guard(n) } { n + 3 }
function d @id("d")(n: I64) -> I64 { n + 4 }
function unused @id("unused")(n: I64) -> I64 { n + 5 }
'''


def cases():
    return [{"id": f"{entry}.{n}", "requirement": entry, "entry": entry,
             "arguments": [f"I64:{n}"],
             "expect": {"kind": "value", "value": {"type": "I64", "value": str(n + offset)}},
             "fuel": 1000, "heap_cells": 1000}
            for offset, entry in enumerate("abcd", 1) for n in range(16)]


def verify(source, scope):
    return {"op": "verify", "base": "s2", "root": "workload",
            "sources": [{"name": "workload", "text": source}],
            "environment": "local-scripted-comparison-v1", "scope": scope,
            "max_cases": 256, "wall_ms": 15000}


def compare(runner, clang, name, candidate):
    tests = cases()
    plan = {"op": "open", "root": "workload", "sources": [{"name": "workload", "text": SOURCE}],
            "requirements": [{"id": entry, "description": f"{entry} adds its declared offset"} for entry in "abcd"],
            "tests": tests}
    baseline = verify(SOURCE, "all")
    baseline["base"] = "s1"
    actions = [plan, baseline, verify(candidate, "affected"), verify(candidate, "all")]
    for snapshot in ["s3", "s4"]:
        actions.extend({"op": "explain", "snapshot": snapshot, "test": case["id"]} for case in tests)
    actions.append({"op": "close"})
    result = subprocess.run([str(runner), "--session", "--clang", str(clang)],
                            input="".join(json.dumps(envelope(action), separators=(",", ":")) + "\n" for action in actions),
                            text=True, capture_output=True, timeout=60, check=True)
    if result.stderr:
        raise RuntimeError(result.stderr)
    lines = result.stdout.splitlines()
    outcomes = [json.loads(line) for line in lines]
    if outcomes[1]["status"] != "checks_satisfied":
        raise AssertionError(outcomes[1])
    affected, full = outcomes[2:4]
    if affected["counts"]["pending"] or full["counts"]["pending"]:
        raise AssertionError("Comparison did not finish all required checks")
    states = outcomes[4:4 + 2 * len(tests)]
    for left, right in zip(states[:len(tests)], states[len(tests):]):
        if left["result"]["status"] != right["result"]["status"] or left["result"]["observed"] != right["result"]["observed"]:
            raise AssertionError({"scenario": name, "affected": left, "full": right})
    return {"scenario": name, "checks": len(tests),
            "affected": {"counts": affected["counts"], "elapsed_ms": affected["elapsed_ms"], "response_bytes": len(lines[2].encode())},
            "full": {"counts": full["counts"], "elapsed_ms": full["elapsed_ms"], "response_bytes": len(lines[3].encode())},
            "all_case_outcomes_equal": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=pathlib.Path, default=pathlib.Path("target/release/lobo-test"))
    parser.add_argument("--clang", type=pathlib.Path, default=pathlib.Path("/usr/bin/clang"))
    args = parser.parse_args()
    candidates = {
        "comments_only": SOURCE + "// new comment\n",
        "unreachable_function_edit": SOURCE.replace("n + 5", "n + 6"),
        "leaf_regression": SOURCE.replace("n + 4", "n + 40"),
        "transitive_regression": SOURCE.replace("n + 1", "n + 10"),
        "contract_only_regression": SOURCE.replace("{ true }", "{ false }"),
        "global_layout_addition": SOURCE + 'record Unused @id("unused.type") { n: I64 }\n',
    }
    report = {"format": "lobo-test-comparison/0.1", "kind": "scripted_native_selection",
              "runner": str(args.runner.resolve()), "clang": str(args.clang.resolve()),
              "runner_sha256": hashlib.sha256(args.runner.read_bytes()).hexdigest(),
              "source_sha256": hashlib.sha256(SOURCE.encode()).hexdigest(),
              "cases_sha256": hashlib.sha256(json.dumps(cases(), sort_keys=True).encode()).hexdigest(),
              "platform": platform.platform(),
              "clang_version": subprocess.run([str(args.clang), "--version"], capture_output=True, text=True, check=True, timeout=5).stdout.splitlines()[0],
              "limitations": "One local run per scenario. Full and affected use the same runner/compiler; checks have independent declared expectations. No agent tokens or repair success measured.",
              "results": [compare(args.runner.resolve(), args.clang, name, candidate) for name, candidate in candidates.items()]}
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
