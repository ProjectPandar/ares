use super::{FillConnectionParams, connect_infill};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, Polyline};

mod decisions;
mod oracles;
mod orca_vectors;
mod ordering;
mod scale_errors;

const KSR_SPACING: f64 = 0.407_079_637_050_628_66;
const KSR_ANCHOR: f32 = f32::from_bits(0x3fd0_6cbe);

fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

#[test]
fn task22o44_distinct_single_hook_matches_pinned_orca() {
    let boundary = ExPolygon::new(
        Polygon::new(vec![
            point(0, 0),
            point(12_000_000, 0),
            point(12_000_000, 8_000_000),
            point(0, 8_000_000),
        ]),
        Vec::new(),
    );
    let before = boundary.clone();
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(0, 2_000_000),
            point(12_000_000, 3_000_000),
        ])],
        &boundary,
        KSR_SPACING,
        FillConnectionParams {
            anchor_length: KSR_ANCHOR,
            anchor_length_max: 20.0,
            multiline: 1,
            dont_sort: false,
        },
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        vec![Polyline::new(vec![
            point(0, 3_628_318),
            point(0, 2_000_000),
            point(12_000_000, 3_000_000),
            point(12_000_000, 4_628_318),
        ])]
    );
    assert_eq!(boundary, before);
}
