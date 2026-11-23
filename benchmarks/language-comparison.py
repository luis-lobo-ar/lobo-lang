#!/usr/bin/env python3
"""Executable fixture comparison; these checks do not measure agent performance."""
import argparse
import hashlib
import itertools
import json
import pathlib
import platform
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURES = ROOT/'benchmarks/language-comparison/fixtures'
LOBO = ROOT/'target/release/lobo'
ARMS = ('lobo-variants', 'lobo-flags', 'rust-variants', 'rust-flags', 'python-flags')
SUFFIXES = {'lobo': '.lobo', 'rust': '.rs', 'python': '.py'}
PREDICTIONS = [(0, 7, 0), (1, 7, 1), (2, 7, 2), (0, 100, 0), (9, 101, 0), (2, 7, 0)]


def source_for(arm):
    return (FIXTURES/(arm+SUFFIXES[arm.split('-')[0]])).read_text()


def expected(kind, amount, action, upper=100):
    if amount < 0 or amount > upper:
        return -100
    transitions = {(0, 0): 1, (1, 1): 2, (2, 2): 0}
    result = transitions.get((kind, action))
    return -900 if result is None else result*1000+amount


def cases():
    return list(itertools.product((-1, 0, 1, 2, 3), (-1, 0, 1, 99, 100, 101), (-1, 0, 1, 2, 3)))


