use serde_json::json;

use super::super::{ScaledPoint, printable_filament_changed_staged};
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
        "extruder_printable_area": [[[0.25, 0.25]], [[0.75, 0.75]]]
    }))
}

#[test]
fn staged_printable_filament_equal_polygons_skip_geometry_callbacks() {
    let old_poly = square(0.0);

    let changed = printable_filament_changed_staged(
        &serde_json::Map::new(),
        (&old_poly, &old_poly),
        |_, _| panic!("diff callback must not run for equal polygons"),
        |_, _| panic!("all-intersection callback must not run for equal polygons"),
        |_, _| panic!("intersection callback must not run for equal polygons"),
    )
    .expect("equal polygons should return false before geometry branch");

    assert!(!changed);
}

#[test]
fn staged_printable_filament_manual_mode_skips_geometry_callbacks() {
    let config = map(json!({ "filament_map_mode": "fmmManual" }));

    let changed = printable_filament_changed_staged(
        &config,
        (&square(0.0), &square(1.0)),
        |_, _| panic!("diff callback must not run for manual mode"),
        |_, _| panic!("all-intersection callback must not run for manual mode"),
        |_, _| panic!("intersection callback must not run for manual mode"),
    )
    .expect("manual mode should return false before geometry branch");

    assert!(!changed);
}

#[test]
fn staged_printable_filament_geometry_branch_returns_true_for_different_ids() {
    let old_poly = square(0.0);
    let new_poly = square(1.0);
    let mut diff_calls = Vec::new();
    let mut intersection_calls = Vec::new();

    let changed = printable_filament_changed_staged(
        &staged_config(),
        (&old_poly, &new_poly),
        |subject, clip| {
            diff_calls.push((subject[0], clip[0]));
            vec![vec![clip[0]]]
        },
        |subject, clips| {
            assert_eq!(subject.len(), 1);
            assert_eq!(clips.len(), 2);
            vec![vec![ScaledPoint { x: 99, y: 99 }]]
        },
        |subject, contour| {
            intersection_calls.push((subject[0], contour[0]));
            if (subject[0] == ScaledPoint { x: 0, y: 0 } && contour[0].x == 250_000)
                || (subject[0] == ScaledPoint { x: 1_000_000, y: 0 } && contour[0].x == 99)
            {
                vec![vec![contour[0]]]
            } else {
                Vec::new()
            }
        },
    )
    .expect("staged geometry branch should run");

    assert!(changed);
    assert_eq!(diff_calls.len(), 2);
    assert_eq!(intersection_calls[0].0, ScaledPoint { x: 0, y: 0 });
    assert_eq!(intersection_calls[0].1.x, 250_000);
    assert_eq!(intersection_calls[1].0, ScaledPoint { x: 0, y: 0 });
    assert_eq!(intersection_calls[2].0, ScaledPoint { x: 0, y: 0 });
    assert_eq!(intersection_calls[3].0, ScaledPoint { x: 1_000_000, y: 0 });
}

#[test]
fn staged_printable_filament_geometry_branch_returns_false_for_equal_ids() {
    let changed = printable_filament_changed_staged(
        &staged_config(),
        (&square(0.0), &square(1.0)),
        |_, clip| vec![vec![clip[0]]],
        |_, _| Vec::new(),
        |_, contour| {
            if contour[0].x == 250_000 {
                vec![vec![contour[0]]]
            } else {
                Vec::new()
            }
        },
    )
    .expect("staged geometry branch should run");

    assert!(!changed);
}
