use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn invalid_support_top_z_distance_fails_during_finalization() {
    assert_invalid_finalization("support_top_z_distance", json!(-0.1));
}

#[test]
fn invalid_support_bottom_z_distance_fails_during_finalization() {
    assert_invalid_finalization("support_bottom_z_distance", json!("fast"));
}

#[test]
fn invalid_enforce_support_layers_fails_during_finalization() {
    assert_invalid_finalization("enforce_support_layers", json!(5001));
}

#[test]
fn zero_top_z_distance_resolves_auto_support_interface_pattern_to_concentric() {
    let auto_zero_gap = finalized_support_paths(json!({
        "support_interface_pattern": "auto",
        "support_top_z_distance": 0.0
    }));
    let omitted_zero_gap = finalized_support_paths(json!({
        "support_top_z_distance": 0.0
    }));
    let explicit_concentric = finalized_support_paths(json!({
        "support_interface_pattern": "concentric",
        "support_top_z_distance": 0.0
    }));

    assert_eq!(auto_zero_gap[0].paths(), explicit_concentric[0].paths());
    assert_eq!(omitted_zero_gap[0].paths(), explicit_concentric[0].paths());
    assert!(auto_zero_gap[0].paths().iter().all(PrintPath::is_closed));
}

#[test]
fn positive_top_z_distance_keeps_auto_support_interface_pattern_rectilinear() {
    let default_gap = finalized_support_paths(json!({
        "support_interface_pattern": "auto"
    }));
    let positive_gap = finalized_support_paths(json!({
        "support_interface_pattern": "auto",
        "support_top_z_distance": 0.2
    }));
    let explicit_concentric = finalized_support_paths(json!({
        "support_interface_pattern": "concentric"
    }));

    assert_eq!(positive_gap[0].paths(), default_gap[0].paths());
    assert_ne!(positive_gap[0].paths(), explicit_concentric[0].paths());
    assert!(positive_gap[0].paths().iter().all(|path| !path.is_closed()));
}

#[test]
fn explicit_rectilinear_ignores_zero_top_z_distance_auto_resolution() {
    let rectilinear_zero_gap = finalized_support_paths(json!({
        "support_interface_pattern": "rectilinear",
        "support_top_z_distance": 0.0
    }));
    let auto_zero_gap = finalized_support_paths(json!({
        "support_interface_pattern": "auto",
        "support_top_z_distance": 0.0
    }));

    assert_ne!(rectilinear_zero_gap[0].paths(), auto_zero_gap[0].paths());
    assert!(
        rectilinear_zero_gap[0]
            .paths()
            .iter()
            .all(|path| !path.is_closed())
    );
}

#[test]
fn zero_top_interface_layers_still_disable_interface_before_zero_gap_auto_resolution() {
    let finalized = finalized_support_paths(json!({
        "support_interface_pattern": "auto",
        "support_top_z_distance": 0.0,
        "support_interface_top_layers": 0
    }));

    assert!(
        finalized[0]
            .paths()
            .iter()
            .all(|path| path.role() != PrintPathRole::SupportMaterialInterface)
    );
    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SupportMaterial)
    );
}

fn assert_invalid_finalization(key: &str, value: Value) {
    let err = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![support_line()])],
        &options(json!({ key: value })),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains(key));
}

fn finalized_support_paths(extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![LayerPrintPaths::new(7, 1.6, vec![support_rectangle()])],
        &options(extra),
    )
    .unwrap()
}

fn support_line() -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
    )
    .unwrap()
}

fn support_rectangle() -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
