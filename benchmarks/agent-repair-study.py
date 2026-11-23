#!/usr/bin/env python3
"""Repeated native library repairs; see decision 0026 for the frozen protocol."""
import argparse
import importlib.util
import json
import os
import pathlib
import platform
import random
import resource
import signal
import statistics
import subprocess
import tempfile
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("pilot", pathlib.Path(__file__).with_name("agent-repair.py"))
pilot = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pilot)
ARMS = ("text_full", "json_full", "json_affected")
DEADLINE = 120
PROTOCOL = ROOT/"decisions/0026-repeated-library-repair-study.md"


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError(f"Fixture drift: expected exactly one {before!r}")
    return source.replace(before, after)


def make_tasks():
    sources = {n: (ROOT/"benchmarks/fixtures/repair-study"/n).read_text() for n in ["collections.lobo", "tree.lobo", "sequences.lobo"]}
    sources["rules.lobo"] = '''module rules @id("study.rules");
function valid @id("study.valid")(value: I64) -> Bool { value >= 0 && value <= 1000 }
function same @id("study.same")(left: I64, right: I64) -> Bool { left == right }
function next_depth @id("study.next_depth")(depth: I64) -> I64 { depth + 1 }
'''
    sources["audit.lobo"] = '''module audit @id("study.audit");
record Work @id("study.audit.work") { remaining: List<I64>, total: I64 }
function total @id("study.total")(values: List<I64>) -> I64 {
  let done = fold(i in List::length(values), work: Work = Work { remaining: values, total: 0 }) {
    Work { remaining: List::tail(work.remaining), total: work.total + List::head(work.remaining) }
  };
  done.total
}
'''
    sources["collections.lobo"] = replace_once(sources["collections.lobo"],
        'import reverse @id("example.collections.reverse");',
        'import reverse @id("example.collections.reverse");\nimport valid @id("study.valid");')
    sources["collections.lobo"] = replace_once(sources["collections.lobo"], "value < 0 || value > 1000", "!valid(value)")
    sources["tree.lobo"] = replace_once(sources["tree.lobo"],
        'import reverse_shared @id("example.collections.reverse");',
        'import reverse_shared @id("example.collections.reverse");\nimport same @id("study.same");\nimport next_depth @id("study.next_depth");')
    sources["tree.lobo"] = replace_once(sources["tree.lobo"], "if key == node.key", "if same(key, node.key)")
    sources["tree.lobo"] = replace_once(sources["tree.lobo"], "depth: node.depth + 1", "depth: next_depth(node.depth)")
    mutations = {
        "collections": [("rules.lobo", "value <= 1000", "value < 1000"),
                        ("collections.lobo", "else { work.sum };", "else { work.sum + value };"),
                        ("collections.lobo", "values: reverse(work.kept)", "values: work.kept")],
        "tree": [("rules.lobo", "left == right", "left <= right"),
                 ("rules.lobo", "depth + 1", "depth"),
                 ("tree.lobo", "keys: traverse(tree, descending)", "keys: traverse(tree, !descending)")]}
    result = {}
    for name, edits in mutations.items():
        broken, singles = dict(sources), []
        for file, before, after in edits:
            broken[file] = replace_once(broken[file], before, after)
            mutant = dict(sources)
            mutant[file] = replace_once(mutant[file], before, after)
            singles.append(mutant)
        result[name] = {"name": name, "root": name+".lobo", "sources": broken,
                        "fixed": dict(sources), "single_mutants": singles}
    return result


TASKS = make_tasks()


def scalar(n):
    return {"type": "I64", "value": str(n)}


def list_value(values):
    return {"type": "List", "value": {"element": "I64", "elements": [scalar(n) for n in values]}}


def list_arg(values):
    return "List:Count:"+str(len(values))+"".join(f",I64:{n}" for n in values)


def record(identity, fields):
    return {"type": "Record", "value": {"id": identity, "fields": fields}}


