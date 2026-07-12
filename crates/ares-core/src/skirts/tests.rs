use super::*;
use crate::{Contour, LayerContours, Point2};

mod brim_envelope;
mod per_object;

#[test]
fn generates_rectangular_skirt_loop_around_contour_bounds() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(-0.5, 0.0),
            Point2::new(0.0, -0.5),
            Point2::new(0.5, 0.0),
            Point2::new(0.0, 0.5),
        ])],
    )];
    let output = generate_skirts(&contours, SkirtOptions::new(1, 2.0, 1, 50.0), 0.45, 1.0).unwrap();
    assert_eq!(output[0].layer_id(), 0);
    assert_eq!(output[0].print_z(), 0.2);
    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-2.5, -2.5),
            Point2::new(2.5, -2.5),
            Point2::new(2.5, 2.5),
            Point2::new(-2.5, 2.5),
        ]
    );
}

#[test]
fn honors_skirt_loop_count_height_and_empty_layers() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, Vec::new()),
    ];
    let output = generate_skirts(&contours, SkirtOptions::new(2, 1.0, 1, 50.0), 0.4, 1.0).unwrap();
    assert_eq!(output.len(), 3);
    assert_eq!(output[0].layer_id(), 0);
    assert_eq!(output[1].layer_id(), 1);
    assert_eq!(output[2].layer_id(), 2);
    assert_eq!(output[0].paths().len(), 2);
    assert!(output[1].paths().is_empty());
    assert!(output[2].paths().is_empty());
}

#[test]
fn enabled_draft_shield_generates_configured_loops_on_all_non_empty_layers() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, Vec::new()),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(2, 1.0, 1, 50.0).with_draft_shield(DraftShield::Enabled),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output.len(), 3);
    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(output[1].paths().len(), 2);
    assert!(output[2].paths().is_empty());
}

#[test]
fn enabled_draft_shield_with_zero_loops_uses_one_effective_loop() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, Vec::new()),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(0, 1.0, 1, 50.0).with_draft_shield(DraftShield::Enabled),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(output[1].paths().len(), 1);
    assert!(output[2].paths().is_empty());
}

#[test]
fn single_loop_draft_shield_defaults_to_disabled() {
    assert!(!SkirtOptions::new(1, 1.0, 1, 50.0).single_loop_draft_shield());
}

#[test]
fn single_loop_draft_shield_uses_outer_configured_loop_after_first_generated_layer() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(2, 1.0, 1, 50.0)
            .with_draft_shield(DraftShield::Enabled)
            .with_single_loop_draft_shield(true),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(output[1].paths().len(), 1);
    assert_eq!(
        output[1].paths()[0].points(),
        &[
            Point2::new(-1.4, -1.4),
            Point2::new(2.4, -1.4),
            Point2::new(2.4, 2.4),
            Point2::new(-1.4, 2.4),
        ]
    );
}

#[test]
fn single_loop_draft_shield_treats_first_non_empty_output_as_first_generated_layer() {
    let contours = vec![
        LayerContours::new(0, 0.2, Vec::new()),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(2, 1.0, 1, 50.0)
            .with_draft_shield(DraftShield::Enabled)
            .with_single_loop_draft_shield(true),
        0.4,
        1.0,
    )
    .unwrap();

    assert!(output[0].paths().is_empty());
    assert_eq!(output[1].paths().len(), 2);
    assert_eq!(output[2].paths().len(), 1);
    assert_eq!(
        output[2].paths()[0].points(),
        &[
            Point2::new(-1.4, -1.4),
            Point2::new(2.4, -1.4),
            Point2::new(2.4, 2.4),
            Point2::new(-1.4, 2.4),
        ]
    );
}

#[test]
fn single_loop_draft_shield_keeps_min_length_on_first_generated_layer_only() {
    let contours = vec![
        LayerContours::new(0, 0.2, Vec::new()),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0)
            .with_draft_shield(DraftShield::Enabled)
            .with_single_loop_draft_shield(true)
            .with_min_skirt_length_mm(50.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert!(output[0].paths().is_empty());
    assert!(output[1].paths().len() > 1);
    assert_eq!(output[2].paths().len(), 1);
    assert_eq!(
        output[2].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
        ]
    );
}

#[test]
fn skirt_start_angle_defaults_to_orca_default() {
    assert_eq!(
        SkirtOptions::new(1, 1.0, 1, 50.0).skirt_start_angle_degrees(),
        -135.0
    );
}

#[test]
fn skirt_start_angle_45_starts_first_path_at_upper_right_corner() {
    let contours = vec![LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_start_angle_degrees(45.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
        ]
    );
}

#[test]
fn skirt_start_angle_zero_starts_first_path_at_lower_right_corner() {
    let contours = vec![LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_start_angle_degrees(0.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
            Point2::new(-1.0, -1.0),
        ]
    );
}

#[test]
fn skirt_start_angle_reorders_only_first_path_on_first_generated_layer() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(2, 1.0, 1, 50.0)
            .with_draft_shield(DraftShield::Enabled)
            .with_skirt_start_angle_degrees(45.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(output[0].paths()[0].points()[0], Point2::new(2.0, 2.0));
    assert_eq!(output[0].paths()[1].points()[0], Point2::new(-1.4, -1.4));
    assert_eq!(output[1].paths()[0].points()[0], Point2::new(-1.0, -1.0));
}

#[test]
fn skirt_start_angle_does_not_reorder_later_single_loop_draft_shield_path() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(2, 1.0, 1, 50.0)
            .with_draft_shield(DraftShield::Enabled)
            .with_single_loop_draft_shield(true)
            .with_skirt_start_angle_degrees(45.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths()[0].points()[0], Point2::new(2.0, 2.0));
    assert_eq!(output[1].paths().len(), 1);
    assert_eq!(output[1].paths()[0].points()[0], Point2::new(-1.4, -1.4));
}

#[test]
fn uses_layer_id_for_skirt_height() {
    let contours = vec![
        LayerContours::new(7, 1.4, vec![unit_square_contour()]),
        LayerContours::new(9, 1.8, vec![unit_square_contour()]),
    ];

    let output = generate_skirts(&contours, SkirtOptions::new(1, 1.0, 1, 50.0), 0.4, 1.0).unwrap();

    assert_eq!(output[0].layer_id(), 7);
    assert_eq!(output[1].layer_id(), 9);
    assert!(output[0].paths().is_empty());
    assert!(output[1].paths().is_empty());
}

#[test]
fn combined_skirt_type_preserves_combined_layer_skirts() {
    let contours = vec![LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_type(SkirtType::Combined),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
        ]
    );
}

fn unit_square_contour() -> Contour {
    Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    ])
}
