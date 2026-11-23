#[path = "support/sql_rows.rs"]
mod sql_rows;
use lobo_core::{compile_bundle, identities::parse_manifest, SourceFile};
use serde_json::{json, Value as Json};
use std::process::Command;
#[allow(dead_code)]
mod support;
use support::Builds;

fn invoke(
    builds: &Builds,
    mode: &str,
    database: &std::path::Path,
    entry: &str,
    args: &[String],
    role: Option<&str>,
) -> Json {
    let mut command = Command::new(builds.0.join(mode));
    command.args([
        "--entry",
        &if entry.contains('.') {
            entry.to_owned()
        } else {
            format!("example.sqlite.users.{entry}")
        },
        "--fuel",
        "1000000",
        "--heap-cells",
        "1000000",
    ]);
    for argument in args {
        command.args(["--arg", argument]);
    }
    if let Some(role) = role {
        command.args(["--sqlite-file", database.to_str().unwrap(), "--sqlite-mode", role]);
    }
    let output = command
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1:symbolize=0")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{mode}/{entry}: {output:?}");
    assert!(output.stderr.is_empty(), "{mode}: {output:?}");
    let result: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "ok", "{result}");
    result["value"]["value"].clone()
}
fn rows(value: &Json) -> Vec<Vec<Json>> {
    assert_eq!(value["case"], "Ok", "{value}");
    value["payload"]["value"]["elements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row["value"]["fields"]["values"]["value"]["elements"]
                .as_array()
                .unwrap()
                .clone()
        })
        .collect()
}
#[test]
fn native_sql_library_persists_binds_decodes_and_rolls_back_in_both_backends() {
    let sources = [
        SourceFile {
            name: "users".into(),
            text: include_str!("../examples/native/sqlite/users.lobo").into(),
            identities: Some(
                parse_manifest(include_str!("../examples/native/sqlite/users.lobo.ids.json")).unwrap(),
            ),
        },
        SourceFile {
            name: "sql".into(),
            text: include_str!("../libraries/sql/sql.lobo").into(),
            identities: Some(parse_manifest(include_str!("../libraries/sql/sql.lobo.ids.json")).unwrap()),
        },
    ];
    let mut sources = sources.to_vec();
    sources.push(SourceFile {
        name: "skill".into(),
        text: include_str!("../.agents/skills/lobo-programming/references/sqlite-example.lobo").into(),
        identities: Some(
            parse_manifest(include_str!(
                "../.agents/skills/lobo-programming/references/sqlite-example.lobo.ids.json"
            ))
            .unwrap(),
        ),
    });
    sources.push(SourceFile {
        name: "rows".into(),
        text: include_str!("fixtures/sql_row_helpers.lobo").into(),
        identities: None,
    });
    let artifact = compile_bundle("users", &sources).unwrap();
    let builds = Builds::new(&artifact);
    for mode in ["native", "llvm", "sanitized", "llvm-sanitized"] {
        let database = builds.0.join(format!("{mode}.sqlite"));
        for (name, expected) in sql_rows::CASES {
            let value = invoke(
                &builds,
                mode,
                &database,
                &format!("test.sql.rows.{name}"),
                &[],
                None,
            );
            if let Some(expected) = expected {
                assert_eq!(value["case"], "Ok", "{mode}/{name}: {value}");
                assert_eq!(value["payload"]["value"], expected, "{mode}/{name}");
            } else {
                assert_eq!(value["case"], "Err", "{mode}/{name}: {value}");
                assert_eq!(
                    value["payload"]["value"]["fields"]["kind"]["value"]["case"], "Decode",
                    "{mode}/{name}"
                );
                assert_eq!(value["payload"]["value"]["fields"]["backend_code"]["value"], "0");
            }
        }
        let denied = invoke(&builds, mode, &database, "list", &[], None);
        assert_eq!(denied["case"], "Err");
        assert_eq!(
            denied["payload"]["value"]["fields"]["kind"]["value"]["case"],
            "Denied"
        );
        for _ in 0..2 {
            assert_eq!(
                invoke(&builds, mode, &database, "setup", &[], Some("schema"))["case"],
                "Ok"
            );
        }
        let skill = rows(&invoke(
            &builds,
            mode,
            &database,
            "skill.sqlite.read",
            &[],
            Some("read"),
        ));
        assert_eq!(skill[0][0]["value"]["payload"]["value"], "42");
        let all = rows(&invoke(&builds, mode, &database, "list", &[], Some("read")));
        assert_eq!(all.len(), 2);
        assert_eq!(all[0][0]["value"]["payload"]["value"], "1");
        assert_eq!(all[1][0]["value"]["payload"]["value"], "2");
        let name = "Robert'); DROP TABLE users;--\0🐺";
        let encoded = name
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let saved = invoke(
            &builds,
            mode,
            &database,
            "save",
            &["I64:3".into(), format!("Text:{encoded}")],
            Some("write"),
        );
        assert_eq!(saved["case"], "Ok");
        let found = rows(&invoke(
            &builds,
            mode,
            &database,
            "get",
            &["I64:3".into()],
            Some("read"),
        ));
        assert_eq!(found[0][1]["value"]["payload"]["value"], name);
        assert!(rows(&invoke(
            &builds,
            mode,
            &database,
            "get",
            &["I64:999".into()],
            Some("read")
        ))
        .is_empty());
        let readonly = invoke(
            &builds,
            mode,
            &database,
            "save",
            &["I64:1".into(), "Text:78".into()],
            Some("read"),
        );
        assert_eq!(readonly["case"], "Err");
        assert_eq!(
            readonly["payload"]["value"]["fields"]["kind"]["value"]["case"],
            "Denied"
        );
        assert_eq!(
            invoke(&builds, mode, &database, "undo", &[], Some("write"))["case"],
            "Ok"
        );
        assert_eq!(
            rows(&invoke(&builds, mode, &database, "list", &[], Some("read"))).len(),
            3
        );
        let values = rows(&invoke(&builds, mode, &database, "typed", &[], Some("read")));
        let cases = values[0]
            .iter()
            .map(|value| value["value"]["case"].clone())
            .collect::<Vec<_>>();
        assert_eq!(
            cases,
            json!(["Null", "Integer", "Decimal", "String", "Binary"])
                .as_array()
                .unwrap()
                .clone()
        );
        assert_eq!(values[0][2]["value"]["payload"]["value"], "0x3ff4000000000000");
        for index in [3, 4] {
            assert_eq!(values[0][index]["value"]["payload"]["value"], "");
        }
        let oracle=Command::new("python3").args(["-c","import json,sqlite3,sys; c=sqlite3.connect(sys.argv[1]); print(json.dumps(c.execute('SELECT id,name FROM users ORDER BY id').fetchall(),ensure_ascii=True)); c.close()",database.to_str().unwrap()]).output().unwrap();
        assert!(oracle.status.success(), "{oracle:?}");
        let observed: Json = serde_json::from_slice(&oracle.stdout).unwrap();
        assert_eq!(observed, json!([[1, "Luis"], [2, "Ada"], [3, name]]));
    }
}
