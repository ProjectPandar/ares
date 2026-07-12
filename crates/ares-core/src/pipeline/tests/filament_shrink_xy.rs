use super::*;

#[test]
fn filament_shrink_xy_scales_pipeline_model_bounds_before_slicing() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "filament_shrink": "80%",
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();
    let bounds = pipeline.model().xy_bounds().unwrap();
    let z_bounds = pipeline.model().z_bounds().unwrap();

    assert_eq!(bounds.min_x, -1.25);
    assert_eq!(bounds.max_x, 1.25);
    assert_eq!(bounds.min_y, -1.25);
    assert_eq!(bounds.max_y, 1.25);
    assert_eq!(z_bounds.min, 0.0);
    assert_eq!(z_bounds.max, 0.4);
}

#[test]
fn omitted_filament_shrink_preserves_pipeline_model_and_contour_bounds() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();
    let model_bounds = pipeline.model().xy_bounds().unwrap();
    let contour_bounds = first_contour_bounds(&pipeline);

    assert_eq!(
        (
            model_bounds.min_x,
            model_bounds.max_x,
            model_bounds.min_y,
            model_bounds.max_y
        ),
        (-1.0, 1.0, -1.0, 1.0)
    );
    assert_eq!(contour_bounds, (-0.5, 0.5, -0.5, 0.5));
}

#[test]
fn filament_shrink_xy_scales_first_layer_contours_before_perimeters() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "filament_shrink": 80,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();

    assert_eq!(
        first_contour_bounds(&pipeline),
        (-0.625, 0.625, -0.625, 0.625)
    );
}

#[test]
fn filament_shrink_xy_accepts_orca_percent_vector_forms() {
    for value in [
        json!("80"),
        json!("80%"),
        json!("80%;100%"),
        json!("80,100"),
        json!([80]),
        json!(["80"]),
        json!(["80%"]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "filament_shrink": value,
            "sparse_infill_density": 0,
            "skirt_loops": 0,
            "brim_width": 0.0
        }))
        .unwrap();

        let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();

        assert_eq!(
            first_contour_bounds(&pipeline),
            (-0.625, 0.625, -0.625, 0.625)
        );
    }
}

#[test]
fn filament_shrink_xy_rejects_invalid_percent_values() {
    for value in [
        json!(0),
        json!(49),
        json!(151),
        json!("NaN"),
        json!("80%;"),
        json!([]),
        json!([80, "bad"]),
        json!(null),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "filament_shrink": value
        }))
        .unwrap();

        let err = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap_err();

        assert!(
            matches!(err, SliceError::InvalidInput(message) if message.contains("filament_shrink"))
        );
    }
}

#[test]
fn filament_shrink_xy_preserves_model_z_for_existing_z_shrinkage_path() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "filament_shrink": "80%",
        "filament_shrinkage_compensation_z": "80%",
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(super::square_pyramid_ascii_stl(), &options).unwrap();
    let bounds = pipeline.model().xy_bounds().unwrap();
    let z_bounds = pipeline.model().z_bounds().unwrap();

    assert_eq!(
        (bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y),
        (-1.25, 1.25, -1.25, 1.25)
    );
    assert_eq!(z_bounds.min, 0.0);
    assert_eq!(z_bounds.max, 0.4);
}

fn first_contour_bounds(pipeline: &SlicingPipeline) -> (f64, f64, f64, f64) {
    let contour = &pipeline.layer_contours()[0].contours()[0];
    contour.points().iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), point| {
            (
                min_x.min(point.x()),
                max_x.max(point.x()),
                min_y.min(point.y()),
                max_y.max(point.y()),
            )
        },
    )
}
