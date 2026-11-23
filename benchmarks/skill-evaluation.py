#!/usr/bin/env python3
"""Explicit Lobo skill versus equivalent flattened knowledge. Decision 0028."""
import argparse
import copy
import fcntl
import hashlib
import importlib.util
import json
import os
import pathlib
import platform
import random
import re
import signal
import statistics
import subprocess
import tempfile
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
SKILL = ROOT/".agents/skills/lobo-programming"
PROTOCOL = ROOT/"decisions/0028-lobo-skill-onboarding-comparison.md"
SPEC = importlib.util.spec_from_file_location("study", pathlib.Path(__file__).with_name("agent-repair-study.py"))
study = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(study)
pilot = study.pilot
DEADLINE = 120

RLE_HEADER = '''module rle @id("skilltask.rle");
import reverse_runs @id("reverse_runs");
record Run @id("skilltask.run") { value: I64, count: I64 }
'''
RLE_HELPER = '''module helpers @id("skilltask.helpers");
import Run @id("skilltask.run");
record Cursor @id("skilltask.cursor") { remaining: List<Run>, reversed: List<Run> }
function reverse_runs @id("reverse_runs")(input: List<Run>) -> List<Run> {
  let done = fold(i in List::length(input), work: Cursor = Cursor { remaining: input, reversed: List::empty<Run>() }) {
    Cursor { remaining: List::tail(work.remaining),
             reversed: List::prepend(List::head(work.remaining), work.reversed) }
  };
  done.reversed
}
function control @id("control")(n: I64) -> I64 { n }
'''
RLE_FIXED = RLE_HEADER + '''record Build @id("skilltask.build") {
  remaining: List<I64>, started: Bool, value: I64, count: I64, out: List<Run>
}
function encode @id("encode")(input: List<I64>) -> List<Run> {
  let done = fold(i in List::length(input), work: Build = Build {
    remaining: input, started: false, value: 0, count: 0, out: List::empty<Run>()
  }) {
    let value = List::head(work.remaining);
    if !work.started {
      Build { remaining: List::tail(work.remaining), started: true, value: value, count: 1, out: work.out }
    } else {
      if value == work.value {
        Build { remaining: List::tail(work.remaining), started: true, value: value, count: work.count + 1, out: work.out }
      } else {
        Build { remaining: List::tail(work.remaining), started: true, value: value, count: 1,
                out: List::prepend(Run { value: work.value, count: work.count }, work.out) }
      }
    }
  };
  let runs = if done.started { List::prepend(Run { value: done.value, count: done.count }, done.out) } else { done.out };
  reverse_runs(runs)
}
'''
CLIP_HEADER = '''module clipping @id("skilltask.clipping");
import extent @id("extent");
variant Outcome @id("skilltask.outcome") { Ok(Text), Bounds(I64) }
'''
CLIP_HELPER = '''module helpers @id("skilltask.helpers");
function extent @id("extent")(text: Text) -> I64 { Bytes::length(Text::bytes(text)) }
function control @id("control")(n: I64) -> I64 { n }
'''
TASKS = {
    "rle": {"root": "program.lobo",
            "description": "Complete encode(input). Return maximal consecutive equal-value runs in input order, each as Run { value, count }. Counts are positive and preserve every input item. Equal values separated by another value form different runs. Empty input returns an empty list. All I64 values are valid.",
            "sources": {"program.lobo": RLE_HEADER+'function encode @id("encode")(input: List<I64>) -> List<Run> { List::empty<Run>() }\n',
                        "helpers.lobo": RLE_HELPER},
            "fixed": {"program.lobo": RLE_FIXED, "helpers.lobo": RLE_HELPER}},
    "clip": {"root": "program.lobo",
             "description": "Complete clip(text,start,count) and repair extent if needed. Indices/counts are Unicode scalars. If start<0 or start>scalar length, return Bounds(start). Otherwise if count<0 or count>length-start, return Bounds(count). Otherwise return Ok of the requested scalar slice. Validate start before count. An empty slice at the end is valid. All I64 start/counts are valid; avoid arithmetic overflow. Preserve text exactly, including combining marks and NUL.",
             "sources": {"program.lobo": CLIP_HEADER+'function clip @id("clip")(text: Text, start: I64, count: I64) -> Outcome { Outcome::Bounds(start) }\n',
                         "helpers.lobo": CLIP_HELPER},
             "fixed": {"program.lobo": CLIP_HEADER+'''function clip @id("clip")(text: Text, start: I64, count: I64) -> Outcome {
  let size = extent(text);
  if start < 0 || start > size { Outcome::Bounds(start) } else {
    if count < 0 || count > size - start { Outcome::Bounds(count) } else {
      Outcome::Ok(Text::slice(text, start, count))
    }
  }
}
''', "helpers.lobo": CLIP_HELPER.replace("Bytes::length(Text::bytes(text))", "Text::length(text)")}}
}


