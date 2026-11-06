//! bbs_3mf.cpp plate metadata may follow model_instance children.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{OrcaInts, Project, load_project};
use std::{collections::BTreeMap, fs};

const KEYS: [&str; 3] = [
    "first_layer_print_sequence",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
];

fn integers(values: &OrcaInts) -> String {
    values
        .0
        .iter()
        .map(|value| value.0.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn imported_fields(project: &Project) -> BTreeMap<&'static str, String> {
    let print = &project.settings().project.print;
    BTreeMap::from([
        (KEYS[0], integers(&print.first_layer_print_sequence)),
        (KEYS[1], integers(&print.other_layers_print_sequence)),
        (
            KEYS[2],
            print.other_layers_print_sequence_nums.0.to_string(),
        ),
    ])
}

fn fixture(name: &str) {
    let fixtures = repo_root().join("tests/parity/filament-group-flush-mechanisms");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(fixtures.join("provenance.json")).unwrap()).unwrap();
    let case = provenance["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap();
    let expected: serde_json::Value =
        serde_json::from_slice(&fs::read(fixtures.join("ordering.json")).unwrap()).unwrap();
    let input = fs::read(fixtures.join(case["input"].as_str().unwrap())).unwrap();
    let input_sha = case["sha256"].as_str().unwrap();
    assert_eq!(sha(&input), input_sha, "unchanged original archive");
    let (reference, _) = native_stream_case::qualify(
        (
            "plate-metadata-order",
            &[
                "bbs_3mf.cpp:3664-3673 metadata and model_instance event dispatch",
                "bbs_3mf.cpp:4654-4661 plate sequence metadata assignments",
                "bbs_3mf.cpp:4738-4748,4986-5005 instance metadata and completion",
            ],
        ),
        name,
        &input,
        input_sha,
        false,
    );
    let block = reference
        .native
        .split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .split_once("; CONFIG_BLOCK_END")
        .unwrap()
        .0;
    let native_fields = block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| KEYS.contains(key))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(native_fields.len(), KEYS.len());
    for key in KEYS {
        assert_eq!(
            native_fields[key],
            expected[name]["native_fields"][key].as_str().unwrap(),
            "Native {key}; input={input_sha}; {:?}",
            reference.root
        );
    }
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let used = usage.split(", ").collect::<Vec<_>>();
    assert_eq!(used.len(), 3);
    assert!(used.iter().all(|value| value.parse::<f64>().unwrap() > 0.0));
    let project = load_project(&input)
        .and_then(|project| project.select_plate(None))
        .unwrap_or_else(|error| {
            panic!(
                "public 3MF plate import: {error}; input={input_sha}; {:?}",
                reference.root
            )
        });
    assert_eq!(project.plates().len(), 1);
    assert_eq!(project.plates()[0].id(), 1);
    assert_eq!(project.plates()[0].instances().len(), 3);
    assert_eq!(project.objects().len(), 3);
    let api_fields = imported_fields(&project);
    for key in KEYS {
        assert_eq!(
            api_fields[key], native_fields[key],
            "imported {key}; input={input_sha}; {:?}",
            reference.root
        );
    }
}

macro_rules! cases {
    ($($name:ident),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name)); }
    )+};
}

cases! {
    x2d_pla_other_layers_ascending_custom_order,
    x2d_pla_other_layers_middle_last_custom_order,
    x2d_pla_short_objects_ascending_prior_state,
    x2d_pla_short_objects_descending_prior_state,
}
