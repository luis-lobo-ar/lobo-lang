use lobo_core::{compile_bundle, identities::parse_manifest, SourceFile};
use serde_json::Value;
use std::{collections::BTreeMap, process::Command};
#[allow(dead_code)]
mod support;
use support::Builds;

fn artifact() -> lobo_core::Artifact {
    let sql = std::fs::read_to_string("libraries/sql/sql.lobo").unwrap();
    let sql_ids =
        parse_manifest(&std::fs::read_to_string("libraries/sql/sql.lobo.ids.json").unwrap()).unwrap();
    let mut ids: BTreeMap<String, String> = [
        ("module", "sql.permissions.test"),
        ("set", "sql.permissions.test.set"),
        ("mask", "sql.permissions.test.mask"),
        ("delegate", "sql.permissions.test.delegate"),
        ("amplify", "sql.permissions.test.amplify"),
        ("empty_open", "sql.permissions.test.empty_open"),
        ("raw", "sql.permissions.test.raw"),
        ("facade", "sql.permissions.test.facade"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect();
    for name in [
        "Permission",
        "Permissions",
        "none",
        "one",
        "allow",
        "abi_mask",
        "restrict",
        "restrict_native",
        "restrict_mask",
        "SqlResult",
    ] {
        ids.insert(format!("sql.{name}"), sql_ids[name].clone());
    }
    let app = r#"module checks;
use sql {Permission,Permissions,none,one,allow,abi_mask,restrict,restrict_native,restrict_mask,SqlResult};
fn set(bits:I64)->Permissions {
 let a=if bits%2==1 {allow(none(),Read)} else {none()};
 let b=if (bits/2)%2==1 {allow(a,Write)} else {a};
 if (bits/4)%2==1 {allow(b,Schema)} else {b}
}
fn mask(bits:I64,duplicate:Bool)->I64 {let requested=set(bits); abi_mask(if duplicate {allow(allow(requested,Read),Read)} else {requested})}
fn delegate(bits:I64)->Native::I64Result actions {native.authorize} {let cap=try restrict_native(Native::sqlite_capability(),set(bits)); Ok(abi_mask(set(bits)))}
fn amplify()->Native::I64Result actions {native.authorize} {let root=Native::sqlite_capability();let child=try restrict_native(root,one(Read));let invalid=try restrict_native(child,one(Write));Ok(42)}
fn empty_open()->Native::BoolResult actions {native.authorize,db.open,db.close} {let child=try restrict_native(Native::sqlite_capability(),none());let db=try Native::sqlite_open(child);Native::sqlite_close(db)}
fn facade()->SqlResult<I64> actions {native.authorize} {try restrict(Native::sqlite_capability(),one(Read)); Ok(1)}
fn raw()->SqlResult<Native::SqliteCapability> actions {native.authorize} {restrict_mask(Native::sqlite_capability(),8)}
"#;
    compile_bundle(
        "app",
        &[
            SourceFile {
                name: "app".into(),
                text: app.into(),
                identities: Some(ids),
            },
            SourceFile {
                name: "sql".into(),
                text: sql,
                identities: Some(sql_ids),
            },
        ],
    )
    .unwrap()
}
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
fn typed_sql_permissions_preserve_sets_roles_and_attenuation() {
    let builds = Builds::new(&artifact());
    for bits in 0..8 {
        for duplicate in [false, true] {
            for mode in ["eval", "native", "sanitized"] {
                let result = builds.invoke(
                    mode,
                    "sql.permissions.test.mask",
                    &[&format!("I64:{bits}"), &format!("Bool:{duplicate}")],
                    100000,
                    100000,
                );
                assert_eq!(
                    result["value"]["value"],
                    (if duplicate { bits | 1 } else { bits }).to_string()
                );
            }
        }
    }
    let file = builds.0.join("permission-test.sqlite");
    let path = file.to_str().unwrap();
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        for (grant, rights) in [
            (None, 0),
            (Some("read"), 1),
            (Some("write"), 3),
            (Some("schema"), 7),
        ] {
            for bits in 0..8 {
                let arg = format!("I64:{bits}");
                let mut args = vec!["--arg", arg.as_str()];
                if let Some(role) = grant {
                    args.extend(["--sqlite-file", path, "--sqlite-mode", role]);
                }
                let result = invoke(&builds, mode, "sql.permissions.test.delegate", &args);
                assert_eq!(
                    result["value"]["value"]["case"],
                    if bits & !rights == 0 { "Ok" } else { "Err" },
                    "{result}"
                );
                if bits & !rights != 0 {
                    assert_eq!(result["value"]["value"]["payload"]["value"], "1");
                }
            }
        }
        for entry in ["sql.permissions.test.amplify", "sql.permissions.test.empty_open"] {
            let result = invoke(
                &builds,
                mode,
                entry,
                &["--sqlite-file", path, "--sqlite-mode", "schema"],
            );
            assert_eq!(result["value"]["value"]["case"], "Err");
            assert_eq!(result["value"]["value"]["payload"]["value"], "1");
        }
        let facade = invoke(&builds, mode, "sql.permissions.test.facade", &[]);
        assert_eq!(
            facade["value"]["value"]["payload"]["value"]["fields"]["kind"]["value"]["case"],
            "Denied"
        );
        let facade = invoke(
            &builds,
            mode,
            "sql.permissions.test.facade",
            &["--sqlite-file", path, "--sqlite-mode", "read"],
        );
        assert_eq!(facade["value"]["value"]["case"], "Ok");
        let raw = invoke(&builds, mode, "sql.permissions.test.raw", &[]);
        assert_eq!(
            raw["value"]["value"]["payload"]["value"]["fields"]["kind"]["value"]["case"], "InvalidInput",
            "{raw}"
        );
    }
    assert!(
        !file.exists(),
        "restriction tests must not create/open database files"
    );
}
