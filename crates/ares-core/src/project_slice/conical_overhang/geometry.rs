use crate::{
    RegionOptions, SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, difference_ex,
        difference_ex_with_safety_offset, intersection_ex, offset_expolygons, union_expolygons,
        xor_ex,
    },
};

use super::{
    super::region_slices::{PostRegion, RegionSurface},
    ValidatedConicalOverhangOptions,
};

const EPSILON_MM: f64 = 0.000_1;
const MITER_LIMIT: f64 = 3.0;
const GEOMETRY_ERROR: &str =
    "project conical overhang geometry is nonfinite or outside the supported Clipper range";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice) struct ConicalOverhangGeometry {
    pub(in crate::project_slice) epsilon_scaled: f32,
    pub(in crate::project_slice) distance_scaled: f32,
    pub(in crate::project_slice) hole_area_scaled: f32,
}

pub(super) fn derive_conical_overhang_geometry(
    options: ValidatedConicalOverhangOptions,
    scale: CoordinateScale,
) -> Result<ConicalOverhangGeometry, SliceError> {
    let factor = scale.factor();
    let epsilon_scaled = (EPSILON_MM / factor) as f32;
    let angle_radians = options.angle_degrees * std::f64::consts::PI / 180.0;
    let tan_angle = angle_radians.tan();
    let distance_scaled = -(tan_angle * options.layer_height_mm / factor) as f32;
    let hole_area_scaled = (options.hole_size_mm2 / factor / factor) as f32;
    if !distance_scaled.is_finite() || !hole_area_scaled.is_finite() {
        return Err(conical_geometry_error());
    }
    Ok(ConicalOverhangGeometry {
        epsilon_scaled,
        distance_scaled,
        hole_area_scaled,
    })
}

pub(super) fn apply_conical_overhang_layer_pair(
    regions: &mut [PostRegion],
    current_layer_index: usize,
    upper_layer_index: usize,
    geometry: ConicalOverhangGeometry,
) -> Result<(), ClipperError> {
    let upper_merged = merged_layer_footprint(regions, upper_layer_index, geometry.epsilon_scaled)?;
    let mut upper_poly = union_expolygons(&upper_merged, &[])?;
    let current_merged =
        merged_layer_footprint(regions, current_layer_index, geometry.epsilon_scaled)?;
    let current_poly = union_expolygons(&current_merged, &[])?;
    protect_small_holes(&current_poly, &mut upper_poly, geometry.hole_area_scaled)?;
    upper_poly = offset_expolygons(
        &upper_poly,
        geometry.distance_scaled,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let pair_start_upper = regions
        .iter()
        .map(|region| surface_expolygons(&region.layers[upper_layer_index].surfaces))
        .collect::<Vec<_>>();

    for region_index in 0..regions.len() {
        if !regions[region_index].options.make_overhang_printable.0 {
            continue;
        }
        let intersected = intersection_ex(&pair_start_upper[region_index], &upper_poly)?;
        let candidates = union_expolygons(&intersected, &[])?;
        let projected = retain_uncovered_islands(candidates, &current_poly)?;
        let current =
            surface_expolygons(&regions[region_index].layers[current_layer_index].surfaces);
        let owned = union_expolygons(&current, &projected)?;
        regions[region_index].layers[current_layer_index].surfaces = internal_surfaces(owned);

        for (other_region_index, other_region) in regions.iter_mut().enumerate() {
            if other_region_index == region_index {
                continue;
            }
            let other = surface_expolygons(&other_region.layers[current_layer_index].surfaces);
            let remaining = difference_ex_with_safety_offset(&other, &projected)?;
            other_region.layers[current_layer_index].surfaces = internal_surfaces(remaining);
        }
    }
    Ok(())
}

fn protect_small_holes(
    current_poly: &[ExPolygon],
    upper_poly: &mut Vec<ExPolygon>,
    hole_area_scaled: f32,
) -> Result<(), ClipperError> {
    if hole_area_scaled <= 0.0 {
        return Ok(());
    }
    for expolygon in current_poly {
        for hole in expolygon.holes() {
            if hole.area().abs() >= f64::from(hole_area_scaled) {
                continue;
            }
            let hole_poly = ExPolygon::new(hole.clone(), Vec::new());
            let hole_slice = std::slice::from_ref(&hole_poly);
            let overlap = intersection_ex(upper_poly, hole_slice)?;
            if !overlap.is_empty() && xor_ex(&overlap, hole_slice)?.is_empty() {
                *upper_poly = difference_ex(upper_poly, hole_slice)?;
            }
        }
    }
    Ok(())
}

fn retain_uncovered_islands(
    candidates: Vec<ExPolygon>,
    current_poly: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut retained = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !difference_ex(std::slice::from_ref(&candidate), current_poly)?.is_empty() {
            retained.push(candidate);
        }
    }
    Ok(retained)
}

fn surface_expolygons(surfaces: &[RegionSurface]) -> Vec<ExPolygon> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

fn internal_surfaces(expolygons: Vec<ExPolygon>) -> Vec<RegionSurface> {
    expolygons
        .into_iter()
        .map(RegionSurface::internal)
        .collect()
}

pub(super) fn conical_geometry_error() -> SliceError {
    SliceError::InvalidInput(GEOMETRY_ERROR.to_owned())
}

pub(in crate::project_slice) fn region_participates_in_merged_footprint(
    options: &RegionOptions,
) -> bool {
    options.bottom_shell_layers.0 > 0
        || options.top_shell_layers.0 > 0
        || options.sparse_infill_density.0 > 0.0
        || options.wall_loops.0 > 0
}

pub(in crate::project_slice) fn merged_layer_footprint(
    regions: &[PostRegion],
    layer_index: usize,
    epsilon_scaled: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut expanded = Vec::new();
    for region in regions {
        if !region_participates_in_merged_footprint(&region.options) {
            continue;
        }
        let expolygons = surface_expolygons(&region.layers[layer_index].surfaces);
        expanded.append(&mut offset_expolygons(
            &expolygons,
            epsilon_scaled,
            JoinType::Miter,
            MITER_LIMIT,
        )?);
    }
    union_expolygons(&[], &expanded)
}

type MergedFootprintFn = fn(&[PostRegion], usize, f32) -> Result<Vec<ExPolygon>, ClipperError>;

const _: fn(
    ValidatedConicalOverhangOptions,
    CoordinateScale,
) -> Result<ConicalOverhangGeometry, SliceError> = derive_conical_overhang_geometry;
const _: fn(&RegionOptions) -> bool = region_participates_in_merged_footprint;
const _: MergedFootprintFn = merged_layer_footprint;
