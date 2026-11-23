use lobo_core::{compile_bundle, Artifact, SourceFile};
use serde_json::{json, Value as Json};
mod support;
use support::Builds;

fn tree() -> Builds {
    let source = std::fs::read_to_string("examples/tree.lobo").unwrap();
    let artifact = compile_bundle(
        "tree",
        &[
            SourceFile {
                identities: None,
                name: "tree".into(),
                text: source,
            },
            SourceFile {
                identities: None,
                name: "sequences".into(),
                text: include_str!("../examples/sequences.lobo").into(),
            },
        ],
    )
    .unwrap();
    let checked = Artifact::from_json(&artifact.to_json().unwrap()).unwrap();
    Builds::new(&checked)
}

fn input(keys: &[i64]) -> String {
    let mut wire = format!("List:Count:{}", keys.len());
    for key in keys {
        wire.push_str(&format!(",I64:{key}"));
    }
    wire
}

fn summary(keys: &[i64], height: i64) -> Json {
    json!({"type":"Record","value":{"id":"example.tree.summary","fields":{
        "keys":{"type":"List","value":{"element":"I64","elements":keys.iter().map(|key|
            json!({"type":"I64","value":key.to_string()})).collect::<Vec<_>>()}},
        "count":{"type":"I64","value":keys.len().to_string()},
        "height":{"type":"I64","value":height.to_string()}
    }}})
}

#[test]
fn frozen_tree_construction_and_inorder_traversal_run_entirely_in_lobo() {
    let builds = tree();
    for mode in ["eval", "native", "sanitized"] {
        for (keys, ordered, height) in [
            (vec![5, 3, 7, 2, 4, 6, 8], vec![2, 3, 4, 5, 6, 7, 8], 3),
            (vec![3, 1, 3, 2], vec![1, 2, 3], 3),
            (vec![], vec![], 0),
            (vec![-2, -3, -1], vec![-3, -2, -1], 2),
            (vec![1, 2, 3, 4], vec![1, 2, 3, 4], 4),
        ] {
            let result = builds.invoke(
                mode,
                "example.tree.summarize",
                &[&input(&keys), "Bool:false"],
                1000000,
                100000,
            );
            assert_eq!(
                result["value"],
                summary(&ordered, height),
                "{mode}: {keys:?}: {result}"
            );
        }
    }
}

#[test]
fn reverse_traversal_and_extreme_keys_preserve_count_and_shape_height() {
    let builds = tree();
    for mode in ["eval", "native", "sanitized"] {
        for (keys, ordered, height) in [
            (vec![5, 3, 7, 2, 4, 6, 8], vec![2, 3, 4, 5, 6, 7, 8], 3),
            (vec![3, 1, 3, 2], vec![1, 2, 3], 3),
            (vec![], vec![], 0),
            (vec![-2, -3, -1], vec![-3, -2, -1], 2),
            (vec![1, 2, 3, 4], vec![1, 2, 3, 4], 4),
            (vec![0], vec![0], 1),
            (vec![0, 0, 0], vec![0], 1),
            (vec![0, i64::MIN, i64::MAX], vec![i64::MIN, 0, i64::MAX], 2),
            (
                vec![i64::MIN, i64::MAX, 0, i64::MIN, i64::MAX],
                vec![i64::MIN, 0, i64::MAX],
                3,
            ),
        ] {
            for descending in [false, true] {
                let expected = if descending {
                    ordered.iter().rev().copied().collect()
                } else {
                    ordered.clone()
                };
                let result = builds.invoke(
                    mode,
                    "example.tree.summarize",
                    &[&input(&keys), if descending { "Bool:true" } else { "Bool:false" }],
                    1000000,
                    100000,
                );
                assert_eq!(
                    result["value"],
                    summary(&expected, height),
                    "{mode}: {keys:?}, descending {descending}: {result}"
                );
            }
        }
    }
}

