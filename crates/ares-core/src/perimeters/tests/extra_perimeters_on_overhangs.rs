use super::*;

fn unsupported_layers() -> [LayerContours; 2] {
    [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(10.0, 0.0, 14.0, 4.0)]),
    ]
}

fn base_options(wall_loops: u32) -> PerimeterOptions {
    PerimeterOptions::new(
        wall_loops,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
}

#[test]
fn extra_perimeters_on_overhangs_adds_one_inset_overhang_loop() {
    let options = base_options(1).with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(perimeters[1].paths().len(), 2);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
    assert_eq!(perimeters[1].paths()[1].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[1].points(),
        rectangle_points(
            10.35707963267949,
            0.3570796326794897,
            13.64292036732051,
            3.642_920_367_320_51
        )
    );
}

#[test]
fn disabled_extra_perimeters_on_overhangs_preserves_single_overhang_loop() {
    let perimeters = generate_perimeters(&unsupported_layers(), base_options(1)).unwrap();

    assert_eq!(perimeters[1].paths().len(), 1);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
}

#[test]
fn disabled_overhang_detection_blocks_extra_overhang_perimeters() {
    let options = base_options(1)
        .with_detect_overhang_wall(false)
        .with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(perimeters[1].paths().len(), 1);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn zero_wall_loops_blocks_extra_overhang_perimeters() {
    let options = base_options(0).with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert!(perimeters[1].paths().is_empty());
}

#[test]
fn extra_overhang_loop_is_inside_configured_internal_walls() {
    let options = base_options(2).with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(perimeters[1].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[1].paths()[1].points(),
        rectangle_points(
            10.35707963267949,
            0.3570796326794897,
            13.64292036732051,
            3.642_920_367_320_51
        )
    );
    assert_eq!(perimeters[1].paths()[2].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[2].points(),
        rectangle_points(
            10.71415926535898,
            0.7141592653589793,
            13.28584073464102,
            3.2858407346410207
        )
    );
}

#[test]
fn extra_overhang_loop_uses_next_loop_spacing() {
    let options = PerimeterOptions::new(
        2,
        0.5,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
    .with_layer_height_mm(0.2)
    .with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(
        perimeters[1].paths()[2].points(),
        rectangle_points(
            10.76415926535898,
            0.7641592653589793,
            13.23584073464102,
            3.2358407346410207
        )
    );
}

#[test]
fn extra_overhang_loop_respects_clockwise_wall_direction() {
    let options = PerimeterOptions::new(
        1,
        0.4,
        0.4,
        WallDirection::Clockwise,
        WallSequence::OuterInner,
    )
    .with_extra_perimeters_on_overhangs(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(
        perimeters[1].paths()[1].points(),
        &[
            Point2::new(10.35707963267949, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 0.3570796326794897),
            Point2::new(10.35707963267949, 0.3570796326794897),
        ]
    );
}

#[test]
fn overhang_reverse_reverses_added_extra_overhang_loop() {
    let options = base_options(1)
        .with_extra_perimeters_on_overhangs(true)
        .with_overhang_reverse(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(
        perimeters[1].paths()[0].points(),
        &[
            Point2::new(10.0, 4.0),
            Point2::new(14.0, 4.0),
            Point2::new(14.0, 0.0),
            Point2::new(10.0, 0.0),
        ]
    );
    assert_eq!(
        perimeters[1].paths()[1].points(),
        &[
            Point2::new(10.35707963267949, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 0.3570796326794897),
            Point2::new(10.35707963267949, 0.3570796326794897),
        ]
    );
}

#[test]
fn overhang_reverse_internal_only_preserves_external_and_reverses_added_extra_loop() {
    let options = base_options(1)
        .with_extra_perimeters_on_overhangs(true)
        .with_overhang_reverse(true)
        .with_overhang_reverse_internal_only(true);

    let perimeters = generate_perimeters(&unsupported_layers(), options).unwrap();

    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
    assert_eq!(
        perimeters[1].paths()[1].points(),
        &[
            Point2::new(10.35707963267949, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 3.642_920_367_320_51),
            Point2::new(13.64292036732051, 0.3570796326794897),
            Point2::new(10.35707963267949, 0.3570796326794897),
        ]
    );
}
