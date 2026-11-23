use std::process::Command;
mod support;

fn semantic(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(diagnostic) = value.get_mut("diagnostic").and_then(|d| d.as_object_mut()) {
        diagnostic.remove("message");
    }
    value
}

#[test]
fn llvm_preserves_checked_math_short_circuiting_contract_order_and_fuel() {
    let source = r#"module m @id("m");
      function add @id("add")(x:I64,y:I64) -> I64 { x+y }
      function subtract @id("sub")(x:I64,y:I64) -> I64 { x-y }
      function multiply @id("mul")(x:I64,y:I64) -> I64 { x*y }
      function divide @id("div")(x:I64,y:I64) -> I64 { x/y }
      function remainder @id("rem")(x:I64,y:I64) -> I64 { x%y }
      function short @id("short")() -> Bool { false && 1/0 == 0 || true }
      function zero @id("zero")() -> I64 { 1/0 }
      function ordered @id("ordered")() -> I64 { add(zero(),9223372036854775807+1) }
      function pre @id("pre")(x:I64) -> I64 requires { zero() == x } { x }
      function post @id("post")(x:I64) -> I64 ensures { result == zero() } { x }
      function loop @id("loop")(n:I64) -> I64 { while(x:I64=0;x<n){x+1} }
    "#;
    let builds = support::Builds::new(&lobo_core::compile(source).unwrap());
    for entry in ["add", "sub", "mul", "div", "rem"] {
        for (a, b) in [
            ("I64:-9223372036854775808", "I64:-1"),
            ("I64:9223372036854775807", "I64:1"),
            ("I64:7", "I64:0"),
            ("I64:-7", "I64:3"),
        ] {
            builds.invoke("native", entry, &[a, b], 100, 100);
        }
    }
    for (entry, args) in [
        ("short", vec![]),
        ("ordered", vec![]),
        ("pre", vec!["I64:1"]),
        ("post", vec!["I64:1"]),
    ] {
        let native = builds.invoke("native", entry, &args, 100, 100);
        let reference = builds.invoke("eval", entry, &args, 100, 100);
        assert_eq!(semantic(native), semantic(reference), "{entry}");
    }
    for fuel in 0..36 {
        assert_eq!(
            semantic(builds.invoke("native", "loop", &["I64:2"], fuel, 0)),
            semantic(builds.invoke("eval", "loop", &["I64:2"], fuel, 0))
        );
    }
}

#[test]
fn llvm_sanitizer_probes_detect_ir_stack_writes_and_runtime_alignment_faults() {
    let artifact =
        lobo_core::compile("module m @id(\"m\"); function f @id(\"f\")(x:I64) -> I64 { 7 }").unwrap();
    let ir = artifact.emit_llvm_sanitized().unwrap();
    assert!(ir.contains("__ubsan_handle_"));
    let root = std::env::temp_dir().join(format!("lobo-llvm-probes-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for (name,probe,expected) in [
        ("asan","  %probe = alloca [2 x i64], align 8\n  %offset = load volatile i64, ptr %arg0, align 8\n  %bad = getelementptr i64, ptr %probe, i64 %offset\n  store volatile i64 7, ptr %bad, align 8\n","AddressSanitizer: stack-buffer-overflow"),
        ("ubsan","  call void @lobo_ir_phase(ptr inttoptr (i64 1 to ptr), i32 0)\n","runtime error: member access within misaligned address")
    ] {
        let injected=ir.replacen("entry:\n",&format!("entry:\n{probe}"),1);
        let file=root.join(format!("{name}.ll")); let binary=root.join(name);
        std::fs::write(&file,injected).unwrap();
        let output=Command::new("clang-18").args(["-O1","-fsanitize=address,undefined","-fno-sanitize-recover=all"]).arg(&file).arg("-o").arg(&binary).output().unwrap();
        assert!(output.status.success(),"{output:?}");
        let output=Command::new(binary).env("ASAN_OPTIONS","detect_leaks=0:halt_on_error=1:symbolize=0").env("UBSAN_OPTIONS","halt_on_error=1:symbolize=0").args(["--entry","f","--arg","I64:2"]).output().unwrap();
        assert!(!output.status.success(),"{name} probe did not fire");
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected),"{name}: {output:?}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn llvm_cli_emits_verifiable_ir_and_builds_standalone_native_code() {
    let root = std::env::temp_dir().join(format!("lobo-llvm-test-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("program.lobo");
    std::fs::write(
        &source,
        r#"module m @id("m");
      function main @id("main")(x: I64) -> I64 { if x < 0 { -x } else { x + 1 } }"#,
    )
    .unwrap();
    let binary = root.join("llvm");
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args([
            "compile",
            source.to_str().unwrap(),
            "--backend",
            "llvm",
            "-o",
            binary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(&std::fs::read(&binary).unwrap()[..4], b"\x7fELF");
    let run = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args([
            "run",
            source.to_str().unwrap(),
            "--backend",
            "llvm",
            "--entry",
            "main",
            "--arg",
            "I64:7",
        ])
        .output()
        .unwrap();
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&run.stdout).unwrap()["value"]["value"],
        "8"
    );
    for (arg, expected) in [("I64:7", "8"), ("I64:-7", "7")] {
        let output = Command::new(&binary)
            .args(["--entry", "main", "--arg", arg])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["value"]["value"], expected);
    }
    let ir = root.join("program.ll");
    let output = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args([
            "compile",
            source.to_str().unwrap(),
            "--emit",
            "llvm",
            "-o",
            ir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let text = std::fs::read_to_string(&ir).unwrap();
    assert!(text.contains("define void @lobo_llvm_fn_0"));
    assert!(text.contains("llvm.sadd.with.overflow.i64"));
    let verify = Command::new("opt-18")
        .args(["-passes=verify", "-disable-output"])
        .arg(&ir)
        .output()
        .unwrap();
    assert!(verify.status.success(), "{verify:?}");
    let again = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .args([
            "compile",
            source.to_str().unwrap(),
            "--backend",
            "llvm",
            "-o",
            binary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!again.status.success());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn llvm_tool_failures_and_backend_selectors_never_fall_back_or_publish_output() {
    let root = std::env::temp_dir().join(format!("lobo-llvm-boundary-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("program.lobo");
    let output = root.join("program");
    std::fs::write(
        &source,
        "module m @id(\"m\"); function f @id(\"f\")() -> I64 { 7 }",
    )
    .unwrap();
    for args in [
        vec!["--backend", "unknown"],
        vec!["--backend", "llvm", "--backend", "c"],
        vec!["--emit", "llvm", "--backend", "c"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_lobo"))
            .args([
                "compile",
                source.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap()["diagnostic"]["code"],
            "invocation.cli.invalid"
        );
        assert!(!output.exists());
    }
    let result = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .env("PATH", "")
        .args([
            "compile",
            source.to_str().unwrap(),
            "--backend",
            "llvm",
            "-o",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    let result: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(result["diagnostic"]["code"], "invocation.native.tool_unavailable");
    assert!(result["diagnostic"]["message"]
        .as_str()
        .unwrap()
        .contains("clang-18"));
    assert!(!output.exists());
    let checked = root.join("program.loboc");
    let result = Command::new(env!("CARGO_BIN_EXE_lobo"))
        .env("PATH", "")
        .args([
            "compile",
            source.to_str().unwrap(),
            "--emit",
            "checked",
            "-o",
            checked.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    std::fs::remove_dir_all(root).unwrap();
}