def collection_expect(values, even):
    invalid = next(((i, n) for i, n in enumerate(values) if not 0 <= n <= 1000), None)
    if invalid is not None:
        i, n = invalid
        payload = record("example.collections.invalid", {"index": scalar(i), "value": scalar(n)})
        case = "InvalidValue"
    else:
        selected = [n for n in values if (n % 2 == 0) == even]
        payload = record("example.collections.summary", {"values": list_value(selected),
                         "count": scalar(len(selected)), "sum": scalar(sum(selected))})
        case = "Ok"
    return {"type": "Variant", "value": {"id": "example.collections.selection", "case": case, "payload": payload}}


def tree_expect(values, descending):
    # Independent pointer BST for oracle height; Lobo uses a flat parent-index table.
    root, height = None, 0
    for n in values:
        if root is None:
            root, height = [n, None, None], 1
            continue
        node, depth = root, 1
        while n != node[0]:
            side = 1 if n < node[0] else 2
            depth += 1
            if node[side] is None:
                node[side] = [n, None, None]
                height = max(height, depth)
                break
            node = node[side]
    keys = sorted(set(values), reverse=descending)
    return record("example.tree.summary", {"keys": list_value(keys), "count": scalar(len(keys)), "height": scalar(height)})


def cases(name, heldout=False):
    collections = [[], [0], [1000], [1, 2, 3, 4], [8, 2, 8, 0], [999, 1000, 1],
                   [-1, 2], [1, 1001, -1], [2, 4, 6, 8], [9, 7, 5, 3]]
    trees = [[], [0], [5, 3, 7, 2, 4, 6, 8], [3, 1, 3, 2], [1, 2, 3, 4], [4, 3, 2, 1],
             [-2, -3, -1], [0, 0, 0], [5, 1, 4, 2, 3], [0, pilot.MIN, pilot.MAX]]
    sequences = [[], [0], [1, 2], [2, 1], [1, 1, 2], [-3, 0, 2], [8, 3, 5, 2, 9], list(range(12)),
                 list(range(12, 0, -1)), [pilot.MIN, pilot.MAX], [1]*12, [0, 9, 0, -9]]
    totals = [[], [0], [1], [-1], [1, -1], list(range(12)), list(range(-12, 0)), [3, 1, 4, 1, 5],
              [999, 1000], [-100, 7, 100], [pilot.MIN, 1], [pilot.MAX, -1]]
    if heldout:
        rng = random.Random(2603)
        collections += [[rng.randint(0, 1000) for _ in range(18)] for _ in range(12)]
        collections += [[1000, 2, 999, 0, 1001], [2, -1, 1001], [1000]*10, [7, 0, -1]]
        trees += [[rng.randint(-30, 30) for _ in range(16)] for _ in range(12)]
        trees += [list(range(16)), list(range(15, -1, -1)), [pilot.MIN, pilot.MAX, 0, pilot.MIN]]
        sequences += [[rng.randint(-100, 100) for _ in range(30)] for _ in range(12)]
        totals += [[rng.randint(-100, 100) for _ in range(30)] for _ in range(12)]
    result = []
    def add(group, entry, args, value):
        result.append({"id": f"{group}.{sum(c['requirement']==group for c in result)}",
                       "requirement": group, "entry": entry, "arguments": args,
                       "expect": {"kind": "value", "value": value}, "fuel": 2000000, "heap_cells": 100000})
    for i in range(max(len(collections)*2, len(trees)*2, len(sequences), len(totals))):
        if i < len(collections)*2:
            values, flag = collections[i//2], bool(i % 2)
            add("collection", "example.collections.select", [list_arg(values), f"Bool:{str(flag).lower()}"], collection_expect(values, flag))
        if i < len(trees)*2:
            values, flag = trees[i//2], bool(i % 2)
            add("tree", "example.tree.summarize", [list_arg(values), f"Bool:{str(flag).lower()}"], tree_expect(values, flag))
        if i < len(sequences):
            values = sequences[i]
            add("reverse", "example.collections.reverse", [list_arg(values)], list_value(values[::-1]))
        if i < len(totals):
            values = totals[i]
            add("aggregate", "study.total", [list_arg(values)], scalar(sum(values)))
    return result


REQUIREMENTS = [
    {"id": "collection", "description": "Validate all input values in [0,1000]; first invalid index/value wins. Keep requested parity in original order, count and sum ONLY selected values."},
    {"id": "tree", "description": "Insert I64 keys in input order into an unbalanced BST. Ignore duplicates. Return unique keys ascending or descending as requested, count and tree height (empty=0, root=1)."},
    {"id": "reverse", "description": "Reverse the complete input list preserving duplicates and values."},
    {"id": "aggregate", "description": "Return the signed sum of input; inputs guarantee each prefix fits I64."}]


class Session(pilot.Session):
    def __init__(self, task, tests):
        self.root, self.cases = task["root"], tests
        self.process = subprocess.Popen([str(pilot.RUNNER), "--session", "--clang", str(pilot.CLANG)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        try:
            response = self.call({"op": "open", "root": self.root, "sources": self.bundle(task["sources"]),
                                  "requirements": REQUIREMENTS, "tests": tests})
            if response["status"] != "opened":
                raise RuntimeError(response)
            self.snapshot = response["snapshot"]
        except BaseException:
            self.close()
            raise

    @staticmethod
    def bundle(sources):
        return [{"name": n, "text": t} for n, t in sorted(sources.items())]

    def verify(self, sources, scope):
        response = self.call({"op": "verify", "base": self.snapshot, "root": self.root,
                              "sources": self.bundle(sources), "environment": "library-study-2026-11-03",
                              "scope": scope, "max_cases": 256, "wall_ms": 15000})
        if "snapshot" in response:
            self.snapshot = response["snapshot"]
        return response


def read_sources(root, task):
    sources = {}
    for name in task["sources"]:
        path = root/name
        if path.is_symlink() or not path.is_file() or path.stat().st_size > 65536:
            raise ValueError("Source must be a regular non-symlink file <=64 KiB: "+name)
        sources[name] = path.read_text()
    return sources


def show(result, arm):
    if arm != "text_full":
        return json.dumps(result, separators=(",", ":"))
    return pilot.render(result, "conventional") if "counts" in result else json.dumps(result, indent=2)


CLIENT = '''import json, os, pathlib, sys, time, uuid
directory = pathlib.Path({address!r})
token = uuid.uuid4().hex
request = directory / (token + ".request.json")
response_file = directory / (token + ".response.json")
staging = directory / (token + ".tmp")
staging.write_text(json.dumps({{"token": token, "args": sys.argv[1:]}}))
os.replace(staging, request)
deadline = time.monotonic() + 40
while time.monotonic() < deadline:
    try:
        response = json.loads(response_file.read_text())
        if response["token"] == token:
            response_file.unlink()
            print(response["text"])
            sys.exit(response["exit"])
    except (FileNotFoundError, json.JSONDecodeError):
        pass
    time.sleep(0.02)
print("Check adapter deadline", file=sys.stderr)
sys.exit(2)
'''


class Adapter:
    """Experiment-only file mailbox with a bounded sequential native session."""
    def __init__(self, root, task, arm, tests):
        self.root, self.task, self.arm = root, task, arm
        self.address = root/".check-io"
        self.address.mkdir(mode=0o700)
        self.session = Session(task, tests)
        self.events = []
        self.stopping = threading.Event()
        self.thread = threading.Thread(target=self.serve, daemon=True)
        self.thread.start()

    def dispatch(self, args):
        start = time.monotonic()
        if args == []:
            sources = read_sources(self.root, self.task)
            result = self.session.verify(sources, "affected" if self.arm == "json_affected" else "all")
        elif args == ["--list"]:
            result = {"status": "ok", "tests": self.session.cases}
        elif isinstance(args, list) and len(args) == 2 and args[0] in ["--detail", "--replay"]:
            result = self.session.call({"op": "explain" if args[0] == "--detail" else "replay",
                                        "snapshot": self.session.snapshot, "test": args[1]})
        else:
            raise ValueError("Usage: check.py [--list | --detail TEST_ID | --replay TEST_ID]")
        text, status = show(result, self.arm), result.get("status")
        if args:
            code = 0 if status == "ok" or "reproduced" in result else 2
        else:
            code = 0 if status == "checks_satisfied" else 1 if status == "failed" else 2
        self.events.append({"args": args, "result": result, "seconds": time.monotonic()-start,
                            "feedback_bytes": len(text.encode()),
                            "source_sha256": {k: pilot.digest(v.encode()) for k, v in sources.items()} if not args else None})
        return {"text": text, "exit": code}

    def serve(self):
        seen = set()
        while not self.stopping.is_set():
            try:
                paths = sorted(self.address.glob("*.request.json"), key=lambda p: p.stat().st_mtime_ns)
                if not paths:
                    self.stopping.wait(0.02)
                    continue
                if len(paths) > 32 or len(seen) >= 256:
                    self.stopping.set()
                    continue
                path = paths[0]
                if self.address.is_symlink() or path.is_symlink() or path.stat().st_size > 4096:
                    raise ValueError("Invalid transport")
                request = json.loads(path.read_text())
                token, args = request["token"], request["args"]
                if not isinstance(token, str) or len(token) != 32 or any(c not in "0123456789abcdef" for c in token):
                    raise ValueError("Invalid token")
                if path.name != token+".request.json":
                    raise ValueError("Token/path mismatch")
                path.unlink()
                if token in seen:
                    continue
                seen.add(token)
            except (OSError, ValueError, KeyError, TypeError):
                self.stopping.wait(0.02)
                continue
            try:
                response = self.dispatch(args)
            except Exception as exc:
                response = {"text": "Adapter error: "+str(exc)[:1024], "exit": 2}
                self.events.append({"args": args, "error": response["text"], "seconds": 0,
                                    "feedback_bytes": len(response["text"].encode())})
            response["token"] = token
            try:
                staging = self.address/(token+".tmp")
                if self.address.is_symlink() or staging.is_symlink():
                    raise ValueError("Symlink transport")
                staging.write_text(json.dumps(response))
                os.replace(staging, self.address/(token+".response.json"))
            except (OSError, ValueError):
                self.stopping.set()

    def close(self):
        self.stopping.set()
        self.thread.join(timeout=40)
        if self.thread.is_alive():
            raise RuntimeError("Adapter did not stop")
        self.session.close()


def schedule():
    rows = []
    for repeat in range(5):
        for t, name in enumerate(TASKS):
            offset = (repeat+t) % 3
            arms = list(ARMS[offset:]+ARMS[:offset])
            if repeat % 2:
                arms.reverse()
            rows += [(repeat, name, a) for a in arms]
    return rows


GUIDE = '''Repair the five-source Lobo project. There are three seeded behavior defects affecting
the TASK named below; all four requirements must hold. Only collections.lobo, tree.lobo,
sequences.lobo, rules.lobo and audit.lobo may change. Preserve public IDs, signatures and
nominal type layouts. Do not alter TASK.txt, check.py or .check-io (tool-generated messages).
Do not read outside this task directory, use another compiler, delegate, access network or
install dependencies.

Run python3 check.py before editing and after each candidate repair. It checks all 64 visible
cases and exits 0 only on success (1=failure, 2=invalid/infrastructure). Summaries show at most
eight failure capsules; frozen inputs and complete observations are available through:
  python3 check.py --list
  python3 check.py --detail TEST_ID
  python3 check.py --replay TEST_ID
Detail/replay refer to the latest checked snapshot, not newer edits. Replay can reproduce a
failing observation; inspect its reproduced field. Eight snapshots are retained including open;
after seven verifies the limit is explicit, never silently reset. Final grading freshly runs
additional inputs satisfying these same requirements. No reference fix or editable oracle is here.

TASK: {name}
REQUIREMENTS:
{requirements}

LANGUAGE GUIDE:
Lobo is immutable and expression-oriented. A function returns its final expression. let x = e;
introduces an immutable local; if c { e } else { e }, match(v) { Case(x) { e }, Empty { e } }
are expressions. function f @id("id")(x: I64) -> I64 { x+1 }. Calls use local names/import aliases.
import alias @id("global.declaration.id"); explicitly links supplied modules. All five sources
are compiled together; imports do not read paths. Preserve existing declarations/layouts.
record R @id("r") { x: I64 } constructs R { x: 1 }; field access r.x.
variant V @id("v") { Some(I64), None } constructs V::Some(1), V::None.
List::empty<I64>(), List::prepend(value,list), List::head(list), List::tail(list), List::length(list).
head/tail require nonempty lists. fold(i in n, state: T = initial) { next_state } runs n iterations,
carrying immutable state. while(state: T = initial; condition) { next_state } repeats until false.
requires/ensures are checked contracts; result names the return value in ensures. && and ||
short-circuit. == != < <= > >= compare values; + - * / % use checked signed I64 arithmetic.
Division truncates toward zero. Overflow, violated contracts and resource limits are diagnostics.
No return keyword, mutation, implicit imports or host IO. Existing code demonstrates the syntax.
'''
PROMPT = "Read TASK.txt and inspect the named project sources. Repair the documented defects, using supplied checks before and after editing. Use list/detail/replay when you need missing case context. Change only named Lobo sources; do not read outside this task directory. Finish with a concise result."


def guide(name):
    return GUIDE.replace("{name}", name).replace("{requirements}", "\n".join(r["id"]+": "+r["description"] for r in REQUIREMENTS))


def trial(index, repeat, name, arm, artifacts):
    task = TASKS[name]
    destination = artifacts/f"{index:02d}-{repeat}-{name}-{arm}"
    destination.mkdir(mode=0o700)
    with tempfile.TemporaryDirectory(prefix="lobo-study-") as directory:
        root = pathlib.Path(directory)
        for file, source in task["sources"].items():
            (root/file).write_text(source)
        fixtures = {"TASK.txt": guide(name), "check.py": CLIENT.format(address=str(root/".check-io"))}
        for file, source in fixtures.items():
            (root/file).write_text(source)
        adapter = Adapter(root, task, arm, cases(name))
        start, cpu_before = time.monotonic(), resource.getrusage(resource.RUSAGE_CHILDREN)
        row = {"repeat": repeat, "task": name, "arm": arm, "command": pilot.command(root),
               "load_before": os.getloadavg(), "timeout": False}
        try:
            with (destination/"events.jsonl").open("w") as stdout, (destination/"stderr.txt").open("w") as stderr:
                proc = subprocess.Popen(pilot.command(root), stdin=subprocess.PIPE, stdout=stdout, stderr=stderr,
                                        text=True, start_new_session=True)
                try:
                    proc.communicate(PROMPT, timeout=DEADLINE)
                except subprocess.TimeoutExpired:
                    row["timeout"] = True
                    os.killpg(proc.pid, signal.SIGTERM)
                    try:
                        proc.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(proc.pid, signal.SIGKILL)
                        proc.wait()
            row.update({"exit_code": proc.returncode, "agent_seconds": time.monotonic()-start})
        finally:
            adapter.close()
        events = []
        for line in (destination/"events.jsonl").read_text().splitlines():
            try:
                events.append(json.loads(line))
            except json.JSONDecodeError:
                row["malformed_event"] = True
        row["usage"], row["events"] = pilot.read_usage(events), adapter.events
        row["agent_errors"] = [e for e in events if e.get("type") in ["error", "turn.failed"]]
        row["transport_errors"] = sum("Check adapter deadline" in e.get("item", {}).get("aggregated_output", "")
                                      for e in events if e.get("type") == "item.completed")
        row["tool_items"] = sum(e.get("type") == "item.completed" and e.get("item", {}).get("type") in
                                 ["command_execution", "file_change", "mcp_tool_call"] for e in events)
        row["fixture_integrity"] = all((root/f).is_file() and not (root/f).is_symlink() and
                                        (root/f).read_text() == v for f, v in fixtures.items())
        allowed = {*task["sources"], *fixtures, ".check-io"}
        def owned_message(path):
            parts = path.relative_to(root).parts
            if len(parts) != 2 or parts[0] != ".check-io" or path.is_symlink():
                return False
            token, _, suffix = parts[1].partition(".")
            return len(token) == 32 and all(c in "0123456789abcdef" for c in token) and suffix in ["request.json", "response.json", "tmp"]
        row["unexpected_files"] = sorted(str(p.relative_to(root)) for p in root.rglob("*")
                                          if str(p.relative_to(root)) not in allowed and not owned_message(p))
        try:
            sources = read_sources(root, task)
            for file, source in sources.items():
                (destination/file).write_text(source)
            row["final_source_sha256"] = {k: pilot.digest(v.encode()) for k, v in sources.items()}
            with Session(task, cases(name, heldout=True)) as judge:
                row["grade"] = judge.verify(sources, "all")
        except Exception as exc:
            row["grading_error"] = str(exc)[:1024]
        row["accepted"] = (not row["timeout"] and row["exit_code"] == 0 and row["fixture_integrity"]
                            and not row["unexpected_files"] and row.get("grade", {}).get("status") == "checks_satisfied"
                            and row["grade"]["full_execution"])
        row["elapsed_seconds"] = time.monotonic()-start
        cpu_after = resource.getrusage(resource.RUSAGE_CHILDREN)
        row["local_child_cpu_seconds"] = (cpu_after.ru_utime+cpu_after.ru_stime)-(cpu_before.ru_utime+cpu_before.ru_stime)
        (destination/"report.json").write_text(json.dumps(row, indent=2))
        return row


def spread(values):
    return {"observed": len(values), "median": statistics.median(values) if values else None,
            "min": min(values) if values else None, "max": max(values) if values else None}


def analyze(rows):
    groups, paired = {}, {}
    for task in TASKS:
        for arm in ARMS:
            subset = [r for r in rows if r["task"] == task and r["arm"] == arm]
            if not subset:
                continue
            events = [e for r in subset for e in r["events"]]
            verifies = [e for e in events if e["args"] == []]
            result = {"trials": len(subset), "accepted": sum(r["accepted"] for r in subset),
                      "timeouts": sum(r["timeout"] for r in subset),
                      "infrastructure_errors": sum(bool(r.get("infrastructure_error") or r.get("grading_error")
                                                       or r.get("agent_errors") or r.get("transport_errors")) for r in subset),
                      "interface_used": sum(any(e["args"] == [] for e in r["events"]) for r in subset),
                      "detail_trials": sum(any(e["args"][:1] == ["--detail"] for e in r["events"]) for r in subset),
                      "replay_trials": sum(any(e["args"][:1] == ["--replay"] for e in r["events"]) for r in subset),
                      "verify_calls": len(verifies), "detail_calls": sum(e["args"][:1] == ["--detail"] for e in events),
                      "replay_calls": sum(e["args"][:1] == ["--replay"] for e in events),
                      "tool_error_calls": sum(bool(e.get("error")) or e.get("result", {}).get("status") in
                                               ["invalid_request", "compile_error", "invalid_candidate", "incomplete"]
                                               for e in events),
                      "elapsed_seconds_all": spread([r["elapsed_seconds"] for r in subset]),
                      "elapsed_seconds_accepted": spread([r["elapsed_seconds"] for r in subset if r["accepted"]]),
                      "host_seconds": sum(e["seconds"] for e in events), "feedback_bytes": sum(e["feedback_bytes"] for e in events),
                      "executed": sum(e.get("result", {}).get("counts", {}).get("executed", 0) for e in verifies),
                      "reused": sum(e.get("result", {}).get("counts", {}).get("reused", 0) for e in verifies)}
            for field in ["input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens"]:
                result[field] = spread([r["usage"][field] for r in subset if r.get("usage") and field in r["usage"]])
            groups[task+"/"+arm] = result
        for before, after in [("text_full", "json_full"), ("json_full", "json_affected")]:
            deltas, accepted_deltas = [], []
            for repeat in range(5):
                a = next((r for r in rows if (r["repeat"], r["task"], r["arm"]) == (repeat, task, before)), None)
                b = next((r for r in rows if (r["repeat"], r["task"], r["arm"]) == (repeat, task, after)), None)
                if a and b:
                    deltas.append(b["elapsed_seconds"]-a["elapsed_seconds"])
                    if a["accepted"] and b["accepted"]:
                        accepted_deltas.append(b["elapsed_seconds"]-a["elapsed_seconds"])
            rng = random.Random(2603)
            boot = sorted(statistics.median(rng.choices(accepted_deltas, k=len(accepted_deltas)))
                          for _ in range(10000)) if accepted_deltas else []
            paired[task+"/"+after+"-minus-"+before] = {
                "complete_pairs": len(deltas), "all_elapsed_deltas_seconds": deltas,
                "both_accepted_pairs": len(accepted_deltas), "accepted_delta": spread(accepted_deltas),
                "exploratory_paired_bootstrap_95_percent": [boot[249], boot[9749]] if boot else None}
    return {"by_task_arm": groups, "paired": paired}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true")
    parser.add_argument("--canary", action="store_true")
    args = parser.parse_args()
    if args.run == args.canary:
        parser.error("Choose exactly one of --canary or --run (uses authenticated quota)")
    artifacts = ROOT/"target"/("agent-study-"+time.strftime("%Y%m%dT%H%M%S"))
    artifacts.mkdir(mode=0o700)
    inputs = {"harness.py": pathlib.Path(__file__), "pilot.py": pathlib.Path(pilot.__file__), "protocol.md": PROTOCOL}
    for label, path in inputs.items():
        (artifacts/label).write_bytes(path.read_bytes())
    frozen = {n: {"root": t["root"], "sources": t["sources"], "visible_cases": cases(n),
                  "acceptance_cases": cases(n, heldout=True)} for n, t in TASKS.items()}
    (artifacts/"inputs.json").write_text(json.dumps(frozen, indent=2))
    report = {"format": "lobo-library-repair-study/0.1", "canary": args.canary,
              "requested_model": pilot.MODEL, "reasoning": "high", "deadline_seconds": DEADLINE,
              "inputs_sha256": pilot.digest((artifacts/"inputs.json").read_bytes()),
              "identities": {label: pilot.digest(path.read_bytes()) for label, path in inputs.items()},
              "runner_sha256": pilot.digest(pilot.RUNNER.read_bytes()), "artifact_directory": str(artifacts),
              "codex_version": subprocess.check_output(["codex", "--version"], text=True).strip(),
              "clang_version": subprocess.check_output([str(pilot.CLANG), "--version"], text=True).splitlines()[0],
              "python_version": platform.python_version(), "platform": platform.platform(), "cpus": os.cpu_count(),
              "prompt": PROMPT, "guides": {n: guide(n) for n in TASKS}, "trials": []}
    order = [(0, "collections", "json_affected"), (0, "tree", "text_full")] if args.canary else schedule()
    for i, (repeat, name, arm) in enumerate(order):
        print(f"Starting {i+1}/{len(order)} repeat={repeat} {name} {arm}", flush=True)
        start = time.monotonic()
        try:
            row = trial(i, repeat, name, arm, artifacts)
        except Exception as exc:
            row = {"repeat": repeat, "task": name, "arm": arm, "accepted": False, "timeout": False,
                   "usage": None, "events": [], "infrastructure_error": str(exc)[:1024],
                   "elapsed_seconds": time.monotonic()-start}
        report["trials"].append(row)
        report["analysis"] = analyze(report["trials"])
        (artifacts/"results.json").write_text(json.dumps(report, indent=2))
        print(f"Finished accepted={row['accepted']} seconds={row['elapsed_seconds']:.2f} calls={len(row['events'])}", flush=True)
    print(artifacts/"results.json", flush=True)


if __name__ == "__main__":
    main()
