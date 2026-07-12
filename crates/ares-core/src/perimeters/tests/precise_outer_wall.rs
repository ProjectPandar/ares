use super::*;

#[test]
fn default_precise_outer_wall_keeps_first_internal_width_but_uses_spacing_for_later_loops() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 6.0, 6.0)],
    )];
    let options = PerimeterOptions::new(
        3,
        0.5,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_layer_height_mm(0.2);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.8070796326794897,
            0.8070796326794897,
            5.19292036732051,
            5.19292036732051
        )
    );
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(0.45, 0.45, 5.55, 5.55)
    );
}

#[test]
fn disabled_precise_outer_wall_uses_spacing_for_first_and_later_inner_outer_loops() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 6.0, 6.0)],
    )];
    let options = PerimeterOptions::new(
        3,
        0.5,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_precise_outer_wall(false)
    .with_layer_height_mm(0.2);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.7641592653589793,
            0.7641592653589793,
            5.235840734641021,
            5.235840734641021
        )
    );
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(
            0.4070796326794897,
            0.4070796326794897,
            5.59292036732051,
            5.59292036732051
        )
    );
}

#[test]
fn rejects_non_positive_rounded_rectangle_spacing() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 6.0, 6.0)],
    )];

    let err = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            2,
            0.1,
            0.1,
            WallDirection::CounterClockwise,
            WallSequence::OuterInner,
        )
        .with_layer_height_mm(1.0),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        SliceError::InvalidInput(message)
            if message.contains("perimeter spacing must be positive")
    ));
}
