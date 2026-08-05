use crate::{
    SliceError,
    geometry::{
        Polygon, difference_polygons_paths, intersection_polygons_paths_with_safety_offset,
    },
    project_slice::{
        prepare_infill::{
            surface_type_detection::types::PreparedSurfaceTypeRecord,
            vertical_shell_projection::types::VerticalShellProjection,
            vertical_shell_trimming::types::VerticalShellTrim,
        },
        region_slices::RegionSurfaceKind,
    },
};

use super::{GeometryStep, geometry_step, range_error};

pub(super) fn trim_record(
    record: &PreparedSurfaceTypeRecord,
    projection: &VerticalShellProjection,
    active: bool,
) -> Result<VerticalShellTrim, SliceError> {
    if !active {
        return Ok(VerticalShellTrim { shell: Vec::new() });
    }
    let polygons_internal = polygons_internal(record);
    geometry_step(GeometryStep::SafetyOffset)?;
    geometry_step(GeometryStep::SafetyIntersection)?;
    let mut shell =
        intersection_polygons_paths_with_safety_offset(&projection.shell, &polygons_internal)
            .map_err(|_| range_error())?;
    geometry_step(GeometryStep::Difference)?;
    shell.extend(
        difference_polygons_paths(&polygons_internal, &projection.holes)
            .map_err(|_| range_error())?,
    );
    geometry_step(GeometryStep::EmptyGate)?;
    if shell.is_empty() {
        return Ok(VerticalShellTrim { shell });
    }
    geometry_step(GeometryStep::SolidAppend)?;
    shell.extend(solid_paths(record));
    Ok(VerticalShellTrim { shell })
}

pub(in crate::project_slice::prepare_infill) fn polygons_internal(
    record: &PreparedSurfaceTypeRecord,
) -> Vec<Polygon> {
    flatten(record, |kind| {
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalVoid
                | RegionSurfaceKind::InternalSolid
        )
    })
}

pub(super) fn solid_paths(record: &PreparedSurfaceTypeRecord) -> Vec<Polygon> {
    flatten(record, |kind| kind == RegionSurfaceKind::InternalSolid)
}

fn flatten(
    record: &PreparedSurfaceTypeRecord,
    selected: impl Fn(RegionSurfaceKind) -> bool,
) -> Vec<Polygon> {
    let mut paths = Vec::new();
    for surface in &record.fill_surfaces {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        if selected(kind) {
            paths.push(expolygon.contour().clone());
            paths.extend(expolygon.holes().iter().cloned());
        }
    }
    paths
}