def cases(name, heldout=False):
    tests = []
    if name == "rle":
        inputs = [[], [1], [1, 1], [1, 2], [2, 2, 1, 1, 1], [1, 2, 1], [0, 0, 0],
                  [-1, -1, 0, -1], [pilot.MIN, pilot.MAX, pilot.MAX], [3]*12, list(range(8)), [1, 1, 2, 1, 1, 2]]
        if heldout:
            rng = random.Random(2801)
            inputs += [[rng.randint(-4, 4) for _ in range(rng.randint(0, 32))] for _ in range(32)]
            inputs += [[pilot.MIN]*20+[pilot.MAX]*20, [2, 1]*30]
        for values in inputs:
            groups = []
            for value in values:
                if groups and groups[-1][0] == value:
                    groups[-1][1] += 1
                else:
                    groups.append([value, 1])
            expected = {"type": "List", "value": {"element": {"Record": "skilltask.run"},
                "elements": [study.record("skilltask.run", {"value": study.scalar(v), "count": study.scalar(n)}) for v, n in groups]}}
            tests.append(("encode", [study.list_arg(values)], expected))
    else:
        inputs = [("", 0, 0), ("", 0, 1), ("abc", 0, 3), ("abc", 1, 1), ("abc", 3, 0), ("abc", 3, 1),
                  ("abc", -1, 0), ("abc", 0, -1), ("🐺aé", 0, 1), ("🐺aé", 1, 1), ("🐺aé", 2, 1),
                  ("🐺aé", 3, 0), ("🐺aé", 4, 0), ("e\u0301", 1, 1), ("a\0b", 1, 1),
                  ("abc", pilot.MAX, pilot.MIN), ("abc", 0, pilot.MAX), ("abc", pilot.MIN, 0),
                  ("abc", 0, pilot.MIN), ("🐺🐺", 1, 1)]
        if heldout:
            rng = random.Random(2802)
            for _ in range(48):
                text = "".join(rng.choice(["a", "🐺", "é", "\u0301", "\0", "中"]) for _ in range(rng.randint(0, 12)))
                start = rng.choice([-1, 0, 1, len(text), len(text)+1, pilot.MAX])
                count = rng.choice([-1, 0, 1, 2, len(text), pilot.MAX])
                inputs.append((text, start, count))
        for text, start, count in inputs:
            if start < 0 or start > len(text):
                case, payload = "Bounds", study.scalar(start)
            elif count < 0 or count > len(text)-start:
                case, payload = "Bounds", study.scalar(count)
            else:
                case, payload = "Ok", {"type": "Text", "value": text[start:start+count]}
            tests.append(("clip", ["Text:"+text.encode().hex(), f"I64:{start}", f"I64:{count}"],
                          {"type": "Variant", "value": {"id": "skilltask.outcome", "case": case, "payload": payload}}))
    result = [{"id": f"behavior.{i}", "requirement": "behavior", "entry": entry, "arguments": args,
               "expect": {"kind": "value", "value": value}, "fuel": 2000000, "heap_cells": 100000}
              for i, (entry, args, value) in enumerate(tests)]
    result += [{"id": f"control.{i}", "requirement": "control", "entry": "control", "arguments": [f"I64:{i}"],
                "expect": {"kind": "value", "value": study.scalar(i)}, "fuel": 1000, "heap_cells": 1000} for i in range(4)]
    return result


