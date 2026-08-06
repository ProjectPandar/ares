use crate::{
    geometry::{ExPolygon, Polygon},
    project_slice::prepare_infill::{
        surface_type_detection::PreparedSurfaceTypeObject,
        vertical_shell_filtering::types::VerticalShellTinyFilterObject,
        vertical_shell_projection::types::VerticalShellProjectionObject,
        vertical_shell_regularization::types::VerticalShellRegularizationObject,
        vertical_shell_trimming::types::VerticalShellTrimObject,
        vertical_shells::types::VerticalShellCacheObject,
    },
};

pub(super) type Allocation = (usize, usize, usize);
pub(super) type SurfaceContent = (u8, u64, u16, u64, u16, Vec<Vec<(i64, i64)>>);

#[derive(Debug, PartialEq)]
pub(super) struct RecordSnapshot {
    pub(super) fields: [Allocation; 6],
    pub(super) fill_points: Vec<Vec<usize>>,
    pub(super) fill_content: Vec<SurfaceContent>,
}

pub(super) fn records(objects: &[PreparedSurfaceTypeObject]) -> Vec<Option<RecordSnapshot>> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .map(|record| {
            record.as_ref().map(|record| RecordSnapshot {
                fields: [
                    (
                        record.perimeters.as_ptr() as usize,
                        record.perimeters.len(),
                        record.perimeters.capacity(),
                    ),
                    (
                        record.thin_fills.as_ptr() as usize,
                        record.thin_fills.len(),
                        record.thin_fills.capacity(),
                    ),
                    (
                        record.slices.as_ptr() as usize,
                        record.slices.len(),
                        record.slices.capacity(),
                    ),
                    (
                        record.fill_surfaces.as_ptr() as usize,
                        record.fill_surfaces.len(),
                        record.fill_surfaces.capacity(),
                    ),
                    (
                        record.fill_expolygons.as_ptr() as usize,
                        record.fill_expolygons.len(),
                        record.fill_expolygons.capacity(),
                    ),
                    (
                        record.fill_no_overlap_expolygons.as_ptr() as usize,
                        record.fill_no_overlap_expolygons.len(),
                        record.fill_no_overlap_expolygons.capacity(),
                    ),
                ],
                fill_points: record
                    .fill_surfaces
                    .iter()
                    .map(|surface| {
                        let expolygon = surface.as_parts().1;
                        std::iter::once(expolygon.contour())
                            .chain(expolygon.holes())
                            .map(|path| path.points().as_ptr() as usize)
                            .collect()
                    })
                    .collect(),
                fill_content: record
                    .fill_surfaces
                    .iter()
                    .map(|surface| {
                        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
                        (
                            kind as u8,
                            thickness.to_bits(),
                            layers,
                            angle.to_bits(),
                            extra,
                            std::iter::once(expolygon.contour())
                                .chain(expolygon.holes())
                                .map(|path| {
                                    path.points()
                                        .iter()
                                        .map(|point| (point.x(), point.y()))
                                        .collect()
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            })
        })
        .collect()
}

#[derive(Debug, PartialEq)]
pub(super) struct Sidecars {
    caches: Vec<Option<[PathGroup; 3]>>,
    projections: Vec<Option<[PathGroup; 2]>>,
    trims: Vec<Option<PathGroup>>,
    regularizations: Vec<Option<ExPolygonGroup>>,
    filters: Vec<Option<ExPolygonGroup>>,
}

type Path = (usize, Vec<(i64, i64)>);
type PathGroup = (usize, Vec<Path>);
type ExPolygonGroup = (usize, Vec<Vec<Path>>);

pub(super) fn sidecars(
    caches: &[VerticalShellCacheObject],
    projections: &[VerticalShellProjectionObject],
    trims: &[VerticalShellTrimObject],
    regularizations: &[VerticalShellRegularizationObject],
    filters: &[VerticalShellTinyFilterObject],
) -> Sidecars {
    Sidecars {
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

fn polygons(values: &[Polygon]) -> PathGroup {
    (values.as_ptr() as usize, values.iter().map(path).collect())
}

fn expolygons(values: &[ExPolygon]) -> ExPolygonGroup {
    (
        values.as_ptr() as usize,
        values
            .iter()
            .map(|value| {
                std::iter::once(value.contour())
                    .chain(value.holes())
                    .map(path)
                    .collect()
            })
            .collect(),
    )
}

fn path(value: &Polygon) -> Path {
    (
        value.points().as_ptr() as usize,
        value
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
    )
}
