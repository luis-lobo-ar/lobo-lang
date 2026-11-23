"""Behavioral oracles and executable checks for the language-comparison fixtures."""
import importlib.util
import pathlib
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "comparison", pathlib.Path(__file__).with_name("language-comparison.py"))
comparison = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(comparison)


class LanguageComparisonTests(unittest.TestCase):
    def test_frozen_behavior_including_priority_and_invalid_transitions(self):
        examples = [((0, 7, 0), 1007), ((1, 7, 1), 2007), ((2, 7, 2), 7),
                    ((0, 100, 0), 1100), ((0, 101, 0), -100),
                    ((9, 101, 0), -100), ((9, 7, 0), -900), ((2, 7, 0), -900)]
        for inputs, expected in examples:
            self.assertEqual(comparison.expected(*inputs), expected)

    def test_reordered_dependencies_and_alternative_counterexamples_are_accepted(self):
        answer = comparison.reference_answer()
        answer['direct_calls'] = ['code', 'step', 'decode', 'valid']
        answer['counterexample'] = [2, 100, 2]
        self.assertTrue(comparison.grade_comprehension(answer)['accepted'])
        answer['outputs'][0] = True
        result = comparison.grade_comprehension(answer)
        self.assertFalse(result['accepted'])
        self.assertFalse(result['checks']['output_0'])

    def test_shallow_answers_and_wrong_dependencies_cannot_pass(self):
        self.assertFalse(comparison.grade_comprehension({})['accepted'])
        answer = comparison.reference_answer()
        answer['direct_calls'] = ['valid', 'decode', 'step']
        answer['impacted'] = ['valid']
        answer['counterexample'] = [0, 99, 0]
        checks = comparison.grade_comprehension(answer)['checks']
        self.assertFalse(checks['direct_calls'])
        self.assertFalse(checks['impacted'])
        self.assertFalse(checks['counterexample'])

    def test_all_five_programs_agree_with_independent_oracle(self):
        for arm in comparison.ARMS:
            with self.subTest(arm=arm), tempfile.TemporaryDirectory() as directory:
                built = comparison.build(arm, comparison.source_for(arm), pathlib.Path(directory))
                self.assertTrue(built['ok'], built['stderr'])
                for inputs in comparison.cases():
                    self.assertEqual(comparison.execute(built, inputs), comparison.expected(*inputs))

    def test_behavior_mutation_is_detected_even_when_it_compiles(self):
        for arm in comparison.ARMS:
            with self.subTest(arm=arm), tempfile.TemporaryDirectory() as directory:
                source = comparison.mutated_source(arm, 'business_rule')
                built = comparison.build(arm, source, pathlib.Path(directory))
                self.assertTrue(built['ok'], built['stderr'])
                self.assertNotEqual(comparison.execute(built, (0, 100, 0)), 1100)

    def test_representation_probes_distinguish_static_rejection_from_wrong_behavior(self):
        for arm in comparison.ARMS:
            for mutation in ('missing_case', 'invalid_tag'):
                with self.subTest(arm=arm, mutation=mutation), tempfile.TemporaryDirectory() as directory:
                    built = comparison.build(arm, comparison.mutated_source(arm, mutation), pathlib.Path(directory))
                    self.assertEqual(built['ok'], not arm.endswith('variants'), built['stderr'])
                    if built['ok']:
                        inputs = (1, 7, 1) if mutation == 'missing_case' else (0, 100, 0)
                        self.assertNotEqual(comparison.execute(built, inputs), comparison.expected(*inputs))


if __name__ == '__main__':
    unittest.main()