class Session(pilot.Session):
    def __init__(self, task, tests):
        self.root, self.cases = task["root"], tests
        self.process = subprocess.Popen([str(pilot.RUNNER), "--session", "--clang", str(pilot.CLANG)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        try:
            response = self.call({"op": "open", "root": self.root, "sources": study.Session.bundle(task["sources"]),
                                  "requirements": [{"id": "behavior", "description": task["description"]},
                                                   {"id": "control", "description": "control(n) returns n unchanged"}], "tests": tests})
            if response["status"] != "opened":
                raise RuntimeError(response)
            self.snapshot = response["snapshot"]
        except BaseException:
            self.close()
            raise

    def verify(self, sources, scope):
        result = self.call({"op": "verify", "base": self.snapshot, "root": self.root,
                            "sources": study.Session.bundle(sources), "environment": "skill-evaluation-0.1",
                            "scope": "all", "max_cases": 256, "wall_ms": 15000})
        if "snapshot" in result:
            self.snapshot = result["snapshot"]
        return result


class Adapter(study.Adapter):
    def __init__(self, root, task, tests):
        self.root, self.task, self.arm = root, task, "json_full"
        self.address = root/".check-io"
        self.address.mkdir(mode=0o700)
        self.session, self.events = Session(task, tests), []
        self.started = time.monotonic()
        self.stopping = threading.Event()
        self.thread = threading.Thread(target=self.serve, daemon=True)
        self.thread.start()

    def dispatch(self, args):
        result = super().dispatch(args)
        self.events[-1]["at_seconds"] = time.monotonic()-self.started
        return result


def skill_files():
    return {str(p.relative_to(SKILL)): p.read_text() for p in sorted(SKILL.rglob("*")) if p.is_file()}


def flat_guide():
    intro = ("Equivalent flattened Lobo knowledge. Embedded links are relative to their original files under "
             "knowledge/skills/lobo-programming; those files are also available there. Canonical specs are under spec/.\n")
    return intro+"\n".join("\n--- FILE: "+name+" ---\n"+text for name, text in skill_files().items())


def fixture_files(arm):
    prefix = ".agents/skills/lobo-programming/" if arm == "skill" else "knowledge/skills/lobo-programming/"
    files = {prefix+name: text for name, text in skill_files().items()}
    files.update({"spec/"+p.name: p.read_text() for p in sorted((ROOT/"spec").glob("*.md"))})
    if arm == "guide":
        files["GUIDE.md"] = flat_guide()
    return files


def schedule():
    return [(repeat, name, arm) for repeat in range(5) for i, name in enumerate(TASKS)
            for arm in (["guide", "skill"] if (repeat+i) % 2 == 0 else ["skill", "guide"])]


def save(path, data):
    staging = path.with_suffix(".tmp")
    staging.write_text(json.dumps(data, indent=2)+"\n")
    os.replace(staging, path)


def read_report(path):
    return json.loads(path.read_text())


def require_pending(report, index):
    if report["trials"][index]["state"] != "pending":
        raise ValueError("Started or finished attempts cannot be replaced")


def check_examples():
    source = SKILL/"references/examples.lobo"
    trials = [
        ("skill.examples.counter", ["I64:4"], study.scalar(4)),
        ("skill.examples.exchange", ["Record:skill.examples.pair:I64:3,I64:9"],
         study.record("skill.examples.pair", {"left": study.scalar(9), "right": study.scalar(3)})),
        ("skill.examples.first", ["Variant:skill.examples.maybe:Case:Empty"], study.scalar(0)),
        ("skill.examples.framed", ["Text:f09f90ba"], {"type": "Text", "value": "<🐺>"})]
    for entry, args, expected in trials:
        cmd = [str(ROOT/"target/release/lobo"), "run", str(source), "--entry", entry]
        for value in args:
            cmd += ["--arg", value]
        data = json.loads(subprocess.check_output(cmd, text=True, timeout=15))
        if data.get("value") != expected:
            raise AssertionError(data)


TASK_TEXT = """Implement the documented Lobo behavior. Only program.lobo and helpers.lobo may change.
Preserve existing public IDs/signatures and nominal types; extra local declarations are allowed.
Use the required knowledge entry below. Read references as needed. All package/spec/client
fixtures are immutable. No network, installs, delegation, other projects or alternate compilers.
Host-required coding-skill files under /home/luis/.codex/skills and /home/luis/.agents/skills may
be read equally in either arm; other outside reference reads are not authorized. Loading cost
and reference reads count in this experiment.

Run python3 check.py before editing and after candidates. It uses native full checking and a
frozen independent oracle. --list, --detail TEST_ID and --replay TEST_ID retrieve visible inputs
and retained observations. Inspect statuses: exit 0 means all visible checks pass, exit 1 a
behavior failure, exit 2 invalid code/infrastructure. Replay can reproduce a failure. Eight
snapshots include open; do not silently reset a full session. Final grading uses additional
inputs satisfying the same behavior, in a fresh full session. Finish with a concise result.

KNOWLEDGE: {knowledge}
BEHAVIOR: {description}
"""


def task_text(name, arm):
    entry = "Use $lobo-programming at .agents/skills/lobo-programming/SKILL.md." if arm == "skill" else "Read GUIDE.md (all language/tool/example knowledge is included)."
    return TASK_TEXT.replace("{knowledge}", entry).replace("{description}", TASKS[name]["description"])


def prompt(arm):
    entry = "Use $lobo-programming. " if arm == "skill" else "Use GUIDE.md. "
    return entry+"Read TASK.txt and the two source files, implement the required behavior using the supplied checks, and change only the named sources."


def trial(index, repeat, name, arm, artifacts):
    task = TASKS[name]
    destination = artifacts/f"{index:02d}-{repeat}-{name}-{arm}"
    destination.mkdir(mode=0o700)
    with tempfile.TemporaryDirectory(prefix="lobo-skill-") as directory:
        root = pathlib.Path(directory)
        fixtures = fixture_files(arm)
        fixtures.update({"TASK.txt": task_text(name, arm), "check.py": study.CLIENT.format(address=str(root/".check-io"))})
        for name_, text in {**fixtures, **task["sources"]}.items():
            path = root/name_
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        adapter = Adapter(root, task, cases(name))
        initial = {n: pilot.digest(t.encode()) for n, t in task["sources"].items()}
        start = time.monotonic()
        row = {"repeat": repeat, "task": name, "arm": arm, "state": "complete", "timeout": False,
               "command": pilot.command(root), "load_before": os.getloadavg()}
        try:
            with (destination/"events.jsonl").open("w") as out, (destination/"stderr.txt").open("w") as err:
                proc = subprocess.Popen(pilot.command(root), stdin=subprocess.PIPE, stdout=out, stderr=err,
                                        text=True, start_new_session=True)
                try:
                    proc.communicate(prompt(arm), timeout=DEADLINE)
                except subprocess.TimeoutExpired:
                    row["timeout"] = True
                    os.killpg(proc.pid, signal.SIGTERM)
                    try:
                        proc.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(proc.pid, signal.SIGKILL)
                        proc.wait()
            row.update(exit_code=proc.returncode, agent_seconds=time.monotonic()-start)
        finally:
            adapter.close()
        events = [json.loads(line) for line in (destination/"events.jsonl").read_text().splitlines()]
        row["usage"], row["events"] = pilot.read_usage(events), adapter.events
        row["agent_errors"] = [e for e in events if e.get("type") in ["error", "turn.failed"]]
        commands = [e["item"]["command"] for e in events if e.get("type") == "item.completed" and e.get("item", {}).get("type") == "command_execution"]
        row["skill_entry_read"] = any("SKILL.md" in c and (".agents/skills/lobo-programming" in c or
                                                          ("cat SKILL.md" in c and "lobo-programming" in c)) for c in commands)
        row["reference_commands"] = [c for c in commands if "references/" in c or "spec/" in c]
        row["external_reference_commands"] = [c for c in commands if "/home/" in c]
        row["tool_items"] = sum(e.get("type") == "item.completed" and e.get("item", {}).get("type") in ["command_execution", "file_change", "mcp_tool_call"] for e in events)
        verifies = [e for e in row["events"] if e["args"] == []]
        edited = [e for e in verifies if e.get("source_sha256") != initial]
        admitted = [e for e in edited if e.get("result", {}).get("status") in ["checks_satisfied", "failed", "incomplete"]]
        row["first_admitted_edited_candidate_seconds"] = admitted[0]["at_seconds"] if admitted else None
        row["compiler_diagnostics"] = [e["result"] for e in edited if e.get("result", {}).get("status") == "compile_error"]
        row["candidate_count"] = len({json.dumps(e.get("source_sha256"), sort_keys=True) for e in edited})
        row["fixture_integrity"] = all((root/f).is_file() and not (root/f).is_symlink() and (root/f).read_text() == text for f, text in fixtures.items())
        allowed = {*fixtures, *task["sources"], ".check-io"}
        def owned(path):
            parts = path.relative_to(root).parts
            if len(parts) != 2 or parts[0] != ".check-io" or path.is_symlink():
                return False
            token, _, suffix = parts[1].partition(".")
            return len(token) == 32 and all(c in "0123456789abcdef" for c in token) and suffix in ["request.json", "response.json", "tmp"]
        row["unexpected_files"] = [str(p.relative_to(root)) for p in root.rglob("*") if not p.is_dir()
                                   and str(p.relative_to(root)) not in allowed and not owned(p)]
        try:
            sources = study.read_sources(root, task)
            row["final_source_sha256"] = {n: pilot.digest(s.encode()) for n, s in sources.items()}
            for file, text in sources.items():
                (destination/file).write_text(text)
            with Session(task, cases(name, True)) as judge:
                row["grade"] = judge.verify(sources, "all")
        except Exception as exc:
            row["grading_error"] = str(exc)[:1024]
        row["accepted"] = (not row["timeout"] and row["exit_code"] == 0 and row["fixture_integrity"]
                            and not row["unexpected_files"] and row.get("grade", {}).get("status") == "checks_satisfied"
                            and row["grade"]["full_execution"])
        row["elapsed_seconds"] = time.monotonic()-start
        save(destination/"report.json", row)
        return row


def identities():
    files = {"harness": pathlib.Path(__file__), "study": pathlib.Path(study.__file__), "pilot": pathlib.Path(pilot.__file__),
             "protocol": PROTOCOL, "runner": pilot.RUNNER}
    return {n: pilot.digest(p.read_bytes()) for n, p in files.items()}


def freeze(artifacts, canary):
    artifacts.mkdir(parents=True, mode=0o700)
    data = {"format": "lobo-skill-evaluation/0.1", "canary": canary, "identities": identities(),
            "requested_model": pilot.MODEL, "reasoning": "high", "deadline_seconds": DEADLINE,
            "codex_version": subprocess.check_output(["codex", "--version"], text=True).strip(),
            "clang_version": subprocess.check_output([str(pilot.CLANG), "--version"], text=True).splitlines()[0],
            "python_version": platform.python_version(), "platform": platform.platform(),
            "cpus": os.cpu_count(), "artifact_directory": str(artifacts), "skill": skill_files(),
            "specs": {p.name: p.read_text() for p in (ROOT/"spec").glob("*.md")},
            "tasks": {n: {"root": t["root"], "sources": t["sources"], "description": t["description"],
                           "visible": cases(n), "acceptance": cases(n, True)} for n, t in TASKS.items()},
            "prompts": {a: prompt(a) for a in ["guide", "skill"]},
            "trials": [{"repeat": r, "task": n, "arm": a, "state": "pending"} for r, n, a in schedule()]}
    for name, path in {"harness.py": pathlib.Path(__file__), "study.py": pathlib.Path(study.__file__),
                       "pilot.py": pathlib.Path(pilot.__file__), "protocol.md": PROTOCOL}.items():
        (artifacts/name).write_bytes(path.read_bytes())
    save(artifacts/"results.json", data)
    return data


def run_block(artifacts, block):
    with (artifacts/"writer.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        path = artifacts/"results.json"
        report = read_report(path)
        if report["identities"] != identities() or report["skill"] != skill_files():
            raise ValueError("Frozen evaluation inputs changed")
        if report["specs"] != {p.name: p.read_text() for p in (ROOT/"spec").glob("*.md")}:
            raise ValueError("Frozen specifications changed")
        if any(row["state"] != "complete" for row in report["trials"][:4*block]):
            raise ValueError("Run earlier blocks first")
        for index in range(4*block, min(4*block+4, 20)):
            require_pending(report, index)
            repeat, name, arm = schedule()[index]
            report["trials"][index]["state"] = "running"
            save(path, report)
            print(f"Starting {index+1}/20 {name} {arm}", flush=True)
            start = time.monotonic()
            try:
                row = trial(index, repeat, name, arm, artifacts)
            except Exception as exc:
                row = {"repeat": repeat, "task": name, "arm": arm, "state": "complete", "accepted": False,
                       "timeout": False, "usage": None, "events": [], "infrastructure_error": str(exc)[:1024],
                       "elapsed_seconds": time.monotonic()-start}
            report["trials"][index] = row
            save(path, report)
            print(f"Finished accepted={row['accepted']} seconds={row['elapsed_seconds']:.2f}", flush=True)


def summarize(report):
    groups = {}
    for task in TASKS:
        for arm in ["guide", "skill"]:
            rows = [r for r in report["trials"] if r["task"] == task and r["arm"] == arm]
            finished = [r for r in rows if r["state"] == "complete"]
            result = {"scheduled": len(rows), "complete": len(finished),
                      "accepted": sum(r["accepted"] for r in finished),
                      "timeouts": sum(r.get("timeout", False) for r in finished),
                      "infrastructure_errors": sum(bool(r.get("infrastructure_error") or r.get("grading_error") or
                                                        r.get("agent_errors")) for r in finished),
                      "skill_entry_read": sum(r.get("skill_entry_read", False) for r in finished),
                      "compiler_diagnostic_count": sum(len(r.get("compiler_diagnostics", [])) for r in finished),
                      "fixture_failures": sum(not r.get("fixture_integrity", False) for r in finished),
                      "elapsed_seconds_all": study.spread([r["elapsed_seconds"] for r in finished
                                                          if r.get("elapsed_seconds") is not None]),
                      "elapsed_seconds_accepted": study.spread([r["elapsed_seconds"] for r in finished
                                                               if r["accepted"] and r.get("elapsed_seconds") is not None]),
                      "first_admitted_edited_candidate_seconds": study.spread([r["first_admitted_edited_candidate_seconds"]
                          for r in finished if r.get("first_admitted_edited_candidate_seconds") is not None]),
                      "candidate_count": study.spread([r["candidate_count"] for r in finished if "candidate_count" in r]),
                      "host_seconds": sum(e["seconds"] for r in finished for e in r.get("events", [])),
                      "feedback_bytes": sum(e["feedback_bytes"] for r in finished for e in r.get("events", []))}
            for key in ["input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens"]:
                result[key] = study.spread([r["usage"][key] for r in finished if r.get("usage") and key in r["usage"]])
            groups[task+"/"+arm] = result
    paired = {}
    for task in TASKS:
        deltas = []
        for repeat in range(5):
            a = next(r for r in report["trials"] if (r["repeat"], r["task"], r["arm"]) == (repeat, task, "guide"))
            b = next(r for r in report["trials"] if (r["repeat"], r["task"], r["arm"]) == (repeat, task, "skill"))
            if a.get("accepted") and b.get("accepted") and a.get("elapsed_seconds") is not None and b.get("elapsed_seconds") is not None:
                deltas.append(b["elapsed_seconds"]-a["elapsed_seconds"])
        rng = random.Random(2800)
        boot = sorted(statistics.median(rng.choices(deltas, k=len(deltas))) for _ in range(10000)) if deltas else []
        paired[task] = {"both_accepted_pairs": len(deltas), "skill_minus_guide_seconds": study.spread(deltas),
                        "deltas": deltas, "exploratory_bootstrap_95_percent": [boot[249], boot[9749]] if boot else None}
    return {"by_task_arm": groups, "paired": paired}


def export(artifacts, destination):
    report = read_report(artifacts/"results.json")
    if report["canary"] or any(r["state"] != "complete" for r in report["trials"]):
        raise ValueError("Only export the full completed scheduled study")
    result = copy.deepcopy(report)
    result["summary"] = summarize(report)
    result["knowledge_sha256"] = {n: pilot.digest(t.encode()) for n, t in result.pop("skill").items()}
    result["specifications_sha256"] = {n: pilot.digest(t.encode()) for n, t in result.pop("specs").items()}
    result["task_plan_sha256"] = {n: pilot.digest(json.dumps(t, sort_keys=True).encode()) for n, t in result.pop("tasks").items()}
    result["raw_report_sha256"] = pilot.digest((artifacts/"results.json").read_bytes())
    for row in result["trials"]:
        for event in row.get("events", []):
            for field in ["bundle", "tests"]:
                value = event.get("result", {}).pop(field, None)
                if value is not None:
                    event["result"][field+"_sha256"] = pilot.digest(json.dumps(value, sort_keys=True).encode())
    save(destination, result)
    print(json.dumps(result["summary"], indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--init", action="store_true")
    parser.add_argument("--canary", action="store_true")
    parser.add_argument("--directory", type=pathlib.Path)
    parser.add_argument("--block", type=int, choices=range(5))
    parser.add_argument("--export", type=pathlib.Path)
    args = parser.parse_args()
    if args.canary:
        artifacts = ROOT/"benchmarks/private-evidence"/("skill-canary-"+time.strftime("%Y%m%dT%H%M%S"))
        freeze(artifacts, True)
        for i, (name, arm) in enumerate([("rle", "skill"), ("clip", "guide")]):
            print("Canary "+name+" "+arm, flush=True)
            row = trial(i, 0, name, arm, artifacts)
            save(artifacts/(name+"-canary.json"), row)
            print(f"Accepted={row['accepted']} skill_read={row['skill_entry_read']} seconds={row['elapsed_seconds']:.2f}", flush=True)
        print(artifacts, flush=True)
    elif args.init:
        artifacts = ROOT/"benchmarks/private-evidence"/("skill-study-"+time.strftime("%Y%m%dT%H%M%S"))
        freeze(artifacts, False)
        print(artifacts, flush=True)
    elif args.directory and args.export:
        export(args.directory.resolve(), args.export)
    elif args.directory and args.block is not None:
        run_block(args.directory.resolve(), args.block)
    else:
        parser.error("Choose --canary, --init, or --directory DIR --block 0..4")


if __name__ == "__main__":
    main()
