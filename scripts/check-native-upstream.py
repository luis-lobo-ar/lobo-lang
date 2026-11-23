#!/usr/bin/env python3
"""Verify pinned native test sources and attribution without network access."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
manifest = json.loads((root / 'tests/upstream/native/manifest.json').read_text())
projects = {item['name']: item for item in manifest['projects']}
known = set()
for item in manifest['files']:
    path = root / item['path']
    assert path.resolve().is_relative_to(root / 'tests/upstream/native'), item['path']
    assert item['path'] not in known, item['path']
    known.add(item['path'])
    project = projects[item['project']]
    assert len(project['commit']) == 40 and project['license']
    assert '/' + project['commit'] + '/' in item['url'], item['url']
    assert item['url'].endswith('/' + item['upstream_path'])
    assert hashlib.sha256(path.read_bytes()).hexdigest() == item['sha256'], item['path']
for group in manifest['adaptation_groups']:
    assert 'tests/upstream/native/' + group['source'] in known, group['source']
    for target in group['tests']:
        assert (root / target).is_file(), target
for project in projects:
    assert any(item['project'] == project and 'LICENSE' in item['upstream_path'] for item in manifest['files']), project
print(json.dumps({'status': 'pass', 'projects': len(projects), 'pinned_files': len(known), 'adaptation_groups': len(manifest['adaptation_groups'])}))
