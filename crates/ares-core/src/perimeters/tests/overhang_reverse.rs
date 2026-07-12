use super::*;
use crate::{LayerContours, Point2};

fn options(overhang_reverse: bool, wall_direction: WallDirection) -> PerimeterOptions {
    PerimeterOptions::new(1, 0.4, 0.4, wall_direction, WallSequence::OuterInner)
        .with_detect_overhang_wall(true)
        .with_overhang_reverse(overhang_reverse)
}

fn options_with_internal_only(
    overhang_reverse: bool,
    overhang_reverse_internal_only: bool,
    wall_direction: WallDirection,
) -> PerimeterOptions {
    PerimeterOptions::new(2, 0.4, 0.4, wall_direction, WallSequence::OuterInner)
        .with_detect_overhang_wall(true)
        .with_overhang_reverse(overhang_reverse)
        .with_overhang_reverse_internal_only(overhang_reverse_internal_only)
}

#[test]
fn disabled_overhang_reverse_preserves_unsupported_second_layer_order() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options(false, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
}

#[test]
fn enabled_overhang_reverse_reverses_unsupported_second_layer_on_odd_zero_based_layer() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options(true, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        &[
            Point2::new(10.0, 4.0),
            Point2::new(14.0, 4.0),
            Point2::new(14.0, 0.0),
            Point2::new(10.0, 0.0),
        ]
    );
}

#[test]
fn enabled_overhang_reverse_does_not_reverse_supported_second_layer() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];

    let perimeters =
        generate_perimeters(&layers, options(true, WallDirection::CounterClockwise)).unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::External);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(0.0, 0.0, 4.0, 4.0)
    );
}

#[test]
fn enabled_overhang_reverse_does_not_reverse_even_zero_based_layer() {
    let perimeters = generate_perimeters(
        &unsupported_layers(2),
        options(true, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(perimeters[2].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[2].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
}

#[test]
fn overhang_reverse_flips_after_clockwise_wall_direction() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options(true, WallDirection::Clockwise),
    )
    .unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
}

#[test]
fn overhang_reverse_reverses_external_and_internal_paths_when_internal_only_is_disabled() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options_with_internal_only(true, false, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        &[
            Point2::new(10.0, 4.0),
            Point2::new(14.0, 4.0),
            Point2::new(14.0, 0.0),
            Point2::new(10.0, 0.0),
        ]
    );
    assert_eq!(perimeters[1].paths()[1].role(), PerimeterRole::Internal);
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
fn overhang_reverse_internal_only_preserves_external_and_reverses_internal_path() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options_with_internal_only(true, true, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
    assert_eq!(perimeters[1].paths()[1].role(), PerimeterRole::Internal);
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
fn overhang_reverse_internal_only_is_inert_without_overhang_reverse() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options_with_internal_only(false, true, WallDirection::CounterClockwise),
    )
    .unwrap();

    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(10.0, 0.0, 14.0, 4.0)
    );
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
fn overhang_reverse_internal_only_flips_internal_after_clockwise_wall_direction() {
    let perimeters = generate_perimeters(
        &unsupported_layers(1),
        options_with_internal_only(true, true, WallDirection::Clockwise),
    )
    .unwrap();

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
        rectangle_points(
            10.35707963267949,
            0.3570796326794897,
            13.64292036732051,
            3.642_920_367_320_51
        )
    );
}

fn unsupported_layers(target_layer_id: usize) -> Vec<LayerContours> {
    let mut layers = Vec::new();
    layers.push(LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    ));
    for layer_id in 1..target_layer_id {
        layers.push(LayerContours::new(
            layer_id,
            0.2 * (layer_id as f64 + 1.0),
            Vec::new(),
        ));
    }
    layers.push(LayerContours::new(
        target_layer_id,
        0.2 * (target_layer_id as f64 + 1.0),
        vec![rectangle(10.0, 0.0, 14.0, 4.0)],
    ));
    layers
}
