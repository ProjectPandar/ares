use crate::{
    SliceError,
    geometry::{
        ExPolygon, FillRule, Polygon, difference_ex_polygons,
        difference_ex_polygons_with_safety_offset, union_ex,
    },
    project_slice::{
        prepare_infill::horizontal_shell_propagation::{
            GeometryStep, gather, geometry_step, range_error,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

pub(super) fn neighbor(
    original: &[RegionSurface],
    mut new_internal_solid: Vec<Polygon>,
) -> Result<Vec<RegionSurface>, SliceError> {
    for surface in original {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        if kind == RegionSurfaceKind::InternalSolid {
            gather::append_expolygon(&mut new_internal_solid, expolygon);
        }
    }

    geometry_step(GeometryStep::SolidUnion)?;
    let unioned = union_ex(&new_internal_solid, FillRule::NonZero).map_err(|_| range_error())?;
    let mut output = unioned
        .iter()
        .cloned()
        .map(|expolygon| RegionSurface::new(RegionSurfaceKind::InternalSolid, expolygon))
        .collect::<Vec<_>>();
    let mut polygons_internal = flatten_expolygons(&unioned);

    let original_internal = original
        .iter()
        .filter_map(|surface| {
            let (kind, expolygon, _, _, _, _) = surface.as_parts();
            (kind == RegionSurfaceKind::Internal).then(|| expolygon.clone())
        })
        .collect::<Vec<_>>();
    geometry_step(GeometryStep::InternalSafetyDifference)?;
    let remaining_internal =
        difference_ex_polygons_with_safety_offset(&original_internal, &polygons_internal)
            .map_err(|_| range_error())?;
    output.extend(
        remaining_internal
            .iter()
            .cloned()
            .map(|expolygon| RegionSurface::new(RegionSurfaceKind::Internal, expolygon)),
    );
    polygons_internal.extend(flatten_expolygons(&remaining_internal));

    for group in external_groups(original) {
        let geometry = group
            .iter()
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>();
        geometry_step(GeometryStep::ExternalGroupDifference)?;
        let fragments =
            difference_ex_polygons(&geometry, &polygons_internal).map_err(|_| range_error())?;
        output.extend(
            fragments
                .into_iter()
                .map(|fragment| group[0].clone_with_expolygon(fragment)),
        );
    }
    Ok(output)
}

fn external_groups(original: &[RegionSurface]) -> Vec<Vec<&RegionSurface>> {
    let mut groups: Vec<Vec<&RegionSurface>> = Vec::new();
    for surface in original {
        if !matches!(
            surface.as_parts().0,
            RegionSurfaceKind::Top | RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge
        ) {
            continue;
        }
        if let Some(group) = groups
            .iter_mut()
            .find(|group| compatible(group[0], surface))
        {
            group.push(surface);
        } else {
            groups.push(vec![surface]);
        }
    }
    groups
}

fn compatible(first: &RegionSurface, candidate: &RegionSurface) -> bool {
    let (first_kind, _, first_thickness, first_layers, first_angle, _) = first.as_parts();
    let (kind, _, thickness, layers, angle, _) = candidate.as_parts();
    first_kind == kind
        && first_thickness == thickness
        && first_layers == layers
        && first_angle == angle
}

fn flatten_expolygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut paths = Vec::new();
    for expolygon in expolygons {
        gather::append_expolygon(&mut paths, expolygon);
    }
    paths
}
