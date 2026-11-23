use lobo_core::{compile_bundle, Artifact, SourceFile};
use serde_json::{json, Value};
mod support;
use support::Builds;

fn bundle(app: &str, dependencies: &[&str]) -> Artifact {
    let load = |name: &str, label: &str| SourceFile {
        identities: None,
        name: label.into(),
        text: std::fs::read_to_string(format!("{}/examples/v1/{name}.lobo", env!("CARGO_MANIFEST_DIR")))
            .unwrap(),
    };
    let mut sources = vec![load(app, "app")];
    for name in dependencies {
        sources.push(load(name, name));
    }
    let a = compile_bundle("app", &sources).unwrap();
    Artifact::from_json(&a.to_json().unwrap()).unwrap()
}
fn number(n: &str) -> Value {
    json!({"type":"I64","value":n})
}

#[test]
fn fixed_collection_pipeline_and_planned_change_pass_all_routes() {
    for (app, first, last, sum) in [
        ("collections", "19996", "0", "49990000"),
        ("collections_changed", "19998", "2", "50000000"),
    ] {
        let builds = Builds::new(&bundle(app, &["sequences"]));
        for mode in ["eval", "native", "sanitized"] {
            let r = builds.invoke(mode, "v1.collections.run", &[], 2_000_000, 2_000_000);
            assert_eq!(r["status"], "ok", "{mode}/{app}: {r}");
            assert_eq!(
                r["value"]["value"]["fields"],
                json!({"count":number("5000"),"first":number(first),"last":number(last),"sum":number(sum)})
            );
            let r = builds.invoke(
                mode,
                "v1.collections.reverse_ints",
                &["List:Count:3,I64:1,I64:2,I64:3"],
                1000,
                1000,
            );
            assert_eq!(
                r["value"]["value"]["elements"],
                json!([number("3"), number("2"), number("1")])
            );
            let r = builds.invoke(
                mode,
                "v1.collections.reverse_texts",
                &["List:Count:2,Text:c3a9,Text:6c6f626f"],
                1000,
                1000,
            );
            assert_eq!(
                r["value"]["value"]["elements"],
                json!([{"type":"Text","value":"lobo"},{"type":"Text","value":"é"}])
            );
            let r = builds.invoke(
                mode,
                "v1.collections.map_texts",
                &["List:Count:2,Text:c3a9,Text:6c6f626f"],
                1000,
                1000,
            );
            assert_eq!(r["value"]["value"]["elements"][0]["value"], "é");
            let r = builds.invoke(
                mode,
                "v1.collections.find_item",
                &["List:Count:2,I64:1,Text:78,I64:7,Text:c3a9"],
                1000,
                1000,
            );
            assert_eq!(r["value"]["value"]["case"], "Ok");
            assert_eq!(
                r["value"]["value"]["payload"]["value"]["fields"]["key"],
                number("7")
            );
            let r = builds.invoke(mode, "v1.collections.find_item", &["List:Count:0"], 1000, 1000);
            assert_eq!(r["value"]["value"]["case"], "Err");
            assert_eq!(r["value"]["value"]["payload"]["value"], "missing");
        }
    }
}

#[test]
fn fixed_document_validation_transformation_and_schema_change_pass_all_routes() {
    for changed in [false, true] {
        let dependencies = if changed {
            [
                "sequences",
                "document_types_changed",
                "document_validate_changed",
                "document_transform_changed",
            ]
        } else {
            [
                "sequences",
                "document_types",
                "document_validate",
                "document_transform",
            ]
        };
        let builds = Builds::new(&bundle("document", &dependencies));
        let items = if changed {
            "Count:3,I64:1,I64:3,Bool:true,Text:6b656570,I64:2,I64:4,Bool:false,Text:6b656570,I64:3,I64:-1,Bool:true,Text:736b6970"
        } else {
            "Count:3,I64:1,I64:3,Bool:true,I64:2,I64:4,Bool:false,I64:3,I64:-1,Bool:true"
        };
        let valid = format!("Record:v1.doc.document:I64:7,Text:c3a9,{items}");
        for mode in ["eval", "native", "sanitized"] {
            let r = builds.invoke(mode, "v1.doc.run", &[&valid], 2_000_000, 2_000_000);
            assert_eq!(r["value"]["value"]["case"], "Ok", "{mode}/{changed}: {r}");
            let report = &r["value"]["value"]["payload"]["value"]["fields"];
            assert_eq!(report["count"], number(if changed { "1" } else { "2" }));
            assert_eq!(report["total"], number(if changed { "6" } else { "4" }));
            let document = &report["document"]["value"]["fields"];
            assert_eq!(document["id"], number("7"));
            assert_eq!(document["name"]["value"], "checked:é");
            let output = document["items"]["value"]["elements"].as_array().unwrap();
            let keys: Vec<_> = output
                .iter()
                .map(|item| item["value"]["fields"]["key"].clone())
                .collect();
            let scores: Vec<_> = output
                .iter()
                .map(|item| item["value"]["fields"]["score"].clone())
                .collect();
            assert_eq!(
                keys,
                if changed {
                    vec![number("1")]
                } else {
                    vec![number("1"), number("3")]
                }
            );
            assert_eq!(
                scores,
                if changed {
                    vec![number("6")]
                } else {
                    vec![number("6"), number("-2")]
                }
            );
            let r = builds.invoke(
                mode,
                "v1.doc.run",
                &["Record:v1.doc.document:I64:7,Text:c3a9,Count:0"],
                2_000_000,
                2_000_000,
            );
            assert_eq!(r["value"]["value"]["case"], "Ok");
            assert_eq!(
                r["value"]["value"]["payload"]["value"]["fields"]["count"],
                number("0")
            );
            for (input, case) in [
                ("Record:v1.doc.document:I64:0,Text:c3a9,Count:0", "DocBadId"),
                ("Record:v1.doc.document:I64:7,Text:,Count:0", "EmptyName"),
            ] {
                let r = builds.invoke(mode, "v1.doc.run", &[input], 2_000_000, 2_000_000);
                assert_eq!(r["value"]["value"]["case"], "Err");
                assert_eq!(r["value"]["value"]["payload"]["value"]["case"], case);
            }
            let bad = if changed {
                "Record:v1.doc.document:I64:7,Text:c3a9,Count:1,I64:0,I64:1,Bool:false,Text:6b656570"
            } else {
                "Record:v1.doc.document:I64:7,Text:c3a9,Count:1,I64:0,I64:1,Bool:false"
            };
            let r = builds.invoke(mode, "v1.doc.run", &[bad], 2_000_000, 2_000_000);
            assert_eq!(r["value"]["value"]["payload"]["value"]["case"], "BadItem");
            assert_eq!(r["value"]["value"]["payload"]["value"]["payload"], number("0"));
            if changed {
                let r = builds.invoke(
                    mode,
                    "v1.doc.run",
                    &["Record:v1.doc.document:I64:7,Text:c3a9,Count:1,I64:1,I64:1,Bool:false,Text:"],
                    2_000_000,
                    2_000_000,
                );
                assert_eq!(r["value"]["value"]["payload"]["value"]["case"], "BadCategory");
            }
        }
    }
}

