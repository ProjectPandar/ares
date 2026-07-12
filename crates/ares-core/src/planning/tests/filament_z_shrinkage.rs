use super::*;

#[test]
fn default_filament_z_shrinkage_preserves_layer_planning() {
    let model = model_with_height(1.0);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(
        layers.iter().map(Layer::print_z).collect::<Vec<_>>(),
        vec![0.2, 0.4, 0.6, 0.8, 1.0]
    );
}

#[test]
fn filament_z_shrinkage_extends_fixed_height_layer_planning() {
    let model = model_with_height(1.0);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "filament_shrinkage_compensation_z": "80%"
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(
        layers.iter().map(Layer::print_z).collect::<Vec<_>>(),
        vec![0.2, 0.4, 0.6, 0.8, 1.0, 1.2]
    );
}

#[test]
fn filament_z_shrinkage_precise_height_aligns_to_compensated_object_top() {
    let model = model_with_height(1.0);
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "filament_shrinkage_compensation_z": 80,
        "precise_z_height": true
    }))
    .unwrap();

    let layers = plan_layers(&model, &options).unwrap();

    assert_eq!(layers.len(), 6);
    assert_eq!(layers.last().unwrap().print_z(), 1.25);
}

#[test]
fn filament_z_shrinkage_accepts_orca_percent_vector_forms() {
    for value in [
        json!(80),
        json!("80"),
        json!("80%"),
        json!("80%;100%"),
        json!("80,100"),
        json!([80]),
        json!(["80"]),
        json!(["80%"]),
    ] {
        let model = model_with_height(1.0);
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "filament_shrinkage_compensation_z": value
        }))
        .unwrap();

        let layers = plan_layers(&model, &options).unwrap();

        assert_eq!(layers.last().unwrap().print_z(), 1.2);
    }
}

#[test]
fn filament_z_shrinkage_rejects_invalid_orca_percent_values() {
    for value in [
        json!(0),
        json!(-1),
        json!(49),
        json!("49%"),
        json!(151),
        json!("151%"),
        json!("NaN"),
        json!("80%;"),
        json!([]),
        json!([80, "bad"]),
        json!(null),
    ] {
        let model = model_with_height(1.0);
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "filament_shrinkage_compensation_z": value
        }))
        .unwrap();

        let err = plan_layers(&model, &options).unwrap_err();

        assert!(
            matches!(err, SliceError::InvalidInput(message) if message.contains("filament_shrinkage_compensation_z"))
        );
    }
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
