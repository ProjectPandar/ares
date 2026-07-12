use super::*;

#[test]
fn inner_outer_wall_sequence_prints_internal_walls_before_external() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.7570796326794897,
            0.7570796326794897,
            3.2429203673205103,
            3.2429203673205103
        )
    );
    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(0.4, 0.4, 3.6, 3.6)
    );
    assert_eq!(perimeters[0].paths()[2].role(), PerimeterRole::External);
}

#[test]
fn outer_inner_wall_sequence_prints_external_wall_first() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::OuterInner,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(
            0.3570796326794897,
            0.3570796326794897,
            3.642_920_367_320_51,
            3.642_920_367_320_51
        )
    );
}

#[test]
fn inner_outer_inner_wall_sequence_uses_inner_outer_on_first_layer() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuterInner,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.7141592653589793,
            0.7141592653589793,
            3.2858407346410207,
            3.2858407346410207
        )
    );
    assert_eq!(perimeters[0].paths()[2].role(), PerimeterRole::External);
}

#[test]
fn inner_outer_inner_wall_sequence_sandwiches_outer_wall_after_first_layer() {
    let layers = [LayerContours::new(
        1,
        0.4,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuterInner,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.7141592653589793,
            0.7141592653589793,
            3.2858407346410207,
            3.2858407346410207
        )
    );
    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::External);
    assert_eq!(perimeters[0].paths()[2].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[2].points(),
        rectangle_points(
            0.3570796326794897,
            0.3570796326794897,
            3.642_920_367_320_51,
            3.642_920_367_320_51
        )
    );
}

#[test]
fn non_inner_outer_sequences_ignore_precise_outer_wall_and_use_spacing_branch() {
    for sequence in [WallSequence::OuterInner, WallSequence::InnerOuterInner] {
        let layers = [LayerContours::new(
            1,
            0.4,
            vec![rectangle(0.0, 0.0, 6.0, 6.0)],
        )];
        let enabled = generate_perimeters(
            &layers,
            PerimeterOptions::new(2, 0.5, 0.4, WallDirection::CounterClockwise, sequence)
                .with_precise_outer_wall(true)
                .with_layer_height_mm(0.2),
        )
        .unwrap();
        let disabled = generate_perimeters(
            &layers,
            PerimeterOptions::new(2, 0.5, 0.4, WallDirection::CounterClockwise, sequence)
                .with_precise_outer_wall(false)
                .with_layer_height_mm(0.2),
        )
        .unwrap();

        assert_eq!(enabled, disabled);
        assert!(enabled[0].paths().iter().any(|path| path.points()
            == rectangle_points(
                0.4070796326794897,
                0.4070796326794897,
                5.59292036732051,
                5.59292036732051
            )));
    }
}
