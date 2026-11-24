#!/usr/bin/env python3
"""Validate, query and draw the additive versioned language reading reference."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT=Path(__file__).resolve().parents[1]
DIRECTORY=ROOT/'docs/agent-reference/1.11.2'


def load():return json.loads((DIRECTORY/'concepts.json').read_text())


def validate(data,root=ROOT):
    ids=[n['id'] for n in data['nodes']]
    if len(ids)!=len(set(ids)):raise ValueError('duplicate concept ID')
    for node in data['nodes']:
        if not node['source_refs']:raise ValueError('missing provenance')
        for ref in node['source_refs']:
            path=(root/ref['path']).resolve()
            if not path.is_relative_to(root.resolve()) or not path.is_file():raise ValueError('invalid source path')
            if hashlib.sha256(path.read_bytes()).hexdigest()!=ref['sha256']:raise ValueError('source changed: '+ref['path'])
            if ref['anchor'] not in path.read_text():raise ValueError('source anchor missing')
    for edge in data['edges']:
        if edge['source'] not in ids or edge['target'] not in ids or not edge['relation']:raise ValueError('invalid relationship')


def lookup(data,topic,neighbors=3):
    if not 0<=neighbors<=8:raise ValueError('neighbors must be 0..8')
    nodes={n['id']:n for n in data['nodes']}
    if topic not in nodes:raise KeyError(topic)
    edges=[e for e in data['edges'] if topic in (e['source'],e['target'])]
    related=[]
    for edge in edges:
        key=edge['target'] if edge['source']==topic else edge['source']
        if key not in related:related.append(key)
    selected=related[:neighbors]
    return dict(language_version=data['language_version'],topic=nodes[topic],related=[nodes[k] for k in selected],relationships=[e for e in edges if e['source'] in {topic,*selected} and e['target'] in {topic,*selected}],omitted_neighbors=len(related)-len(selected))


def diagram(data):
    graph=ET.Element('mxGraphModel',adaptiveColors='auto',dx='1600',dy='1500',grid='1',gridSize='10')
    root=ET.SubElement(graph,'root');ET.SubElement(root,'mxCell',id='0');ET.SubElement(root,'mxCell',id='1',parent='0')
    # Topic lanes keep common dependencies nearby. The JSON remains the machine-facing graph.
    lanes=[['blocks','records','updates','generics','imports','identities','json'],['fold','while','lists','variants','match','try','wire-input'],['callbacks','actions','permissions','resources','limits','native','bytes','text']]
    colors=['#dae8fc','#d5e8d4','#ffe6cc']
    by_id={n['id']:n for n in data['nodes']}
    for col,lane in enumerate(lanes):
        for row,key in enumerate(lane):
            node=by_id[key]
            cell=ET.SubElement(root,'mxCell',id=key,value=node['title']+'\n['+node['category']+']',vertex='1',parent='1',style='rounded=1;whiteSpace=wrap;html=0;fillColor='+colors[col]+';fontSize=14;')
            ET.SubElement(cell,'mxGeometry',x=str(70+col*500),y=str(60+row*200),width='220',height='70',attrib={'as':'geometry'})
    for i,edge in enumerate(data['edges']):
        cell=ET.SubElement(root,'mxCell',id='edge-'+str(i),value=edge['relation'],edge='1',parent='1',source=edge['source'],target=edge['target'],style='edgeStyle=orthogonalEdgeStyle;rounded=0;jettySize=auto;html=0;endArrow=block;fontSize=11;labelBackgroundColor=default;')
        ET.SubElement(cell,'mxGeometry',relative='1',attrib={'as':'geometry'})
    ET.indent(graph)
    return ET.tostring(graph,encoding='unicode')+'\n'


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('operation',choices=['check','lookup','draw']);p.add_argument('topic',nargs='?');p.add_argument('--neighbors',type=int,default=3);a=p.parse_args();data=load();validate(data)
    if a.operation=='draw':(DIRECTORY/'language-relationships.drawio').write_text(diagram(data));print(DIRECTORY/'language-relationships.drawio')
    elif a.operation=='lookup':
        try:print(json.dumps(lookup(data,a.topic,a.neighbors),indent=2))
        except (KeyError,ValueError) as exc:p.error(str(exc))
    else:print(json.dumps(dict(nodes=len(data['nodes']),relationships=len(data['edges']),provenance='verified')))
