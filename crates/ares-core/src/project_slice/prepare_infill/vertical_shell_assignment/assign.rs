use crate::{
    SliceError,
    geometry::{ExPolygon, difference_ex, intersection_polygons_ex},
    project_slice::{
        prepare_infill::{
            surface_type_detection::types::PreparedSurfaceTypeRecord,
            vertical_shell_assignment::{GeometryStep, geometry_step, range_error, record_commit},
            vertical_shell_filtering::types::VerticalShellTinyFilter,
            vertical_shell_trimming::trim::polygons_internal,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[derive(Debug)]
pub(super) enum StagedAssignment {
    Noop,
    Replace {
        new_internal: Vec<ExPolygon>,
        new_internal_void: Vec<ExPolygon>,
        new_internal_solid: Vec<ExPolygon>,
    },
}

pub(super) fn stage_record(
    record: &PreparedSurfaceTypeRecord,
    filter: &VerticalShellTinyFilter,
) -> Result<StagedAssignment, SliceError> {
    if filter.filtered_shell.is_empty() {
        return Ok(StagedAssignment::Noop);
    }

    let internal_paths = polygons_internal(record);
    let internal = collect_kind(record, RegionSurfaceKind::Internal);
    let internal_void = collect_kind(record, RegionSurfaceKind::InternalVoid);

    geometry_step(GeometryStep::SolidIntersection)?;
    let new_internal_solid = intersection_polygons_ex(&internal_paths, &filter.filtered_shell)
        .map_err(|_| range_error())?;
    geometry_step(GeometryStep::InternalDifference)?;
    let new_internal =
        difference_ex(&internal, &filter.filtered_shell).map_err(|_| range_error())?;
    geometry_step(GeometryStep::InternalVoidDifference)?;
    let new_internal_void =
        difference_ex(&internal_void, &filter.filtered_shell).map_err(|_| range_error())?;

    Ok(StagedAssignment::Replace {
        new_internal,
        new_internal_void,
        new_internal_solid,
    })
}

pub(super) fn commit(record: &mut PreparedSurfaceTypeRecord, staged: StagedAssignment) {
    let StagedAssignment::Replace {
        new_internal,
        new_internal_void,
        new_internal_solid,
    } = staged
    else {
        return;
    };

    record_commit();
    record.fill_surfaces.retain(|surface| {
        matches!(
            surface.as_parts().0,
            RegionSurfaceKind::Top | RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge
        )
    });
    record.fill_surfaces.extend(
        new_internal
            .into_iter()
            .map(|expolygon| RegionSurface::new(RegionSurfaceKind::Internal, expolygon)),
    );
    record.fill_surfaces.extend(
        new_internal_void
            .into_iter()
            .map(|expolygon| RegionSurface::new(RegionSurfaceKind::InternalVoid, expolygon)),
    );
    record.fill_surfaces.extend(
        new_internal_solid
            .into_iter()
            .map(|expolygon| RegionSurface::new(RegionSurfaceKind::InternalSolid, expolygon)),
    );
}

fn collect_kind(record: &PreparedSurfaceTypeRecord, selected: RegionSurfaceKind) -> Vec<ExPolygon> {
    record
        .fill_surfaces
        .iter()
        .filter_map(|surface| {
            let (kind, expolygon, _, _, _, _) = surface.as_parts();
            (kind == selected).then(|| expolygon.clone())
        })
        .collect()
}
