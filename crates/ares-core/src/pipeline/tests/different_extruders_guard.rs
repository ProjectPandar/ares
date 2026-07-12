use serde_json::json;

use crate::{SliceError, SliceOptions, run_slicing_pipeline, slice};

use super::square_pyramid_ascii_stl;

#[test]
fn rejects_different_extruder_types_without_variant_support() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
}

#[test]
fn rejects_different_nozzle_volume_types_without_variant_support() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "High Flow"],
        "extruder_variant_list": ["Direct Drive Standard"]
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
}

#[test]
fn accepts_matching_extruder_characteristics() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();

    assert!(!pipeline.layers().is_empty());
}

#[test]
fn rejects_different_extruders_even_when_variant_list_supports_them() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "High Flow"],
        "extruder_variant_list": ["Direct Drive Standard,Bowden High Flow", "Bowden High Flow"]
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
}

#[test]
fn propagates_invalid_extruder_variant_list_before_unsupported_error() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"],
        "extruder_variant_list": "Direct Drive Standard"
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("extruder_variant_list"));
}

#[test]
fn rejects_different_extruders_before_model_loading() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "nozzle_diameter": [0.4, 0.4],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }))
    .unwrap();

    let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
    assert!(
        !err.to_string()
            .contains("unsupported or malformed model input")
    );
}

#[tokio::test]
async fn public_slice_rejects_different_extruder_types() {
    let err = slice(
        square_pyramid_ascii_stl(),
        options(json!({
            "nozzle_diameter": [0.4, 0.4],
            "extruder_type": ["Direct Drive", "Bowden"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
}

#[tokio::test]
async fn public_slice_rejects_different_nozzle_volume_types() {
    let err = slice(
        square_pyramid_ascii_stl(),
        options(json!({
            "nozzle_diameter": [0.4, 0.4],
            "extruder_type": ["Direct Drive", "Direct Drive"],
            "nozzle_volume_type": ["Standard", "High Flow"]
        })),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("different extruders"));
}

#[test]
fn propagates_invalid_extruder_type_before_unsupported_error() {
    let err = run_slicing_pipeline(
        square_pyramid_ascii_stl(),
        &options(json!({
            "nozzle_diameter": [0.4, 0.4],
            "extruder_type": ["Direct Drive", "Invalid"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("extruder_type"));
    assert!(!err.to_string().contains("different extruders"));
}

#[test]
fn propagates_invalid_nozzle_volume_type_before_unsupported_error() {
    let err = run_slicing_pipeline(
        square_pyramid_ascii_stl(),
        &options(json!({
            "nozzle_diameter": [0.4, 0.4],
            "extruder_type": ["Direct Drive", "Direct Drive"],
            "nozzle_volume_type": ["Standard", "Invalid"]
        })),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_volume_type"));
    assert!(!err.to_string().contains("different extruders"));
}

#[test]
fn propagates_invalid_nozzle_diameter_before_unsupported_error() {
    let err = run_slicing_pipeline(
        square_pyramid_ascii_stl(),
        &options(json!({
            "nozzle_diameter": [0.4, 0.0],
            "extruder_type": ["Direct Drive", "Bowden"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_diameter"));
    assert!(!err.to_string().contains("different extruders"));
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
