use super::helpers::polygon;
use crate::geometry::Polygon;
use crate::geometry::clipper::{JoinType, offset_paths_tree};

fn ring(cx: i64, radius: i64, clockwise: bool) -> Polygon {
    let mut points = (0..8)
        .map(|index| {
            let angle = std::f64::consts::TAU * f64::from(index) / 8.0;
            (
                cx + (radius as f64 * angle.cos()).round() as i64,
                (radius as f64 * angle.sin()).round() as i64,
            )
        })
        .collect::<Vec<_>>();
    if clockwise {
        points.reverse();
    }
    polygon(&points)
}

#[test]
fn negative_polytree_offset_preserves_grouped_root_order() {
    let mut paths = Vec::new();
    for center in [0, 5_000_000] {
        paths.push(ring(center, 2_000_000, false));
        paths.push(ring(center, 1_000_000, true));
    }
    let output = offset_paths_tree(&paths, -171_576.02, JoinType::Miter, 3.0).unwrap();
    let roots = output.children().collect::<Vec<_>>();
    let centers = roots
        .iter()
        .map(|root| {
            let points = root.contour().points();
            let min = points.iter().map(|point| point.x()).min().unwrap();
            let max = points.iter().map(|point| point.x()).max().unwrap();
            (min + max) / 2
        })
        .collect::<Vec<_>>();

    assert_eq!(centers, [0, 5_000_000]);
    assert!(roots.iter().all(|root| root.children().count() == 1));
}
