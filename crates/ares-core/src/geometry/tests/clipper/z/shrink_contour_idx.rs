use crate::geometry::{ExPolygon, JoinType, Point, Polygon, shrink_expolygon_with_contour_idx};

fn square(min: i64, max: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

/// `shrink_expolygon_with_contour_idx` (`SupportCommon.cpp:486-570`): the
/// grown hole (2..48) cuts the shrunk contour (8..92) corner; contour
/// vertices keep 0, the hole corner keeps 1, the crossings become -1.
#[test]
fn tags_vertices_with_their_source_contour() {
    let mut hole = square(10_000, 40_000);
    hole.reverse();
    let expolygon = ExPolygon::new(square(0, 100_000), vec![hole]);
    let out = shrink_expolygon_with_contour_idx(&expolygon, 8_000.0, JoinType::Miter, 1.2).unwrap();
    assert_eq!(out.len(), 1);
    let mut vertices = out[0].clone();
    vertices.sort_unstable();
    assert_eq!(
        vertices,
        vec![
            (8_000, 48_000, -1),
            (8_000, 92_000, 0),
            (48_000, 8_000, -1),
            (48_000, 48_000, 1),
            (92_000, 8_000, 0),
            (92_000, 92_000, 0),
        ]
    );
}

/// No hole: one inward offset tagged 0; nothing survives past the middle.
#[test]
fn contour_only_offsets_are_tagged_zero_or_vanish() {
    let expolygon = ExPolygon::new(square(0, 100_000), Vec::new());
    let out = shrink_expolygon_with_contour_idx(&expolygon, 8_000.0, JoinType::Miter, 1.2).unwrap();
    assert!(out.len() == 1 && out[0].iter().all(|&(_, _, z)| z == 0));
    let gone =
        shrink_expolygon_with_contour_idx(&expolygon, 60_000.0, JoinType::Miter, 1.2).unwrap();
    assert!(gone.is_empty());
}
