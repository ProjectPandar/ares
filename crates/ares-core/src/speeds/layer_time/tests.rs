use super::super::*;
use crate::{ExtrusionMove, Point2};

const EPS: f64 = 0.000001;

#[test]
fn disabled_layer_cooling_keeps_capped_speeds() {
    let output = generate_speed_moves(
        &[line_layer(vec![travel(0.0), external(10.0, 1.0)])],
        layer_time_options(100.0, 50.0, 50.0, 20.0, 1.0).with_slow_down_for_layer_cooling(false),
    );

    assert_eq!(output[0].moves()[1].speed_mm_s(), 100.0);
}

#[test]
fn slows_adjustable_prints_to_reach_layer_time() {
    let output = generate_speed_moves(
        &[line_layer(vec![travel(0.0), external(10.0, 1.0)])],
        layer_time_options(100.0, 50.0, 50.0, 2.0, 1.0),
    );

    assert_close(output[0].moves()[1].speed_mm_s(), 10.0 / 2.002);
}

#[test]
fn respects_slow_down_min_speed_when_target_is_unreachable() {
    let output = generate_speed_moves(
        &[line_layer(vec![travel(0.0), external(10.0, 1.0)])],
        layer_time_options(100.0, 50.0, 50.0, 20.0, 10.0),
    );

    assert_eq!(output[0].moves()[1].speed_mm_s(), 10.0);
}

#[test]
fn keeps_speeds_when_layer_already_meets_threshold() {
    let output = generate_speed_moves(
        &[line_layer(vec![travel(0.0), external(10.0, 1.0)])],
        layer_time_options(100.0, 50.0, 50.0, 0.05, 1.0),
    );

    assert_eq!(output[0].moves()[1].speed_mm_s(), 100.0);
}

#[test]
fn non_adjustable_prints_contribute_time_without_being_slowed() {
    let output = generate_speed_moves(
        &[line_layer(vec![
            travel(0.0),
            external(10.0, 1.0),
            internal(20.0, 2.0),
        ])],
        layer_time_options(100.0, 100.0, 50.0, 1.0, 1.0).with_dont_slow_down_outer_wall(true),
    );

    assert_eq!(output[0].moves()[1].speed_mm_s(), 100.0);
    assert_close(output[0].moves()[2].speed_mm_s(), 10.0 / 0.901);
}

#[test]
fn fastest_adjustable_moves_are_lowered_before_slower_moves() {
    let output = generate_speed_moves(
        &[line_layer(vec![
            travel(0.0),
            external(10.0, 1.0),
            internal(20.0, 2.0),
        ])],
        layer_time_options(100.0, 50.0, 50.0, 0.35, 1.0),
    );

    assert_close(output[0].moves()[1].speed_mm_s(), 10.0 / 0.15035);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 50.0);
}

#[test]
fn pre_first_extrusion_travel_updates_position_and_later_travel_contributes_time() {
    let output = generate_speed_moves(
        &[line_layer(vec![
            travel(100.0),
            external(110.0, 1.0),
            travel(200.0),
        ])],
        SpeedOptions::new(90.0, 100.0, 50.0)
            .with_slow_down_for_layer_cooling(true)
            .with_slow_down_layer_time(1.5)
            .with_slow_down_min_speed(1.0),
    );

    assert_eq!(output[0].moves()[0].speed_mm_s(), 90.0);
    assert_close(output[0].moves()[1].speed_mm_s(), 10.0 / 0.5015);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 90.0);
}

#[test]
fn zero_min_speed_uses_unlimited_proportional_slowdown() {
    let output = generate_speed_moves(
        &[line_layer(vec![
            travel(0.0),
            external(10.0, 1.0),
            internal(20.0, 2.0),
        ])],
        layer_time_options(100.0, 50.0, 50.0, 0.6, 0.0),
    );

    let factor = 0.6006 / 0.3;
    assert_close(output[0].moves()[1].speed_mm_s(), 100.0 / factor);
    assert_close(output[0].moves()[2].speed_mm_s(), 50.0 / factor);
}

fn layer_time_options(
    external_speed: f64,
    internal_speed: f64,
    sparse_speed: f64,
    layer_time: f64,
    min_speed: f64,
) -> SpeedOptions {
    SpeedOptions::new(120.0, external_speed, sparse_speed)
        .with_internal_perimeter_speed(internal_speed)
        .with_slow_down_for_layer_cooling(true)
        .with_slow_down_layer_time(layer_time)
        .with_slow_down_min_speed(min_speed)
}

fn line_layer(moves: Vec<ExtrusionMove>) -> LayerExtrusionMoves {
    LayerExtrusionMoves::new(1, 0.4, moves, 0.0)
}

fn travel(x: f64) -> ExtrusionMove {
    ExtrusionMove::new(
        ToolpathMoveKind::Travel,
        PrintPathRole::ExternalPerimeter,
        Point2::new(x, 0.0),
        None,
    )
}

fn external(x: f64, e: f64) -> ExtrusionMove {
    print(PrintPathRole::ExternalPerimeter, x, e)
}

fn internal(x: f64, e: f64) -> ExtrusionMove {
    print(PrintPathRole::InternalPerimeter, x, e)
}

fn print(role: PrintPathRole, x: f64, e: f64) -> ExtrusionMove {
    ExtrusionMove::new(ToolpathMoveKind::Print, role, Point2::new(x, 0.0), Some(e))
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= EPS,
        "expected {actual} to be within {EPS} of {expected}"
    );
}
