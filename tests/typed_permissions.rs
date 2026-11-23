use lobo_core::{compile_bundle, SourceFile};
use serde_json::Value;
use std::process::Command;
#[allow(dead_code)]
mod support;
use support::Builds;

fn sources(app: &str) -> Vec<SourceFile> {
    vec![
        SourceFile {
            name: "app".into(),
            text: app.into(),
            identities: None,
        },
        SourceFile {
            name: "access".into(),
            text: std::fs::read_to_string("libraries/native/permissions.lobo")
                .expect("typed permissions library"),
            identities: None,
        },
        SourceFile {
            name: "os".into(),
            text: include_str!("../libraries/native/native.lobo").into(),
            identities: None,
        },
    ]
}
const APP: &str = r#"module checks;
use access {Permission,Permissions,none,one,allow,abi_mask,restrict};
use os {IoResult,restrict_capability,legacy_mask};
fn set(bits:I64)->Permissions {
 let a=if bits%2==1 {allow(none(),ClockRead)} else {none()};
 let b=if (bits/2)%2==1 {allow(a,RandomRead)} else {a};
 let c=if (bits/4)%2==1 {allow(b,Listen)} else {b};
 if (bits/8)%2==1 {allow(c,Connect)} else {c}
}
fn mask(bits:I64,duplicate:Bool)->I64 {
 let requested=set(bits);
 abi_mask(if duplicate {allow(allow(allow(allow(requested,ClockRead),ClockRead),RandomRead),RandomRead)} else {requested})
}
fn delegate(bits:I64)->Native::I64Result actions {native.authorize} {
 let cap=try restrict(Native::capability(),set(bits)); Ok(abi_mask(set(bits)))
}
fn amplify()->Native::CapabilityResult actions {native.authorize} {
 let root=Native::capability(); let child=try restrict(root,one(ClockRead)); restrict(child,one(Listen))
}
fn empty_clock()->Native::I64Result actions {native.authorize,clock.read} {
 let cap=try restrict(Native::capability(),none()); Native::monotonic_ms(cap)
}
fn wrapped()->IoResult<I64> actions {native.authorize} {try restrict_capability(Native::capability(),one(Listen)); Ok(1)}
fn raw(raw_mask:I64)->IoResult<I64> actions {native.authorize} {try legacy_mask(Native::capability(),raw_mask); Ok(1)}
identities {module:"permissions.test",access.Permission:"lobo.permissions.Permission",access.Permissions:"lobo.permissions.Permissions",access.none:"lobo.permissions.none",access.one:"lobo.permissions.one",access.allow:"lobo.permissions.allow",access.abi_mask:"lobo.permissions.abi_mask",access.restrict:"lobo.permissions.restrict",os.IoResult:"lobo.os.result",os.restrict_capability:"lobo.os.restrict_permissions",os.legacy_mask:"lobo.os.restrict_capability",set:"permissions.test.set",mask:"permissions.test.mask",delegate:"permissions.test.delegate",amplify:"permissions.test.amplify",empty_clock:"permissions.test.empty_clock",wrapped:"permissions.test.wrapped",raw:"permissions.test.raw"}
"#;
fn invoke(builds: &Builds, mode: &str, entry: &str, options: &[&str]) -> Value {
    let out = Command::new(builds.0.join(mode))
        .args(["--entry", entry])
        .args(options)
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn typed_permissions_preserve_set_and_runtime_authority_semantics() {
    let artifact = compile_bundle("app", &sources(APP)).unwrap();
    let builds = Builds::new(&artifact);
    for bits in 0..16 {
        for duplicate in [false, true] {
            let expected = if duplicate { bits | 3 } else { bits };
            for mode in ["eval", "native", "sanitized"] {
                let value = builds.invoke(
                    mode,
                    "permissions.test.mask",
                    &[&format!("I64:{bits}"), &format!("Bool:{duplicate}")],
                    100000,
                    100000,
                );
                assert_eq!(value["value"]["value"], expected.to_string());
            }
        }
    }
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for grants in 0..16 {
            for bits in 0..16 {
                let arg = format!("I64:{bits}");
                let mut options = vec!["--arg", arg.as_str()];
                if grants & 1 != 0 {
                    options.extend(["--native-clock", "true"]);
                }
                if grants & 2 != 0 {
                    options.extend(["--native-random", "true"]);
                }
                if grants & 4 != 0 {
                    options.extend(["--native-listen", "127.0.0.1:0"]);
                }
                if grants & 8 != 0 {
                    options.extend(["--native-connect", "127.0.0.1:12345"]);
                }
                let result = invoke(&builds, mode, "permissions.test.delegate", &options);
                let allowed = bits & !grants == 0;
                assert_eq!(
                    result["value"]["value"]["case"],
                    if allowed { "Ok" } else { "Err" },
                    "grant={grants} request={bits}: {result}"
                );
                assert_eq!(
                    result["value"]["value"]["payload"]["value"],
                    if allowed { bits.to_string() } else { "1".into() }
                );
            }
        }
        let exhausted = invoke(
            &builds,
            mode,
            "permissions.test.delegate",
            &["--arg", "I64:0", "--native-handles", "1"],
        );
        assert_eq!(exhausted["value"]["value"]["case"], "Err");
        assert_eq!(exhausted["value"]["value"]["payload"]["value"], "11");
        for entry in ["permissions.test.amplify", "permissions.test.empty_clock"] {
            let denied = invoke(
                &builds,
                mode,
                entry,
                &["--native-clock", "true", "--native-listen", "127.0.0.1:0"],
            );
            assert_eq!(denied["value"]["value"]["case"], "Err");
            assert_eq!(denied["value"]["value"]["payload"]["value"], "1");
        }
        let wrapped = invoke(&builds, mode, "permissions.test.wrapped", &[]);
        assert_eq!(wrapped["value"]["value"]["payload"]["value"]["case"], "Denied");
        let granted = invoke(
            &builds,
            mode,
            "permissions.test.wrapped",
            &["--native-listen", "127.0.0.1:0"],
        );
        assert_eq!(granted["value"]["value"]["case"], "Ok");
        let raw_empty = invoke(&builds, mode, "permissions.test.raw", &["--arg", "I64:0"]);
        assert_eq!(raw_empty["value"]["value"]["case"], "Ok");
        for mask in ["I64:-1", "I64:16"] {
            let raw = invoke(&builds, mode, "permissions.test.raw", &["--arg", mask]);
            assert_eq!(raw["value"]["value"]["payload"]["value"]["case"], "InvalidInput");
        }
    }
}
#[test]
fn typed_permissions_reject_integer_and_cross_domain_arguments() {
    for replacement in ["5", "Permission::ClockRead"] {
        let bad = APP.replace(
            "restrict(Native::capability(),set(bits))",
            &format!("restrict(Native::capability(),{replacement})"),
        );
        let diagnostic = compile_bundle("app", &sources(&bad)).unwrap_err();
        assert_eq!(diagnostic.code, "compile.type.mismatch", "{diagnostic:?}");
    }

    let app = APP
        .replace("use access {", "use db {SqlPermissions};\nuse access {")
        .replace(
            "module:\"permissions.test\"",
            "db.SqlPermissions:\"lobo.sql.placeholder\",module:\"permissions.test\"",
        )
        .replace(
            "restrict(Native::capability(),set(bits))",
            "restrict(Native::capability(),SqlPermissions {read:true,write:false,schema:false})",
        );
    let mut files = sources(&app);
    let sql_ids =
        lobo_core::identities::parse_manifest(include_str!("../libraries/sql/sql.lobo.ids.json")).unwrap();
    files[0].text = files[0]
        .text
        .replace("lobo.sql.placeholder", &sql_ids["Permissions"]);
    files.push(SourceFile {
        name: "db".into(),
        text: include_str!("../libraries/sql/sql.lobo").into(),
        identities: Some(sql_ids),
    });
    let diagnostic = compile_bundle("app", &files).unwrap_err();
    assert_eq!(diagnostic.code, "compile.type.mismatch", "{diagnostic:?}");
}
