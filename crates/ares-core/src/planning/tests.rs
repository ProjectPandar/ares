use super::*;
use crate::{InputFormat, Model, Point3, SliceOptions, Triangle};
use serde_json::json;

mod filament_z_shrinkage;

#[test]
fn slice_options_expose_default_and_custom_layer_heights() {
    let defaults = SliceOptions::default();
    assert_eq!(defaults.layer_height().unwrap(), 0.2);
    assert_eq!(defaults.initial_layer_print_height().unwrap(), 0.2);

    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.25,
        "initial_layer_print_height": 0.3
    }))
    .unwrap();

    assert_eq!(options.layer_height().unwrap(), 0.25);
    assert_eq!(options.initial_layer_print_height().unwrap(), 0.3);
}

#[test]
fn slice_options_reject_invalid_layer_heights() {
    let options: SliceOptions = serde_json::from_value(json!({ "layer_height": 0.0 })).unwrap();
    assert!(matches!(
        options.layer_height(),
        Err(SliceError::InvalidInput(_))
    ));

    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_print_height": -0.1
    }))
    .unwrap();
    assert!(matches!(
        options.initial_layer_print_height(),
        Err(SliceError::InvalidInput(_))
    ));

    for layer_height in [0.0000005, 0.000001] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "layer_height": layer_height })).unwrap();
        assert!(matches!(
            options.layer_height(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn initial_layer_print_height_accepts_numeric_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "initial_layer_print_height": "0.24"
    }))
    .unwrap();

    assert_eq!(options.initial_layer_print_height().unwrap(), 0.24);
}

#[test]
fn plan_layers_uses_initial_layer_print_height_then_regular_height_without_shortening_final_layer()
{
    let model = model_with_height(0.55);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.15,
        "initial_layer_print_height": 0.3
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].id(), 0);
    assert_eq!(layers[0].height(), 0.3);
    assert_eq!(layers[0].print_z(), 0.3);
    assert_eq!(layers[1].height(), 0.15);
    assert_eq!(layers[1].print_z(), 0.45);
    assert_eq!(layers[2].height(), 0.15);
    assert_eq!(layers[2].print_z(), 0.6);
}

#[test]
fn plan_layers_rejects_empty_flat_and_non_finite_models() {
    let empty = Model::new(InputFormat::Stl, Vec::new());
    assert!(matches!(
        plan_layers(&empty, &SliceOptions::default()),
        Err(crate::SliceError::InvalidInput(_))
    ));

    let flat = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ])],
    );
    assert!(matches!(
        plan_layers(&flat, &SliceOptions::default()),
        Err(crate::SliceError::InvalidInput(_))
    ));

    let non_finite = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, f32::NAN),
            Point3::new(0.0, 1.0, 0.4),
        ])],
    );
    assert!(matches!(
        plan_layers(&non_finite, &SliceOptions::default()),
        Err(crate::SliceError::InvalidInput(_))
    ));
}

#[test]
fn plan_layers_stops_at_model_top_when_fixed_height_lands_exactly() {
    let model = two_layer_model();
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].print_z(), 0.2);
    assert_eq!(layers[1].print_z(), 0.4);
}

#[test]
fn precise_z_height_defaults_to_false() {
    assert!(!SliceOptions::default().precise_z_height().unwrap());
}

#[test]
fn parses_precise_z_height_boolean() {
    let enabled: SliceOptions =
        serde_json::from_value(json!({ "precise_z_height": true })).unwrap();
    let disabled: SliceOptions =
        serde_json::from_value(json!({ "precise_z_height": false })).unwrap();

    assert!(enabled.precise_z_height().unwrap());
    assert!(!disabled.precise_z_height().unwrap());
}

#[test]
fn rejects_non_boolean_precise_z_height() {
    for value in [json!(1), json!("true"), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "precise_z_height": value })).unwrap();

        assert!(matches!(
            options.precise_z_height(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn default_layer_planning_does_not_truncate_final_layer_to_model_top() {
    let model = model_with_height(1.31);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "min_layer_height": 0.07
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 7);
    assert_eq!(layers.last().unwrap().height(), 0.2);
    assert_eq!(layers.last().unwrap().print_z(), 1.4);
}

#[test]
fn default_layer_planning_does_not_parse_precise_adjustment_bounds() {
    let model = model_with_height(1.31);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "min_layer_height": -0.01,
        "max_layer_height": -0.01,
        "nozzle_diameter": 0.004
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 7);
    assert_eq!(layers.last().unwrap().print_z(), 1.4);
}

#[test]
fn precise_z_height_adjusts_last_five_layers_to_model_top() {
    let model = model_with_height(1.31);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "min_layer_height": 0.07,
        "precise_z_height": true
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 7);
    assert_eq!(layers[0].height(), 0.2);
    assert_eq!(layers[1].height(), 0.2);
    for layer in &layers[2..] {
        assert_eq!(layer.height(), 0.182);
    }
    assert_eq!(layers.last().unwrap().print_z(), 1.31);
}

#[test]
fn precise_z_height_leaves_short_series_unchanged() {
    let model = model_with_height(0.71);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "precise_z_height": true
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 4);
    assert_eq!(
        layers.iter().map(Layer::print_z).collect::<Vec<_>>(),
        vec![0.2, 0.4, 0.6, 0.8]
    );
}

#[test]
fn default_layer_planning_stops_when_next_regular_midpoint_reaches_model_top() {
    let model = model_with_height(1.3);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 6);
    assert_eq!(layers.last().unwrap().print_z(), 1.2);
}

#[test]
fn precise_z_height_keeps_default_series_when_gap_exceeds_min_layer_bound() {
    let model = model_with_height(1.11);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "min_layer_height": 0.19,
        "precise_z_height": true
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 6);
    assert_eq!(
        layers.iter().map(Layer::print_z).collect::<Vec<_>>(),
        vec![0.2, 0.4, 0.6, 0.8, 1.0, 1.2]
    );
}

#[test]
fn precise_z_height_respects_nozzle_min_when_resolving_default_max_height() {
    let model = model_with_height(1.17);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.05,
        "initial_layer_print_height": 0.05,
        "min_layer_height": 0.2,
        "max_layer_height": 0.0,
        "nozzle_diameter": [0.05],
        "precise_z_height": true
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 23);
    assert_eq!(layers.last().unwrap().print_z(), 1.17);
}

fn two_layer_model() -> Model {
    model_with_height(0.4)
}

fn model_with_height(height: f32) -> Model {
    Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, height / 2.0),
            Point3::new(0.0, 1.0, height),
        ])],
    )
}
