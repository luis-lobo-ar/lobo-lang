"""Executable strict response profile, rendered from the same field definitions used by validation."""
import json
import re

INTEGER={'kind':'integer','rule':'JSON integer token, not float or boolean'}
BOOLEAN={'kind':'boolean'}
TEXT={'kind':'string'}


def array(item,length=None):
    return dict(kind='array',items=item,**({'length':length} if length is not None else {}))


def obj(required,optional=None):return dict(kind='object',required=required,optional=optional or {},additional_properties=False)


def input_shape(family):
    if family=='ledger':return obj(dict(initial=INTEGER,events=array(dict(kind='tuple',items=[INTEGER,INTEGER]))))
    if family=='scheduler':return obj(dict(jobs=array(obj(dict(id=INTEGER,cost=INTEGER,deps=array(INTEGER)))),budget=INTEGER))
    return obj(dict(data=dict(kind='string',pattern='(?:[0-9a-fA-F]{2})*'),limit=INTEGER))


def summary_shape(family):
    return {'ledger':obj(dict(balance=INTEGER,rejected=INTEGER,peak=INTEGER,audit=array(INTEGER))),'scheduler':obj(dict(order=array(INTEGER),spent=INTEGER,blocked=INTEGER)),'frames':obj(dict(frames=array(dict(kind='string',pattern='(?:[0-9a-f]{2})*')),consumed=INTEGER,error=INTEGER))}[family]


def shape(stage,family):
    if stage!='format':return obj(dict(predictions=array(INTEGER,6),admissions=array(BOOLEAN,4)),dict(notes=TEXT))
    return obj(dict(predictions=array(summary_shape(family),6),witness=input_shape(family),direct_calls=array(TEXT),impacted=array(TEXT),fault_location=TEXT,counterexample=input_shape(family)),dict(notes=TEXT))


def validate(schema,value,path='$'):
    errors=[];kind=schema['kind']
    types={'object':dict,'array':list,'tuple':list,'integer':int,'boolean':bool,'string':str}
    if type(value) is not types[kind]:return [path+': expected '+kind]
    if kind=='object':
        missing=set(schema['required'])-set(value);extra=set(value)-set(schema['required'])-set(schema['optional'])
        if missing:errors.append(path+': missing '+','.join(sorted(missing)))
        if extra:errors.append(path+': unknown '+','.join(sorted(extra)))
        for k,s in (schema['required']|schema['optional']).items():
            if k in value:errors+=validate(s,value[k],path+'.'+k)
    elif kind in ('array','tuple'):
        length=len(schema['items']) if kind=='tuple' else schema.get('length')
        if length is not None and len(value)!=length:errors.append(path+': expected length '+str(length))
        for i,v in enumerate(value):
            if kind=='tuple':
                if i>=len(schema['items']):continue
                sub=schema['items'][i]
            else:sub=schema['items']
            errors+=validate(sub,v,path+'['+str(i)+']')
    elif kind=='string' and 'pattern' in schema and not re.fullmatch(schema['pattern'],value):errors.append(path+': encoding does not match '+schema['pattern'])
    return errors


def errors(stage,family,value):return validate(shape(stage,family),value)


def documentation(stage,family):
    return ('RESPONSE CONTRACT v2. Return exactly one JSON object, without Markdown. All objects in this profile are closed: every required field must occur, unknown fields and duplicate keys are invalid. Integers use integer tokens, never 1.0 or true. '+('witness and counterexample are INPUT objects only; do not put frames, consumed, error, summary, expected, original or mutant explanations inside them unless the field is declared by the input profile. ' if stage=='format' else '')+'Put any explanation in the optional top-level notes string. Shape validity and semantic correctness are separate checks. Input domain bounds in the task still apply.\n'+json.dumps(shape(stage,family),ensure_ascii=False,sort_keys=True)+'\n')


def decode(text):
    def pairs(values):
        result={}
        for k,v in values:
            if k in result:raise ValueError('duplicate key')
            result[k]=v
        return result
    try:return json.loads(text,object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(ValueError('nonfinite JSON')))
    except (ValueError,TypeError):return None
