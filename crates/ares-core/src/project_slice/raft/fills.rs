//! Raft layer fills, ported from `FillSupportBase::fill_surface`
//! (`OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3610-3634`) and
//! `make_fill_lines` (`:2920-2961`), wired to the ported
//! `connect_base_support`.
//!
//! Grid anchor: the object bounding-box center rotated by `-angle`
//! (`Fill::_infill_direction`, `FillBase.cpp:288-290`; the filler's
//! bbox is the object bbox, `SupportCommon.cpp:1454-1455`). Only
//! OuterLow→OuterHigh intersection pairs become vertical fill lines;
//! the connection boundary is the `-0.5·spacing` inset outer contour.

use super::fill_params::RaftFillPattern;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillSpec {
    /// Infill angle in radians.
    pub(crate) angle: f64,
    /// Flow spacing in mm.
    pub(crate) spacing: f64,
    /// `flow.spacing()` of the layer's own extrusion flow, mm: the sheath
    /// offsets use it (`SupportCommon.cpp:733`), while the filler keeps
    /// `spacing`. Raft base layers are taller than the object layer the
    /// filler spacing is derived from (`:1476` vs `:1479`).
    pub(crate) sheath_spacing: f64,
    /// Density in [0, 1].
    pub(crate) density: f32,
    /// `with_sheath` (`SupportCommon.cpp:1516`): the flange (raft layer 0)
    /// draws a perimeter loop before the fill.
    pub(crate) sheath: bool,
    /// `no_sort` (`:1517`): each sheath region is one ordered collection.
    pub(crate) no_sort: bool,
    /// `tree_branch_diameter_double_wall_area_scaled`
    /// (`SupportParameters.hpp:186-188`), lattice units².
    pub(crate) tree_double_wall_area: f64,
    /// `first_layer_flow.width()` for the flange `closing_ex` inflate
    /// (`0.5·width`), mm.
    pub(crate) width_mm: f64,
    /// `Fill::new_from_type(base_fill_pattern / raft_interface_fill_pattern)`
    /// (`SupportCommon.cpp:1450-1451`).
    pub(crate) pattern: RaftFillPattern,
    /// Region preprocessing before the fill
    /// (`fill_expolygons_with_sheath_generate_paths` vs
    /// `extrude_interface`, `SupportCommon.cpp:725-727,1729`).
    pub(crate) preprocess: RaftFillPreprocess,
}

/// `SupportCommon.cpp:725-727` applies `closing_ex(polygons, ε)` to the
/// non-sheath raft base/interface layers, while the contact layer
/// (`extrude_interface`, `:1729`) wraps its polygons in
/// `union_safety_offset_ex` (+10 units, `ClipperUtils.hpp:17`).
#[derive(Clone, Copy, Debug)]
pub(crate) enum RaftFillPreprocess {
    /// `closing_ex(polygons, float(SCALED_EPSILON))`.
    Closing,
    /// `union_safety_offset_ex` (Clipper +10 lattice units).
    SafetyOffset,
}

/// One entry of a raft layer's `support_fills.entities`; `extrude_support`
/// chains the top-level entries (`GCode.cpp:6199-6201`).
#[derive(Debug, PartialEq)]
pub(crate) enum RaftEntity {
    /// `ExtrusionPath` (`extrusion_entities_append_paths`,
    /// `ExtrusionEntity.hpp:581-590`).
    Path(Polyline),
    /// `ExtrusionPathOriented`: appended with `can_reverse = false`.
    Oriented(Polyline),
    /// `ExtrusionEntityCollection` with `no_sort = true`, printed in order.
    NoSort(Vec<Polyline>),
}

