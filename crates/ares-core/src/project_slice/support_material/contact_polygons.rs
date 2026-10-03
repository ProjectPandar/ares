//! Normal Grid/Snug `SupportMaterial.cpp::fill_contact_layer`.

use super::{contact_layer::TopContactLayer, pattern::SupportPattern};
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, difference_polygons_paths,
    intersection_polygons_paths, offset_paths, opening_paths, union_polygons_paths,
};

pub(super) struct ContactPolygons<'a> {
    pub(super) scale: CoordinateScale,
    pub(super) layer_index: usize,
    pub(super) grid: SupportGridParams,
    pub(super) reduce_interfaces: bool,
    pub(super) no_interface_offset: f32,
    pub(super) lower_layer_polygons: &'a [Polygon],
    pub(super) trimming_polygons: &'a [Polygon],
    pub(super) all_trimming_polygons: &'a [Polygon],
    pub(super) contact_polygons: &'a [Polygon],
    pub(super) overhang_polygons: Vec<Polygon>,
    pub(super) enforcer_polygons: Vec<Polygon>,
}

pub(super) fn fill_grid_contact_layer(
    layer: &mut TopContactLayer,
    input: ContactPolygons<'_>,
) -> Result<(), ClipperError> {
    let pattern = SupportPattern::new(
        input.contact_polygons,
        input.trimming_polygons,
        &input.grid,
        input.scale,
    )?;
    let projected =
        pattern.extract_support(input.contact_polygons, input.trimming_polygons, -3, true);
    let dense_lower = if input.reduce_interfaces {
        opening_paths(
            input.lower_layer_polygons,
            input.no_interface_offset * 0.5,
            input.no_interface_offset * (0.6_f32 + 0.5),
            JoinType::Square,
            0.0,
        )?
    } else {
        Vec::new()
    };
    layer.polygons = if input.reduce_interfaces {
        dense_contact_polygons(
            &input.overhang_polygons,
            &dense_lower,
            &projected,
            input.trimming_polygons,
            &input,
        )?
    } else {
        pattern.extract_support(
            input.contact_polygons,
            input.trimming_polygons,
            input.grid.expansion_to_slice,
            true,
        )
    };
    layer.contact_polygons = Some(projected);
    if !input.enforcer_polygons.is_empty()
        && !input.all_trimming_polygons.is_empty()
        && input.layer_index > 0
    {
        let pattern = SupportPattern::new(
            &input.enforcer_polygons,
            input.all_trimming_polygons,
            &input.grid,
            input.scale,
        )?;
        let projected = pattern.extract_support(
            &input.enforcer_polygons,
            input.all_trimming_polygons,
            -3,
            true,
        );
        let mut polygons = if input.reduce_interfaces {
            dense_contact_polygons(
                &input.enforcer_polygons,
                &dense_lower,
                &projected,
                input.all_trimming_polygons,
                &input,
            )?
        } else {
            pattern.extract_support(
                &input.enforcer_polygons,
                input.all_trimming_polygons,
                input.grid.expansion_to_slice,
                true,
            )
        };
        let needs_union = !layer.polygons.is_empty();
        layer.polygons.append(&mut polygons);
        if needs_union {
            layer.polygons = union_polygons_paths(&layer.polygons)?;
        }
    }
    layer.overhang_polygons = Some(input.overhang_polygons);
    if !input.enforcer_polygons.is_empty() {
        // Upstream ultimately retains the original enforcer polygons, not
        // the grid projection used for generating the printable interface.
        layer.enforcer_polygons = Some(input.enforcer_polygons);
    }
    Ok(())
}

fn dense_contact_polygons(
    polygons: &[Polygon],
    dense_lower: &[Polygon],
    projected: &[Polygon],
    trimming: &[Polygon],
    input: &ContactPolygons<'_>,
) -> Result<Vec<Polygon>, ClipperError> {
    let dense = difference_polygons_paths(polygons, dense_lower)?;
    if dense.is_empty() {
        return Ok(Vec::new());
    }
    // This expand uses the default Miter/3, not SUPPORT_SURFACES_OFFSET_PARAMETERS.
    let expanded = offset_paths(
        &dense,
        input.no_interface_offset * 0.1,
        JoinType::Miter,
        3.0,
    )?;
    let dense = difference_polygons_paths(&expanded, trimming)?;
    let dense = intersection_polygons_paths(&dense, projected)?;
    let pattern = SupportPattern::new(&dense, trimming, &input.grid, input.scale)?;
    Ok(pattern.extract_support(&dense, trimming, input.grid.expansion_to_slice, false))
}
