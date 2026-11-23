use lobo_core::{compile_bundle, Artifact, SourceFile};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn file(name: &str, text: &str) -> SourceFile {
    SourceFile {
        identities: None,
        name: name.into(),
        text: text.into(),
    }
}

#[test]
fn imported_function_and_nominal_record_are_shared_in_native_and_reference_programs() {
    let files = vec![
        file("main.lobo", "module app @id(\"app\"); import Pair @id(\"lib.pair\"); import make @id(\"lib.make\"); function main @id(\"main\")() -> Pair { make(41) }"),
        file("lib.lobo", "module library @id(\"lib\"); record Pair @id(\"lib.pair\") { value: I64 } function make @id(\"lib.make\")(n: I64) -> Pair { Pair { value: n + 1 } }"),
    ];
    let artifact = compile_bundle("main.lobo", &files).unwrap();
    let loaded = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    let builds = Builds::new(&loaded);
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &[], 1000, 0)["value"],
            json!({"type":"Record","value":{"id":"lib.pair","fields":{"value":{"type":"I64","value":"42"}}}})
        );
    }
    let mut changed = files.clone();
    changed[1].text = changed[1].text.replace("n + 1", "n + 2");
    let updated = compile_bundle("main.lobo", &changed).unwrap();
    assert_ne!(artifact.to_json().unwrap(), updated.to_json().unwrap());
    let builds = Builds::new(&updated);
    for mode in ["eval", "native", "sanitized"] {
        let value = builds.invoke(mode, "main", &[], 1000, 0);
        assert_eq!(value["value"]["value"]["id"], "lib.pair");
        assert_eq!(value["value"]["value"]["fields"]["value"]["value"], "43");
    }
}

fn sample() -> Vec<SourceFile> {
    vec![file("app", "module app @id(\"app\"); import make @id(\"lib.make\"); function main @id(\"main\")() -> I64 { make(41).value }"),
    file("lib", "module library @id(\"lib\"); record Pair @id(\"lib.pair\") { value: I64 } function make @id(\"lib.make\")(n: I64) -> Pair { Pair { value: n + 1 } }")]
}

#[test]
fn source_order_labels_and_comments_are_not_linked_artifact_identity() {
    let mut files = sample();
    let a = compile_bundle("app", &files).unwrap();
    files.reverse();
    let b = compile_bundle("app", &files).unwrap();
    files[1].name = "renamed".into();
    files[0].text.insert_str(0, "// moved\n");
    let c = compile_bundle("renamed", &files).unwrap();
    assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
    assert_eq!(a.to_json().unwrap(), c.to_json().unwrap());
    assert_eq!(a.emit_c().unwrap(), c.emit_c().unwrap());
}

#[test]
fn local_names_are_separate_but_imported_nominal_types_remain_the_original_type() {
    let mut files = sample();
    files[0].text="module app @id(\"app\"); import Foreign @id(\"lib.pair\"); import make @id(\"lib.make\"); record Pair @id(\"app.pair\") { value: I64 } function main @id(\"main\")() -> I64 { let local = Pair { value: 21 }; let remote: Foreign = make(41); local.value + remote.value }".into();
    let builds = Builds::new(&compile_bundle("app", &files).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(builds.invoke(mode, "main", &[], 1000, 0)["value"]["value"], "63");
    }
    files[0].text = files[0].text.replace("remote: Foreign", "remote: Pair");
    assert_eq!(
        compile_bundle("app", &files).unwrap_err().code,
        "compile.type.mismatch"
    );
    let sources = [file("a", "module library @id(\"a\"); function import @id(\"lib.import\")(n: I64) -> I64 { n + 1 } function call @id(\"lib.call\")() -> I64 { import(41) }"),
        file("b", "module client @id(\"b\"); function main @id(\"main\")(import: I64) -> I64 { import + 1 }")];
    let builds = Builds::new(&compile_bundle("b", &sources).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "main", &["I64:41"], 1000, 0)["value"]["value"],
            "42"
        );
        assert_eq!(
            builds.invoke(mode, "lib.call", &[], 1000, 0)["value"]["value"],
            "42"
        );
    }
}

