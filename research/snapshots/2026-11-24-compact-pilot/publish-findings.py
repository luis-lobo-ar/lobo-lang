from pathlib import Path
import json, hashlib, importlib.util, subprocess
ROOT=Path.cwd(); P=ROOT/'benchmarks/private-evidence/compact-study-20261124'
spec=importlib.util.spec_from_file_location('compact',ROOT/'benchmarks/matched-study/compact.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
def write(path,value):
 path=ROOT/path;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(value,indent=2)+'\n')
def insert(path,text):
 p=ROOT/path;old=p.read_text();head,body=old.split('\n',1);p.write_text(head+'\n\n'+text.strip()+'\n'+body)
reg=json.loads((P/'registration/manifest.json').read_text());result=json.loads((ROOT/'benchmarks/results/2026-11-24-compact-pilot.json').read_text());prior=json.loads((P/'prior-paper-pins.json').read_text());index=json.loads((ROOT/'research/local-evidence.json').read_text())
assert index['entries']==prior['entries']
assert all(s.sha(ROOT/f['path'])==f['sha256'] for f in prior['pins'])
assert all(s.sha(ROOT/path)==v for path,v in reg['code'].items())
assert s.sha(s.L.C.BASE.COMPILER)==reg['compiler_sha256']
assert result['attempted']==24 and len(result['unattempted'])==0 and len(result['audits'])==24
assert all(all(v for k,v in a.items() if k!='index') for a in result['audits'])
assert result['all_participant_usage']['known']==26 and result['all_participant_usage']['unknown']==0
exports={'reading-guide.txt':s.CLEAN}|{topic+'.txt':s.card(topic) for topic in s.TOPICS}
assert all((s.PACKET_PATH.parent/name).read_bytes()==text.encode() for name,text in exports.items())
validation=json.loads((P/'validation/results.json').read_text());probes=json.loads((P/'validation/signature-probes.json').read_text())
assert len(validation)==4 and all(len(r['values'])==12 and all(v['ok'] for v in r['values']) and len(r['admissions'])==4 and all(v['ok'] for v in r['admissions']) and r['mutation']['killed'] for r in validation)
assert len(probes)==28 and all(p['ok'] for p in probes)
changed=subprocess.run(['git','diff','--name-only','089ef07b29bfd8b732873578f1874b95bffe70ca'],capture_output=True,text=True,check=True).stdout.splitlines()
for path in changed:
 assert path.startswith(('benchmarks/matched-study/compact','benchmarks/matched-study/test_compact','docs/agent-reference/compact-pilot-2026-11-24/','research/snapshots/2026-11-24-compact-pilot/','benchmarks/results/2026-11-24-compact','decisions/0068')) or path=='AGENTS.md',path
