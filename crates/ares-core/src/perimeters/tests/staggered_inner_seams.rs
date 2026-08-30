use super::*;

#[test]
fn staggered_inner_seams_defaults_false_parses_bool_and_rejects_invalid() {
    let default = SliceOptions::default().perimeter_options().unwrap();
    assert!(!default.staggered_inner_seams());

    let enabled: SliceOptions =
        serde_json::from_value(json!({ "staggered_inner_seams": true })).unwrap();
    assert!(enabled.perimeter_options().unwrap().staggered_inner_seams());

    for value in [json!("true"), json!(1), json!(null)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "staggered_inner_seams": value })).unwrap();
        let err = options.perimeter_options().unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("staggered_inner_seams"));
    }
}

#[test]
fn staggered_inner_seams_shifts_internal_back_seam_without_moving_external_loop() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = PerimeterOptions::new(
        2,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
    .with_seam_position(SeamPosition::Back)
    .with_staggered_inner_seams(true);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(
        perimeters[0].paths()[0].points(),
        &[
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
        ]
    );
    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[1].points(),
        &[
            Point2::new(3.24292036732051, 3.64292036732051),
            Point2::new(0.3570796326794897, 3.64292036732051),
            Point2::new(0.3570796326794897, 0.3570796326794897),
            Point2::new(3.64292036732051, 0.3570796326794897),
            Point2::new(3.64292036732051, 3.64292036732051),
        ]
    );
}