#[test]
fn missing_conflicting_and_self_imports_fail_without_resolving_any_external_sources() {
    for (imports, code) in [
        ("import absent @id(\"not.supplied\");", "compile.module.invalid"),
        ("import wrong @id(\"lib\");", "compile.module.invalid"),
        ("import self @id(\"main\");", "compile.module.invalid"),
        ("import main @id(\"lib.make\");", "compile.name.duplicate"),
        (
            "import x @id(\"lib.make\"); import x @id(\"lib.pair\");",
            "compile.name.duplicate",
        ),
    ] {
        let mut files = sample();
        files[0].text =
            format!("module app @id(\"app\"); {imports} function main @id(\"main\")() -> I64 {{ 0 }}");
        let error = compile_bundle("app", &files).unwrap_err();
        assert_eq!(error.code, code, "{imports}");
        assert_eq!(error.source.as_deref().map(String::as_str), Some("app"));
    }
    let mut files = sample();
    files[1].text = files[1].text.replace("lib.make", "main");
    assert_eq!(
        compile_bundle("app", &files).unwrap_err().code,
        "compile.identity.duplicate"
    );
}

#[test]
fn dependency_diagnostics_name_exact_text_and_own_scope_not_the_root_file() {
    let mut files = sample();
    files[1].text="// 🐺\r\nmodule library @id(\"lib\"); record Pair @id(\"lib.pair\") { value: I64 } function make @id(\"lib.make\")(n: I64) -> Pair { Pair { value: missing } }".into();
    let error = compile_bundle("app", &files).unwrap_err();
    assert_eq!(error.source.as_deref().map(String::as_str), Some("lib"));
    assert_eq!(error.subject.as_deref(), Some("lib.make"));
    let range = error.range.unwrap();
    assert_eq!(&files[1].text[range.start.byte..range.end.byte], "missing");
    assert_eq!(range.start.line, 1);
}

#[test]
fn a_stable_id_is_not_a_source_visibility_escape_hatch() {
    let files = vec![
        file(
            "app",
            "module app @id(\"app\"); function main @id(\"main\")() -> I64 { hidden() }",
        ),
        file(
            "lib",
            "module library @id(\"lib\"); function secret @id(\"hidden\")() -> I64 { 42 }",
        ),
    ];
    assert_eq!(
        compile_bundle("app", &files).unwrap_err().code,
        "compile.name.unresolved"
    );
    let mut files = sample();
    let encoded = compile_bundle("app", &files).unwrap().to_json().unwrap();
    let original: Json = serde_json::from_str(&encoded).unwrap();
    let mut forged = original.clone();
    forged["program"]["scopes"][0]["imports"] = json!([]);
    assert_eq!(
        Artifact::from_json(&forged.to_string()).unwrap_err().code,
        "compile.name.unresolved"
    );
    files[0].text="module app @id(\"app\"); import Pair @id(\"lib.pair\"); function main @id(\"main\")() -> I64 { Pair { value: 1 }.value }".into();
    let mut forged: Json =
        serde_json::from_str(&compile_bundle("app", &files).unwrap().to_json().unwrap()).unwrap();
    forged["program"]["scopes"][0]["imports"] = json!([]);
    assert_eq!(
        Artifact::from_json(&forged.to_string()).unwrap_err().code,
        "compile.module.visibility"
    );
}

#[test]
fn ownership_and_import_metadata_are_revalidated_on_load() {
    let artifact = compile_bundle("app", &sample()).unwrap();
    let original: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    for change in 0..4 {
        let mut data = original.clone();
        match change {
            0 => data["program"]["scopes"][0]["declarations"] = json!([]),
            1 => data["program"]["scopes"][1]["declarations"]
                .as_array_mut()
                .unwrap()
                .push(json!("main")),
            2 => data["program"]["scopes"][0]["imports"][0]["id"] = json!("unknown"),
            _ => data["program"]["module"] = json!("other"),
        }
        assert_eq!(
            Artifact::from_json(&data.to_string()).unwrap_err().code,
            "compile.module.invalid"
        );
    }
}

