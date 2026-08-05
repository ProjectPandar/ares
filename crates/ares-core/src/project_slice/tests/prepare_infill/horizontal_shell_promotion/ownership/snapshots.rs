use crate::{
    geometry::{ExPolygon, Polygon},
    project_slice::prepare_infill::{
        vertical_shell_filtering::types::VerticalShellTinyFilterObject,
        vertical_shell_projection::types::VerticalShellProjectionObject,
        vertical_shell_regularization::types::VerticalShellRegularizationObject,
        vertical_shell_trimming::types::VerticalShellTrimObject,
        vertical_shells::types::VerticalShellCacheObject,
    },
};

type PathSnapshot = (usize, Vec<(i64, i64)>);
type PolygonGroupSnapshot = (usize, Vec<PathSnapshot>);
type ExPolygonGroupSnapshot = (usize, Vec<Vec<PathSnapshot>>);

#[derive(Debug, PartialEq)]
pub(super) struct SidecarSnapshots {
    caches: Vec<Option<[PolygonGroupSnapshot; 3]>>,
    projections: Vec<Option<[PolygonGroupSnapshot; 2]>>,
    trims: Vec<Option<PolygonGroupSnapshot>>,
    regularizations: Vec<Option<ExPolygonGroupSnapshot>>,
    filters: Vec<Option<ExPolygonGroupSnapshot>>,
}

pub(super) fn sidecar_snapshots(
    caches: &[VerticalShellCacheObject],
    projections: &[VerticalShellProjectionObject],
    trims: &[VerticalShellTrimObject],
    regularizations: &[VerticalShellRegularizationObject],
    filters: &[VerticalShellTinyFilterObject],
) -> SidecarSnapshots {
    SidecarSnapshots {
        caches: caches
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| {
                record.as_ref().map(|record| {
                    [
                        polygons(&record.top_surfaces),
                        polygons(&record.bottom_surfaces),
                        polygons(&record.holes),
                    ]
                })
            })
            .collect(),
        projections: projections
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| {
                record
                    .as_ref()
                    .map(|record| [polygons(&record.shell), polygons(&record.holes)])
            })
            .collect(),
        trims: trims
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| record.as_ref().map(|record| polygons(&record.shell)))
            .collect(),
        regularizations: regularizations
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| {
                record
                    .as_ref()
                    .map(|record| expolygons(&record.regularized_shell))
            })
            .collect(),
        filters: filters
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| {
                record
                    .as_ref()
                    .map(|record| expolygons(&record.filtered_shell))
            })
            .collect(),
    }
}

fn polygons(polygons: &[Polygon]) -> PolygonGroupSnapshot {
    (
        polygons.as_ptr() as usize,
        polygons.iter().map(path).collect(),
    )
}

fn expolygons(expolygons: &[ExPolygon]) -> ExPolygonGroupSnapshot {
    (
        expolygons.as_ptr() as usize,
        expolygons
            .iter()
            .map(|expolygon| {
                std::iter::once(expolygon.contour())
                    .chain(expolygon.holes())
                    .map(path)
                    .collect()
            })
            .collect(),
    )
}

fn path(path: &Polygon) -> PathSnapshot {
    (
        path.points().as_ptr() as usize,
        path.points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
    )
}
