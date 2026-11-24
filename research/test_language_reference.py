import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import unittest
import xml.etree.ElementTree as ET

spec=importlib.util.spec_from_file_location('language_reference',Path(__file__).with_name('language_reference.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)


class ReferenceTests(unittest.TestCase):
    def test_graph_provenance_and_unique_nodes(self):
        data=r.load();r.validate(data)
        self.assertGreaterEqual(len(data['nodes']),18)
        self.assertEqual(data['language_version'],'core/1.11')
        self.assertFalse(data['exposed_in_context_trials'])

    def test_bounded_topic_lookup(self):
        result=r.lookup(r.load(),'try',neighbors=3)
        self.assertEqual(result['topic']['id'],'try')
        self.assertLessEqual(len(result['related']),3)
        self.assertIn('No implicit resource cleanup',result['topic']['pitfalls'])
        self.assertRaises(KeyError,r.lookup,r.load(),'unknown')
        self.assertRaises(ValueError,r.lookup,r.load(),'try',neighbors=100)

    def test_missing_edge_target_is_rejected(self):
        data=copy.deepcopy(r.load());data['edges'][0]['target']='missing'
        self.assertRaises(ValueError,r.validate,data)

    def test_source_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'source.md').write_text('rule')
            data={'nodes':[{'id':'n','source_refs':[{'path':'source.md','sha256':'0'*64,'anchor':'rule'}]}],'edges':[]}
            self.assertRaises(ValueError,r.validate,data,root)

    def test_drawio_encodes_every_node_and_relationship(self):
        data=r.load();xml=ET.fromstring(r.diagram(data))
        self.assertEqual(xml.attrib['adaptiveColors'],'auto')
        cells=xml.findall('./root/mxCell');ids=[x.attrib['id'] for x in cells]
        self.assertEqual(len(ids),len(set(ids)))
        self.assertEqual(sum(x.get('vertex')=='1' for x in cells),len(data['nodes']))
        edges=[x for x in cells if x.get('edge')=='1']
        self.assertEqual(len(edges),len(data['edges']))
        self.assertTrue(all(x.find('mxGeometry').get('relative')=='1' for x in edges))

    def test_reading_examples_on_frozen_native_compiler(self):
        compiler=r.ROOT/'benchmarks/private-evidence/matched-study-20261123/freeze/lobo'
        with tempfile.TemporaryDirectory() as d:
            binary=Path(d)/'reading'
            build=subprocess.run([str(compiler),'compile',str(r.DIRECTORY/'reading-examples.lobo'),'-o',str(binary)],capture_output=True,text=True,timeout=35)
            self.assertEqual(build.returncode,0,build.stdout+build.stderr)
            for entry,value in [('slice_count',3),('spread',11),('counted',6),('stopped',4),('list_order',2),('propagate',9),('propagate_error',True)]:
                run=subprocess.run([str(binary),'--entry','agentref.'+entry],capture_output=True,text=True,timeout=10)
                self.assertEqual(run.returncode,0,run.stderr+run.stdout)
                actual=json.loads(run.stdout)['value']
                self.assertEqual(actual['type'],'Bool' if type(value) is bool else 'I64')
                self.assertEqual(actual['value'],value if type(value) is bool else str(value))

    def test_native_count_example_kills_end_offset_mistake(self):
        compiler=r.ROOT/'benchmarks/private-evidence/matched-study-20261123/freeze/lobo'
        with tempfile.TemporaryDirectory() as d:
            folder=Path(d);source=folder/'reading-examples.lobo'
            source.write_text((r.DIRECTORY/source.name).read_text().replace('slice(data, 1, 3)','slice(data, 1, 2)'))
            (folder/(source.name+'.ids.json')).write_text((r.DIRECTORY/(source.name+'.ids.json')).read_text())
            binary=folder/'mutant'
            build=subprocess.run([str(compiler),'compile',str(source),'-o',str(binary)],capture_output=True,text=True,timeout=35)
            self.assertEqual(build.returncode,0,build.stdout+build.stderr)
            run=subprocess.run([str(binary),'--entry','agentref.slice_count'],capture_output=True,text=True,timeout=10)
            self.assertEqual(run.returncode,0,run.stderr+run.stdout)
            self.assertNotEqual(json.loads(run.stdout)['value']['value'],'3')


if __name__=='__main__':unittest.main()