#[test]
fn imports_preserve_action_bounds_contract_purity_and_runtime_resource_subjects() {
    let dependency=file("lib","module library @id(\"lib\"); function leaf @id(\"lib.leaf\")() -> I64 actions { demo.observe } { 42 }");
    for (header, body, code) in [
        ("", "if false { leaf() } else { 0 }", "compile.action.undeclared"),
        (
            "requires { if false { leaf() == 42 } else { true } }",
            "0",
            "compile.action.contract",
        ),
    ] {
        let root=file("app",&format!("module app @id(\"app\"); import leaf @id(\"lib.leaf\"); function main @id(\"main\")() -> I64 {header} {{ {body} }}"));
        assert_eq!(
            compile_bundle("app", &[root, dependency.clone()])
                .unwrap_err()
                .code,
            code
        );
    }
    let files=vec![file("app","module app @id(\"app\"); import allocate @id(\"lib.allocate\"); function main @id(\"main\")() -> List<I64> { allocate() }"),
    file("lib","module library @id(\"lib\"); function allocate @id(\"lib.allocate\")() -> List<I64> { List::prepend(1,List::empty<I64>()) }")];
    let builds = Builds::new(&compile_bundle("app", &files).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let fault = builds.invoke(mode, "main", &[], 1000, 3);
        assert_eq!(fault["diagnostic"]["code"], "resource.memory.exhausted");
        assert_eq!(fault["diagnostic"]["subject"], "lib.allocate");
        assert!(fault.get("value").is_none());
    }
}

#[test]
fn mutual_function_dependencies_have_no_initialization_order() {
    let files=vec![file("a","module a @id(\"module.a\"); import b @id(\"b\"); function a @id(\"a\")(n: I64) -> I64 { if n == 0 { 0 } else { b(n-1) } }"),
    file("b","module b @id(\"module.b\"); import a @id(\"a\"); function b @id(\"b\")(n: I64) -> I64 { if n == 0 { 1 } else { a(n-1) } }")];
    let builds = Builds::new(&compile_bundle("a", &files).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        assert_eq!(
            builds.invoke(mode, "a", &["I64:5"], 1000, 0)["value"]["value"],
            "1"
        );
    }
}

#[test]
fn aggregate_source_and_token_admission_is_bounded_before_linking() {
    let files = sample();
    assert_eq!(
        compile_bundle("missing", &files).unwrap_err().code,
        "compile.module.invalid"
    );
    let mut duplicate = files.clone();
    duplicate[1].name = "app".into();
    assert_eq!(
        compile_bundle("app", &duplicate).unwrap_err().code,
        "compile.module.invalid"
    );
    let mut too_many = vec![];
    for i in 0..17 {
        too_many.push(file(&format!("s{i}"), ""));
    }
    assert_eq!(
        compile_bundle("s0", &too_many).unwrap_err().code,
        "resource.frontend.limit"
    );
    assert_eq!(
        compile_bundle("large", &[file("large", &" ".repeat(1048577))])
            .unwrap_err()
            .code,
        "resource.frontend.limit"
    );
    let mut sources = vec![];
    for i in 0..4 {
        let bindings = (0..900).map(|j| format!("let x{j} = 1; ")).collect::<String>();
        sources.push(file(
            &format!("s{i}"),
            &format!("module m @id(\"m{i}\"); function f @id(\"f{i}\")() -> I64 {{ {bindings} 0 }}"),
        ));
    }
    let Err(error) = compile_bundle("s0", &sources) else {
        panic!("aggregate token limit must reject the bundle");
    };
    assert_eq!(error.code, "resource.frontend.limit");
}

