"""Behavior checks for the review inventory, independent of repository counts."""
import unittest
from dx_inventory import lobo_functions, mask_lobo

class InventoryTests(unittest.TestCase):
    def test_comments_and_strings_do_not_create_functions_or_branches(self):
        source='// fn fake() { if true {} }\nfn real()->Text { "fn fake() { if true {} }" }'
        functions=lobo_functions(source)
        self.assertEqual([f['name'] for f in functions],['real'])
        self.assertEqual(functions[0]['if_tokens'],0)
        self.assertEqual(mask_lobo(source).count('\n'),source.count('\n'))

    def test_action_and_contract_braces_are_not_the_function_body(self):
        source='fn fetch()->I64 actions {db.read,db.close} requires {if true {true} else {false}} {let value=1; if value==1 {value} else {0}}'
        fn=lobo_functions(source)[0]
        self.assertEqual(fn['actions'],['db.close','db.read'])
        self.assertEqual(fn['if_tokens'],1)
        self.assertEqual(fn['let_tokens'],1)

    def test_forwarded_fields_and_unused_names_are_candidates_not_semantics(self):
        source='fn step(s:S)->S {let ignored=1; S {x:s.x,y:s.y+1}}'
        fn=lobo_functions(source)[0]
        self.assertEqual(fn['forwarded_fields'],1)
        self.assertEqual(fn['unused_binding_candidates'],['ignored'])

if __name__=='__main__':unittest.main()
