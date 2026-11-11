//! FilamentGroup.cpp preferences/memory/AMS and ToolOrderUtils.cpp carried layer state.
use super::super::super::{
    native_api_cli::{NativeCase, sha},
    native_stream_case, repo_root,
};
use crate::OrcaRunner;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
};

const KEYS: [&str; 16] = [
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
    "first_layer_print_sequence",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
    "machine_load_filament_time",
    "machine_unload_filament_time",
    "extruder_printable_height",
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
    let fixture_root = repo_root().join("tests/parity/filament-group-flush-mechanisms");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture_root.join("provenance.json")).unwrap()).unwrap();
    let case = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap();
    let expectations: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture_root.join(case["expectations_file"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    let expected = &expectations[name];
    let project = fs::read(fixture_root.join(case["input"].as_str().unwrap())).unwrap();
    let input_sha = sha(&project);
    assert_eq!(input_sha, case["sha256"].as_str().unwrap());
    let mut zip = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    let saved: serde_json::Value =
        serde_json::from_reader(zip.by_name("Metadata/project_settings.config").unwrap()).unwrap();
    assert_eq!(saved["filament_map"], expected["seed_map"]);
    assert_eq!(saved["filament_map_mode"], "Auto For Flush");
    assert_eq!(saved["nozzle_volume_type"], json!(["Standard", "Standard"]));
    let mut plate = String::new();
    zip.by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut plate)
        .unwrap();
    assert!(plate.contains(&format!(
        "<metadata key=\"filament_maps\" value=\"{}\" />",
        expected["plate_seed"].as_str().unwrap()
    )));
    assert!(plate.contains("<metadata key=\"filament_map_mode\" value=\"Auto For Flush\" />"));
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush-mechanisms",
            &[
                "FilamentGroup.cpp:36-374 source traversal and preferences",
                "FilamentGroup.cpp:427-566,901-922 remembered candidates and empty-inventory AMS",
                "ToolOrderUtils.cpp:748-1141,1324-1494 custom sequence and retained layer state",
                "ToolOrdering.cpp:1537-1747 physical constraints and valid master preference",
                "Print.cpp:4908-4941 and PrintObject.cpp:349-515 restriction producers",
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
        reference.root.join("filament-group-flush-mechanisms-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": name, "input_sha256": input_sha, "fixture_provenance": {"case": case, "expectations": expected},
            "native_sha256": sha(reference.native.as_bytes()), "api_sha256": sha(&api), "cli_sha256": sha(&cli),
            "native_fields": native_fields, "api_fields": api_fields, "cli_fields": cli_fields,
            "native_usage_mm": usage, "public_api_cli_complete_bytes_equal": api == cli,
            "strict_whole_api_comparison": whole_api, "strict_whole_cli_comparison": whole_cli,
            "whole_output_parity_claimed": false,
            "scope": "literal source map preferences and ordering; empty AMS inventory only, no live colour matching"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        used,
        vec![true; usize::try_from(expected["logical_filament_count"].as_u64().unwrap()).unwrap()],
        "all configured Native logical filaments are used"
    );
    for key in KEYS {
        assert_eq!(
            native_fields[key],
            expected["native_fields"][key].as_str().unwrap(),
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
    ($($name:ident),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name)); }
    )+};
}

cases! {
    x2d_pla_five_filaments_equal_cost_first_assignment,
    x2d_pla_five_filaments_last_filament_flush_bias,
    x2d_pla_three_filaments_remembered_small_gap,
    x2d_pla_three_filaments_outside_absolute_gap,
    x2d_pla_other_layers_ascending_custom_order,
    x2d_pla_other_layers_middle_last_custom_order,
    x2d_pla_short_objects_ascending_prior_state,
    x2d_pla_short_objects_descending_prior_state,
    x2d_pla_auto_map_first_filament_head1_only,
    x2d_pla_auto_map_second_filament_head1_only,
    x2d_pla_auto_map_third_filament_head1_only,
    x2d_pla_auto_map_second_head_height_below_model,
}

#[test]
fn h2c_manual_multi_nozzle_grouping_matches_native() {
    let project = fs::read(
        repo_root()
            .join("tests/parity/filament-group-flush-mechanisms/h2c_manual_multi_nozzle.3mf"),
    )
    .unwrap();
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let native = NativeCase::slice(&runner, &project, "h2c-manual-multi-nozzle-");
    native.compare_prefix_and_maps(GenerationMetadata::deterministic(2026, 11, 10, 2, 0, 0));
}

#[test]
fn h2c_manual_multi_nozzle_two_used_filaments_remain_unsupported() {
    let project = fs::read(repo_root().join(
        "tests/parity/filament-group-flush-mechanisms/h2c_manual_multi_nozzle_two_filaments.3mf",
    ))
    .unwrap();
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native harness required");
    let native = NativeCase::slice(&runner, &project, "h2c-manual-two-filaments-");
    let usage = native
        .native
        .lines()
        .find_map(|line| line.strip_prefix("; filament used [mm] = "))
        .unwrap();
    assert!(
        usage
            .split(", ")
            .all(|value| value.parse::<f64>().unwrap() > 0.0)
    );
    assert!(native.native.contains("; filament_map = 1,2,2\n"));
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 11, 10, 2, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    let error = result.unwrap_err();
    fs::write(native.root.join("ares.api.error"), format!("{error:?}")).unwrap();
    assert!(
        matches!(error, ares_core::SliceError::InvalidInput(ref detail) if detail == "unsupported automatic filament grouping context")
    );
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(native.root.join("ares.cli.gcode"))
        .arg(native.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(native.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(!cli.status.success());
    assert!(
        String::from_utf8_lossy(&cli.stderr)
            .contains("unsupported automatic filament grouping context")
    );
}
