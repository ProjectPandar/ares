//! ToolOrdering.cpp:1838–1846,3053–3084 and Print.cpp:4656–4753 public map/column publication.
use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
};

const KEYS: [&str; 7] = [
    "filament_map",
    "filament_volume_map",
    "filament_nozzle_map",
    "filament_self_index",
    "filament_retraction_length",
    "extruder_ams_count",
    "group_algo_with_time",
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
    let fixture_root = repo_root().join("tests/parity/per-head-retraction-column");
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
    assert_eq!(saved["filament_map"], json!(["1", "1"]));
    assert_eq!(saved["filament_map_mode"], "Auto For Flush");
    let mut plate = String::new();
    zip.by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut plate)
        .unwrap();
    assert!(plate.contains("<metadata key=\"filament_maps\" value=\"1 1\"/>"));
    assert!(plate.contains("<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>"));
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-group-flush",
            &[
                "ToolOrdering.cpp:1532,1604 pinned empty print-time term",
                "ToolOrdering.cpp:1838-1846 FlushMode and master preference",
                "ToolOrdering.cpp:3053-3084 static map result publication",
                "Print.cpp:4656-4753 used-map merge and rematerialization",
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
    let whole_api =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    let whole_cli =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli);
    fs::write(
        reference.root.join("filament-group-flush-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":name, "input_sha256":input_sha, "fixture_provenance":case,
            "saved_project_map":saved["filament_map"],
            "saved_project_mode":saved["filament_map_mode"],
            "saved_plate_map":"1 1", "saved_plate_mode":"Auto For Flush",
            "native_sha256":sha(reference.native.as_bytes()),
            "api_sha256":sha(&api), "cli_sha256":sha(&cli),
            "native_fields":native_fields, "api_fields":api_fields, "cli_fields":cli_fields,
            "fields_match":native_fields == api_fields && native_fields == cli_fields,
            "public_api_cli_complete_bytes_equal":api == cli,
            "strict_whole_api_comparison":whole_api,
            "strict_whole_cli_comparison":whole_cli,
            "whole_output_parity_claimed":false,
            "scope":"exact Native/API/CLI map, source-shaped volume/global-nozzle maps, materialized self index, nullable retract override and CLI context fields; complete strict diagnostics kept independently"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_ne!(
        native_fields["filament_map"], "1,1",
        "computed Native premise"
    );
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
    x2d_pla_concentric_ironing_auto_map,
    x2d_pla_auto_map_toolchange_boundary,
    x2d_pla_variant_order_stream,
    x2d_pla_variant_order_extrusion_speed,
    x2d_pla_toolchange_without_virtual_move_comments,
    x2d_pla_toolchange_with_virtual_move_comments,
}
