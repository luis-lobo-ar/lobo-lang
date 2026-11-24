import copy
import importlib.util
import json
from pathlib import Path
import unittest

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('learning',HERE/'learning.py')
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)


class LearningTests(unittest.TestCase):
    def test_canonical_format_answers_pass_every_check(self):
        for family in s.C.BASE.FAMILIES:
            result=s.grade(dict(stage='format',family=family),s.corpus.format_correct(family))
            self.assertTrue(result['accepted'],(family,result))

    def test_contract_accepts_notes_but_rejects_embedded_metadata(self):
        for family in s.C.BASE.FAMILIES:
            answer=s.corpus.format_correct(family)
            self.assertEqual(s.contract.errors('format',family,answer),[])
            answer['notes']='A concise explanation belongs here.'
            self.assertEqual(s.contract.errors('format',family,answer),[])
            answer['witness']['expected']={}
            self.assertTrue(s.contract.errors('format',family,answer))

    def test_contract_rejects_missing_fields_float_bool_and_duplicate_json_keys(self):
        answer=s.corpus.format_correct('ledger')
        del answer['witness']['initial']
        self.assertTrue(s.contract.errors('format','ledger',answer))
        for wrong in (1.0,True):
            answer=s.corpus.format_correct('ledger');answer['predictions'][0]['balance']=wrong
            self.assertTrue(s.contract.errors('format','ledger',answer))
        self.assertIsNone(s.contract.decode('{"predictions":[],"predictions":[]}'))

    def test_valid_shape_does_not_pass_wrong_behavior(self):
        answer=s.corpus.format_correct('ledger');answer['predictions'][0]['balance']+=1
        self.assertFalse(s.grade({'stage':'format','family':'ledger'},answer)['accepted'])
        self.assertEqual(s.contract.errors('format','ledger',answer),[])

    def test_nested_input_and_root_unknown_fields_are_explicitly_closed(self):
        answer=s.corpus.format_correct('scheduler');answer['witness']['jobs'][0]['result']=0
        self.assertTrue(any('unknown result' in e for e in s.contract.errors('format','scheduler',answer)))
        answer=s.corpus.format_correct('scheduler');answer['explanation']='wrong location'
        self.assertTrue(s.contract.errors('format','scheduler',answer))
        text=s.contract.documentation('format','scheduler')
        self.assertIn('unknown fields and duplicate keys are invalid',text)
        self.assertIn('top-level notes',text)

    def test_format_treatment_only_adds_contract(self):
        row=dict(stage='format',family='frames',language='lobo',arm='ambiguous')
        before=s.prompt(row);row['arm']='explicit';after=s.prompt(row)
        self.assertEqual(after,before+'\n'+s.contract.documentation('format','frames'))
        self.assertEqual(before.count('Task family:'),1)

    def test_reference_treatment_only_adds_reference(self):
        row=dict(stage='reference',family='ranges',language='lobo',arm='base')
        before=s.prompt(row);row['arm']='flat'
        self.assertEqual(s.prompt(row),before+'\n'+s.reference('flat'))

    def test_shared_admission_declarations_are_supplied_once(self):
        for stage,form in (('reference',0),('graph',1)):
            for topic in s.corpus.TOPICS:
                row=dict(stage=stage,family=topic,language='lobo',arm='base' if stage=='reference' else 'flat')
                self.assertEqual(s.prompt(row).count(s.corpus.micro(topic,form)['head']),1)

    def test_graph_and_flat_have_identical_facts_and_examples(self):
        flat=s.reference('flat');graph=s.reference('graph')
        self.assertEqual(s.reference_facts_hash('flat'),s.reference_facts_hash('graph'))
        parsed=json.loads(graph.split('\nCHECKED EXAMPLES\n')[0])
        source=s.REF.load()
        self.assertEqual(parsed['concepts'],source['nodes'])
        edges=[dict(source=k,**e) for k,values in parsed['adjacency'].items() for e in values]
        key=lambda e:(e['source'],e['relation'],e['target'])
        self.assertEqual(sorted(edges,key=key),sorted(source['edges'],key=key))
        for node in source['nodes']:
            for value in [node['title'],node['semantics'],*node['pitfalls']]:self.assertIn(value,flat)
            for ref in node['source_refs']:
                for value in ref.values():self.assertIn(value,flat)
        for edge in source['edges']:
            self.assertIn('Relationship: '+edge['source']+' | '+edge['relation']+' | '+edge['target'],flat)
        self.assertEqual(flat.split('\nCHECKED EXAMPLES\n')[1],graph.split('\nCHECKED EXAMPLES\n')[1])

    def test_schedule_is_bounded_balanced_and_stages_are_separate(self):
        rows=s.schedule();self.assertEqual(len(rows),84)
        self.assertEqual([sum(r['stage']==p for r in rows) for p in s.STAGES],[36,24,24])
        for stage in s.STAGES:
            for effort in ('high','low'):
                selected=[r for r in rows if r['stage']==stage and r['reasoning']==effort]
                self.assertEqual(len(selected),18 if stage=='format' else 12)
                self.assertEqual(set(r['language'] for r in selected),{'lobo','rust','python'} if stage=='format' else {'lobo'})
        for family in s.corpus.TOPICS:
            self.assertNotEqual(s.corpus.micro(family,0),s.corpus.micro(family,1))
            old={a['source'] for a in s.corpus.micro(family,0)['admissions']}
            self.assertTrue(all(a['source'] not in old for a in s.corpus.micro(family,1)['admissions']))

    def test_fresh_format_inputs_and_stable_identity_values(self):
        for family in s.C.BASE.FAMILIES:
            old=[json.loads(json.dumps(x)) for x in s.C.inputs(family)+s.C.BASE.prediction_inputs(family)]
            self.assertTrue(all(a not in old for a in s.corpus.format_inputs(family)))
            previous=json.loads(s.C.BASE.sources(family,'lobo')['logic.lobo.ids.json'])['identities']
            current=json.loads(s.corpus.format_sources(family,'lobo')['logic.lobo.ids.json'])['identities']
            self.assertEqual(sorted(previous.values()),sorted(current.values()))

    def test_correct_answers_and_item_failures(self):
        for topic in s.corpus.TOPICS:
            for form,stage in enumerate(('reference','graph')):
                answer=s.corpus.micro_correct(topic,form)
                self.assertTrue(s.grade(dict(stage=stage,family=topic),answer)['accepted'])
                answer['predictions'][0]+=1
                result=s.grade(dict(stage=stage,family=topic),answer)
                self.assertFalse(result['accepted']);self.assertEqual(result['passed'],9)


if __name__=='__main__':unittest.main()
