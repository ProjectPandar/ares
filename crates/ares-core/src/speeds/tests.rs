use super::*;
use crate::{ExtrusionMove, Point2};

mod acceleration;
mod ironing;
mod jerk;
mod overhang;
mod small_perimeter;
mod solid_infill;
mod top_bottom_solid_surface;
mod volumetric_rate_smoothing;

#[test]
fn assigns_feedrates_by_move_kind_and_role() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(1.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 1.0),
                Some(0.2),
            ),
        ],
        0.2,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].layer_id(), 1);
    assert_eq!(output[0].print_z(), 0.2);
    assert_eq!(output[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(output[0].moves()[1].kind(), ToolpathMoveKind::Print);
    assert_eq!(
        output[0].moves()[1].role(),
        PrintPathRole::ExternalPerimeter
    );
    assert_eq!(output[0].moves()[2].role(), PrintPathRole::SparseInfill);
    assert_eq!(output[0].moves()[2].point(), Point2::new(1.0, 1.0));
    assert_eq!(output[0].moves()[0].speed_mm_s(), 120.0);
    assert_eq!(output[0].moves()[1].speed_mm_s(), 60.0);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 100.0);
    assert_eq!(output[0].moves()[0].acceleration_mm_s2(), Some(10000.0));
    assert_eq!(output[0].moves()[1].acceleration_mm_s2(), Some(500.0));
    assert_eq!(output[0].moves()[2].acceleration_mm_s2(), Some(500.0));
    assert_eq!(output[0].moves()[0].jerk_mm_s(), None);
    assert_eq!(output[0].moves()[1].jerk_mm_s(), None);
    assert_eq!(output[0].moves()[2].jerk_mm_s(), None);
    assert_eq!(output[0].moves()[0].feedrate_mm_min(), 7200.0);
    assert_eq!(output[0].moves()[1].feedrate_mm_min(), 3600.0);
    assert_eq!(output[0].moves()[2].feedrate_mm_min(), 6000.0);
    assert_eq!(output[0].moves()[1].e_position(), Some(0.1));
}

#[test]
fn preserves_empty_represented_layers_for_speeds() {
    let layers = [LayerExtrusionMoves::new(7, 1.4, Vec::new(), 0.0)];
    let options = SpeedOptions::new(120.0, 60.0, 100.0);
    let output = generate_speed_moves(&layers, options);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].layer_id(), 7);
    assert!(output[0].moves().is_empty());
}

#[test]
fn assigns_skirt_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::Skirt,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0).with_skirt_speed(50.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 50.0);
    assert_eq!(output[0].moves()[0].feedrate_mm_min(), 3000.0);
}

#[test]
fn assigns_bridge_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::Bridge,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    let output = generate_speed_moves(
        &layers,
        SpeedOptions::new(120.0, 60.0, 100.0).with_bridge_speed(25.0),
    );
    assert_eq!(output[0].moves()[0].speed_mm_s(), 25.0);
    assert_eq!(output[0].moves()[0].feedrate_mm_min(), 1500.0);
}

#[test]
fn assigns_internal_bridge_speed() {
    let layers = [LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::Bridge,
                Point2::new(1.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::InternalBridge,
                Point2::new(2.0, 0.0),
                Some(0.2),
            ),
        ],
        0.2,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_bridge_speed(25.0)
        .with_internal_bridge_speed(37.5);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 25.0);
    assert_eq!(output[0].moves()[1].speed_mm_s(), 37.5);
    assert_eq!(output[0].moves()[1].feedrate_mm_min(), 2250.0);
}

#[test]
fn assigns_internal_perimeter_speed() {
    let base = SpeedOptions::new(120.0, 60.0, 100.0);
    assert_eq!(
        base.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::InternalPerimeter),
        60.0
    );

    let configured = base.with_internal_perimeter_speed(35.0);
    assert_eq!(
        configured.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::InternalPerimeter),
        35.0
    );
}

