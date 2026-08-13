mod apply;
mod collision;
mod contour;
mod graph;
mod scale;
mod touching;
mod types;

use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Polyline};
use apply::apply_connections;
use graph::build_working_graph;
use scale::{scaled_epsilon, scaled_f32, scaled_f64};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FillConnectionParams {
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) multiline: i32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn connect_infill(
    infill_ordered: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(!infill_ordered.is_empty());
    debug_assert!(infill_ordered.iter().all(Polyline::is_valid));
    debug_assert!(!boundary.contour().points().is_empty());
    debug_assert!(spacing.is_finite() && spacing > 0.0);
    debug_assert!(params.anchor_length >= 0.0);
    debug_assert!(params.anchor_length_max >= 0.01);
    debug_assert!(params.anchor_length_max >= params.anchor_length);
    debug_assert!(params.multiline >= 1);

    let anchor_length = scaled_f32(params.anchor_length, scale);
    let anchor_length_max = scaled_f32(params.anchor_length_max, scale);
    let scaled_spacing = scaled_f64(spacing, scale);
    let epsilon = scaled_epsilon(scale);
    let graph = build_working_graph(infill_ordered, boundary, spacing, scale)?;
    Ok(apply_connections(
        graph,
        anchor_length,
        anchor_length_max,
        scaled_spacing,
        params.multiline,
        params.dont_sort,
        epsilon,
    ))
}

#[cfg(test)]
mod tests;