#[test]
fn agent_bundle_stdin_is_source_only_and_explicit_file_inputs_compile_natively() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let files = sample();
    let envelope = json!({"format":"lobo-source/0.1","root":"app","sources":files});
    let mut child = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .env("PATH", "")
        .args(["check", "--bundle-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(envelope.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.stderr.is_empty());
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{result}");
    let builds = Builds::new(&compile_bundle("app", &sample()).unwrap());
    let root = builds.0.join("app.lobo");
    let dependency = builds.0.join("lib.lobo");
    std::fs::write(&root, &files[0].text).unwrap();
    std::fs::write(&dependency, &files[1].text).unwrap();
    for mode in ["run", "eval"] {
        let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
            .arg(mode)
            .arg(&root)
            .arg("--source")
            .arg(&dependency)
            .args(["--entry", "main"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let result: Json = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["value"]["value"], "42");
    }
    let mut oversized = vec![b' '; lobo_core::MAX_SOURCE_BYTES];
    oversized.push(0xff);
    std::fs::write(&root, oversized).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .arg("check")
        .arg(&root)
        .arg("--source")
        .arg(&dependency)
        .output()
        .unwrap();
    let failure: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(failure["diagnostic"]["code"], "resource.frontend.limit");
    std::fs::remove_file(root).unwrap();
    std::fs::remove_file(dependency).unwrap();
}

#[test]
fn real_source_files_share_one_reverse_implementation_and_cursor_layout() {
    let sources = [
        file(
            "reverse",
            &std::fs::read_to_string("examples/modules/reverse.lobo").unwrap(),
        ),
        file(
            "mirror",
            &std::fs::read_to_string("examples/modules/mirror.lobo").unwrap(),
        ),
        file(
            "sequences",
            &std::fs::read_to_string("examples/modules/sequences.lobo").unwrap(),
        ),
        file("common", include_str!("../examples/sequences.lobo")),
    ];
    let artifact = compile_bundle("reverse", &sources).unwrap();
    let encoded: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(
        encoded["program"]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["id"] == "example.modules.reverse")
            .count(),
        1
    );
    assert_eq!(encoded["program"]["records"].as_array().unwrap().len(), 1);
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        for (entry, expected) in [
            ("example.modules.reversed", vec![0, 9, -1, 5]),
            ("example.modules.mirrored", vec![5, -1, 9, 0]),
        ] {
            let value = builds.invoke(
                mode,
                entry,
                &["List:Count:4,I64:5,I64:-1,I64:9,I64:0"],
                1000,
                1000,
            );
            assert_eq!(
                value["value"]["value"]["elements"],
                json!(expected
                    .iter()
                    .map(|n| json!({"type":"I64","value":n.to_string()}))
                    .collect::<Vec<_>>())
            );
        }
    }
}

#[test]
fn collection_and_tree_workloads_share_one_cursor_and_algorithm_without_changing_entry_ids() {
    let sources = [
        file("collections", include_str!("../examples/collections.lobo")),
        file("tree", include_str!("../examples/tree.lobo")),
        file("sequences", include_str!("../examples/sequences.lobo")),
    ];
    let artifact = compile_bundle("collections", &sources).unwrap();
    let encoded: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    assert_eq!(
        encoded["program"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| record["name"] == "Cursor")
            .count(),
        1
    );
    let builds = Builds::new(&artifact);
    for mode in ["eval", "native", "sanitized"] {
        for entry in ["example.collections.reverse", "example.tree.reverse"] {
            let result = builds.invoke(mode, entry, &["List:Count:3,I64:1,I64:2,I64:3"], 1000, 1000);
            assert_eq!(
                result["value"]["value"]["elements"],
                json!([
                    {"type":"I64","value":"3"}, {"type":"I64","value":"2"}, {"type":"I64","value":"1"}
                ])
            );
        }
    }
}

