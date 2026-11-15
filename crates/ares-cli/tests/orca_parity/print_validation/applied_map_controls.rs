//! Print applied map precedes ToolOrdering.cpp:1288-1303 late recommendation.
use ares_core::{ConfigOverrides, GenerationMetadata, ProjectSettings, SliceError, SlicerContext};
use serde_json::json;

#[test]
fn print_applied_map_global_manual_does_not_override_plate_auto() {
    super::layer_nozzle::verify(
        json!({"raft_layers":"2","support_filament":"2","support_interface_filament":"2","filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"initial_layer_print_height":"0.3","filament_map_mode":"Manual"}),
    );
}

#[test]
fn print_applied_map_flat_shared_map_retains_height_error() {
    let fields = json!({"nozzle_diameter":["0.2"],"layer_height":"0.2","initial_layer_print_height":"0.3","raft_layers":"2","support_filament":"2","support_interface_filament":"2","filament_diameter":["1.75","1.75"],"filament_map":["1","1"]});
    let settings: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&settings, false);
    let directory = super::evidence_directory("print-map-flat-");
    std::fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(
            &json!({"settings":fields,"actual":actual.as_ref().map_err(ToString::to_string)}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(actual,Err(SliceError::Validate { message: ref text, .. }) if text=="Layer height cannot exceed nozzle diameter."),
        "{actual:?}; {directory:?}"
    );
}

#[test]
fn print_applied_map_invalid_input_still_rejected_before_print() {
    let input =
        include_bytes!("../../../../../tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf");
    let directory = super::evidence_directory("print-map-invalid-");
    for (index, map) in [json!(["1", "0"]), json!(["1", "2"])]
        .into_iter()
        .enumerate()
    {
        let fields = json!({"filament_diameter":["1.75","1.75"],"filament_map":map});
        let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
        let actual = ares_core::validate_project_plate(input, None, &overrides);
        std::fs::write(
            directory.join(format!("{index}.json")),
            serde_json::to_vec_pretty(
                &json!({"overrides":fields,"actual":actual.as_ref().map_err(ToString::to_string)}),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            matches!(actual,Err(SliceError::InvalidInput(ref text)) if text=="invalid Orca option filament_map"),
            "{actual:?}; {directory:?}"
        );
    }
}

#[test]
fn print_applied_map_flat_missing_map_lookup_guard_is_preserved() {
    let fields = json!({"nozzle_diameter":["0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["1"],"enable_support":"1","support_filament":"2","support_interface_filament":"2"});
    let settings: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&settings, false);
    let directory = super::evidence_directory("print-map-missing-");
    std::fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(
            &json!({"settings":fields,"actual":actual.as_ref().map_err(ToString::to_string)}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(actual,Err(SliceError::Validate { message: ref text, .. }) if text=="invalid filament_map"),
        "{actual:?}; {directory:?}"
    );
}

#[test]
fn print_applied_map_unchanged_recommendation_preserves_export_bytes() {
    // Print.cpp:4661-4755: a single-colour applied map equals the recommendation.
    let fields = json!({"before_layer_change_gcode":"G92 E0\n"});
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let metadata = GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let gcode = runtime
        .block_on(crate::runner::application::slice_project_plate_validated(
            super::RELATIVE_INPUT,
            metadata,
            None,
            &overrides,
            SlicerContext::new(0),
        ))
        .unwrap();
    let directory = super::evidence_directory("print-map-export-");
    std::fs::write(directory.join("ares.gcode"), &gcode).unwrap();
    let input = directory.join("input.3mf");
    std::fs::write(&input, super::RELATIVE_INPUT).unwrap();
    let runner = crate::runner::OrcaRunner::from_env().unwrap().unwrap();
    let mut comparisons = Vec::new();
    for label in ["native", "native-replay"] {
        let output = directory.join(label);
        std::fs::create_dir(&output).unwrap();
        crate::runner::stages::run_live(
            runner.bin(),
            [
                "--before-layer-change-gcode".to_owned(),
                "G92 E0\n".to_owned(),
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_string_lossy().into_owned(),
                input.to_string_lossy().into_owned(),
            ],
            crate::runner::stages::Stage::Slice,
            &output,
        )
        .unwrap();
        let native = std::fs::read(output.join("plate_1.gcode")).unwrap();
        let comparison = crate::golden::compare_ordered_bytes_generator_only(&native, &gcode);
        comparisons.push(json!({
            "label":label,"native_sha256":super::sha256(&native),"comparison":comparison,
        }));
        std::fs::write(
            directory.join("result.json"),
            serde_json::to_vec_pretty(&json!({
                "input_sha256":super::sha256(super::RELATIVE_INPUT),"overrides":fields,
                "ares_sha256":super::sha256(&gcode),"comparisons":comparisons,
            }))
            .unwrap(),
        )
        .unwrap();
        comparison.unwrap_or_else(|error| panic!("{error}; {directory:?}"));
    }
    let exported = std::str::from_utf8(&gcode).unwrap();
    assert!(exported.lines().any(|line| line == "; filament_map = 1"));
    assert!(
        exported
            .lines()
            .any(|line| line == "; before_layer_change_gcode = G92 E0\\n")
    );
}
