use crate::{
    geometry::{ExPolygon, Polygon},
    project_slice::{
        prepare_infill::{
            horizontal_shell_propagation::types::SourceKind,
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

pub(super) fn source_paths(
    record: &PreparedSurfaceTypeRecord,
    working_fill: &[RegionSurface],
    kind: SourceKind,
) -> Vec<Polygon> {
    let wanted = kind.surface_kind();
    let mut paths = Vec::new();
    append_matching(&mut paths, &record.slices, |surface_kind| {
        surface_kind == wanted
    });
    append_matching(&mut paths, working_fill, |surface_kind| {
        surface_kind == wanted
    });
    paths
}

pub(super) fn neighbor_internal_paths(fill: &[RegionSurface]) -> Vec<Polygon> {
    flatten_matching(fill, |kind| {
        matches!(
            kind,
            RegionSurfaceKind::Internal | RegionSurfaceKind::InternalSolid
        )
    })
}

pub(super) fn repair_clip_paths(fill: &[RegionSurface]) -> Vec<Polygon> {
    flatten_matching(fill, |kind| {
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalVoid
        )
    })
}

fn flatten_matching(
    surfaces: &[RegionSurface],
    predicate: impl Fn(RegionSurfaceKind) -> bool,
) -> Vec<Polygon> {
    let mut paths = Vec::new();
    append_matching(&mut paths, surfaces, predicate);
    paths
}

fn append_matching(
    paths: &mut Vec<Polygon>,
    surfaces: &[RegionSurface],
    predicate: impl Fn(RegionSurfaceKind) -> bool,
) {
    for surface in surfaces {
        let (kind, expolygon, _, _, _, _) = surface.as_parts();
        if predicate(kind) {
            append_expolygon(paths, expolygon);
        }
    }
}

pub(super) fn append_expolygon(paths: &mut Vec<Polygon>, expolygon: &ExPolygon) {
    paths.push(expolygon.contour().clone());
    paths.extend(expolygon.holes().iter().cloned());
}
