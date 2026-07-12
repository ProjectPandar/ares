use serde_json::json;

use super::super::instance_sync_state::{
    StagedInstanceApplyState, StagedPrintStep, sync_changed_instance_printable_filament_staged,
};
use super::super::{PrintableFilamentGeometryOps, ScaledPoint};
use crate::Point2;

fn map(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .expect("test value must be object")
        .clone()
}

fn square(offset: f64) -> Vec<Point2> {
    vec![
        Point2::new(offset, 0.0),
        Point2::new(offset + 1.0, 0.0),
        Point2::new(offset + 1.0, 1.0),
        Point2::new(offset, 1.0),
    ]
}

fn staged_config() -> serde_json::Map<String, serde_json::Value> {
    map(json!({
        "printable_area": [[0, 0], [1, 0], [1, 1]],
        "extruder_printable_area": [[[0.25, 0.25]]]
    }))
}

fn staged_instance(
    transform: i32,
    printable: bool,
    print_volume_state: i32,
    convex_hull: Vec<Point2>,
) -> StagedInstanceApplyState {
    StagedInstanceApplyState {
        convex_hull,
        transform,
        print_volume_state,
        printable,
    }
}

fn first_diff_result(_: &[ScaledPoint], clip: &[ScaledPoint]) -> Vec<Vec<ScaledPoint>> {
    vec![vec![clip[0]]]
}

fn no_diff(_: &[ScaledPoint], _: &[ScaledPoint]) -> Vec<Vec<ScaledPoint>> {
    Vec::new()
}

fn no_all_intersection(_: &[Vec<ScaledPoint>], _: &[Vec<ScaledPoint>]) -> Vec<Vec<ScaledPoint>> {
    Vec::new()
}

fn no_intersection(_: &[ScaledPoint], _: &[ScaledPoint]) -> Vec<Vec<ScaledPoint>> {
    Vec::new()
}

#[test]
fn instance_sync_returns_wipe_tower_and_gcode_export_when_printable_filament_changes() {
    let mut old = staged_instance(1, false, 10, square(0.0));
    let new = staged_instance(2, true, 20, square(1.0));
    let mut first_intersection_call = None;

    let steps = sync_changed_instance_printable_filament_staged(
        &mut old,
        &new,
        &staged_config(),
        PrintableFilamentGeometryOps {
            diff: first_diff_result,
            all_intersection: no_all_intersection,
            intersection: |subject: &[ScaledPoint], contour: &[ScaledPoint]| {
                first_intersection_call.get_or_insert((subject[0], contour[0]));
                if subject[0] == (ScaledPoint { x: 0, y: 0 }) && contour[0].x == 250_000 {
                    vec![vec![contour[0]]]
                } else {
                    Vec::new()
                }
            },
        },
    )
    .expect("instance sync should succeed");

    assert_eq!(
        steps,
        vec![StagedPrintStep::WipeTower, StagedPrintStep::GCodeExport]
    );
    assert_eq!(
        first_intersection_call,
        Some((
            ScaledPoint { x: 0, y: 0 },
            ScaledPoint {
                x: 250_000,
                y: 250_000
            }
        ))
    );
    assert_eq!(old.transform, 2);
    assert!(old.printable);
    assert_eq!(old.print_volume_state, 20);
}

#[test]
fn instance_sync_returns_no_steps_but_still_copies_fields_when_predicate_is_false() {
    let mut old = staged_instance(1, false, 10, square(0.0));
    let new = staged_instance(2, true, 20, square(1.0));

    let steps = sync_changed_instance_printable_filament_staged(
        &mut old,
        &new,
        &staged_config(),
        PrintableFilamentGeometryOps {
            diff: first_diff_result,
            all_intersection: no_all_intersection,
            intersection: no_intersection,
        },
    )
    .expect("instance sync should succeed");

    assert!(steps.is_empty());
    assert_eq!(old.transform, 2);
    assert!(old.printable);
    assert_eq!(old.print_volume_state, 20);
}

#[test]
fn instance_sync_propagates_predicate_errors_without_copying_fields() {
    let mut old = staged_instance(1, false, 10, square(0.0));
    let new = staged_instance(2, true, 20, square(1.0));
    let config = map(json!({ "printable_area": "bad" }));

    let err = sync_changed_instance_printable_filament_staged(
        &mut old,
        &new,
        &config,
        PrintableFilamentGeometryOps {
            diff: no_diff,
            all_intersection: no_all_intersection,
            intersection: no_intersection,
        },
    )
    .expect_err("bad printable_area should propagate");

    assert_eq!(
        err.to_string(),
        "printable_area must be an array of [x,y] points"
    );
    assert_eq!(old.transform, 1);
    assert!(!old.printable);
    assert_eq!(old.print_volume_state, 10);
}
