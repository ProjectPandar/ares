use std::io::{Cursor, Read};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::{Value, json};

use super::{RELATIVE_ERROR, RELATIVE_INPUT};

#[test]
fn processed_project_entry_rejects_missing_reset() {
    let result = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(
            crate::runner::application::slice_project_plate_processed_validated(
                RELATIVE_INPUT,
                GenerationMetadata::default(),
                None,
                &ConfigOverrides::default(),
                SlicerContext::new(0),
            ),
        );
    assert!(
        matches!(result, Err(SliceError::Validate { message: ref text, .. }) if text == RELATIVE_ERROR),
        "observed error: {:?}",
        result.as_ref().err()
    );
}

#[test]
fn stl_entry_rejects_relative_without_layer_reset() {
    let options = serde_json::from_value(json!({
        "use_relative_e_distances": true, "gcode_flavor": "marlin",
    }))
    .unwrap();
    let input = std::fs::read(crate::runner::repo_root().join("tests/parity/cube10.stl")).unwrap();
    let result = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_stl_validated(input, options));
    assert!(
        matches!(result, Err(SliceError::InvalidInput(ref text)) if text == RELATIVE_ERROR),
        "observed error: {:?}",
        result.as_ref().err()
    );
}

#[test]
fn profile_entries_reject_relative_without_reset() {
    let mut archive = zip::ZipArchive::new(Cursor::new(RELATIVE_INPUT)).unwrap();
    let mut bytes = Vec::new();
    archive
        .by_name("Metadata/project_settings.config")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let settings: Value = serde_json::from_slice(&bytes).unwrap();
    let profiles = ["machine", "process", "filament"].map(|kind| {
        let mut fields = if kind == "machine" {
            settings.clone()
        } else if kind == "filament" {
            json!({"filament_type": ["PLA"], "filament_diameter": ["1.75"], "filament_flow_ratio": ["1"]})
        } else {
            json!({})
        };
        for key in [
            "compatible_printers",
            "compatible_printers_condition",
            "compatible_prints",
            "compatible_prints_condition",
        ] {
            fields.as_object_mut().unwrap().remove(key);
        }
        fields["type"] = json!(kind);
        fields["name"] = json!(kind);
        fields["inherits"] = json!("");
        fields["instantiation"] = json!("true");
        fields
    });
    let catalog = json!({"vendors": [], "imports": [{"profiles": profiles}]}).to_string();
    let selection =
        json!({"machine": "machine", "process": "process", "filaments": ["filament"]}).to_string();
    let stl = std::fs::read(crate::runner::repo_root().join("tests/parity/cube10.stl")).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let overrides = ConfigOverrides::default();
    for (input, filename, placement) in [
        (RELATIVE_INPUT, "input.3mf", ares_core::Placement::None),
        (stl.as_slice(), "input.stl", ares_core::Placement::Center),
    ] {
        let request = ProfileSliceRequest {
            filename,
            catalog_json: &catalog,
            selection_json: &selection,
            plate: None,
            overrides: &overrides,
            placement,
        };
        let result = runtime.block_on(ares_core::slice_with_profiles_validated(
            input,
            request,
            GenerationMetadata::default(),
            SlicerContext::new(0),
        ));
        assert!(
            matches!(result, Err(SliceError::Validate { message: ref text, .. }) if text == RELATIVE_ERROR),
            "observed error: {:?}",
            result.as_ref().err()
        );
        let result = runtime.block_on(ares_core::slice_with_profiles_processed_validated(
            input,
            request,
            GenerationMetadata::default(),
            SlicerContext::new(0),
        ));
        assert!(
            matches!(result, Err(SliceError::Validate { message: ref text, .. }) if text == RELATIVE_ERROR),
            "observed error: {:?}",
            result.as_ref().err()
        );
    }
}
