use crate::geometry::{ExPolygon, Point, Polygon, RegionExpansion, RegionExpansionParameters};

pub(super) fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

pub(super) fn expolygon(contour: &[(i64, i64)], holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(polygon(contour), holes)
}

pub(super) fn params(values: (f32, f32, usize, f32, f64, f64)) -> RegionExpansionParameters {
    let (
        initial_step,
        other_step,
        num_other_steps,
        max_inflation,
        arc_tolerance,
        shortest_edge_length,
    ) = values;
    RegionExpansionParameters {
        tiny_expansion: 1.0,
        initial_step,
        other_step,
        num_other_steps,
        max_inflation,
        arc_tolerance,
        shortest_edge_length,
    }
}

type ExpansionSnapshot = (u32, u32, Vec<(i64, i64)>);

pub(super) fn snapshots(expansions: &[RegionExpansion]) -> Vec<ExpansionSnapshot> {
    expansions
        .iter()
        .map(|expansion| {
            (
                expansion.src_id,
                expansion.boundary_id,
                expansion
                    .polygon
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect(),
            )
        })
        .collect()
}
