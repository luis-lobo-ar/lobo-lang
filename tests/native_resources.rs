use lobo_core::{compile, Artifact};
use serde_json::Value as Json;
use std::process::Command;
mod support;
use support::Builds;

const SOURCE: &str = r#"module resources;
fn probe()->I64 actions {native.authorize,net.listen,net.close} {
  let cap=Native::capability();
  match Native::listen(cap,0) {
    Err(code)=>code,
    Ok(listener)=>match Native::close_listener(listener) {Err(code)=>100+code,Ok(done)=>0}
  }
}
fn stale()->I64 actions {native.authorize,net.listen,net.close} {
  let cap=Native::capability();
  match Native::listen(cap,0) {
    Err(code)=>100+code,
    Ok(listener)=>{
      let alias=listener;
      let closed=Native::close_listener(listener);
      match Native::close_listener(alias) {Err(code)=>code,Ok(done)=>0}
    }
  }
}
fn input(listener:Native::Listener)->I64 {0}
fn leak()->Native::Capability actions {native.authorize} {Native::capability()}
fn clock()->Native::I64Result actions {native.authorize,clock.read} {Native::monotonic_ms(Native::capability())}
fn random(n:I64)->Native::BytesResult actions {native.authorize,random.read} {Native::random_bytes(Native::capability(),n)}
identities {module:"resources",probe:"resources.probe",stale:"resources.stale",input:"resources.input",leak:"resources.leak",clock:"resources.clock",random:"resources.random"}
"#;
fn invoke(builds: &Builds, mode: &str, entry: &str, options: &[&str]) -> Json {
    let output = Command::new(builds.0.join(mode))
        .args(["--entry", entry, "--fuel", "1000000", "--heap-cells", "100000"])
        .args(options)
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert!(output.stderr.is_empty(), "{mode}: {output:?}");
    let json: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.success(), json["status"] == "ok");
    json
}
#[test]
fn runtime_grants_default_to_deny_and_cannot_be_replaced_by_action_labels() {
    let artifact = compile(SOURCE).unwrap();
    let builds = Builds::new(&Artifact::from_json(&artifact.to_json().unwrap()).unwrap());
    assert_eq!(
        builds.invoke("eval", "resources.probe", &[], 10000, 100000)["diagnostic"]["code"],
        "invocation.native.required"
    );
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        assert_eq!(
            invoke(&builds, mode, "resources.probe", &[])["value"]["value"],
            "1"
        );
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.probe",
                &["--native-listen", "127.0.0.1:0"]
            )["value"]["value"],
            "0"
        );
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.stale",
                &["--native-listen", "127.0.0.1:0"]
            )["value"]["value"],
            "5"
        );
        for options in [
            vec!["--native-listen", "0.0.0.0:0"],
            vec!["--native-listen", "127.0.0.1:-1"],
            vec!["--native-listen", "127.0.0.1:0", "--native-listen", "127.0.0.1:0"],
        ] {
            assert_eq!(
                invoke(&builds, mode, "resources.probe", &options)["diagnostic"]["code"],
                "invocation.cli.invalid"
            );
        }
    }
}
#[test]
fn opaque_resources_cannot_cross_input_output_or_literal_boundaries() {
    let artifact = compile(SOURCE).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        assert_eq!(
            invoke(&builds, mode, "resources.input", &["--arg", "I64:1"])["diagnostic"]["code"],
            "invocation.arguments.invalid"
        );
        assert_eq!(
            invoke(&builds, mode, "resources.input", &["--arg", "List:I64:257"])["diagnostic"]["code"],
            "invocation.arguments.invalid"
        );
        assert_eq!(
            invoke(&builds, mode, "resources.leak", &[])["diagnostic"]["code"],
            "invocation.native.resource_value"
        );
    }
    let bad = SOURCE.replace("let alias=listener;", "let alias:Native::Listener=1;");
    assert_eq!(compile(&bad).unwrap_err().code, "compile.type.mismatch");
    let bad = SOURCE.replace("Native::close_listener(listener)", "Native::close_listener(cap)");
    assert_eq!(compile(&bad).unwrap_err().code, "compile.type.mismatch");
}
#[test]
fn os_calls_need_declared_actions_and_are_unavailable_in_contracts_or_reference_execution() {
    for (before,after,expected) in [
        ("fn leak()->Native::Capability actions {native.authorize}","fn leak()->Native::Capability","compile.action.undeclared"),
        ("{Native::monotonic_ms(Native::capability())}","requires {Native::monotonic_ms(Native::capability()) == Native::monotonic_ms(Native::capability())} {Native::monotonic_ms(Native::capability())}","compile.action.contract"),
    ] {assert_eq!(compile(&SOURCE.replace(before,after)).unwrap_err().code,expected);}
    let artifact = compile(SOURCE).unwrap();
    assert_eq!(
        artifact.run("resources.probe", vec![], 1000).unwrap_err().code,
        "invocation.native.required"
    );
}
#[test]
fn clock_and_random_enforce_grants_and_bounds() {
    let builds = Builds::new(&compile(SOURCE).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let denied = invoke(&builds, mode, "resources.clock", &[]);
        assert_eq!(denied["value"]["value"]["case"], "Err");
        assert_eq!(denied["value"]["value"]["payload"]["value"], "1");
        let allowed = invoke(&builds, mode, "resources.clock", &["--native-clock", "true"]);
        assert_eq!(allowed["value"]["value"]["case"], "Ok");
        assert!(
            allowed["value"]["value"]["payload"]["value"]
                .as_str()
                .unwrap()
                .parse::<i64>()
                .unwrap()
                >= 0
        );
        for n in [0, 1, 32, 65536] {
            let atom = format!("I64:{n}");
            let random = invoke(
                &builds,
                mode,
                "resources.random",
                &["--arg", &atom, "--native-random", "true"],
            );
            assert_eq!(random["value"]["value"]["case"], "Ok");
            assert_eq!(
                random["value"]["value"]["payload"]["value"]
                    .as_str()
                    .unwrap()
                    .len(),
                n * 2
            );
        }
        for n in [-1, 65537, i64::MAX] {
            let atom = format!("I64:{n}");
            let bad = invoke(
                &builds,
                mode,
                "resources.random",
                &["--arg", &atom, "--native-random", "true"],
            );
            assert_eq!(bad["value"]["value"]["case"], "Err");
            assert_eq!(bad["value"]["value"]["payload"]["value"], "2");
        }
    }
}

