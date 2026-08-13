use crate::{
    OrcaFloats, RegionOptions, SliceError,
    geometry::{
        ClipperError, CoordinateScale, Point, Polygon, intersection_polygons_paths,
        union_polygons_paths,
    },
    project_slice::{
        layers::PlannedLayer, perimeters::flow::resolve_thick_solid_infill_bridge_flow,
    },
};

use super::types::{BridgeCandidateObject, CandidateSurface};

const TARGET_FLOW_HEIGHT_FACTOR: f32 = 0.9;
const EPSILON: f64 = 1.0e-4;
const AABB_INFLATION_MM: f64 = 7.0;

pub(in crate::project_slice) struct CandidateClusterLayer<'a> {
    pub(in crate::project_slice) layer_index: usize,
    pub(in crate::project_slice) print_z: f64,
    pub(in crate::project_slice) bridge_height: f32,
    pub(in crate::project_slice) candidates: &'a [CandidateSurface],
}

pub(in crate::project_slice) fn cluster_candidate_layers(
    layers: &[CandidateClusterLayer<'_>],
    scale: CoordinateScale,
) -> Result<Vec<Vec<usize>>, ClipperError> {
    let mut coverage_by_layer = Vec::with_capacity(layers.len());
    for layer in layers {
        coverage_by_layer.push(layer_coverage(layer.candidates, scale)?);
    }

    let mut clusters: Vec<Vec<usize>> = Vec::new();
    for (position, layer) in layers.iter().enumerate() {
        let starts_new_cluster = if position == 0 {
            true
        } else {
            let previous = &layers[position - 1];
            previous.print_z
                < layer.print_z
                    - f64::from(layer.bridge_height * TARGET_FLOW_HEIGHT_FACTOR)
                    - EPSILON
                || intersection_polygons_paths(
                    &coverage_by_layer[position - 1],
                    &coverage_by_layer[position],
                )?
                .is_empty()
        };
        if starts_new_cluster {
            clusters.push(vec![layer.layer_index]);
        } else {
            clusters
                .last_mut()
                .expect("the first candidate layer starts a cluster")
                .push(layer.layer_index);
        }
    }
    Ok(clusters)
}

pub(in crate::project_slice) fn cluster_candidate_object(
    candidates: &BridgeCandidateObject,
    planned_layers: &[PlannedLayer],
    ordered_region_options: &[&RegionOptions],
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<Vec<Vec<usize>>, SliceError> {
    let flow = resolve_thick_solid_infill_bridge_flow(ordered_region_options[0], nozzle_diameters)?;
    let layers = candidates
        .surfaces_by_layer
        .iter()
        .map(|(&layer_index, candidates)| CandidateClusterLayer {
            layer_index,
            print_z: planned_layers[layer_index].print_z,
            bridge_height: flow.height,
            candidates,
        })
        .collect::<Vec<_>>();
    cluster_candidate_layers(&layers, scale).map_err(geometry_error)
}

fn layer_coverage(
    candidates: &[CandidateSurface],
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut coverage = Vec::new();
    for candidate in candidates {
        coverage.push(inflated_candidate_aabb(candidate, scale));
        coverage = union_polygons_paths(&coverage)?;
    }
    Ok(coverage)
}

fn inflated_candidate_aabb(candidate: &CandidateSurface, scale: CoordinateScale) -> Polygon {
    let mut bounds = candidate
        .new_polygons
        .first()
        .map(polygon_bounds)
        .unwrap_or((0, 0, 0, 0, false));
    for polygon in candidate.new_polygons.iter().skip(1) {
        let next = polygon_bounds(polygon);
        if next.4 {
            if bounds.4 {
                bounds.0 = bounds.0.min(next.0);
                bounds.1 = bounds.1.min(next.1);
                bounds.2 = bounds.2.max(next.2);
                bounds.3 = bounds.3.max(next.3);
            } else {
                bounds = next;
            }
        }
    }
    let (min_x, min_y, max_x, max_y, _) = bounds;
    let inflation = (AABB_INFLATION_MM / scale.factor()).round() as i64;
    let min_x = min_x - inflation;
    let min_y = min_y - inflation;
    let max_x = max_x + inflation;
    let max_y = max_y + inflation;
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn polygon_bounds(polygon: &Polygon) -> (i64, i64, i64, i64, bool) {
    let Some(&first) = polygon.points().first() else {
        return (0, 0, 0, 0, false);
    };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x(), first.y(), first.x(), first.y());
    for &point in &polygon.points()[1..] {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }
    (min_x, min_y, max_x, max_y, min_x < max_x && min_y < max_y)
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "bridge candidate-layer coverage is outside the supported Clipper range".to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("bridge candidate-layer clustering uses closed paths only")
        }
    }
}

#[cfg(test)]
mod tests;
