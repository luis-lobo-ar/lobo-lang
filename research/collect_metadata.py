#!/usr/bin/env python3
"""Retrieve primary bibliographic metadata; never infer a venue from an aggregator."""
import argparse
import hashlib
import html.parser
import json
import pathlib
import re
import time
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent


class CitationParser(html.parser.HTMLParser):
    def __init__(self):
        super().__init__()
        self.fields = {}

    def handle_starttag(self, tag, attrs):
        data = dict(attrs)
        if tag == 'meta' and data.get('name', '').startswith('citation_'):
            self.fields.setdefault(data['name'], []).append(data.get('content', ''))


def get(url, cache, label):
    request = urllib.request.Request(url, headers={'User-Agent': 'Lobo-literature-review/0.1'})
    with urllib.request.urlopen(request, timeout=20) as response:
        content = response.read(8 * 1024 * 1024)
    (cache/label).write_bytes(content)
    return content


def bib_escape(text):
    return text.replace('&', r'\&').replace('%', r'\%').replace('_', r'\_')


def collect():
    source = json.loads((ROOT/'sources.json').read_text())
    cache = ROOT/'.cache'
    cache.mkdir(mode=0o700, exist_ok=True)
    records = []
    for paper in source['papers']:
        row = dict(paper)
        try:
            if 'arxiv' in row:
                url = 'https://arxiv.org/abs/'+row['arxiv']
                content = get(url, cache, row['key']+'.html')
                parser = CitationParser()
                parser.feed(content.decode())
                meta = parser.fields
                row['title'] = meta['citation_title'][0]
                row['authors'] = meta['citation_author']
                row['date'] = meta.get('citation_date', [''])[0]
                row['updated'] = meta.get('citation_online_date', [''])[0]
                row['year'] = row['date'][:4]
                base = row['arxiv'].split('v')[0]
                versions = re.findall(re.escape(base)+r'v(\d+)', content.decode())
                version = row['arxiv'].split('v')[1] if 'v' in row['arxiv'] else max(map(int, versions), default=1)
                row['pinned_id'] = base+'v'+str(version)
                row['url'] = 'https://arxiv.org/abs/'+row['pinned_id']
                row['doi'] = '10.48550/arXiv.'+base
                row['metadata_sha256'] = hashlib.sha256(content).hexdigest()
                row['metadata_fields'] = {k: v for k, v in meta.items() if k != 'citation_abstract'}
                time.sleep(0.4)
            elif 'zenodo' in row:
                url = 'https://zenodo.org/api/records/'+row['zenodo']
                content = get(url, cache, row['key']+'.json')
                data = json.loads(content)
                meta = data['metadata']
                row.update(title=meta['title'], authors=[a['name'] for a in meta['creators']],
                           date=meta['publication_date'], year=meta['publication_date'][:4],
                           doi=meta['doi'], url='https://zenodo.org/records/'+row['zenodo'],
                           primary_api=url, resource_type=meta['resource_type'],
                           files=[{'name': f['key'], 'url': f['links']['self']} for f in data['files']],
                           metadata_sha256=hashlib.sha256(content).hexdigest())
            row['accessed'] = source['accessed']
        except Exception as error:
            row['retrieval_error'] = str(error)
        records.append(row)
        print(row['key']+': '+row.get('pinned_id', row.get('title', row.get('retrieval_error', '?'))), flush=True)
    output = {'accessed': source['accessed'], 'method': source['scope'], 'records': records,
              'collector_sha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
    (ROOT/'catalog.json').write_text(json.dumps(output, ensure_ascii=False, indent=2)+'\n')
    entries = []
    for row in records:
        if 'title' not in row:
            continue
        fields = {'title': '{'+row['title']+'}', 'author': ' and '.join(row['authors']),
                  'year': row['year'], 'url': row['url'], 'note': row['status']+'; accessed '+source['accessed']}
        if 'pinned_id' in row:
            fields.update(eprint=row['pinned_id'], archivePrefix='arXiv')
        if 'doi' in row:
            fields['doi'] = row['doi']
        entries.append('@misc{'+row['key']+',\n'+',\n'.join('  '+key+' = {'+bib_escape(value)+'}' for key, value in fields.items())+'\n}')
    (ROOT/'references.bib').write_text('\n\n'.join(entries)+'\n')
    if any('retrieval_error' in row for row in records):
        raise SystemExit('Some primary metadata retrievals failed; catalog retains failures')


if __name__ == '__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    collect()
