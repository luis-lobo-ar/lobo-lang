#!/usr/bin/env python3
"""Bounded real-agent feedback pilot. See decision 0025 before interpreting results."""
import argparse
import hashlib
import json
import os
import pathlib
import platform
import select
import signal
import statistics
import subprocess
import tempfile
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target/release/lobo-test"
CLANG = pathlib.Path("/usr/bin/clang")
MODEL = "gpt-6.1-sol"
LIMIT = 90
MIN, MAX = -(2**63), 2**63-1


def task(signature, body, fixed, description, result="I64"):
    prefix = 'module repair @id("repair");\nfunction target @id("target")(' + signature + ") -> " + result + " { "
    suffix = ' }\nfunction control @id("control")(n: I64) -> I64 { n * 2 }\n'
    return {"source": prefix + body + suffix, "fixed": prefix + fixed + suffix,
            "description": description, "result": result}


TASKS = {
    "clamp": task("x: I64, lo: I64, hi: I64",
                  "if x < lo { lo } else { if x > hi { lo } else { x } }",
                  "if x < lo { lo } else { if x > hi { hi } else { x } }",
                  "For lo <= hi, return x clamped to the inclusive interval [lo, hi]."),
    "overlap": task("a: I64, b: I64, c: I64, d: I64", "a < d && c < b",
                    "a <= d && c <= b",
                    "For closed intervals [a,b] and [c,d], a<=b and c<=d, return whether they intersect. Touching endpoints intersect.", "Bool"),
    "floor_div": task("a: I64, b: I64", "a / b",
                      "let q = a / b; if a % b != 0 && ((a < 0) != (b < 0)) { q - 1 } else { q }",
                      "Return mathematical floor(a/b) for nonzero b. Inputs exclude a=I64_MIN,b=-1. All other I64 inputs are valid."),
    "ceil_div": task("n: I64, d: I64", "(n + d - 1) / d",
                     "let q = n / d; if n % d == 0 { q } else { q + 1 }",
                     "For n>=0 and d>0, return mathematical ceil(n/d). Every such I64 input is valid, including I64_MAX. Avoid intermediate overflow."),
    "distance": task("a: I64, b: I64", "a - b", "if a < b { b - a } else { a - b }",
                     "Return absolute mathematical distance |a-b|. Inputs guarantee that the distance fits in I64. Signed inputs and either ordering are valid."),
}


def inputs(name, heldout=False):
    if name == "clamp":
        visible = [(-1, 0, 10), (0, 0, 10), (5, 0, 10), (10, 0, 10), (11, 0, 10), (9, 3, 3)]
        extra = [(x, lo, hi) for lo, hi in [(MIN, MAX), (-7, -2), (0, 0), (MAX, MAX)]
                 for x in [MIN, -8, -2, 0, 1, 6, MAX]]
    elif name == "overlap":
        visible = [(0, 2, 2, 4), (0, 1, 2, 3), (0, 4, 1, 2), (2, 2, 2, 2), (-4, -2, -2, 0), (1, 3, 0, 1)]
        intervals = [(MIN, MIN), (MIN, MAX), (-3, -1), (-1, 1), (0, 0), (2, 4), (MAX, MAX)]
        extra = [(a, b, c, d) for a, b in intervals for c, d in intervals]
    elif name == "floor_div":
        visible = [(-7, 3), (7, -3), (7, 3), (-7, -3), (-6, 3), (0, 2)]
        extra = [(a, b) for a in [MIN, MIN+1, -20, -1, 0, 1, 20, MAX]
                 for b in [MIN, -7, -1, 1, 7, MAX] if (a, b) != (MIN, -1)]
    elif name == "ceil_div":
        visible = [(0, 1), (1, 2), (7, 3), (6, 3), (MAX, 2), (MAX, MAX)]
        extra = [(n, d) for n in [0, 1, 2, 19, MAX-1, MAX] for d in [1, 2, 3, 17, MAX-1, MAX]]
    else:
        visible = [(1, 4), (4, 1), (-4, -1), (-1, -4), (0, 0), (MIN, MIN)]
        extra = [(a, b) for a in [MIN, MIN+1, -20, -1, 0, 1, 20, MAX]
                 for b in [MIN, -20, -1, 0, 1, 20, MAX] if abs(a-b) <= MAX]
    return list(dict.fromkeys(visible + (extra if heldout else [])))


