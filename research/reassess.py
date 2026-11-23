#!/usr/bin/env python3
"""Reproduce research input accounting; this script launches no agents or model calls."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def verify_files(root, files):
    for item in files:
        path = root / item.get('snapshot', item.get('path', ''))
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != item['sha256']:
            raise ValueError(f'Changed or missing pinned input: {path}')


def knowledge_sizes(files):
    seen = set()
    totals = {}
    for item in files:
        key = item['version'], item['original_name']
        if key in seen:
            raise ValueError(f'Duplicate knowledge input: {key}')
        seen.add(key)
        totals[item['version']] = totals.get(item['version'], 0) + item['bytes']
    return totals


def compare_representation(legacy, source, metadata):
    return {'legacy_source_bytes': len(legacy), 'current_source_bytes': len(source),
            'current_metadata_bytes': len(metadata), 'current_combined_bytes': len(source) + len(metadata),
            'source_reduction_percent': (1 - len(source) / len(legacy)) * 100,
            'combined_reduction_percent': (1 - (len(source) + len(metadata)) / len(legacy)) * 100,
            'billed_token_effect': None}


def summarize_trials(trials):
    known = sum(isinstance(t.get('usage'), dict) and type(t['usage'].get('input_tokens')) is int
                for t in trials)
    return {'scheduled': len(trials), 'accepted': sum(t.get('accepted') is True for t in trials),
            'usage_known': known, 'usage_unknown': len(trials) - known}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True, help='New output JSON file')
    args = parser.parse_args()
    manifest = json.loads((ROOT / 'research/snapshots/2026-11-23-reassessment/knowledge.json').read_text())
    verify_files(ROOT, manifest['files'])
    for item in manifest['files']:
        if (ROOT / item['snapshot']).stat().st_size != item['bytes']:
            raise ValueError('Knowledge byte count disagrees with its pinned file')
    index = json.loads((ROOT / 'research/local-evidence.json').read_text())
    for entry in index['entries']:
        verify_files(ROOT, entry['files'])
    names = ['2026-11-03-agent-repair-pilot.json', '2026-11-03-library-repair-study.json',
             '2026-11-03-skill-evaluation.json']
    studies = {name: summarize_trials(json.loads((ROOT / 'benchmarks/results' / name).read_text())['trials'])
               for name in names}
    sources = ['examples/mvc.lobo', 'examples/mvc.compact.lobo', 'examples/mvc.compact.lobo.ids.json']
    report = {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'knowledge_bytes': knowledge_sizes(manifest['files']),
              'knowledge_current_snapshot_commit': manifest['current_source_commit'],
              'mvc': compare_representation(*[(ROOT / name).read_bytes() for name in sources]),
              'historical_agent_studies': studies, 'paper_entries_verified': len(index['entries']),
              'new_model_calls': 0, 'agent_construction_effect': None, 'agent_comprehension_effect': None,
              'limits': ['Bytes are not tokenizer counts or billed usage.',
                         'Knowledge coverage expanded between versions; this is not an equal-information trial.',
                         'Historical repeats are not independent task families.']}
    with args.out.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'status': 'pass', 'output': str(args.out)}))


if __name__ == '__main__':
    main()
