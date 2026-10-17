//! `SeamPlacer.cpp:1083-1092`: raw support distance, NOT clamped seam penalty.
use super::super::LayerPlan;
use crate::geometry::{CoordinateScale, ExPolygon, Line, LineDistanceTree};

pub(super) fn populate(previous: &[ExPolygon], layer: &mut LayerPlan, scale: CoordinateScale) {
    // ExPolygon.hpp:175-193: contour then holes, preserving their line order.
    let lines: Vec<_> = previous
        .iter()
        .flat_map(|slice| std::iter::once(slice.contour()).chain(slice.holes()))
        .flat_map(|polygon| {
            let points = polygon.points();
            points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .take(points.len())
                .map(|(&a, &b)| Line::new(a, b))
        })
        .collect();
    let tree = LineDistanceTree::new(&lines);
    layer.unsupported_distances = Some(
        layer
            .candidates
            .points
            .iter()
            .map(|candidate| {
                let point = [
                    f64::from(candidate.position.x),
                    f64::from(candidate.position.y),
                ];
                // AABBTreeLines.hpp:333-346 explicitly returns +infinity for no line.
                let distance =
                    tree.nearest_mm(point, scale.factor())
                        .map_or(f64::INFINITY, |hit| {
                            hit.squared_distance.sqrt()
                                * f64::from(tree.outside_mm(point, scale.factor()))
                        });
                let width = layer.candidates.perimeters[candidate.perimeter_index].flow_width;
                (distance + f64::from(0.4_f32 * width)) as f32
            })
            .collect(),
    );
}
