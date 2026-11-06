//! ToolOrdering.cpp:1477–1501,1604–1616 and FilamentGroup.cpp:191–197 public scoring context.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
};

const DEFAULT_MATRIX: &str = "0,100,1000,100,0,800,1000,800,0,0,900,20,900,0,1000,20,1000,0";
const FAST_MATRIX: &str = "0,10,100,10,0,80,100,80,0,0,3600,80,3600,0,4000,80,4000,0";
const TIME_MATRIX: &str = "0,100,1000,100,0,800,1000,800,0,0,900,1,900,0,1000,1,1000,0";
const KEYS: [&str; 11] = [
    "flush_volumes_matrix",
    "filament_map",
    "filament_volume_map",
    "filament_nozzle_map",
    "filament_self_index",
    "filament_retraction_length",
    "extruder_ams_count",
    "group_algo_with_time",
    "machine_load_filament_time",
    "machine_unload_filament_time",
    "flush_multiplier",
];

fn fields(text: &str) -> BTreeMap<&str, &str> {
    let block = text.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let values = block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .filter(|(key, _)| KEYS.contains(key))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(values.len(), KEYS.len());
    values
}

fn fixture(
    name: &str,
    (map, nozzle, matrix, flag, load, unload): (&str, &str, &str, &str, &str, &str),
) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let fixture_root = repo_root().join("tests/parity/filament-group-flush-context");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture_root.join("provenance.json")).unwrap()).unwrap();
    let case = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap();
    let project = fs::read(fixture_root.join(case["input"].as_str().unwrap())).unwrap();
    let input_sha = sha(&project);
    assert_eq!(input_sha, case["sha256"].as_str().unwrap());
    let mut zip = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    let saved: serde_json::Value =
        serde_json::from_reader(zip.by_name("Metadata/project_settings.config").unwrap()).unwrap();
    assert_eq!(saved["filament_map"], json!(["1", "1", "1"]));
    assert_eq!(saved["filament_map_mode"], "Auto For Flush");
    let mut plate = String::new();
    zip.by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut plate)
        .unwrap();
    assert!(plate.contains("<metadata key=\"filament_maps\" value=\"1 1 1\" />"));
    assert!(plate.contains("<metadata key=\"filament_map_mode\" value=\"Auto For Flush\" />"));
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-context",
            &[
                "ToolOrdering.cpp:1477-1501 float matrix and selected multiplier",
                "ToolOrdering.cpp:1532,1604-1616 empty print-time table and actual change time",
                "FilamentGroup.cpp:191-197 combined flush/change-time score",
                "ToolOrdering.cpp:3053-3084 static result publication",
            ],
        ),
        name,
        &project,
        &input_sha,
        false,
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI; {:?}", reference.root);
    let cli = fs::read(cli_path).unwrap();
    let native_fields = fields(&reference.native);
    let api_fields = fields(std::str::from_utf8(&api).unwrap());
    let cli_fields = fields(std::str::from_utf8(&cli).unwrap());
    let usage = reference
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    let used = usage
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let whole_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let whole_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    fs::write(reference.root.join("filament-group-flush-context-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":name, "input_sha256":input_sha, "fixture_provenance":case,
            "native_sha256":sha(reference.native.as_bytes()), "api_sha256":sha(&api), "cli_sha256":sha(&cli),
            "native_fields":native_fields, "api_fields":api_fields, "cli_fields":cli_fields,
            "native_usage_mm":usage, "public_api_cli_complete_bytes_equal":api == cli,
            "strict_whole_api_comparison":whole_api, "strict_whole_cli_comparison":whole_cli,
            "whole_output_parity_claimed":false,
            "scope":"literal selected matrix and map/column fields; usage parsed only to verify all three Native filaments are used"
        })).unwrap()).unwrap();
    assert_eq!(used.len(), 3, "three logical Native filaments");
    assert!(used.iter().all(|value| *value > 0.0));
    for (key, value) in [
        ("filament_map", map),
        ("filament_nozzle_map", nozzle),
        ("flush_volumes_matrix", matrix),
        ("group_algo_with_time", flag),
        ("machine_load_filament_time", load),
        ("machine_unload_filament_time", unload),
        ("filament_volume_map", "0,0,0"),
        ("filament_self_index", "1,2,3"),
    ] {
        assert_eq!(
            native_fields[key], value,
            "Native {key}; {:?}",
            reference.root
        );
    }
    assert_eq!(
        api, cli,
        "complete public API/CLI bytes; {:?}",
        reference.root
    );
    for key in KEYS {
        assert_eq!(
            api_fields[key], native_fields[key],
            "API {key}; {:?}",
            reference.root
        );
        assert_eq!(
            cli_fields[key], native_fields[key],
            "CLI {key}; {:?}",
            reference.root
        );
    }
}

macro_rules! cases {
    ($($name:ident => $expected:expr),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name), $expected); }
    )+};
}

cases! {
    x2d_pla_three_objects_default_purge => ("1,2,1", "0,1,0", DEFAULT_MATRIX, "1", "29", "29"),
    x2d_pla_three_objects_fast_purge => ("1,1,2", "0,0,1", FAST_MATRIX, "1", "29", "29"),
    x2d_pla_short_middle_zero_load_time => ("1,2,1", "0,1,0", TIME_MATRIX, "0", "0", "0"),
    x2d_pla_short_middle_high_load_time => ("2,1,1", "1,0,0", TIME_MATRIX, "0", "1000", "0"),
}
