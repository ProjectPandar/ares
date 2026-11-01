mod apply;
pub(super) mod collision;
pub(super) mod contour;
pub(super) mod graph;
mod hooks;
mod scale;
mod touching;
pub(super) mod types;

use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, Polygon, Polyline};
use apply::apply_connections;
use graph::build_working_graph;
use scale::{scaled_epsilon, scaled_f32, scaled_f64};

pub(crate) struct FillBoundary<'a> {
    pub(crate) polygons: &'a [Polygon],
    pub(crate) bbox: BoundingBox,
}

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
    debug_assert!(params.multiline >= 1);
    // `anchor_length_max < 0.05` (`dont_connect`) is short-circuited inside
    // `connect_infill_polygons` — the `>= 0.01` invariant only holds on the
    // connection path, not this `chain_or_connect_infill` entry.

    let mut boundaries = Vec::with_capacity(boundary.holes().len() + 1);
    boundaries.push(boundary.contour().clone());
    boundaries.extend(boundary.holes().iter().cloned());
    let bbox =
        BoundingBox::from_expolygon(boundary).expect("the fill boundary contour must be nonempty");
    connect_infill_polygons(
        infill_ordered,
        FillBoundary {
            polygons: &boundaries,
            bbox,
        },
        spacing,
        params,
        scale,
    )
}

pub(crate) fn connect_infill_polygons(
    infill_ordered: Vec<Polyline>,
    boundary: FillBoundary<'_>,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `chain_or_connect_infill` (`FillBase.cpp:1822`): `dont_connect()`
    // (`anchor_length_max < 0.05`, `FillBase.hpp:52`) returns the polylines
    // chained (`size > 1`) rather than boundary-connected.
    if params.anchor_length_max < 0.05 {
        let mut polylines = infill_ordered;
        if polylines.len() > 1 {
            crate::project_slice::chain_polylines(&mut polylines, scale);
        }
        return Ok(polylines);
    }
    let anchor_length = scaled_f32(params.anchor_length, scale);
    let anchor_length_max = scaled_f32(params.anchor_length_max, scale);
    let scaled_spacing = scaled_f64(spacing, scale);
    let epsilon = scaled_epsilon(scale);
    let graph = build_working_graph(
        infill_ordered,
        boundary.polygons,
        boundary.bbox,
        spacing,
        scale,
    )?;
    let out = apply_connections(
        graph,
        anchor_length,
        anchor_length_max,
        scaled_spacing,
        params.multiline,
        params.dont_sort,
        epsilon,
    );
    Ok(out)
}
