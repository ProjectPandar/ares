use crate::{
    geometry::ExPolygon,
    project_slice::{
        group_fills, prepare_infill::combine_infill, region_slices::RegionSurfaceKind,
    },
};

use super::super::focused::fixture::*;

const LAYER: usize = 1;

#[test]
fn task22o73_priority_uses_ordered_donut_paths_and_accumulates_raw_empty_groups() {
    let mut graph = graph();
    let outer = rectangle(0, 0, 10_000_000, 10_000_000).contour().clone();
    let mut first_hole = rectangle(2_000_000, 2_000_000, 5_000_000, 8_000_000)
        .contour()
        .clone();
    let mut second_hole = rectangle(6_000_000, 2_000_000, 8_000_000, 8_000_000)
        .contour()
        .clone();
    first_hole.reverse();
    second_hole.reverse();
    let a = ExPolygon::new(outer, vec![first_hole.clone(), second_hole.clone()]);
    let mut b_hole = rectangle(3_500_000, 3_250_000, 4_000_000, 3_750_000)
        .contour()
        .clone();
    b_hole.reverse();
    let b = ExPolygon::new(
        rectangle(3_000_000, 3_000_000, 4_500_000, 4_000_000)
            .contour()
            .clone(),
        vec![b_hole],
    );
    let c = rectangle(10_000_001, 1_000_000, 10_000_006, 2_000_000);
    let d = rectangle(10_000_011, 1_000_000, 10_000_015, 2_000_000);
    let mut surfaces = [
        surface(RegionSurfaceKind::BottomBridge, a.clone(), 0),
        surface(RegionSurfaceKind::BottomBridge, b, 0),
        surface(RegionSurfaceKind::BottomBridge, c, 0),
        surface(RegionSurfaceKind::BottomBridge, d, 0),
    ];
    for (surface, angle) in surfaces.iter_mut().zip([3.0, 2.0, 1.0, 0.0]) {
        surface.set_bridge_angle(angle);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = surfaces.into_iter().collect();
    let before = graph_snapshot(&graph);

    let first = group_fills::group_fills_base(external(&graph), 0, LAYER).unwrap();
    let second = group_fills::group_fills_base(external(&graph), 0, LAYER).unwrap();

    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(first.surface_fills.len(), 4);
    assert_eq!(
        first
            .surface_fills
            .iter()
            .map(|fill| fill.representative.bridge_angle)
            .collect::<Vec<_>>(),
        [3.0, 2.0, 1.0, 0.0]
    );
    assert_eq!(first.surface_fills[0].expolygons, [a]);
    assert_eq!(
        first.surface_fills[0].expolygons[0].holes(),
        [first_hole, second_hole]
    );
    assert_eq!(first.surface_fills[1].expolygons.len(), 1);
    assert_eq!(
        bounds(&first.surface_fills[1].expolygons[0]),
        (3_000_000, 3_000_000, 4_500_000, 4_000_000)
    );
    assert_eq!(first.surface_fills[1].expolygons[0].holes().len(), 1);
    assert!(first.surface_fills[2].expolygons.is_empty());
    assert!(first.surface_fills[3].expolygons.is_empty());
    assert_eq!(
        first
            .surface_fills
            .iter()
            .map(|fill| &fill.expolygons)
            .collect::<Vec<_>>(),
        second
            .surface_fills
            .iter()
            .map(|fill| &fill.expolygons)
            .collect::<Vec<_>>()
    );
    combine_infill::dispose(graph);
}

fn bounds(expolygon: &ExPolygon) -> (i64, i64, i64, i64) {
    expolygon.contour().points().iter().fold(
        (i64::MAX, i64::MAX, i64::MIN, i64::MIN),
        |(min_x, min_y, max_x, max_y), point| {
            (
                min_x.min(point.x()),
                min_y.min(point.y()),
                max_x.max(point.x()),
                max_y.max(point.y()),
            )
        },
    )
}
