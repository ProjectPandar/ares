use super::*;

#[test]
fn default_seam_position_is_aligned() {
    let options = SliceOptions::default().perimeter_options().unwrap();

    assert_eq!(options.seam_position(), SeamPosition::Aligned);
}

#[test]
fn invalid_seam_position_is_rejected() {
    let options: SliceOptions = serde_json::from_value(json!({
        "seam_position": "front"
    }))
    .unwrap();

    let err = options.perimeter_options().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("seam_position"));
}

#[test]
fn back_seam_position_rotates_external_perimeter_to_rear_vertex() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = PerimeterOptions::new(
        1,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
    .with_seam_position(SeamPosition::Back);

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
}

#[test]
fn back_seam_position_rotates_internal_perimeters_to_rear_vertex() {
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
    .with_seam_position(SeamPosition::Back);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[1].points(),
        &[
            Point2::new(3.64292036732051, 3.64292036732051),
            Point2::new(0.3570796326794897, 3.64292036732051),
            Point2::new(0.3570796326794897, 0.3570796326794897),
            Point2::new(3.64292036732051, 0.3570796326794897),
        ]
    );
}