#[test]
fn assigns_brim_external_perimeter_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::Brim,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0).with_skirt_speed(50.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 60.0);
    assert_eq!(output[0].moves()[0].feedrate_mm_min(), 3600.0);
}

#[test]
fn first_layer_speeds_apply_only_to_first_layer_supported_roles() {
    let layers = [
        LayerExtrusionMoves::new(
            0,
            0.2,
            vec![
                ExtrusionMove::new(
                    ToolpathMoveKind::Travel,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(0.0, 0.0),
                    None,
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(1.0, 0.0),
                    Some(0.1),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::InternalPerimeter,
                    Point2::new(2.0, 0.0),
                    Some(0.2),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::SparseInfill,
                    Point2::new(3.0, 0.0),
                    Some(0.3),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::Brim,
                    Point2::new(4.0, 0.0),
                    Some(0.4),
                ),
            ],
            0.4,
        ),
        LayerExtrusionMoves::new(
            1,
            0.4,
            vec![
                ExtrusionMove::new(
                    ToolpathMoveKind::Travel,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(0.0, 0.0),
                    None,
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(1.0, 0.0),
                    Some(0.5),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::SparseInfill,
                    Point2::new(2.0, 0.0),
                    Some(0.6),
                ),
            ],
            0.2,
        ),
    ];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_first_layer_speed(30.0)
        .with_first_layer_infill_speed(45.0)
        .with_first_layer_travel_speed(80.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 80.0);
    assert_eq!(output[0].moves()[1].speed_mm_s(), 30.0);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 30.0);
    assert_eq!(output[0].moves()[3].speed_mm_s(), 45.0);
    assert_eq!(output[0].moves()[4].speed_mm_s(), 30.0);
    assert_eq!(output[1].moves()[0].speed_mm_s(), 120.0);
    assert_eq!(output[1].moves()[1].speed_mm_s(), 60.0);
    assert_eq!(output[1].moves()[2].speed_mm_s(), 100.0);
}

#[test]
fn z_travel_feedrate_falls_back_to_layer_travel_speed_when_zero() {
    let options = SpeedOptions::new(120.0, 60.0, 100.0).with_first_layer_travel_speed(45.0);

    assert_eq!(options.travel_speed_z_mm_s(), 0.0);
    assert_eq!(options.z_travel_feedrate_for_layer(true), 2700.0);
    assert_eq!(options.z_travel_feedrate_for_layer(false), 7200.0);
}

#[test]
fn positive_z_travel_speed_overrides_all_z_travel_layers() {
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_first_layer_travel_speed(45.0)
        .with_travel_speed_z(25.0);

    assert_eq!(options.travel_speed_z_mm_s(), 25.0);
    assert_eq!(options.z_travel_feedrate_for_layer(true), 1500.0);
    assert_eq!(options.z_travel_feedrate_for_layer(false), 1500.0);
    assert_eq!(
        options.feedrate_for_layer(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            true
        ),
        2700.0
    );
    assert_eq!(
        options.feedrate_for_role(ToolpathMoveKind::Travel, PrintPathRole::ExternalPerimeter),
        7200.0
    );
}

#[test]
fn first_layer_speeds_do_not_override_skirt_or_bridge_roles() {
    let layers = [LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::Skirt,
                Point2::new(1.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::Bridge,
                Point2::new(2.0, 0.0),
                Some(0.2),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::InternalBridge,
                Point2::new(3.0, 0.0),
                Some(0.3),
            ),
        ],
        0.3,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_skirt_speed(50.0)
        .with_bridge_speed(25.0)
        .with_internal_bridge_speed(35.0)
        .with_first_layer_speed(30.0)
        .with_first_layer_infill_speed(45.0)
        .with_first_layer_travel_speed(80.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 50.0);
    assert_eq!(output[0].moves()[1].speed_mm_s(), 25.0);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 35.0);
}
