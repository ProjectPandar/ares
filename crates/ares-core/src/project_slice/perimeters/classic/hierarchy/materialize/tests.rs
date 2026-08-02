use crate::geometry::{ExPolygon, Point, Polygon};

use crate::project_slice::perimeters::classic::onion::RawShellDepth;

use super::materialize;

fn polygon(tag: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(tag, 0),
        Point::new(tag + 4, 0),
        Point::new(tag + 4, 4),
        Point::new(tag, 4),
    ])
}

fn expolygon(tag: i64, holes: &[i64]) -> ExPolygon {
    ExPolygon::new(
        polygon(tag),
        holes.iter().map(|tag| polygon(*tag)).collect(),
    )
}

#[test]
fn task22o4_materializes_normal_then_smaller_contours_and_holes() {
    let shells = vec![RawShellDepth {
        depth: 0,
        normal: vec![expolygon(10, &[11, 12]), expolygon(20, &[])],
        smaller_width: vec![expolygon(30, &[31])],
    }];
    let buckets = materialize(1, &shells);
    assert_eq!(buckets.contours.len(), 2);
    assert!(buckets.contours[1].is_empty());
    assert_eq!(
        buckets.contours[0]
            .iter()
            .map(|loop_| loop_.polygon.points()[0].x())
            .collect::<Vec<_>>(),
        [10, 20, 30]
    );
    assert_eq!(
        buckets.holes[0]
            .iter()
            .map(|loop_| loop_.polygon.points()[0].x())
            .collect::<Vec<_>>(),
        [11, 12, 31]
    );
    assert_eq!(
        buckets.contours[0]
            .iter()
            .map(|loop_| loop_.is_smaller_width_perimeter)
            .collect::<Vec<_>>(),
        [false, false, true]
    );
    assert!(buckets.contours[0].iter().all(|loop_| loop_.is_contour));
    assert!(buckets.holes[0].iter().all(|loop_| !loop_.is_contour));
}

#[test]
fn task22o4_materialization_uses_explicit_depth_and_effective_collapse() {
    let shells = vec![
        RawShellDepth {
            depth: 0,
            normal: vec![expolygon(0, &[])],
            smaller_width: Vec::new(),
        },
        RawShellDepth {
            depth: 1,
            normal: vec![expolygon(10, &[])],
            smaller_width: Vec::new(),
        },
    ];
    let buckets = materialize(1, &shells);
    assert_eq!(buckets.contours[1][0].depth, 1_u16);
    assert_eq!(buckets.contours[1][0].polygon.points()[0].x(), 10);

    let empty = materialize(-1, &shells);
    assert!(empty.contours.is_empty());
    assert!(empty.holes.is_empty());
}
