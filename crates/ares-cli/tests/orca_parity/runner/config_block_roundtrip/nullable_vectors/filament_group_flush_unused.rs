//! ToolOrdering.cpp:3066–3081 merges used extruder/volume entries, not global nozzle IDs.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
};

const KEYS: [&str; 10] = [
    "filament_map",
    "filament_volume_map",
    "filament_nozzle_map",
    "filament_self_index",
    "filament_retraction_length",
    "filament_printable",
    "extruder_ams_count",
    "group_algo_with_time",
    "flush_multiplier",
    "flush_volumes_matrix",
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

fn fixture(name: &str) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let fixture_root = repo_root().join("tests/parity/filament-group-flush-unused");
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
    assert_eq!(saved["filament_map"], case["seed_map"]);
    assert_eq!(saved["filament_volume_map"], case["seed_volume_map"]);
    assert_eq!(saved["filament_map_mode"], "Auto For Flush");
    assert_eq!(saved["nozzle_volume_type"], json!(["Standard", "Standard"]));
    let mut plate = String::new();
    zip.by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut plate)
        .unwrap();
    assert!(plate.contains(&format!(
        "<metadata key=\"filament_maps\" value=\"{}\" />",
        case["plate_seed"].as_str().unwrap()
    )));
    assert!(plate.contains("<metadata key=\"filament_map_mode\" value=\"Auto For Flush\" />"));
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-unused",
            &[
                "ToolOrdering.cpp:3066-3081 used-entry extruder/volume and full nozzle publication",
                "FilamentGroupUtils.cpp:346-358 update_used_filament_values",
                "MultiNozzleUtils.cpp:25-41,144-163,319-369 full static result conversion",
                "Print.cpp:4656-4753 publication and column refresh",
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
        .map(|value| value.parse::<f64>().unwrap() > 0.0)
        .collect::<Vec<_>>();
    let whole_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let whole_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    fs::write(
        reference.root.join("filament-group-flush-unused-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": name, "input_sha256": input_sha, "fixture_provenance": case,
            "native_sha256": sha(reference.native.as_bytes()), "api_sha256": sha(&api), "cli_sha256": sha(&cli),
            "native_fields": native_fields, "api_fields": api_fields, "cli_fields": cli_fields,
            "native_usage_mm": usage, "public_api_cli_complete_bytes_equal": api == cli,
            "strict_whole_api_comparison": whole_api, "strict_whole_cli_comparison": whole_cli,
            "whole_output_parity_claimed": false,
            "scope": "used-entry publication, unused seeds, distinct global nozzle map and refreshed filament columns"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        used,
        [true, true, false],
        "two used Native logical filaments"
    );
    for key in KEYS {
        assert_eq!(
            native_fields[key],
            case["expected_native"][key].as_str().unwrap(),
            "Native {key}; {:?}",
            reference.root
        );
    }
    assert_eq!(
        api, cli,
        "complete public API/CLI bytes; {:?}",
        reference.root
    );
    for key in ["filament_map", "filament_volume_map", "filament_nozzle_map"] {
        assert_eq!(
            api_fields[key].split(',').nth(2),
            native_fields[key].split(',').nth(2),
            "API unused {key}; {:?}",
            reference.root
        );
        assert_eq!(
            cli_fields[key].split(',').nth(2),
            native_fields[key].split(',').nth(2),
            "CLI unused {key}; {:?}",
            reference.root
        );
    }
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
    ($($name:ident),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name)); }
    )+};
}

cases! {
    x2d_pla_two_objects_unused_filament_head1,
    x2d_pla_two_objects_unused_filament_head2,
    x2d_pla_two_objects_unused_filament_high_flow,
}