/// The raft layer's `support_fills.entities` (`SupportCommon.cpp:1476-1520`):
/// `fill_expolygons_with_sheath_generate_paths` (`:705-757`) with a sheath,
/// otherwise the plain fill. `Print.cpp:2693` collects their points for
/// the skirt hull.
pub(crate) fn raft_layer_entities(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<RaftEntity>, ClipperError> {
    if !spec.sheath {
        return Ok(
            raft_layer_fill(layer_polygons, spec, Point::new(0, 0), scale)?
                .into_iter()
                .map(RaftEntity::Path)
                .collect(),
        );
    }
    debug_assert!(matches!(spec.preprocess, RaftFillPreprocess::Closing));
    if spec.density == 0.0 {
        return tree::tree_supports_generate_paths(layer_polygons, spec, scale);
    }
    sheath::fill_with_sheath(layer_polygons, spec, scale)
}

mod sheath;
mod tree;

/// `closing_ex(const Polygons&, delta1, delta2)` (`ClipperUtils.cpp:599-604`):
/// `shrink_paths<PolyTree>(expand_paths(polygons, delta1), delta2)`, which
/// keeps clockwise hole polygons as holes.
fn closing_polygons(
    polygons: &[Polygon],
    expand: f32,
    shrink: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    use crate::geometry::{JoinType, offset_paths, offset_paths_tree};
    let expanded = offset_paths(polygons, expand, JoinType::Miter, 3.0)?;
    Ok(offset_paths_tree(&expanded, -shrink, JoinType::Miter, 3.0)?
        .into_expolygons(CoordinateScale::Normal))
}

/// `Polyline::clip_end` (`Polyline.cpp`): shorten the polyline tail by
/// `distance`, dropping exhausted segments.
mod geom;
use geom::checked_scale_f32;
pub(in crate::project_slice) mod support_base;

mod patterns;
pub(in crate::project_slice) use patterns::concentric;

/// Fill one raft layer with the filler `Fill::new_from_type(pattern)` builds
/// (`SupportCommon.cpp:1450-1451`); the inline body is
/// `FillSupportBase::fill_surface`.
pub(crate) fn raft_layer_fill(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let expolygons = match spec.preprocess {
        // `union_safety_offset_ex` = `offset_ex(polygons, 10.f)`
        // (`ClipperUtils.hpp:17,364`).
        RaftFillPreprocess::SafetyOffset => {
            crate::geometry::union_safety_offset_ex(layer_polygons)?
        }
        // `closing_ex(polygons, float(SCALED_EPSILON))`
        // (`SupportCommon.cpp:727`): `SCALED_EPSILON = scale_(1e-4)`.
        RaftFillPreprocess::Closing => {
            let epsilon = scale.scaled_delta(1e-4) as f32;
            closing_polygons(layer_polygons, epsilon, epsilon)?
        }
    };
    fill_expolygons(expolygons, spec, reference, scale)
}

/// `fill_expolygons_generate_paths` (`SupportCommon.cpp:432-443`); the
/// `SupportBase` body is `FillSupportBase::fill_surface`.
fn fill_expolygons(
    expolygons: Vec<ExPolygon>,
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut out = Vec::new();
    for expolygon in expolygons {
        match spec.pattern {
            RaftFillPattern::SupportBase => {}
            RaftFillPattern::Rectilinear => {
                out.extend(patterns::rectilinear(&expolygon, spec, reference, scale)?);
                continue;
            }
            RaftFillPattern::Honeycomb => {
                out.extend(patterns::honeycomb(&expolygon, spec, scale)?);
                continue;
            }
            RaftFillPattern::Concentric => {
                out.extend(patterns::concentric(
                    &expolygon,
                    spec.spacing,
                    spec.density,
                    scale,
                )?);
                continue;
            }
            RaftFillPattern::Grid => {
                out.extend(patterns::grid(&expolygon, spec, reference, scale)?);
                continue;
            }
        }
        out.extend(support_base::fill_surface(
            &expolygon,
            support_base::SupportBaseFillParams {
                spacing: spec.spacing,
                density: spec.density,
                angle: spec.angle as f32,
                reference,
                reference_angle: spec.angle as f32,
                outer_offset: checked_scale_f32(scale, -0.5 * spec.spacing)?,
            },
            scale,
        )?);
    }
    Ok(out)
}