def expected(name, args):
    if name == "clamp":
        x, lo, hi = args
        return min(max(x, lo), hi)
    if name == "overlap":
        a, b, c, d = args
        return max(a, c) <= min(b, d)
    a, b = args
    return {"floor_div": lambda: a // b, "ceil_div": lambda: -(-a // b),
            "distance": lambda: abs(a-b)}[name]()


def test_cases(name, heldout=False):
    cases = []
    for i, args in enumerate(inputs(name, heldout)):
        value = expected(name, args)
        cases.append({"id": f"target.{i}", "requirement": "repair", "entry": "target",
                      "arguments": [f"I64:{a}" for a in args],
                      "expect": {"kind": "value", "value": {"type": TASKS[name]["result"],
                                 "value": value if isinstance(value, bool) else str(value)}},
                      "fuel": 1000, "heap_cells": 1000})
    for n in range(4):
        cases.append({"id": f"control.{n}", "requirement": "control", "entry": "control",
                      "arguments": [f"I64:{n}"], "expect": {"kind": "value",
                      "value": {"type": "I64", "value": str(n*2)}}, "fuel": 1000, "heap_cells": 1000})
    return cases


def schedule():
    return [(name, arm) for i, name in enumerate(TASKS) for arm in
            (["conventional", "structured"] if i % 2 == 0 else ["structured", "conventional"])]


def read_usage(events):
    usages = [e["usage"] for e in events if e.get("type") == "turn.completed" and "usage" in e]
    return usages[-1] if usages else None


def read_source(root):
    path = root / "program.lobo"
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 65536:
        raise ValueError("Candidate must be a regular, non-symlink file at most 64 KiB")
    return path.read_text()


