#[cfg(not(test))]
use crate::geometry::opening_paths;
#[cfg(test)]
use crate::geometry::{ClipperError, opening_paths_with_interstage};

use crate::{
    ProcessEnsureVerticalShellThickness, RegionOptions, SliceError,
    geometry::{
        CoordinateScale, JoinType, Polygon, SAFETY_OFFSET, difference_polygons_paths,
        intersection_polygons_paths, intersection_polygons_paths_with_safety_offset, offset_paths,
    },
    project_slice::{
        perimeters::types::PerimeterInputRecord,
        prepare_infill::horizontal_shell_propagation::{
            GeometryStep, gather, geometry_step, range_error, rebuild, types::NeighborOutcome,
        },
        region_slices::RegionSurface,
    },
};

pub(super) struct NeighborContext<'a> {
    pub(super) scale: CoordinateScale,
    pub(super) source_input: &'a PerimeterInputRecord,
    pub(super) neighbor_input: Option<&'a PerimeterInputRecord>,
    pub(super) options: &'a RegionOptions,
    pub(super) neighbor_fill: &'a [RegionSurface],
}

pub(super) fn process_neighbor(
    context: NeighborContext<'_>,
    solid: &mut Vec<Polygon>,
) -> Result<NeighborOutcome, SliceError> {
    let internal = gather::neighbor_internal_paths(context.neighbor_fill);
    geometry_step(GeometryStep::SafetyIntersection)?;
    let mut new_internal_solid = intersection_polygons_paths_with_safety_offset(solid, &internal)
        .map_err(|_| range_error())?;
    if new_internal_solid.is_empty() {
        return Ok(NeighborOutcome::EmptyIntersection);
    }

    let factor = first_factor(context.options);
    if factor > 0.0 {
        geometry_step(GeometryStep::NeighborExternalWidthScale)?;
        let neighbor = context
            .neighbor_input
            .expect("a None neighbor cannot produce an intersection");
        let margin = scaled_width(context.scale, neighbor.ext_perimeter_flow.width)? * factor;
        geometry_step(GeometryStep::FirstOpeningShrink)?;
        let opened = opening_with_step(
            &new_internal_solid,
            margin,
            GeometryStep::FirstOpeningExpand,
        )?;
        geometry_step(GeometryStep::FirstTooNarrowDifference)?;
        let too_narrow =
            difference_polygons_paths(&new_internal_solid, &opened).map_err(|_| range_error())?;
        if !too_narrow.is_empty() {
            geometry_step(GeometryStep::FirstTrimDifference)?;
            let trimmed = difference_polygons_paths(&new_internal_solid, &too_narrow)
                .map_err(|_| range_error())?;
            *solid = trimmed.clone();
            new_internal_solid = trimmed;
        }
    }

    geometry_step(GeometryStep::SourceSolidWidthScale)?;
    let margin = scaled_width(context.scale, context.source_input.solid_infill_flow.width)?
        * second_factor(context.options);
    geometry_step(GeometryStep::SecondOpeningShrink)?;
    let opened = opening_with_step(
        &new_internal_solid,
        margin,
        GeometryStep::SecondOpeningExpand,
    )?;
    geometry_step(GeometryStep::SecondTooNarrowDifference)?;
    let too_narrow =
        difference_polygons_paths(&new_internal_solid, &opened).map_err(|_| range_error())?;
    if !too_narrow.is_empty() {
        geometry_step(GeometryStep::RepairExpansion)?;
        let expanded = repair_expansion(&too_narrow, margin)?;
        let local_internal = gather::repair_clip_paths(context.neighbor_fill);
        geometry_step(GeometryStep::RepairIntersection)?;
        let repaired =
            intersection_polygons_paths(&expanded, &local_internal).map_err(|_| range_error())?;
        new_internal_solid.extend(repaired);
    }

    rebuild::neighbor(context.neighbor_fill, new_internal_solid).map(NeighborOutcome::Rebuilt)
}

fn opening_with_step(
    paths: &[Polygon],
    margin: f32,
    expansion_step: GeometryStep,
) -> Result<Vec<Polygon>, SliceError> {
    #[cfg(not(test))]
    {
        let _ = expansion_step;
        opening_paths(paths, margin, margin + SAFETY_OFFSET, JoinType::Miter, 5.0)
            .map_err(|_| range_error())
    }
    #[cfg(test)]
    opening_paths_with_interstage(
        paths,
        [margin, margin + SAFETY_OFFSET],
        JoinType::Miter,
        5.0,
        |_| geometry_step(expansion_step).map_err(|_| ClipperError::CoordinateOutOfRange),
    )
    .map_err(|_| range_error())
}

fn repair_expansion(paths: &[Polygon], margin: f32) -> Result<Vec<Polygon>, SliceError> {
    offset_paths(paths, margin, JoinType::Miter, 3.0).map_err(|_| range_error())
}

#[cfg(test)]
pub(super) fn opening_for_test(paths: &[Polygon], margin: f32) -> Result<Vec<Polygon>, SliceError> {
    opening_with_step(paths, margin, GeometryStep::FirstOpeningExpand)
}

#[cfg(test)]
pub(super) fn repair_expansion_for_test(
    paths: &[Polygon],
    margin: f32,
) -> Result<Vec<Polygon>, SliceError> {
    repair_expansion(paths, margin)
}

pub(super) fn scaled_width(scale: CoordinateScale, width: f32) -> Result<f32, SliceError> {
    scale
        .checked_scale(f64::from(width))
        .map(|scaled| scaled as f32)
        .ok_or_else(range_error)
}

pub(super) fn should_stop_after_empty(options: &RegionOptions) -> bool {
    options.sparse_infill_density.0 == 0.0
        || matches!(
            options.ensure_vertical_shell_thickness,
            ProcessEnsureVerticalShellThickness::None
                | ProcessEnsureVerticalShellThickness::CriticalOnly
        )
}

pub(super) fn first_factor(options: &RegionOptions) -> f32 {
    if options.sparse_infill_density.0 == 0.0 {
        1.0_f32
    } else {
        match options.ensure_vertical_shell_thickness {
            ProcessEnsureVerticalShellThickness::None => 0.5_f32,
            ProcessEnsureVerticalShellThickness::CriticalOnly => 0.2_f32,
            ProcessEnsureVerticalShellThickness::Moderate => 0.0_f32,
            ProcessEnsureVerticalShellThickness::EnsureAll => {
                unreachable!("EnsureAll is gated before O26 geometry")
            }
        }
    }
}

pub(super) fn second_factor(options: &RegionOptions) -> f32 {
    match options.ensure_vertical_shell_thickness {
        ProcessEnsureVerticalShellThickness::None => 1.0_f32,
        ProcessEnsureVerticalShellThickness::CriticalOnly
        | ProcessEnsureVerticalShellThickness::Moderate => 3.0_f32,
        ProcessEnsureVerticalShellThickness::EnsureAll => {
            unreachable!("EnsureAll is gated before O26 geometry")
        }
    }
}