#[test]
fn forty_level_trees_do_not_need_one_language_call_frame_per_level() {
    let builds = tree();
    let ordered = (-20..20).collect::<Vec<_>>();
    let reversed = ordered.iter().rev().copied().collect::<Vec<_>>();
    for mode in ["eval", "native", "sanitized"] {
        for keys in [&ordered, &reversed] {
            for descending in [false, true] {
                let result = builds.invoke(
                    mode,
                    "example.tree.summarize",
                    &[&input(keys), if descending { "Bool:true" } else { "Bool:false" }],
                    20000000,
                    100000,
                );
                assert_eq!(
                    result["value"],
                    summary(if descending { &reversed } else { &ordered }, 40),
                    "{mode}: {result}"
                );
            }
        }
    }
}

#[test]
fn balanced_tree_variations_and_duplicates_keep_known_height_without_host_tree_building() {
    let builds = tree();
    let shape = [8, 4, 12, 2, 6, 10, 14, 1, 3, 5, 7, 9, 11, 13, 15];
    let mut seed = 0x6d20a815u32;
    for case in 0..32 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let scale = i64::from((seed >> 24) % 7 + 1);
        let shift = i64::from((seed >> 16) % 256) - 128;
        // Positive affine changes preserve the frozen shape; the harness does not build or traverse it.
        let mut keys = shape.iter().map(|key| key * scale + shift).collect::<Vec<_>>();
        keys.extend(keys.clone().into_iter().rev());
        let ordered = (1..=15).map(|key| key * scale + shift).collect::<Vec<_>>();
        for mode in ["eval", "native", "sanitized"] {
            let result = builds.invoke(
                mode,
                "example.tree.summarize",
                &[&input(&keys), "Bool:false"],
                2000000,
                100000,
            );
            assert_eq!(
                result["value"],
                summary(&ordered, 4),
                "{mode}, case {case}: {result}"
            );
        }
    }
}

#[test]
fn constructed_node_table_contains_actual_parent_edges_not_just_sorted_keys() {
    let builds = tree();
    for mode in ["eval", "native", "sanitized"] {
        let result = builds.invoke(
            mode,
            "example.tree.build_tree",
            &[&input(&[5, 3, 7, 2, 4, 6, 8])],
            1000000,
            100000,
        );
        let tree = &result["value"]["value"]["fields"];
        assert_eq!(tree["height"]["value"], "3");
        let nodes = tree["nodes"]["value"]["elements"].as_array().unwrap();
        assert_eq!(nodes.len(), 7);
        for (node, (index, parent, key, depth)) in nodes.iter().zip([
            (6, 2, 8, 3),
            (5, 2, 6, 3),
            (4, 1, 4, 3),
            (3, 1, 2, 3),
            (2, 0, 7, 2),
            (1, 0, 3, 2),
            (0, -1, 5, 1),
        ]) {
            let fields = &node["value"]["fields"];
            for (name, expected) in [
                ("index", index),
                ("parent", parent),
                ("key", key),
                ("depth", depth),
            ] {
                assert_eq!(fields[name]["value"], expected.to_string(), "{mode}: {node}");
            }
        }
        assert_eq!(tree["root"]["value"]["case"], "Found");
        assert_eq!(tree["root"]["value"]["payload"], *nodes.last().unwrap());
    }
}

#[test]
fn tree_exhaustion_never_returns_a_partial_successful_summary() {
    let builds = tree();
    for mode in ["eval", "native", "sanitized"] {
        for (fuel, heap, code) in [
            (20, 100000, "resource.fuel.exhausted"),
            (1000000, 28, "resource.memory.exhausted"),
        ] {
            let result = builds.invoke(
                mode,
                "example.tree.summarize",
                &[&input(&[5, 3, 7, 2, 4, 6, 8]), "Bool:false"],
                fuel,
                heap,
            );
            assert_eq!(result["status"], "error", "{mode}: {result}");
            assert_eq!(result["diagnostic"]["code"], code, "{mode}: {result}");
            assert_eq!(result["diagnostic"]["phase"], "execute", "{mode}: {result}");
            assert!(result.get("value").is_none(), "{mode}: {result}");
        }
    }
}