class Session:
    def __init__(self, definition, cases):
        self.definition, self.cases = definition, cases
        self.process = subprocess.Popen([str(RUNNER), "--session", "--clang", str(CLANG)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL)
        try:
            response = self.call({"op": "open", "root": "program",
                                  "sources": [{"name": "program", "text": definition["source"]}],
                                  "requirements": [{"id": "repair", "description": definition["description"]},
                                                   {"id": "control", "description": "control(n) is twice n"}],
                                  "tests": cases})
            if response["status"] != "opened":
                raise RuntimeError(response)
            self.snapshot = response["snapshot"]
        except BaseException:
            self.close()
            raise

    def call(self, action):
        self.process.stdin.write((json.dumps({"format": "lobo-test/0.1", "action": action}) + "\n").encode())
        self.process.stdin.flush()
        if not select.select([self.process.stdout], [], [], 35)[0]:
            raise TimeoutError("Runner response deadline")
        line = self.process.stdout.readline(8*1024*1024+1)
        if len(line) > 8*1024*1024 or not line.endswith(b"\n"):
            raise RuntimeError("Incomplete or oversized runner response")
        return json.loads(line)

    def verify(self, source, scope):
        response = self.call({"op": "verify", "base": self.snapshot, "root": "program",
                              "sources": [{"name": "program", "text": source}],
                              "environment": "local-agent-pilot-2026-11-03", "scope": scope,
                              "max_cases": 256, "wall_ms": 10000})
        if "snapshot" in response:
            self.snapshot = response["snapshot"]
        return response

    def close(self):
        if self.process.poll() is None:
            self.process.stdin.close()
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        self.process.stdout.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


GUIDE = """Lobo repair task. Only program.lobo may change. Preserve module, function IDs/signatures,
and control(n)=2*n. Repair target to satisfy REQUIREMENT below for all valid inputs.
No dependencies, networking, delegation, alternate compiler, or reading files outside this directory.
Do not change check.py or TASK.txt. Run python3 check.py before and after your repair.
It exits 0 only when visible checks pass; 1 means failed and 2 means infrastructure/invalid code.
python3 check.py --list shows the frozen visible cases. --detail TEST_ID explains the current
checked candidate. The final judge uses additional inputs satisfying the same requirement.

Language: module name @id("id"); function name @id("id")(x: I64) -> I64 { expression }
Bool values true/false. let name = expression; expression (immutable binding).
if condition { expression } else { expression }. Comparisons == != < <= > >=;
boolean && || !, arithmetic + - * / %. I64 arithmetic is checked and can fault on overflow.
Division truncates toward zero; remainder has the dividend sign. No mutation or return keyword.
I64 range [-9223372036854775808,9223372036854775807]. No loops are needed here.

REQUIREMENT: {requirement}
"""

CLIENT = """import json, os, pathlib, sys, time, uuid
directory = pathlib.Path({address!r})
token = uuid.uuid4().hex
request = {{"token": token, "args": sys.argv[1:]}}
(directory / "request.tmp").write_text(json.dumps(request))
os.replace(directory / "request.tmp", directory / "request.json")
deadline = time.monotonic() + 40
while time.monotonic() < deadline:
    try:
        response = json.loads((directory / "response.json").read_text())
        if response["token"] == token:
            print(response["text"])
            sys.exit(response["exit"])
    except (FileNotFoundError, json.JSONDecodeError):
        pass
    time.sleep(0.02)
print("Check adapter deadline", file=sys.stderr)
sys.exit(2)
"""


def guide_for(name):
    return GUIDE.replace("{requirement}", TASKS[name]["description"])


def digest(data):
    return hashlib.sha256(data).hexdigest()


def render(response, arm):
    if arm == "structured":
        return json.dumps(response, separators=(",", ":"))
    counts = response.get("counts")
    if counts is None:
        return "ERROR: " + json.dumps(response)
    lines = [f"Ran {counts['executed']} native tests; {counts['passed']} passed, "
             f"{counts['failed']} failed, {counts['pending']} pending. {response['status']}"]
    for failure in response.get("failures", []):
        lines.append("FAIL: " + json.dumps(failure, separators=(",", ":")))
    lines.append(f"Additional failures: {response.get('failures_omitted', 0)}")
    return "\n".join(lines)


class Adapter:
    """Experiment transport only; one sequential session, no general command execution."""
    def __init__(self, root, address, name, arm):
        self.root, self.name, self.arm = root, name, arm
        self.address = address
        self.address.mkdir(mode=0o700)
        self.session = Session(TASKS[name], test_cases(name))
        self.checks = []
        self.requests = []
        self.stopping = threading.Event()
        self.thread = threading.Thread(target=self.serve, daemon=True)
        self.thread.start()

    def serve(self):
        previous = None
        while not self.stopping.is_set():
            try:
                path = self.address / "request.json"
                if self.address.is_symlink() or path.is_symlink():
                    raise ValueError("Symlinked transport")
                if path.stat().st_size > 4096:
                    raise ValueError("Oversized request")
                request = json.loads(path.read_text())
                token = request["token"]
                if not isinstance(token, str) or len(token) != 32 or any(c not in "0123456789abcdef" for c in token):
                    raise ValueError("Invalid request token")
                if token == previous:
                    self.stopping.wait(0.02)
                    continue
                previous = token
                args = request["args"]
                self.requests.append(args)
            except (OSError, ValueError, KeyError, TypeError):
                self.stopping.wait(0.02)
                continue
            try:
                if args == ["--list"]:
                    response = {"text": json.dumps(self.session.cases), "exit": 0}
                elif isinstance(args, list) and len(args) == 2 and args[0] == "--detail":
                    detail = self.session.call({"op": "explain", "snapshot": self.session.snapshot, "test": args[1]})
                    response = {"text": json.dumps(detail), "exit": 0}
                elif args == []:
                    start = time.monotonic()
                    source = read_source(self.root)
                    result = self.session.verify(source, "affected" if self.arm == "structured" else "all")
                    text = render(result, self.arm)
                    code = 0 if result["status"] == "checks_satisfied" else 1 if result["status"] == "failed" else 2
                    response = {"text": text, "exit": code}
                    self.checks.append({"source_sha256": digest(source.encode()), "result": result,
                                        "elapsed_seconds": time.monotonic()-start,
                                        "feedback_bytes": len(text.encode())})
                else:
                    raise ValueError("Usage: check.py [--list | --detail TEST_ID]")
            except Exception as exc:
                response = {"text": "Adapter error: " + str(exc)[:1024], "exit": 2}
            response["token"] = token
            # Refuse agent-replaced output symlinks; atomic replace does not follow the destination.
            staging = self.address / "response.tmp"
            try:
                if self.address.is_symlink() or staging.is_symlink():
                    raise ValueError("Symlinked transport")
                staging.write_text(json.dumps(response))
                os.replace(staging, self.address / "response.json")
            except (OSError, ValueError):
                self.stopping.set()

    def close(self):
        self.stopping.set()
        self.thread.join(timeout=40)
        if self.thread.is_alive():
            raise RuntimeError("Adapter failed to stop")
        self.session.close()


def command(root):
    return ["codex", "exec", "--ignore-user-config", "--ephemeral", "--json",
            "--sandbox", "workspace-write", "--skip-git-repo-check",
            "--disable", "multi_agent", "--disable", "apps", "--disable", "plugins",
            "--disable", "plugin_discovery_prefetch", "--disable", "remote_plugin", "--disable", "memories",
            "-C", str(root), "-m", MODEL, "-c", 'model_reasoning_effort="high"',
            "-c", 'approval_policy="never"', "-"]


PROMPT = "Read TASK.txt and program.lobo. Repair the function as specified, use the provided checks before and after editing, and finish with a concise result. Change only program.lobo. Do not read outside this task directory."


def trial(index, name, arm, artifacts):
    with tempfile.TemporaryDirectory(prefix="lobo-pilot-") as temp:
        host = pathlib.Path(temp)
        root = host / "task"
        root.mkdir()
        (root / "program.lobo").write_text(TASKS[name]["source"])
        fixtures = {"TASK.txt": guide_for(name),
                    "check.py": CLIENT.format(address=str(root / ".check-io"))}
        for label, contents in fixtures.items():
            (root / label).write_text(contents)
        adapter = Adapter(root, root / ".check-io", name, arm)
        destination = artifacts / f"{index:02d}-{name}-{arm}"
        destination.mkdir(mode=0o700)
        (destination / "prompt.txt").write_text(PROMPT)
        start = time.monotonic()
        report = {"task": name, "arm": arm, "load_before": os.getloadavg(), "command": command(root),
                  "initial_source_sha256": digest(TASKS[name]["source"].encode()),
                  "visible_plan_sha256": digest(json.dumps(test_cases(name), sort_keys=True).encode()),
                  "acceptance_plan_sha256": digest(json.dumps(test_cases(name, heldout=True), sort_keys=True).encode())}
        timed_out = False
        try:
            with (destination / "events.jsonl").open("w") as stdout, (destination / "stderr.txt").open("w") as stderr:
                proc = subprocess.Popen(command(root), stdin=subprocess.PIPE, stdout=stdout, stderr=stderr,
                                        text=True, start_new_session=True)
                try:
                    proc.communicate(PROMPT, timeout=LIMIT)
                except subprocess.TimeoutExpired:
                    timed_out = True
                    os.killpg(proc.pid, signal.SIGTERM)
                    try:
                        proc.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(proc.pid, signal.SIGKILL)
                        proc.wait()
            report.update({"agent_seconds": time.monotonic()-start, "timeout": timed_out, "exit_code": proc.returncode})
        finally:
            adapter.close()
        events = []
        for line in (destination / "events.jsonl").read_text().splitlines():
            try:
                events.append(json.loads(line))
            except json.JSONDecodeError:
                report["malformed_event"] = True
        report["usage"] = read_usage(events)
        report["tool_calls"] = sum(e.get("type") == "item.completed" and
                                   e.get("item", {}).get("type") in ["command_execution", "mcp_tool_call", "file_change"] for e in events)
        report["checks"] = adapter.checks
        report["check_requests"] = adapter.requests
        report["interface_used"] = bool(adapter.checks)
        report["fixtures_intact"] = all((root / label).is_file() and not (root / label).is_symlink()
                                         and (root / label).read_text() == content for label, content in fixtures.items())
        report["unexpected_files"] = sorted(str(p.relative_to(root)) for p in root.rglob("*")
                                             if str(p.relative_to(root)) not in ["TASK.txt", "program.lobo", "check.py",
                                                 ".check-io", ".check-io/request.json", ".check-io/response.json"])
        try:
            source = read_source(root)
            (destination / "program.lobo").write_text(source)
            report["final_source_sha256"] = digest(source.encode())
            with Session(TASKS[name], test_cases(name, heldout=True)) as judge:
                report["final_grade"] = judge.verify(source, "all")
        except Exception as exc:
            report["grading_error"] = str(exc)[:1024]
        report["accepted"] = (not timed_out and proc.returncode == 0 and report["fixtures_intact"]
                              and not report["unexpected_files"]
                              and report.get("final_grade", {}).get("status") == "checks_satisfied"
                              and report.get("final_grade", {}).get("full_execution") is True)
        report["elapsed_seconds"] = time.monotonic()-start
        (destination / "report.json").write_text(json.dumps(report, indent=2))
        return report


def summaries(trials):
    result = {}
    for arm in ["conventional", "structured"]:
        rows = [r for r in trials if r["arm"] == arm]
        times = [r["elapsed_seconds"] for r in rows]
        result[arm] = {"trials": len(rows), "accepted": sum(r["accepted"] for r in rows),
                       "interface_used": sum(r["interface_used"] for r in rows),
                       "timeouts": sum(r["timeout"] for r in rows),
                       "infrastructure_errors": sum(bool(r.get("infrastructure_error") or r.get("grading_error")) for r in rows)}
        if times:
            result[arm]["elapsed_seconds_all_trials"] = {"median": statistics.median(times), "min": min(times), "max": max(times)}
        for field in ["input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens"]:
            values = [r["usage"][field] for r in rows if r.get("usage") and field in r["usage"]]
            result[arm][field] = {"observed": len(values), "median": statistics.median(values) if values else None,
                                  "min": min(values) if values else None, "max": max(values) if values else None}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true", help="Run exactly the ten registered model trials (uses authenticated quota)")
    parser.add_argument("--canary", action="store_true", help="One excluded wiring check, not pilot data")
    args = parser.parse_args()
    if not args.run and not args.canary:
        parser.error("Choose --run or --canary explicitly")
    if args.run and args.canary:
        parser.error("Run the canary separately")
    artifacts = ROOT / "target" / ("agent-repair-" + time.strftime("%Y%m%dT%H%M%S"))
    artifacts.mkdir(mode=0o700)
    (artifacts / "protocol.md").write_bytes((ROOT / "decisions/0025-real-agent-testing-pilot.md").read_bytes())
    (artifacts / "harness.py").write_bytes(pathlib.Path(__file__).read_bytes())
    report = {"format": "lobo-agent-repair-pilot/0.1", "canary": args.canary,
              "model": MODEL, "reasoning": "high", "deadline_seconds": LIMIT,
              "runner_sha256": digest(RUNNER.read_bytes()), "harness_sha256": digest(pathlib.Path(__file__).read_bytes()),
              "protocol_sha256": digest((ROOT / "decisions/0025-real-agent-testing-pilot.md").read_bytes()),
              "codex_version": subprocess.check_output(["codex", "--version"], text=True).strip(),
              "clang_version": subprocess.check_output([str(CLANG), "--version"], text=True).splitlines()[0],
              "platform": platform.platform(), "python_version": platform.python_version(),
              "cpus": os.cpu_count(), "artifact_directory": str(artifacts), "trials": [],
              "limitations": "Five heterogeneous pairs, no within-task repetitions. Same native backend; feedback+selection bundle, not comparison against existing frameworks. No general productivity conclusion. Shared host and cache/latency confounding."}
    order = [("clamp", "structured")] if args.canary else schedule()
    for i, (name, arm) in enumerate(order):
        print(f"Starting {i+1}/{len(order)} {name} {arm}", flush=True)
        start = time.monotonic()
        try:
            row = trial(i, name, arm, artifacts)
        except Exception as exc:
            row = {"task": name, "arm": arm, "accepted": False, "interface_used": False,
                   "timeout": False, "usage": None, "checks": [],
                   "infrastructure_error": str(exc)[:1024], "elapsed_seconds": time.monotonic()-start}
        report["trials"].append(row)
        report["summary"] = summaries(report["trials"])
        (artifacts / "results.json").write_text(json.dumps(report, indent=2))
        print(f"Finished accepted={row['accepted']} checks={len(row['checks'])} seconds={row['elapsed_seconds']:.2f}", flush=True)
    print(str(artifacts / "results.json"), flush=True)


if __name__ == "__main__":
    main()