#[test]
fn fixed_fractional_aggregation_and_negative_validation_change_pass_all_routes() {
    let float = |bits: &str| json!({"type":"F64","value":bits});
    for changed in [false, true] {
        let builds = Builds::new(&bundle(
            if changed { "numerical_changed" } else { "numerical" },
            &["sequences"],
        ));
        for mode in ["eval", "native", "sanitized"] {
            let input = if changed {
                "List:Count:3,F64:1.5,F64:2.25,F64:0.0"
            } else {
                "List:Count:3,F64:1.5,F64:2.25,F64:-0.75"
            };
            let r = builds.invoke(mode, "v1.numerical.run", &[input], 2_000_000, 2_000_000);
            assert_eq!(r["value"]["value"]["case"], "Ok", "{mode}/{changed}: {r}");
            let stats = &r["value"]["value"]["payload"]["value"]["fields"];
            assert_eq!(stats["count"], number("3"));
            assert_eq!(
                stats["sum"],
                float(if changed {
                    "0x400e000000000000"
                } else {
                    "0x4008000000000000"
                })
            );
            assert_eq!(
                stats["mean"],
                float(if changed {
                    "0x3ff4000000000000"
                } else {
                    "0x3ff0000000000000"
                })
            );
            assert_eq!(
                stats["variance"],
                float(if changed {
                    "0x3fec000000000000"
                } else {
                    "0x3ffa000000000000"
                })
            );
            for (input, case, index) in [
                ("List:Count:0", "Empty", None),
                (
                    "List:Count:3,F64:1.0,F64:0x7ff8000000000001,F64:0x7ff0000000000000",
                    "NonFinite",
                    Some("1"),
                ),
                (
                    "List:Count:2,F64:0xfff0000000000000,F64:1.0",
                    "NonFinite",
                    Some("0"),
                ),
                (
                    "List:Count:2,F64:0x7fefffffffffffff,F64:0x7fefffffffffffff",
                    "ArithmeticNonFinite",
                    None,
                ),
            ] {
                let r = builds.invoke(mode, "v1.numerical.run", &[input], 2_000_000, 2_000_000);
                assert_eq!(r["value"]["value"]["case"], "Err");
                let error = &r["value"]["value"]["payload"]["value"];
                assert_eq!(error["case"], case, "{r}");
                if let Some(index) = index {
                    assert_eq!(error["payload"], number(index));
                }
            }
            for value in ["F64:-0.0", "F64:1.5"] {
                let input = format!("List:Count:1,{value}");
                let r = builds.invoke(mode, "v1.numerical.run", &[&input], 2_000_000, 2_000_000);
                assert_eq!(r["value"]["value"]["case"], "Ok");
                assert_eq!(
                    r["value"]["value"]["payload"]["value"]["fields"]["variance"],
                    float("0x0000000000000000")
                );
            }
            if changed {
                let r = builds.invoke(
                    mode,
                    "v1.numerical.run",
                    &["List:Count:3,F64:1.5,F64:2.25,F64:-0.75"],
                    2_000_000,
                    2_000_000,
                );
                assert_eq!(r["value"]["value"]["payload"]["value"]["case"], "Negative");
                assert_eq!(r["value"]["value"]["payload"]["value"]["payload"], number("2"));
            }
        }
    }
}
