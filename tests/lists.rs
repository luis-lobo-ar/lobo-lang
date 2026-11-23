use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn program(body: &str, output: &str) -> String {
    format!("module m @id(\"m\"); function main @id(\"main\")() -> {output} {{ {body} }}")
}

#[test]
fn lists_construct_share_traverse_and_serialize_in_reference_and_native() {
    let source = program("let empty = List::empty<I64>(); let a = List::prepend(1, empty); let b = List::prepend(2, a); if List::length(a) == 1 && List::length(empty) == 0 && List::head(b) == 2 { List::tail(b) } else { empty }", "List<I64>");
    let artifact = compile(&source).unwrap();
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let expected = json!({"type":"List","value":{"element":"I64","elements":[{"type":"I64","value":"1"}]}});
    assert_eq!(
        serde_json::to_value(loaded.run("main", vec![], 100).unwrap()).unwrap(),
        expected
    );
    let output = loaded.run_native(&["--entry".into(), "main".into()]).unwrap();
    assert!(output.status.success(), "{output:?}");
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["value"], expected);
}

#[test]
fn growable_state_supports_range_and_sum_without_a_new_loop() {
    let source = "module m @id(\"m\"); record State @id(\"state\") { xs: List<I64>, sum: I64 }
      function main @id(\"main\")(n: I64) -> I64 {
        let xs = fold(i in n, list: List<I64> = List::empty<I64>()) { List::prepend(i, list) };
        let done = fold(j in List::length(xs), state: State = State { xs: xs, sum: 0 }) {
          State { sum: state.sum + List::head(state.xs), xs: List::tail(state.xs) }
        }; done.sum
      }";
    let artifact = compile(source).unwrap();
    for (n, sum) in [(0, 0), (1, 0), (1000, 499500)] {
        assert_eq!(
            artifact.run("main", vec![Value::I64(n)], 100000).unwrap(),
            Value::I64(sum)
        );
        let output = artifact
            .run_native(&[
                "--entry".into(),
                "main".into(),
                "--arg".into(),
                format!("I64:{n}"),
            ])
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["value"]["value"], sum.to_string());
    }
}

#[test]
fn list_checks_reject_wrong_types_arity_and_unknown_element_types() {
    for (body, code) in [
        ("List::head(1)", "compile.list.invalid"),
        ("List::prepend(true, List::empty<I64>())", "compile.type.mismatch"),
        ("List::length()", "compile.list.invalid"),
        ("List::empty<Missing>()", "compile.type.unknown"),
    ] {
        assert_eq!(compile(&program(body, "I64")).unwrap_err().code, code, "{body}");
    }
}