#[test]
fn root_authority_must_be_acquired_at_entry_and_passed_to_callees() {
    let source=SOURCE.replace("identities {",r#"fn acquire()->Native::Capability actions {native.authorize} {Native::capability()}
fn nested()->I64 actions {native.authorize} {let cap=acquire();0}
fn cached()->I64 actions {native.authorize} {let first=Native::capability();let second=acquire();0}
fn delegated()->I64 actions {native.authorize,net.listen,net.close} {let cap=Native::capability();use_cap(cap)}
fn use_cap(cap:Native::Capability)->I64 actions {net.listen,net.close} {
  match Native::listen(cap,0) {Err(code)=>code,Ok(listener)=>{let closed=Native::close_listener(listener);0}}
}
identities {acquire:"resources.acquire",nested:"resources.nested",cached:"resources.cached",delegated:"resources.delegated",use_cap:"resources.use_cap","#);
    let builds = Builds::new(&compile(&source).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.cached",
                &["--native-listen", "127.0.0.1:0"]
            )["diagnostic"]["code"],
            "invocation.native.authority"
        );
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.nested",
                &["--native-listen", "127.0.0.1:0"]
            )["diagnostic"]["code"],
            "invocation.native.authority"
        );
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.delegated",
                &["--native-listen", "127.0.0.1:0"]
            )["value"]["value"],
            "0"
        );
    }
}

#[test]
fn delegated_capabilities_can_only_lose_rights() {
    let source = SOURCE.replace(
        "identities {",
        r#"fn attenuated(mask:I64)->I64 actions {native.authorize,random.read} {
 let root=Native::capability();
 match Native::restrict_capability(root,mask) {
  Err(code)=>100+code,
  Ok(narrow)=>match Native::random_bytes(narrow,1) {Err(code)=>code,Ok(data)=>0}
 }
}
fn widen()->I64 actions {native.authorize} {
 let root=Native::capability();
 match Native::restrict_capability(root,1) {
  Err(code)=>100+code,
  Ok(narrow)=>match Native::restrict_capability(narrow,3) {Err(code)=>code,Ok(wide)=>0}
 }
}
identities {attenuated:"resources.attenuated",widen:"resources.widen","#,
    );
    let builds = Builds::new(&compile(&source).unwrap());
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let both = ["--native-clock", "true", "--native-random", "true"];
        for (mask, code) in [(0, 1), (1, 1), (2, 0), (3, 0), (16, 102), (-1, 102)] {
            let atom = format!("I64:{mask}");
            let mut options = both.to_vec();
            options.extend(["--arg", &atom]);
            assert_eq!(
                invoke(&builds, mode, "resources.attenuated", &options)["value"]["value"],
                code.to_string()
            );
        }
        assert_eq!(
            invoke(&builds, mode, "resources.widen", &both)["value"]["value"],
            "1"
        );
        assert_eq!(
            invoke(
                &builds,
                mode,
                "resources.attenuated",
                &["--native-clock", "true", "--arg", "I64:3"]
            )["value"]["value"],
            "101"
        );
    }
}
