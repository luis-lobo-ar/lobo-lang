use lobo_core::{compile, Artifact, Value};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn program(body: &str, output: &str) -> String {
    format!("module m @id(\"m\"); function main @id(\"main\")() -> {output} {{ {body} }}")
}
fn native(artifact: &Artifact) -> Json {
    let output = artifact.run_native(&["--entry".into(), "main".into()]).unwrap();
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice::<Json>(&output.stdout).unwrap()["value"].clone()
}

#[test]
fn unicode_literals_and_escapes_survive_checked_and_native_round_trips() {
    let source = program(r#""A\u0000\n\"\\🐺\uD83D\uDC3A""#, "Text");
    let artifact = compile(&source).unwrap();
    let artifact = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let expected = json!({"type":"Text","value":"A\0\n\"\\🐺🐺"});
    assert_eq!(
        serde_json::to_value(artifact.run("main", vec![], 100).unwrap()).unwrap(),
        expected
    );
    assert_eq!(native(&artifact), expected);
}

#[test]
fn byte_and_scalar_views_are_distinct_immutable_and_composable() {
    let source = program(
        r#"let t = "aé🐺"; let b = Text::bytes(t); if Text::length(t) == 3 && Bytes::length(b) == 7 && Text::at(t,2) == 128058 && Bytes::at(b,1) == 195 { Text::concat(Text::slice(t,1,2),"!") } else { "bad" }"#,
        "Text",
    );
    let artifact = compile(&source).unwrap();
    assert_eq!(native(&artifact), json!({"type":"Text","value":"é🐺!"}));
    assert_eq!(
        serde_json::to_value(artifact.run("main", vec![], 1000).unwrap()).unwrap(),
        native(&artifact)
    );
    let source = program("let a = Bytes::push(Bytes::empty(),255); let b = Bytes::push(a,0); if Bytes::length(a) == 1 { Bytes::slice(b,1,1) } else { a }","Bytes");
    let artifact = compile(&source).unwrap();
    assert_eq!(native(&artifact), json!({"type":"Bytes","value":"00"}));
}

#[test]
fn buffers_reject_wrong_arity_types_and_malformed_text_literals() {
    for (body, code) in [
        ("Text::length(1)", "compile.type.mismatch"),
        ("Bytes::at(Bytes::empty())", "compile.buffer.invalid"),
        ("Text::unknown(1)", "compile.buffer.invalid"),
        (r#""\uD800""#, "compile.syntax.invalid"),
        (r#""\q""#, "compile.syntax.invalid"),
    ] {
        assert_eq!(compile(&program(body, "I64")).unwrap_err().code, code, "{body}");
    }
    let scalar = compile(&program("Bytes::length(Bytes::empty())", "I64")).unwrap();
    assert_eq!(scalar.run("main", vec![], 100).unwrap(), Value::I64(0));
}

#[test]
fn decoded_buffer_input_limit_precedes_heap_spending_and_execution() {
    let artifact =
        compile("module m @id(\"m\"); function f @id(\"f\")(b: Bytes) -> I64 { Bytes::length(b) }").unwrap();
    let input = Value::Bytes(lobo_core::BufferValue::from_bytes(vec![0; 8 * 1024 * 1024 + 1]));
    assert_eq!(
        artifact.run_with_heap("f", vec![input], 0, 0).unwrap_err().code,
        "resource.input.limit"
    );
}

#[test]
fn independent_utf8_corpus_agrees_in_evaluator_optimized_and_leak_checked_native() {
    let source = "module m @id(\"m\"); record Scan @id(\"scan\") { remaining: List<Bytes>, flags: List<Bool> }
      function main @id(\"main\")(input: List<Bytes>) -> List<Bool> {
        let done = fold(i in List::length(input), state: Scan = Scan { remaining: input, flags: List::empty<Bool>() }) {
          Scan { remaining: List::tail(state.remaining), flags: List::prepend(Text::valid(List::head(state.remaining)),state.flags) }
        }; done.flags
      }";
    let builds = Builds::new(&compile(source).unwrap());
    let mut cases = vec![
        vec![],
        vec![0xc2, 0x80],
        vec![0xdf, 0xbf],
        vec![0xe0, 0xa0, 0x80],
        vec![0xed, 0x9f, 0xbf],
        vec![0xef, 0xbf, 0xbf],
        vec![0xf0, 0x90, 0x80, 0x80],
        vec![0xf4, 0x8f, 0xbf, 0xbf],
        vec![0xef, 0xbb, 0xbf],
        vec![0xc0, 0x80],
        vec![0xe0, 0x80, 0x80],
        vec![0xed, 0xa0, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
        vec![0xf5, 0x80, 0x80, 0x80],
        vec![0xed, 0xa1, 0x8c, 0xed, 0xbe, 0xb4],
    ];
    cases.extend((0..=255).map(|b| vec![b]));
    let mut seed = 0x28c4e7d1u32;
    for length in 2..=5 {
        for _ in 0..256 {
            let mut bytes = Vec::new();
            for _ in 0..length {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                bytes.push((seed >> 24) as u8);
            }
            cases.push(bytes);
        }
    }
    let mut wire = format!("List:Count:{}", cases.len());
    for case in &cases {
        wire.push_str(",Bytes:");
        for byte in case {
            wire.push_str(&format!("{byte:02x}"));
        }
    }
    let expected = cases
        .iter()
        .rev()
        .map(|bytes| json!({"type":"Bool","value":std::str::from_utf8(bytes).is_ok()}))
        .collect::<Vec<_>>();
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[&wire], 1000000, 100000)["value"]["value"]["elements"],
            json!(expected),
            "{mode}"
        );
    }
}

#[test]
fn buffer_heap_fuel_bounds_and_failure_phases_agree() {
    let source = "module m @id(\"m\");
      function push @id(\"push\")() -> Bytes { Bytes::push(Bytes::empty(),255) }
      function concat @id(\"concat\")() -> Text { Text::concat(\"a\",\"🐺\") }
      function view @id(\"view\")() -> Text { Text::from_bytes(Text::bytes(\"é\")) }
      function decode @id(\"decode\")(b: Bytes) -> Text { Text::from_bytes(b) }
      function at @id(\"at\")(b: Bytes, i: I64) -> I64 { Bytes::at(b,i) }
      function contract @id(\"contract\")() -> I64 requires { Bytes::length(Bytes::push(Bytes::empty(),1)) == 1 } { 42 }";
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "push", &[], 5, 4)["value"],
            json!({"type":"Bytes","value":"ff"})
        );
        assert_eq!(
            builds.invoke(mode, "push", &[], 4, 0)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "push", &[], 5, 3)["diagnostic"]["code"],
            "resource.memory.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "concat", &[], 9, 4)["value"],
            json!({"type":"Text","value":"a🐺"})
        );
        assert_eq!(
            builds.invoke(mode, "view", &[], 6, 0)["value"],
            json!({"type":"Text","value":"é"})
        );
        assert_eq!(
            builds.invoke(mode, "decode", &["Bytes:ff"], 4, 4)["diagnostic"]["code"],
            "runtime.text.invalid_utf8"
        );
        assert_eq!(
            builds.invoke(mode, "decode", &["Bytes:ff"], 3, 4)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "at", &["Bytes:ff", "I64:-1"], 10, 4)["diagnostic"]["code"],
            "runtime.bytes.bounds"
        );
        let error = builds.invoke(mode, "contract", &[], 100, 0);
        assert_eq!(error["diagnostic"]["phase"], "precondition");
        assert_eq!(error["diagnostic"]["code"], "resource.memory.exhausted");
    }
}

#[test]
fn nested_buffer_arguments_and_guarded_typed_decoding_preserve_shapes() {
    let source="module m @id(\"m\"); record Row @id(\"row\") { text: Text, bytes: Bytes }
      variant Decoded @id(\"decoded\") { Ok(Text), Invalid }
      function row @id(\"row.echo\")(r: Row) -> Row { r }
      function echo @id(\"echo\")(xs: List<Text>) -> List<Text> { xs }
      function decode @id(\"decode\")(b: Bytes) -> Decoded { if Text::valid(b) { Decoded::Ok(Text::from_bytes(b)) } else { Decoded::Invalid } }
      function bulk @id(\"bulk\")(xs: List<I64>) -> Bytes { Bytes::from_list(xs) }";
    let artifact = compile(source).unwrap();
    let builds = Builds::new(&artifact);
    let expected = json!({"type":"Record","value":{"id":"row","fields":{"text":{"type":"Text","value":"a🐺"},"bytes":{"type":"Bytes","value":"00ff"}}}});
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(
                mode,
                "row.echo",
                &["Record:row:Text:61F09F90BA,Bytes:00FF"],
                100,
                8
            )["value"],
            expected
        );
        assert_eq!(
            builds.invoke(mode, "echo", &["List:Count:2,Text:,Text:00"], 100, 14)["value"]["value"]
                ["elements"],
            json!([{"type":"Text","value":""},{"type":"Text","value":"\0"}])
        );
        assert_eq!(
            builds.invoke(mode, "decode", &["Bytes:ff"], 100, 4)["value"]["value"]["case"],
            "Invalid"
        );
        assert_eq!(
            builds.invoke(mode, "decode", &["Bytes:f09f90ba"], 100, 4)["value"]["value"]["payload"],
            json!({"type":"Text","value":"🐺"})
        );
        assert_eq!(
            builds.invoke(mode, "bulk", &["List:Count:3,I64:0,I64:127,I64:255"], 100, 16)["value"],
            json!({"type":"Bytes","value":"007fff"})
        );
        assert_eq!(
            builds.invoke(mode, "bulk", &["List:Count:1,I64:256"], 100, 4)["diagnostic"]["code"],
            "runtime.bytes.range"
        );
        for bad in [
            "Record:row:Text:ff,Bytes:",
            "Record:row:Text:c080,Bytes:",
            "Record:row:Text:0,Bytes:",
            "Record:row:Text:zz,Bytes:",
            "Record:row:Text:,Bytes:00,",
        ] {
            assert_eq!(
                builds.invoke(mode, "row.echo", &[bad], 0, 0)["diagnostic"]["code"],
                "invocation.arguments.invalid",
                "{mode}/{bad}"
            );
        }
    }
}

#[test]
fn empty_and_scalar_ranges_and_equality_have_explicit_fuel_rules() {
    let source = "module m @id(\"m\");
      function eq @id(\"eq\")(a: Text,b: Text) -> Bool { a == b }
      function at @id(\"at\")(t: Text,i: I64) -> I64 { Text::at(t,i) }
      function slice @id(\"slice\")(t: Text,start: I64,count: I64) -> Text { Text::slice(t,start,count) }";
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "eq", &["Text:6162", "Text:6162"], 6, 8)["value"]["value"],
            true
        );
        assert_eq!(
            builds.invoke(mode, "eq", &["Text:6162", "Text:6162"], 5, 8)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "eq", &["Text:6162", "Text:6362"], 5, 8)["value"]["value"],
            false
        );
        assert_eq!(
            builds.invoke(mode, "eq", &["Text:c3a9", "Text:65cc81"], 4, 8)["value"]["value"],
            false
        );
        assert_eq!(
            builds.invoke(mode, "at", &["Text:f09f90ba", "I64:-1"], 4, 4)["diagnostic"]["code"],
            "runtime.text.bounds"
        );
        assert_eq!(
            builds.invoke(mode, "at", &["Text:f09f90ba", "I64:1"], 7, 4)["diagnostic"]["code"],
            "resource.fuel.exhausted"
        );
        assert_eq!(
            builds.invoke(mode, "at", &["Text:f09f90ba", "I64:1"], 8, 4)["diagnostic"]["code"],
            "runtime.text.bounds"
        );
        assert_eq!(
            builds.invoke(mode, "slice", &["Text:f09f90ba", "I64:1", "I64:0"], 100, 4)["value"],
            json!({"type":"Text","value":""})
        );
        assert_eq!(
            builds.invoke(mode, "slice", &["Text:", "I64:0", "I64:0"], 100, 0)["value"],
            json!({"type":"Text","value":""})
        );
        assert_eq!(
            builds.invoke(
                mode,
                "slice",
                &["Text:61", "I64:1", "I64:9223372036854775807"],
                100,
                4
            )["diagnostic"]["code"],
            "runtime.text.bounds"
        );
    }
}

#[test]
fn all_json_control_escapes_and_literal_identity_boundaries_are_preserved() {
    let text = (0u8..32).map(char::from).collect::<String>() + "\"\\/\u{7f}\u{2028}\u{2029}é🐺";
    let source = program(&serde_json::to_string(&text).unwrap(), "Text");
    let builds = Builds::new(&compile(&source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[], 100, 0)["value"],
            json!({"type":"Text","value":text})
        );
    }
    for identity in [r#""m\u0061""#, r#""é""#, r#""with space""#] {
        let source = format!("module m @id({identity}); function f @id(\"f\")() -> I64 {{ 0 }}");
        let error = compile(&source).unwrap_err();
        assert_eq!(error.code, "compile.syntax.invalid");
        let range = error.range.unwrap();
        assert_eq!(&source[range.start.byte..range.end.byte], identity);
    }
    let source = program(r#""🦊\u0000🐺""#, "I64");
    let error = compile(&source).unwrap_err();
    let range = error.range.unwrap();
    assert_eq!(&source[range.start.byte..range.end.byte], r#""🦊\u0000🐺""#);
}

#[test]
fn checked_buffer_nodes_and_literal_limits_are_revalidated() {
    let artifact = compile(&program(r#"Text::concat("a","b")"#, "Text")).unwrap();
    let original: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    for (pointer, value, code) in [
        (
            "/program/functions/0/body/value/family",
            json!("External"),
            "compile.buffer.invalid",
        ),
        (
            "/program/functions/0/body/value/arguments",
            json!([]),
            "compile.buffer.invalid",
        ),
        (
            "/program/functions/0/body/value/arguments/0/value",
            json!({"type":"I64","value":"1"}),
            "compile.type.mismatch",
        ),
        (
            "/program/functions/0/body/value",
            json!({"kind":"Literal","value":{"type":"Bytes","value":"00"}}),
            "compile.syntax.invalid",
        ),
    ] {
        let mut forged = original.clone();
        *forged.pointer_mut(pointer).unwrap() = value;
        assert_eq!(Artifact::from_json(&forged.to_string()).unwrap_err().code, code);
    }
    let mut large = original;
    large["program"]["functions"][0]["body"]["value"] =
        json!({"kind":"Literal","value":{"type":"Text","value":"x".repeat(1_048_577)}});
    assert_eq!(
        Artifact::from_json(&large.to_string()).unwrap_err().code,
        "resource.frontend.limit"
    );
}

#[test]
fn text_result_byte_limit_matches_actual_json_at_the_exact_boundary() {
    let payload_limit = 8 * 1024 * 1024;
    let prefix = serde_json::to_vec(&json!({"type":"Text","value":""}))
        .unwrap()
        .len();
    let source = format!(
        "module m @id(\"m\"); function main @id(\"main\")(over: Bool) -> Text {{
      let data=fold(i in 23, b: Bytes = Text::bytes(\"x\")) {{ Bytes::concat(b,b) }};
      let n=if over {{ {} }} else {{ {} }}; Text::from_bytes(Bytes::slice(data,0,n))
    }}",
        payload_limit - prefix + 1,
        payload_limit - prefix
    );
    let builds = Builds::new(&compile(&source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "main", &["Bool:false"], 50000000, 3000000);
        assert_eq!(
            serde_json::to_vec(&result["value"]).unwrap().len(),
            payload_limit,
            "{mode}"
        );
        assert_eq!(
            builds.invoke(mode, "main", &["Bool:true"], 50000000, 3000000)["diagnostic"]["code"],
            "resource.output.limit"
        );
    }
    let escaped=program("let data=fold(i in 21, b: Bytes = Text::bytes(\"\\u0000\")) { Bytes::concat(b,b) }; Text::from_bytes(data)","Text");
    let builds = Builds::new(&compile(&escaped).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[], 10000000, 1000000)["diagnostic"]["code"],
            "resource.output.limit"
        );
    }
}

#[test]
fn native_decoded_input_admission_rejects_oversized_buffers_before_allocation() {
    let artifact =
        compile("module m @id(\"m\"); function f @id(\"f\")(b: Bytes) -> I64 { Bytes::length(b) }").unwrap();
    let builds = Builds::new(&artifact);
    // Construct argv inside the probe to exercise the adapter beyond the kernel's argv-size ceiling.
    let source=format!("#define main lobo_entry_main\n{}\n#undef main\n
      static char *probe_arg; static void probe_cleanup(void) {{ free(probe_arg); }}
      int main(void) {{ size_t n=8*1024*1024+1; probe_arg=malloc(7+2*n); if (!probe_arg) return 3;
        atexit(probe_cleanup); memcpy(probe_arg,\"Bytes:\",6); memset(probe_arg+6,'0',2*n); probe_arg[6+2*n]=0;
        char *args[]={{\"probe\",\"--entry\",\"f\",\"--arg\",probe_arg,\"--fuel\",\"0\",\"--heap-cells\",\"0\"}};
        return lobo_entry_main(9,args);
      }}",artifact.emit_c().unwrap());
    std::fs::write(builds.0.join("probe.c"), source).unwrap();
    for (binary, flags) in [
        ("probe", vec!["-O3"]),
        (
            "probe-sanitized",
            vec!["-O1", "-fsanitize=address,undefined", "-fno-sanitize-recover=all"],
        ),
    ] {
        let out = std::process::Command::new("clang")
            .arg("-std=c11")
            .args(flags)
            .arg(builds.0.join("probe.c"))
            .arg("-o")
            .arg(builds.0.join(binary))
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        let out = std::process::Command::new(builds.0.join(binary))
            .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
            .output()
            .unwrap();
        assert!(out.stderr.is_empty(), "{out:?}");
        let result: Json = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(result["diagnostic"]["code"], "resource.input.limit");
        assert_eq!(result["diagnostic"]["phase"], "invoke");
    }
}

#[test]
fn buffer_operands_keep_actions_contract_purity_and_first_fault_order() {
    let declarations =
        "module m @id(\"m\"); function marked @id(\"marked\")() -> Text actions { demo.observe } { \"x\" }";
    let source =
        format!("{declarations} function main @id(\"main\")() -> Text {{ Text::concat(marked(),\"!\") }}");
    assert_eq!(compile(&source).unwrap_err().code, "compile.action.undeclared");
    let contract=format!("{declarations} function main @id(\"main\")() -> I64 actions {{ demo.observe }} requires {{ Text::length(marked()) > 0 }} {{ 0 }}");
    assert_eq!(compile(&contract).unwrap_err().code, "compile.action.contract");
    let source = "module m @id(\"m\");
      function zero @id(\"zero\")() -> Text { let n=1/0; \"x\" }
      function over @id(\"over\")() -> Text { let n=9223372036854775807+1; \"x\" }
      function main @id(\"main\")() -> Text { Text::concat(zero(),over()) }";
    let builds = Builds::new(&compile(source).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "main", &[], 100, 0);
        assert_eq!(result["diagnostic"]["code"], "runtime.integer.zero_divisor");
        assert_eq!(result["diagnostic"]["subject"], "zero");
    }
}

#[test]
fn executable_text_example_frames_unicode_and_returns_original_invalid_octets() {
    let artifact = compile(include_str!("../examples/text.lobo")).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "example.text.greeting", &[], 1000, 100)["value"],
            json!({"type":"Text","value":"[Hello, 🐺!\n]"})
        );
        assert_eq!(
            builds.invoke(mode, "example.text.frame", &["Text:f09f90ba"], 1000, 12)["value"],
            json!({"type":"Text","value":"[🐺]"})
        );
        assert_eq!(
            builds.invoke(mode, "example.text.decode", &["Bytes:ff"], 1000, 4)["value"]["value"]["payload"],
            json!({"type":"Bytes","value":"ff"})
        );
    }
}

#[test]
fn scalar_indexing_preserves_encoding_boundaries_and_bom_as_data() {
    let points = [
        0xfeff, 0, 0x7f, 0x80, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xffff, 0x10000, 0x10ffff,
    ];
    let text = points
        .iter()
        .copied()
        .map(|n| char::from_u32(n).unwrap())
        .collect::<String>();
    let wire = format!(
        "Text:{}",
        text.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let source="module m @id(\"m\"); function main @id(\"main\")(text: Text) -> List<I64> {
      fold(i in Text::length(text), codes: List<I64> = List::empty<I64>()) { List::prepend(Text::at(text,i),codes) }
    }";
    let builds = Builds::new(&compile(source).unwrap());
    let expected = points
        .iter()
        .rev()
        .map(|n| json!({"type":"I64","value":n.to_string()}))
        .collect::<Vec<_>>();
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[&wire], 10000, 1000)["value"]["value"]["elements"],
            json!(expected)
        );
    }
}