write(P/'development/final-audit.json',dict(date='2026-11-24',registration_code_pins=len(reg['code']),all_code_pins_match=True,compiler_sha256=reg['compiler_sha256'],prior_entries=len(prior['entries']),prior_pins=len(prior['pins']),all_old_entries_and_pins_match=True,all_24_prompt_grade_acceptance_tool_audits_match=True,standalone_exports_match=True,validation=dict(native_evaluator_values=48,static_admissions=16,killed_compiled_mutations=4,signature_probes=28),changed_paths_since_prior_head=changed))
report='''# Small checked-reference pilot

Date: 2026-11-24. Decision [0068](../decisions/0068-small-guide-and-fact-pilot.md).
All 24 registered attempts completed on frozen compiler/extension 1.11.2, Core 1.11.
Short checked topic cards improved comprehension on these selected tasks. Guide cleanup alone did not.
No language, library, editor, old guide/reference/graph or historical score changed.

## What changed

The earlier study exposed missing builtin signatures and contextual grammar/contract rules. This pilot
adds a [reading-only guide and four cards](../docs/agent-reference/compact-pilot-2026-11-24/README.md).
The reading guide preserves the old semantic paragraphs verbatim, removes authoring/check instructions,
and adds a source-only reading cue. It shrinks from 3,443 to 2,003 UTF-8 bytes. Each targeted prompt adds
one relevant card, 744–934 bytes, rather than the entire reference. These are byte counts, not tokens.

The cards state buffer parameter/result types, byte versus scalar units, runtime fault boundaries,
nominal constructors and updates, inference limits, contextual try spellings, and contract Bool/purity
rules. They explicitly distinguish static admission from runtime success and proof. The machine-readable
packet keeps source hashes and anchors outside delivered prose. It covers these four topics, not the
whole language. General rules are taught; scored source names and gold answers are absent from cards.

## Registered procedure

Commit `ef3d410` was pushed before calls. The [registration](snapshots/2026-11-24-compact-pilot/registration/manifest.json)
pins 18 code/material files, 24 exact prompts, the compiler hash, validations, schedule and contrasts.
Four fresh program/admission sets cover ranges, nominal values/inference, propagation and contracts.
Each has six value predictions and four independent static-admission judgments. Every arm gets the same
program, question, Unicode input legend and strict v2 response contract. Correct identity metadata is
host-supplied. There is no scoring relaxation, repair or posthoc replacement.

Three arms use the existing guide, the cleaned reading guide, or the cleaned guide plus its topic card.
GPT-5.6-Luna runs 12 high-reasoning attempts, then 12 low, with rotated arm order within topics. Each
attempt has a fresh ephemeral read-only session, zero tools, a 120-second wall limit and no hard token cap.
The old process runner is reused through in-memory hooks; its disk bytes stay frozen. Two source-free
availability checks are excluded from scores and included in costs. All attempts completed normally;
there were no retries, replacements, tool calls, timeouts, infrastructure failures or unattempted tasks.

These examples were authored after reviewing earlier errors. They are development-targeted transfer
instances, not independently authored blind holdouts. The three preregistered comparisons are old→clean,
clean→targeted, and old→targeted. The second estimates the addition of facts conditional on cleanup.
There is no facts-only arm or measured interaction.

## Results

| Context | Whole tasks accepted | Correct items | Value predictions | Static admissions |
| --- | ---: | ---: | ---: | ---: |
| Existing guide | 0/8 | 68/80 (85.0%) | 47/48 | 21/32 |
| Reading-only guide | 0/8 | 67/80 (83.8%) | 48/48 | 19/32 |
| Reading guide + relevant card | 8/8 | 80/80 (100%) | 48/48 | 32/32 |

All 24 response shapes were valid. Every targeted-card task passed at both reasoning settings: 4/4
high and 4/4 low. High item scores are old35/40, clean34/40, targeted40/40; low scores are33/40,
33/40 and40/40. All arms already predict values well. Errors concentrate in static admissions, so
whole-task acceptance magnifies a small number of mistakes: a task needs all ten answers correct.
The targeted arm gains eight matched tasks against either baseline, with no reverse transition.

The [results](../benchmarks/results/2026-11-24-compact-pilot.json) retain all 25 failed item checks in
the other arms. Errors include rejecting accepted payload-free `Case()` construction; inferring generic
function arguments only from an expected return type; rejecting a statically accepted out-of-range byte
operation; confusing `Text::valid(Bytes)` with Text input; admitting direct propagation in contracts;
rejecting an ordinary declared function named try; treating a false postcondition as static rejection;
and allowing an actionful call inside an unreachable contract branch. One old/low reply also predicts
a fold result instead of propagation of its error. Some rules were already partly stated in the old
semantic paragraphs; the treatment adds precision and salience. It does not identify which sentence
caused each correction or prove the model's internal reason for a wrong judgment.

All outcomes, delivered prompts, exact answers, canonical answers and raw-file hashes are public in the
[trial bundles](../benchmarks/fixtures/compact-study-20261124/trial-00.json), indices00..23. No negative
result was removed. The grade replay agrees for every attempt, including every unsuccessful reply.

## Costs and timing

Ratios are medians of eight matched reported input-plus-output token totals, including failed attempts.
They are neither ratios of guide bytes nor billing estimates.

| Contrast | Item difference | Median token ratio | Median wall ratio |
| --- | ---: | ---: | ---: |
| Reading / existing | -1 | 0.9764 (2.4% lower) | 0.8748 |
| Targeted / reading | +13 | 1.0009 (0.09% higher) | 0.5971 |
| Targeted / existing | +12 | 0.9778 (2.2% lower) | 0.6251 |

Cleanup alone saves some reported tokens without improved accuracy. Adding precise facts to the cleaned
guide substantially improves these answers with effectively unchanged median token use. Against the
existing guide, the combined condition improves accuracy while using slightly fewer median tokens.
That combined token ratio ranges0.8451–0.9861; targeted/clean ranges0.8675–1.0100. Aggregate arm totals
are131,213,127,577 and125,184 tokens respectively. Aggregate reductions and median paired reductions
are different summaries and must not be substituted for each other.

The targeted condition has fewer output tokens (2,447 versus7,589 old and6,309 clean); this partly
offsets its additional facts. Reported reasoning is a subset of output. Cached prefixes, model-service
behavior and reasoning length affect costs/timing. The combined median wall ratio0.6251 is an observed
37.5% reduction in response wall time, not native program speed or an established inference-speed gain.
No local heavy compiler suite ran during participant timing.

Scored usage is367,629 input plus16,345 output, totaling383,974 tokens. Two availability calls add28,240.
All26 calls total412,214 reported tokens. Cached input310,656 is part of395,859 input; reasoning15,625
is part of16,355 output. Neither is added again. All terminal usage records are known. Billing, parent-agent
development tokens, underlying provider-request counts and immutable model weights remain unknown.

## Interpretation and next step

The promising change is compact, task-relevant, compiler-checked rules. Simply removing text did not
improve comprehension. Keep the packet available as a tested research artifact; do not replace frozen
study inputs or silently promote it to a universal reference. A later selected trial should use independent
fresh questions, broader topics and another model, then separately test bounded lookup/tool-assisted use.
No further participant run is part of this task.

The observed 100% is a small-pilot ceiling on four selected program instances repeated across reasoning
settings. Items share programs, and high precedes low; they are not80 independent samples. Prior errors
informed topic selection and card content. Whole-task0/8→8/8 must not be compared directly with older
studies' different tasks. We cannot claim full-language comprehension, generalization, noninferiority,
causal reasoning-level differences, better repository construction, Rust/Python superiority, production
correctness or native runtime gains. This supports a context-delivery hypothesis, not a language-feature claim.

## Verification and preservation

TDD preserves initial missing-module failures for the new driver and reporter, followed by six passing
new tests. Final harness tests pass40/40 and research tests15/15. Before participant calls, the frozen
compiler validates48 native/evaluator value observations,16 static cases,28 positive/negative signature
probes for14 operations, and four compiled behavior mutations that fail the expected results. These
are calibration observations, not extra model attempts or independent research replications.

The compiler SHA-256 is unchanged. Its earlier312-pass regression receipt is reused under decision0068;
that full compiler suite was not rerun here. No extension check is newly claimed. The
[verification receipt](../benchmarks/results/2026-11-24-compact-verification.json) records exact checks,
registration/prompt audits, code/export hashes and raw-archive integrity. Paper local-043 preserves all42
prior entries and141 pins, including the older scores and limitations. Public snapshots contain the
registered materials and development outcomes; local raw archives also retain native binaries and CLI
streams. Local archives are not off-device backups. Product and tested materials stay frozen. Stop at24.
'''
(ROOT/'research/compact-pilot-2026-11-24.md').write_text(report)
insert('CHANGELOG.md','''## 2026-11-24 — Small checked-reference pilot

Completed24 preregistered source-only attempts at frozen1.11.2. Existing/clean/targeted guides give
68/80,67/80,80/80 correct items and0/8,0/8,8/8 whole tasks. Short topic cards fix these selected
admission errors at both high/low reasoning. Cleanup alone gives no accuracy gain. Combined median
paired reported tokens fall2.2%; billing and generalization remain unknown. All outcomes, TDD failures,
new native/static/mutation checks and prior evidence are preserved. The unchanged312-pass compiler
receipt is reused, not newly rerun. No compiler/library/editor or old reference changes. See
[the report](research/compact-pilot-2026-11-24.md), decision0068 and paper local-043.
''')
insert('research/README.md','''## Small checked-reference pilot

The [24-attempt pilot](compact-pilot-2026-11-24.md) tests reading-only cleanup and relevant checked
cards at frozen1.11.2. Existing/clean/targeted item scores are68/80,67/80,80/80. All targeted tasks
pass at both reasoning settings, with combined median paired reported tokens2.2% lower than the old
guide. Cleanup alone does not improve accuracy. This is a selected four-topic development sample;
all errors, costs and limits remain in paper local-043. No more trials or product changes are implied.
''')
insert('research/local-evidence.md','''## Compact checked-reference pilot

Paper local-043 adds [the small three-arm pilot](compact-pilot-2026-11-24.md) and
[all24 results](../benchmarks/results/2026-11-24-compact-pilot.json). Targeted cards give80/80 items
and8/8 tasks versus68/80 and0/8 old; cleanup alone gives67/80 and0/8. Both reasoning settings benefit
on these selected tasks; combined median paired reported tokens are2.2% lower. The sample does not
establish a general language or model advantage. All42 prior entries and141 pins remain unchanged.
''')
p=ROOT/'benchmarks/results/README.md';text=p.read_text();anchor='| --- | --- | --- | --- |\n';row='| 2026-11-24 | Compact checked-reference pilot | All24 attempts complete; old/clean/targeted68/80,67/80,80/80 items and0/8,0/8,8/8 tasks. Combined median paired reported tokens2.2% lower; selected topics, no generalization. All26 usage records and prior evidence retained. | [Report](../../research/compact-pilot-2026-11-24.md), [results](2026-11-24-compact-pilot.json), [verification](2026-11-24-compact-verification.json) |\n';assert anchor in text;p.write_text(text.replace(anchor,anchor+row,1))
p=ROOT/'decisions/0068-small-guide-and-fact-pilot.md';text=p.read_text().replace('Status: selected; register before participant calls.','Status: complete; all24 registered attempts retained.');text+='''
## Observed outcome

Registration `ef3d410` preceded calls. Existing/clean/targeted arms give68/80,67/80,80/80 items and
0/8,0/8,8/8 whole tasks. Targeted cards pass4/4 high and4/4 low; cleanup alone does not improve
accuracy. Combined median paired reported tokens fall2.2%, with unknown billing. Preserve the additive
packet and all results as development-targeted research evidence; no universal default switch or claim
of general language/model superiority. See [the report](../research/compact-pilot-2026-11-24.md).
All26 participant/availability usage records are known. Stop at24; no additional calls or product edits.
''';p.write_text(text)
p=ROOT/'AGENTS.md';text=p.read_text();start=text.index('The user selected decision0068:');end=text.index('\n\nDecision 0067',start);text=text[:start]+'''Decision0068 is complete:24 attempts,26 known usage records, old/clean/targeted68/80,67/80,80/80
items and0/8,0/8,8/8 tasks. The checked topic cards help these selected cases at both reasoning levels;
guide cleanup alone does not. Combined median paired reported tokens are2.2% lower. Read
research/compact-pilot-2026-11-24.md before interpretation. Keep product1.11.2, new registered driver,
tested cards/reading guide and prior studies/reference/graph frozen. Preserve local-043 and all previous
entries/pins. The312-pass compiler receipt is reused, not a new full-suite run. Stop at24; no additional
trials, reference edits or product expansion without new user selection.''' +text[end:];p.write_text(text)
p=s.PACKET_PATH.parent/'README.md';text=p.read_text();text+='''
## Preserved text and outcome

Use [the reading guide](reading-guide.txt) with one relevant card:
[ranges](ranges.txt), [nominal values](nominal.txt), [propagation](propagation.txt), or
[contracts](contracts.txt). These text exports exactly match the registered rendered materials.

[The completed pilot](../../../research/compact-pilot-2026-11-24.md) gives80/80 correct items with
cards, compared with68/80 old and67/80 clean. This selected four-topic result does not establish
full-language comprehension. Keep the tested texts frozen; changes require a new study/material version.
''';p.write_text(text)
print('Findings and ledgers written; all registered inputs, old entries and141 pins verified.')
