//! The non-`FillSupportBase` raft fillers: `SupportParameters` selects
//! `ipRectilinear` for dense bases/interfaces and `ipHoneycomb` for a
//! honeycomb `support_base_pattern` (`SupportParameters.hpp:133-135`).
//! Raft fillers are fresh `Fill` objects: `layer_id = size_t(-1)` (no layer
//! alternation), `overlap = 0`, `link_max_length = 0`
//! (`link_max_length_factor = 0`, `SupportCommon.cpp:1436`) and default
//! `FillParams` apart from `density` and `dont_adjust = true` (`:453-455`).

use super::RaftFillSpec;
use crate::fill::honeycomb::{HoneycombFillParams, fill_honeycomb};
use crate::fill::rectilinear::{MonotonicFillParams, fill_rectilinear_surface};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polyline, offset_expolygon,
};

/// `FillParams::anchor_length{_max}` defaults (`FillBase.hpp:60-61`).
const DEFAULT_ANCHOR_LENGTH: f32 = 1000.0;

mod concentric;
pub(in crate::project_slice) use concentric::concentric;

/// `FillGrid::fill_surface` (`FillRectilinear.cpp:3422-3446`), with
/// the fresh support filler's `layer_id = size_t(-1)` (odd).
pub(super) fn grid(
    expolygon: &ExPolygon,
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    use crate::fill::multiline::{MultilineFillParams, Sweep, fill_surface};
    let params = MultilineFillParams {
        spacing: spec.spacing,
        overlap: 0.0,
        angle: spec.angle as f32 + std::f32::consts::FRAC_PI_2,
        reference,
        density: spec.density,
        multiline: 1,
        anchor_length: DEFAULT_ANCHOR_LENGTH,
        anchor_length_max: DEFAULT_ANCHOR_LENGTH,
        dont_sort: false,
    };
    let sweeps = [
        Sweep {
            angle: 0.0,
            shift: 0.0,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: 0.0,
        },
    ];
    let mut paths = fill_surface(expolygon, params, &sweeps, scale)?;
    for path in &mut paths {
        path.reverse();
    }
    Ok(paths)
}

/// `FillRectilinear::fill_surface` with `multiline == 1` →
/// `fill_surface_by_lines` (`FillRectilinear.cpp:3387-3399`).
pub(super) fn rectilinear(
    expolygon: &ExPolygon,
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let params = MonotonicFillParams {
        dont_alternate_fill_direction: false,
        layer_angle_fixed: false,
        spacing: spec.spacing,
        overlap: 0.0,
        density: spec.density,
        angle: spec.angle as f32,
        layer_index: 0,
        thickness_layers: 1,
        fixed_angle: true,
        bridge_angle: None,
        reference_point: reference,
        dont_adjust: true,
        anchor_length_max: DEFAULT_ANCHOR_LENGTH,
        link_max_length: 0.0,
        consistent_pattern: false,
        horiz_move: 0.0,
        extended_object_min_x: None,
        extended_object_max_x: None,
        locked_zag: false,
    };
    Ok(fill_rectilinear_surface(expolygon, params, scale)?.polylines)
}

/// `Fill::fill_surface` (`FillBase.cpp:102-118`): offset by
/// `overlap - 0.5·spacing`, then `FillHoneycomb::_fill_surface_single` per
/// component.
pub(super) fn honeycomb(
    expolygon: &ExPolygon,
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let params = HoneycombFillParams {
        dont_alternate_fill_direction: false,
        spacing: spec.spacing,
        density: spec.density,
        multiline: 1,
        smooth_factor: 0.0,
        resolution: 0.0125,
        angle: spec.angle as f32,
        layer_index: 0,
        thickness_layers: 1,
        fixed_angle: true,
        bridge_angle: None,
        anchor_length: DEFAULT_ANCHOR_LENGTH,
        anchor_length_max: DEFAULT_ANCHOR_LENGTH,
    };
    let delta = (-0.5 * spec.spacing / scale.factor()) as f32;
    let mut out = Vec::new();
    for component in offset_expolygon(expolygon, delta, JoinType::Miter, 3.0)? {
        out.extend(fill_honeycomb(&component, params, scale)?);
    }
    Ok(out)
}