def build(arm, source, directory):
    language = arm.split('-')[0]
    path = directory/('program'+SUFFIXES[language])
    path.write_text(source)
    binary = directory/'program'
    if language == 'lobo':
        command = [str(LOBO), 'compile', str(path), '-o', str(binary)]
        invocation = [str(binary), '--entry', 'comparison.run']
    elif language == 'rust':
        command = ['rustc', '--edition=2021', '-O', str(path), '-o', str(binary)]
        invocation = [str(binary)]
    else:
        command = [sys.executable, '-m', 'py_compile', str(path)]
        invocation = [sys.executable, str(path)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
    return {'ok': result.returncode == 0, 'exit_code': result.returncode, 'command': command,
            'stdout': result.stdout, 'stderr': result.stderr, 'invocation': invocation, 'language': language}


def execute(built, inputs):
    command = list(built['invocation'])
    for value in inputs:
        command += ['--arg', 'I64:'+str(value)] if built['language'] == 'lobo' else [str(value)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=5, check=True)
    if built['language'] == 'lobo':
        return int(json.loads(result.stdout)['value']['value'])
    return int(result.stdout.strip())


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError('Mutation target is not unique: '+repr(before))
    return source.replace(before, after)


def mutated_source(arm, mutation):
    source = source_for(arm)
    if mutation == 'business_rule':
        return replace_once(source, '<= 100', '< 100')
    targets = {
        'lobo-variants': {
            'missing_case': (', Paid(n) { 2000 + n }', ''),
            'invalid_tag': ('State::Draft(amount)', 'State::Missing(amount)')},
        'lobo-flags': {
            'missing_case': ('if state.kind == 2 { 2000 + state.amount } else { -900 }', '-900'),
            'invalid_tag': ('State { kind: 0, amount: amount }', 'State { kind: 99, amount: amount }')},
        'rust-variants': {
            'missing_case': ('        State::Paid(n) => 2000 + n,\n', ''),
            'invalid_tag': ('State::Draft(amount)', 'State::Missing(amount)')},
        'rust-flags': {
            'missing_case': ('        2 => 2000 + state.amount,\n', ''),
            'invalid_tag': ('State { kind: 0, amount }', 'State { kind: 99, amount }')},
        'python-flags': {
            'missing_case': ('    if kind == 2:\n        return 2000 + amount\n', ''),
            'invalid_tag': ('    if kind == 0:\n        return (0, amount)',
                            '    if kind == 0:\n        return (99, amount)')},
    }
    return replace_once(source, *targets[arm][mutation])


def reference_answer():
    return {'outputs': [expected(*x) for x in PREDICTIONS], 'witness': [1, 7, 1],
            'direct_calls': ['valid', 'decode', 'step', 'code'], 'impacted': ['valid', 'run'],
            'fault_location': 'valid', 'counterexample': [0, 100, 0]}


def triple(value):
    return isinstance(value, list) and len(value) == 3 and all(type(x) is int and -1000 <= x <= 1000 for x in value)


def names(value, wanted):
    return isinstance(value, list) and all(isinstance(x, str) for x in value) and len(value) == len(wanted) and set(value) == set(wanted)


def grade_comprehension(answer):
    answer = answer if isinstance(answer, dict) else {}
    outputs = answer.get('outputs')
    checks = {}
    for i, inputs in enumerate(PREDICTIONS):
        checks['output_'+str(i)] = (isinstance(outputs, list) and len(outputs) == len(PREDICTIONS)
                                   and type(outputs[i]) is int and outputs[i] == expected(*inputs))
    witness = answer.get('witness')
    checks['witness'] = triple(witness) and expected(*witness) == 2007
    checks['direct_calls'] = names(answer.get('direct_calls'), ['valid', 'decode', 'step', 'code'])
    checks['impacted'] = names(answer.get('impacted'), ['valid', 'run'])
    checks['fault_location'] = answer.get('fault_location') == 'valid'
    counterexample = answer.get('counterexample')
    checks['counterexample'] = (triple(counterexample) and counterexample[0] in (0, 1, 2)
                                and counterexample[2] in (0, 1, 2)
                                and expected(*counterexample) != expected(*counterexample, upper=99))
    return {'accepted': all(checks.values()), 'passed': sum(checks.values()), 'total': len(checks), 'checks': checks}


def validate():
    rows = []
    for arm in ARMS:
        source = source_for(arm)
        with tempfile.TemporaryDirectory(prefix='lobo-comparison-') as directory:
            built = build(arm, source, pathlib.Path(directory))
            observations = []
            if built['ok']:
                for inputs in cases():
                    actual = execute(built, inputs)
                    observations.append({'inputs': inputs, 'expected': expected(*inputs), 'actual': actual,
                                         'passed': actual == expected(*inputs)})
            probes = []
            for mutation in ('missing_case', 'invalid_tag', 'business_rule'):
                with tempfile.TemporaryDirectory(prefix='lobo-probe-') as probe_directory:
                    probe_source = mutated_source(arm, mutation)
                    probe = build(arm, probe_source, pathlib.Path(probe_directory))
                    # All three mutations affect this input; keep compile rejection distinct from execution failure.
                    inputs = (1, 7, 1) if mutation == 'missing_case' else (0, 100, 0)
                    actual = execute(probe, inputs) if probe['ok'] else None
                    probes.append({'mutation': mutation, 'source_sha256': hashlib.sha256(probe_source.encode()).hexdigest(),
                                   'build': probe, 'inputs': inputs, 'expected': expected(*inputs), 'actual': actual,
                                   'caught_before_execution': not probe['ok'],
                                   'behavior_failed': actual != expected(*inputs) if probe['ok'] else None})
            rows.append({'arm': arm, 'source_sha256': hashlib.sha256(source.encode()).hexdigest(),
                         'source_bytes': len(source.encode()), 'build': built, 'cases': observations,
                         'passed': sum(o['passed'] for o in observations), 'total': len(cases()), 'probes': probes})
    versions = {}
    for name in ('rustc', 'python3', 'clang'):
        result = subprocess.run([name, '--version'], capture_output=True, text=True, check=True)
        versions[name] = result.stdout.splitlines()[0]
    return {'date': '2026-11-03', 'kind': 'fixture-equivalence-and-compiler-probes-not-agent-results',
            'hypotheses_tested_by_agents': [], 'comprehension_agent_scores': None,
            'platform': platform.platform(), 'versions': versions,
            'lobo_binary_sha256': hashlib.sha256(LOBO.read_bytes()).hexdigest(),
            'harness_sha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
            'unavailable_toolchains': [name for name in ('ocamlc', 'dafny', 'lean', 'tsc') if shutil.which(name) is None],
            'rows': rows, 'grader_reference': grade_comprehension(reference_answer()),
            'limits': ['One tiny state-workflow family; bounded equivalence, not universal semantics',
                       'Deterministic compiler/behavior checks do not measure agent construction or comprehension',
                       'No speed, token-efficiency or human-comprehension ranking']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=pathlib.Path, required=True, help='New JSON output path; existing results are never overwritten')
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Refusing to overwrite an existing result')
    report = validate()
    with args.output.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'arms': len(report['rows']), 'passed': sum(r['passed'] for r in report['rows']),
                      'total': sum(r['total'] for r in report['rows']), 'agent_trials': 0}))
    if any(not row['build']['ok'] or row['passed'] != row['total'] for row in report['rows']):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