#[test]
fn exact_heap_thresholds_cleanup_and_contract_fault_phases_agree() {
    let source = "module m @id(\"m\");
      function one @id(\"one\")() -> List<I64> { List::prepend(42, List::empty<I64>()) }
      function empty @id(\"empty\")() -> List<I64> { List::empty<I64>() }
      function head @id(\"head\")() -> I64 { List::head(List::empty<I64>()) }
      function fail @id(\"fail\")() -> I64 { let xs = List::prepend(1, List::empty<I64>()); 1 / 0 }
      function contract @id(\"contract\")() -> I64 requires { List::length(List::prepend(42, List::empty<I64>())) == 1 } { 42 }
      function nested @id(\"nested\")() -> I64 requires { List::length(one()) == 1 } { 42 }
      function post @id(\"post\")() -> I64 ensures { List::length(List::prepend(result, List::empty<I64>())) == 1 } { 42 }";
    let builds = Builds::new(&compile(source).unwrap());
    for (entry, heap, code, phase) in [
        ("one", 3, "resource.memory.exhausted", "execute"),
        ("head", 0, "runtime.list.empty", "execute"),
        ("fail", 4, "runtime.integer.zero_divisor", "execute"),
        ("contract", 0, "resource.memory.exhausted", "precondition"),
        ("nested", 0, "resource.memory.exhausted", "precondition"),
        ("post", 0, "resource.memory.exhausted", "postcondition"),
    ] {
        for mode in ["eval", "native", "sanitized"] {
            let result = builds.invoke(mode, entry, &[], 100, heap);
            assert_eq!(result["diagnostic"]["code"], code, "{mode}/{entry}");
            assert_eq!(result["diagnostic"]["phase"], phase, "{mode}/{entry}");
        }
    }
    for (entry, heap) in [("one", 4), ("empty", 0), ("contract", 4), ("post", 4)] {
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(builds.invoke(mode, entry, &[], 100, heap)["status"], "ok");
        }
    }
    for mode in ["eval", "native", "sanitized"] {
        let error = builds.invoke(mode, "one", &[], 0, 4);
        assert_eq!(error["diagnostic"]["code"], "resource.fuel.exhausted");
        assert_eq!(error["diagnostic"]["subject"], "one", "{mode}");
    }
}

#[test]
fn nested_list_inputs_validate_before_spending_and_preserve_typed_shapes() {
    let source = "module m @id(\"m\");
      record Row @id(\"row:1\") { values: List<I64>, flag: Bool }
      variant Maybe @id(\"maybe\") { None, Some(List<Row>) }
      function echo @id(\"echo\")(x: List<Maybe>) -> List<Maybe> { x }
      function nested @id(\"nested\")(x: List<List<I64>>) -> List<List<I64>> { x }";
    let artifact = compile(source).unwrap();
    let builds = Builds::new(&artifact);
    let number = json!({"type":"I64","value":"7"});
    let values = json!({"type":"List","value":{"element":"I64","elements":[number]}});
    let row = json!({"type":"Record","value":{"id":"row:1","fields":{"values":values,"flag":{"type":"Bool","value":true}}}});
    let rows = json!({"type":"List","value":{"element":{"Record":"row:1"},"elements":[row]}});
    let expected = json!({"type":"List","value":{"element":{"Variant":"maybe"},"elements":[{"type":"Variant","value":{"id":"maybe","case":"Some","payload":rows}}]}});
    let good = "List:Count:1,Case:Some,Count:1,Count:1,I64:7,Bool:true";
    // Outer Maybe width=2, Row width=2, I64 width=1: total 5+5+4=14 cells.
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(builds.invoke(mode, "echo", &[good], 100, 14)["value"], expected);
        assert_eq!(
            builds.invoke(mode, "echo", &[good], 0, 13)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
        for bad in [
            "List:Count:01",
            "List:Count:-1",
            "List:Count:+1",
            "List:Count:0,",
            "List:Count:1,Case:Unknown",
            "List:Count:1,Case:Some,Count:1,Count:1,Bool:true,Bool:true",
            "List:Count:1,Case:None,I64:1",
        ] {
            assert_eq!(
                builds.invoke(mode, "echo", &[bad], 0, 0)["diagnostic"]["code"],
                "invocation.arguments.invalid",
                "{mode}/{bad}"
            );
        }
        let nested = builds.invoke(mode, "nested", &["List:Count:2,Count:0,Count:1,I64:7"], 100, 12);
        assert_eq!(nested["status"], "ok", "{nested}");
        assert_eq!(nested["value"]["value"]["element"], json!({"List":"I64"}));
    }
}

#[test]
fn list_type_nesting_generic_closers_and_compound_metadata_are_bounded() {
    let nested = format!("{}I64{}", "List<".repeat(8), ">".repeat(8));
    let source = program(
        &format!("List::empty<{}I64{}>()", "List<".repeat(7), ">".repeat(7)),
        &nested,
    );
    assert!(compile(&source).is_ok());
    let source = program("let xs: List<I64>=List::empty<I64>(); List::length(xs)", "I64");
    assert_eq!(
        compile(&source).unwrap().run("main", vec![], 100).unwrap(),
        Value::I64(0)
    );
    let source = program(
        &format!("List::empty<{}I64{}>()", "List<".repeat(8), ">".repeat(8)),
        &format!("List<{nested}>"),
    );
    assert_eq!(compile(&source).unwrap_err().code, "resource.frontend.limit");
    let cycle = "module m @id(\"m\"); record R @id(\"r\") { next: List<R> } function main @id(\"main\")() -> I64 { 0 }";
    assert!(
        compile(cycle).is_ok(),
        "Core 0.10 admits recursion through list storage"
    );
    let name = "x".repeat(90_000);
    let source = format!("module m @id(\"m\"); record R @id(\"r\") {{ {name}: I64 }} function main @id(\"main\")() -> List<R> {{ let r = R {{ {name}: 1 }}; fold(i in 100, xs: List<R> = List::empty<R>()) {{ List::prepend(r,xs) }} }}");
    let artifact = compile(&source).unwrap();
    let Err(error) = artifact.run("main", vec![], 10000) else {
        panic!("oversized output must fail");
    };
    assert_eq!(error.code, "resource.output.limit");
    let output = artifact.run_native(&["--entry".into(), "main".into()]).unwrap();
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["diagnostic"]["code"], "resource.output.limit");
}

#[test]
fn shared_sublists_do_not_expand_unbounded_results_and_long_tails_drop_iteratively() {
    let source = program("let inner = fold(i in 1000, xs: List<I64> = List::empty<I64>()) { List::prepend(i,xs) }; fold(j in 1000, ys: List<List<I64>> = List::empty<List<I64>>()) { List::prepend(inner,ys) }", "List<List<I64>>");
    let artifact = compile(&source).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "main", &[], 100000, 10000);
        assert_eq!(
            result["diagnostic"]["code"], "resource.output.limit",
            "{mode}: {result}"
        );
        assert_eq!(result["diagnostic"]["phase"], "invoke");
    }
    let list = lobo_core::ListValue::from_values(lobo_core::Type::I64, (0..100000).map(Value::I64).collect());
    let alias = list.clone();
    drop(list);
    assert_eq!(alias.len(), 100000);
    drop(alias);
}

#[test]
fn frozen_collection_workload_runs_entirely_in_lobo_with_a_predicate_variation() {
    let artifact = lobo_core::compile_bundle(
        "collections",
        &[
            lobo_core::SourceFile {
                identities: None,
                name: "collections".into(),
                text: include_str!("../examples/collections.lobo").into(),
            },
            lobo_core::SourceFile {
                identities: None,
                name: "sequences".into(),
                text: include_str!("../examples/sequences.lobo").into(),
            },
        ],
    )
    .unwrap();
    let builds = Builds::new(&artifact);
    for (input, even, selected, sum) in [
        (vec![5, 2, 8, 3, 0, 10], true, vec![2, 8, 0, 10], 20),
        (vec![], true, vec![], 0),
        (vec![1, 3, 5], true, vec![], 0),
        (vec![1000, 1000], true, vec![1000, 1000], 2000),
        (vec![5, 2, 8, 3, 0, 10], false, vec![5, 3], 8),
    ] {
        let mut wire = format!("List:Count:{}", input.len());
        for value in input {
            wire.push_str(&format!(",I64:{value}"));
        }
        let expected = json!({"type":"Variant","value":{"id":"example.collections.selection","case":"Ok","payload":{"type":"Record","value":{"id":"example.collections.summary","fields":{
            "values":{"type":"List","value":{"element":"I64","elements":selected.iter().map(|n|json!({"type":"I64","value":n.to_string()})).collect::<Vec<_>>()}},
            "count":{"type":"I64","value":selected.len().to_string()},"sum":{"type":"I64","value":sum.to_string()}
        }}}}});
        for mode in ["eval", "native", "sanitized"] {
            assert_eq!(
                builds.invoke(
                    mode,
                    "example.collections.select",
                    &[&wire, if even { "Bool:true" } else { "Bool:false" }],
                    10000,
                    10000
                )["value"],
                expected,
                "{mode}"
            );
        }
    }
    for (wire, index, value) in [
        ("List:Count:3,I64:2,I64:-1,I64:4", 1, -1),
        ("List:Count:2,I64:1001,I64:-1", 0, 1001),
    ] {
        for mode in ["eval", "native", "sanitized"] {
            let output = builds.invoke(
                mode,
                "example.collections.select",
                &[wire, "Bool:true"],
                10000,
                10000,
            );
            assert_eq!(output["value"]["value"]["case"], "InvalidValue");
            let fields = &output["value"]["value"]["payload"]["value"]["fields"];
            assert_eq!(fields["index"]["value"], index.to_string());
            assert_eq!(fields["value"]["value"], value.to_string());
        }
    }
    for mode in ["eval", "native", "sanitized"] {
        let fault = builds.invoke(
            mode,
            "example.collections.select",
            &["List:Count:1,I64:2", "Bool:true"],
            1000,
            11,
        );
        assert_eq!(fault["diagnostic"]["code"], "resource.memory.exhausted");
        assert_eq!(fault["diagnostic"]["subject"], "example.collections.reverse");
    }
}

#[test]
fn serialized_byte_limit_includes_metadata_and_accepts_the_exact_boundary() {
    use lobo_core::{ListValue, RecordValue, Type};
    use std::collections::BTreeMap;
    let base = Value::List(ListValue::from_values(
        Type::Record("r".into()),
        (0..100)
            .map(|_| {
                Value::Record(RecordValue {
                    id: "r".into(),
                    fields: BTreeMap::from([("x".into(), Value::I64(1)), ("flag".into(), Value::Bool(true))]),
                })
            })
            .collect(),
    ));
    let base_bytes = serde_json::to_vec(&base).unwrap().len();
    let extra = 8 * 1024 * 1024 - base_bytes;
    let field = "x".repeat(1 + extra / 100);
    let false_count = extra % 100;
    let text = format!("module m @id(\"m\"); record R @id(\"r\") {{ {field}: I64, flag: Bool }} function main @id(\"main\")(over: Bool) -> List<R> {{ fold(i in 100, xs: List<R> = List::empty<R>()) {{ List::prepend(R {{ {field}: 1, flag: !(i < {false_count} || (over && i == {false_count})) }},xs) }} }}");
    let artifact = compile(&text).unwrap();
    let builds = Builds::new(&artifact);
    let value = artifact
        .run("main", vec![Value::Bool(false)], 10000)
        .expect("exactly 8 MiB is valid");
    assert_eq!(serde_json::to_vec(&value).unwrap().len(), 8 * 1024 * 1024);
    for mode in ["eval", "native", "sanitized"] {
        let good = builds.invoke(mode, "main", &["Bool:false"], 10000, 1000);
        assert_eq!(serde_json::to_vec(&good["value"]).unwrap().len(), 8 * 1024 * 1024);
        assert_eq!(
            builds.invoke(mode, "main", &["Bool:true"], 10000, 1000)["diagnostic"]["code"],
            "resource.output.limit"
        );
    }
}

#[test]
fn compiler_artifacts_and_values_remain_sendable_between_host_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Artifact>();
    assert_send_sync::<Value>();
}

#[test]
fn list_equality_charges_each_pair_and_respects_record_declaration_order() {
    let source = "module m @id(\"m\"); record R @id(\"r\") { z: List<I64>, a: List<I64> }
      function eq @id(\"eq\")(a: List<I64>, b: List<I64>) -> Bool { a == b }
      function records @id(\"records\")(a: R, b: R) -> Bool { a == b }";
    let builds = Builds::new(&compile(source).unwrap());
    let xs = "List:Count:3,I64:1,I64:2,I64:3";
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "eq", &[xs, xs], 7, 24)["value"],
            json!({"type":"Bool","value":true})
        );
        assert_eq!(
            builds.invoke(mode, "eq", &[xs, xs], 6, 24)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "eq", &[xs, "List:Count:0"], 4, 12)["value"]["value"],
            false
        );
        let a = "Record:r:Count:1,I64:1,Count:3,I64:1,I64:2,I64:3";
        let b = "Record:r:Count:1,I64:2,Count:3,I64:1,I64:2,I64:3";
        assert_eq!(
            builds.invoke(mode, "records", &[a, b], 5, 100)["value"]["value"],
            false
        );
    }
}

#[test]
fn reloaded_artifacts_cannot_smuggle_invalid_list_operations_or_values() {
    let artifact = compile(&program("List::prepend(1, List::empty<I64>())", "List<I64>")).unwrap();
    let original: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    for (pointer, value, code) in [
        (
            "/program/functions/0/body/value/op",
            json!("untrusted"),
            "compile.list.invalid",
        ),
        (
            "/program/functions/0/body/value/arguments",
            json!([]),
            "compile.list.invalid",
        ),
        (
            "/program/functions/0/body/value/arguments/1/element",
            json!({"Record":"missing"}),
            "compile.type.unknown",
        ),
        (
            "/program/functions/0/body/value",
            json!({"kind":"Literal","value":{"type":"List","value":{"element":"I64","elements":[]}}}),
            "compile.syntax.invalid",
        ),
    ] {
        let mut forged = original.clone();
        *forged.pointer_mut(pointer).unwrap() = value;
        assert_eq!(Artifact::from_json(&forged.to_string()).unwrap_err().code, code);
    }
    let source = "module m @id(\"m\"); function f @id(\"f\")(xs: List<I64>) -> I64 { List::length(xs) }";
    let artifact = compile(source).unwrap();
    let bad = Value::List(lobo_core::ListValue::from_values(
        lobo_core::Type::I64,
        vec![Value::Bool(true)],
    ));
    assert_eq!(
        artifact.run("f", vec![bad], 0).unwrap_err().code,
        "invocation.arguments.invalid"
    );
    let huge = Value::List(lobo_core::ListValue::from_values(
        lobo_core::Type::I64,
        (0..100000).map(Value::I64).collect(),
    ));
    assert_eq!(
        artifact.run("f", vec![huge], 0).unwrap_err().code,
        "resource.input.limit"
    );
}
