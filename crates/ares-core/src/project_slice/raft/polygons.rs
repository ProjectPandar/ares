//! Raft polygon chain, ported from the raft-only path of upstream
//! `generate_raft_base` (`OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:244-390`)
//! and the layer-0 contact derivation:
//!
//! 1. overhangs for layer 0 = the first object layer `lslices`
//!    (`top_contact_layers`, `SupportMaterial.cpp:2130-2145`).
//! 2. `contact_polygons = expand(overhangs, raft_expansion)`
//!    (`detect_contacts` layer_id==0, `SupportMaterial.cpp:1573-1577`).
//! 3. `contacts->polygons = SupportGridPattern(contact).extract_support(
//!    expansion_to_slice, fill_holes=true)`
//!    (`fill_contact_layer` else-branch — `reduce_interfaces` is false
//!    at layer 0, `SupportMaterial.cpp:1861-1887`).
//! 4. `interface_polygons = expand(contacts, inflate_factor_fine,
//!    jtSquare)` (`SupportCommon.cpp:295-331`); fine = 0.5mm when
//!    `raft_layers > 1`, else EPSILON.
//! 5. `base = union(interface_polygons)` (no support columns in the
//!    raft-only path, `:347-359`).
//! 6. 1st layer polygons = `base`, further expanded by
//!    `max(0, raft_first_layer_expansion - fine)` (`:279-281`, `:366-371`).

use crate::geometry::support_grid::{SupportGridParams, SupportGridPattern};
use crate::geometry::{
    ClipperError, Coord, ExPolygon, FillRule, JoinType, Polygon, offset_expolygons, union_ex,
};

/// Scaled millimeters (`scale_`).
const INFLATE_FACTOR_FINE_MM: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftPolygonScale {
    /// Lattice units per millimetre (`CoordinateScale::factor()`).
    pub(crate) units_per_mm: f64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RaftPolygons {
    /// 1st print layer polygons (may be expanded beyond `base`).
    pub(crate) first_layer: Vec<Polygon>,
    /// Base raft layer polygons (layers 1..base_raft_layers).
    pub(crate) base: Vec<Polygon>,
    /// Interface raft layer polygons (layers 1..interface_raft_layers).
    pub(crate) interface: Vec<Polygon>,
    /// Contact layer polygons (from top_contacts).
    pub(crate) contact: Vec<Polygon>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftPolygonParams {
    /// `raft_expansion` in lattice units.
    pub(crate) raft_expansion: Coord,
    /// `raft_first_layer_expansion` in lattice units.
    pub(crate) first_layer_expansion: Coord,
    pub(crate) raft_layers: usize,
    pub(crate) grid: SupportGridParams,
    pub(crate) scale: RaftPolygonScale,
}

pub(crate) fn raft_polygons(
    first_layer_lslices: &[ExPolygon],
    params: &RaftPolygonParams,
) -> Result<RaftPolygons, crate::geometry::ClipperError> {
    // 1) Layer-0 overhangs: the whole first object layer.
    let mut overhangs = Vec::new();
    for expolygon in first_layer_lslices {
        overhangs.push(expolygon.contour().clone());
        overhangs.extend(expolygon.holes().iter().cloned());
    }
    if overhangs.is_empty() {
        return Ok(RaftPolygons::default());
    }

    // 2) Expand for stability (`detect_contacts:1576`).
    let contacts_input = if params.raft_expansion > 0 {
        offset_expolygons(
            first_layer_lslices,
            params.raft_expansion as f32,
            JoinType::Miter,
            3.0,
        )?
        .into_iter()
        .flat_map(|expolygon| {
            let (contour, holes) = expolygon.into_parts();
            std::iter::once(contour).chain(holes)
        })
        .collect::<Vec<_>>()
    } else {
        overhangs
    };

    // 3) Grid stretching of the contact silhouette.
    let pattern = SupportGridPattern::new(&contacts_input, &[], &params.grid)?;
    let contact =
        pattern.extract_support(&contacts_input, &[], params.grid.expansion_to_slice, true);

    // 4) Interface expansion (`SupportCommon.cpp:295-331`).
    let fine = if params.raft_layers > 1 {
        (INFLATE_FACTOR_FINE_MM * params.scale.units_per_mm) as Coord
    } else {
        0
    };
    let interface = if fine > 0 {
        expand_square(&contact, fine)?
    } else {
        contact.clone()
    };

    // 5) Base = union(interface) (raft-only: no columns to merge).
    let base = interface.clone();

    // 6) 1st layer expansion (`:279-281`, `:366-371`).
    let first_inflate = (params.first_layer_expansion - fine).max(0);
    let first_layer = if first_inflate > 0 {
        expand_square(&base, first_inflate)?
    } else {
        base.clone()
    };

    Ok(RaftPolygons {
        first_layer,
        base,
        interface,
        contact,
    })
}

/// `expand(polygons, delta, SUPPORT_SURFACES_OFFSET_PARAMETERS)` —
/// square joins with no miter limit (`PrintObject.cpp:4303`).
fn expand_square(
    polygons: &[Polygon],
    delta: Coord,
) -> Result<Vec<Polygon>, crate::geometry::ClipperError> {
    let expolygons = union_ex(polygons, FillRule::NonZero)?;
    let expanded = offset_expolygons(&expolygons, delta as f32, JoinType::Square, 0.0)?;
    Ok(expanded
        .into_iter()
        .flat_map(|expolygon| {
            let (contour, holes) = expolygon.into_parts();
            std::iter::once(contour).chain(holes)
        })
        .collect())
}

#[cfg(test)]
mod tests;