#[test]
fn type_only_modules_share_variants_and_only_inline_recursive_layouts_are_rejected() {
    let files=[file("app","module app @id(\"app\"); import Box @id(\"lib.box\"); import Reply @id(\"lib.reply\"); function main @id(\"main\")() -> Reply { Reply::Ok(Box { value: 42 }) }"),
        file("types","module types @id(\"lib\"); record Box @id(\"lib.box\") { value: I64 } variant Reply @id(\"lib.reply\") { Ok(Box), Empty }")];
    let builds = Builds::new(&compile_bundle("app", &files).unwrap());
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(mode, "main", &[], 1000, 0);
        assert_eq!(result["value"]["value"]["id"], "lib.reply");
        assert_eq!(result["value"]["value"]["payload"]["value"]["id"], "lib.box");
    }
    let files=[file("a","module a @id(\"a\"); import Y @id(\"b.y\"); record X @id(\"a.x\") { next: List<Y> } function main @id(\"main\")() -> I64 { 0 }"),
        file("b","module b @id(\"b\"); import X @id(\"a.x\"); record Y @id(\"b.y\") { next: List<X> }")];
    assert!(compile_bundle("a", &files).is_ok());
    let mut inline = files.clone();
    for source in &mut inline {
        source.text = source.text.replace("List<Y>", "Y").replace("List<X>", "X");
    }
    let fault = compile_bundle("a", &inline).unwrap_err();
    assert_eq!(fault.code, "compile.record.cycle");
    assert!(fault.source.is_none() && fault.range.is_none());
}

#[test]
fn bundle_wire_and_artifact_versions_do_not_implicitly_migrate() {
    let files = sample();
    let wire = json!({"format":"lobo-source/0.1","root":"app","sources":files});
    let artifact = lobo_core::compile_bundle_json(&wire.to_string()).unwrap();
    let mut old: Json = serde_json::from_str(&artifact.to_json().unwrap()).unwrap();
    old["format"] = "lobo-checked/0.8".into();
    old["language"] = "core/0.8-draft".into();
    old["program"].as_object_mut().unwrap().remove("scopes");
    assert_eq!(
        Artifact::from_json(&old.to_string()).unwrap_err().code,
        "invocation.artifact.version"
    );
    old["format"] = lobo_core::FORMAT.into();
    old["language"] = lobo_core::LANGUAGE.into();
    assert_eq!(
        Artifact::from_json(&old.to_string()).unwrap_err().code,
        "compile.module.invalid"
    );
    let mut invalid = wire;
    invalid["format"] = "lobo-source/99".into();
    assert_eq!(
        lobo_core::compile_bundle_json(&invalid.to_string())
            .unwrap_err()
            .code,
        "invocation.bundle.version"
    );
}

#[test]
fn a_noncallable_import_reports_its_source_alias_not_a_resolved_id_length() {
    let id = format!("record.{}", "x".repeat(100));
    let text = format!(
        "module app @id(\"app\"); import F @id(\"{id}\"); function main @id(\"main\")() -> I64 {{ F() }}"
    );
    let sources = [
        file("app", &text),
        file(
            "types",
            &format!("module types @id(\"types\"); record Foo @id(\"{id}\") {{ value: I64 }}"),
        ),
    ];
    let error = compile_bundle("app", &sources).unwrap_err();
    assert_eq!(error.code, "compile.name.unresolved");
    let range = error.range.unwrap();
    assert_eq!(&text[range.start.byte..range.end.byte], "F");
}

#[test]
fn forged_namespace_metadata_is_admitted_before_building_large_context_maps() {
    let base = lobo_core::compile("module m @id(\"m\"); function f @id(\"f\")() -> I64 { 0 }").unwrap();
    let mut data: Json = serde_json::from_str(&base.to_json().unwrap()).unwrap();
    let template = data["program"]["functions"][0].clone();
    let mut functions = vec![];
    let mut declarations = vec![];
    for i in 0..16385 {
        let name = format!("f{i}");
        let mut function = template.clone();
        function["name"] = name.clone().into();
        function["id"] = name.clone().into();
        functions.push(function);
        declarations.push(json!(name));
    }
    data["program"]["functions"] = json!(functions);
    data["program"]["scopes"][0]["declarations"] = json!(declarations);
    let outcome = Artifact::from_json(&data.to_string());
    assert_eq!(
        outcome.err().map(|e| e.code),
        Some("resource.frontend.limit".into())
    );
}
