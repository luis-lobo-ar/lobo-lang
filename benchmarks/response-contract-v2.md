# Response contract v2

This research response profile fixes the closed-input ambiguity recorded in paper local-041.
The [executable definition](matched-study/response_contract.py) generates the exact disclosure text and
validates the response from the same field definitions. It does not change Lobo source syntax or any
earlier study's prompt, parser or score.

Every response is one JSON object. Every object is closed: all required fields must occur, unknown fields
are rejected, and duplicate keys are rejected at every depth. Integer fields require JSON integer tokens.
Floating-point tokens such as `1.0` and booleans do not satisfy an integer field. Byte strings use the
specified hexadecimal encoding. Array lengths stated in the profile are exact.

For the cross-language program tasks, required top-level fields are predictions, witness, direct_calls,
impacted, fault_location and counterexample. Predictions contains exactly six complete Summary objects.
The witness and counterexample are input objects with exactly the task's declared fields. Explanatory
output fields such as expected, summary, frames or original belong outside those input objects unless
the input profile explicitly declares that field.

The optional top-level notes field is a string for concise explanation. A note never replaces a required
field or alters how an input is executed. The primary grader never removes unknown properties or repairs
a reply. Invalid and partial replies remain evidence.

For Lobo-specific reading tasks, required fields are predictions and admissions. Predictions contains six
JSON integers for the entrypoint results. Admissions contains four JSON booleans for static-check
acceptance. Optional notes has the same meaning. Static acceptance does not claim runtime success,
termination or proof of a contract.

Format validity checks syntax, field placement, closure, lexical types, encoding and stated array lengths.
Task checks independently evaluate predictions, witnesses, dependency questions and type judgments.
Task-specific domain bounds and required behavior remain in the question. A well-shaped wrong answer
fails task checks; a correct answer with undeclared metadata fails format validity. Reports retain both
outcomes and whole-task acceptance.

Decision [0067](../decisions/0067-response-contract-reference-and-graph-experiments.md) compares the same
strict grader under ambiguous wording versus explicit disclosure. Both arms allow top-level notes.
There is no provider-native output constraint, automatic repair or retry. Reference and graph stages
both receive the explicit contract and use fresh questions separate from the format stage.
