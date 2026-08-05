mod snapshots;

use crate::project_slice::{
    prepare_infill::{
        horizontal_shell_promotion, surface_type_detection::PreparedSurfaceTypeObject,
    },
    region_slices::RegionSurfaceKind,
    tests::support::KsrArchive,
};

#[test]
fn task22o25_active_promotion_moves_exact_graph_and_changes_only_internal_kinds() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"1#\"",
    );
    let input = super::fixture::prepare_o24(archive.bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let outer = [
        input.objects.as_ptr() as usize,
        input.caches.as_ptr() as usize,
        input.projections.as_ptr() as usize,
        input.trims.as_ptr() as usize,
        input.regularizations.as_ptr() as usize,
        input.filters.as_ptr() as usize,
    ];
    let record_vectors = [
        input
            .objects
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        input
            .caches
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect(),
        input
            .projections
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect(),
        input
            .trims
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect(),
        input
            .regularizations
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect(),
        input
            .filters
            .iter()
            .map(|value| value.records.as_ptr() as usize)
            .collect(),
    ];
    let record_fields = record_field_allocations(&input.objects);
    let sidecars = snapshots::sidecar_snapshots(
        &input.caches,
        &input.projections,
        &input.trims,
        &input.regularizations,
        &input.filters,
    );
    let fill_paths = fill_path_allocations(&input.objects);
    let fill_content = fill_content_without_kind(&input.objects);
    let before_kinds = fill_kinds(&input.objects);

    let output = horizontal_shell_promotion::prepare(input).unwrap();

    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        [
            output.objects.as_ptr() as usize,
            output.caches.as_ptr() as usize,
            output.projections.as_ptr() as usize,
            output.trims.as_ptr() as usize,
            output.regularizations.as_ptr() as usize,
            output.filters.as_ptr() as usize,
        ],
        outer
    );
    assert_eq!(
        [
            output
                .objects
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect::<Vec<_>>(),
            output
                .caches
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect(),
            output
                .projections
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect(),
            output
                .trims
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect(),
            output
                .regularizations
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect(),
            output
                .filters
                .iter()
                .map(|value| value.records.as_ptr() as usize)
                .collect(),
        ],
        record_vectors
    );
    assert_eq!(record_field_allocations(&output.objects), record_fields);
    assert_eq!(
        snapshots::sidecar_snapshots(
            &output.caches,
            &output.projections,
            &output.trims,
            &output.regularizations,
            &output.filters,
        ),
        sidecars
    );
    assert_eq!(fill_path_allocations(&output.objects), fill_paths);
    assert_eq!(fill_content_without_kind(&output.objects), fill_content);
    let after_kinds = fill_kinds(&output.objects);
    assert_eq!(before_kinds.len(), after_kinds.len());
    for (before, after) in before_kinds.into_iter().zip(after_kinds) {
        assert_eq!(
            after,
            if before == RegionSurfaceKind::Internal {
                RegionSurfaceKind::InternalSolid
            } else {
                before
            }
        );
    }
    assert!(fill_paths.iter().flatten().any(|paths| paths.len() > 1));
    horizontal_shell_promotion::dispose(output);
}

fn record_field_allocations(
    objects: &[PreparedSurfaceTypeObject],
) -> Vec<Option<[(usize, usize, usize); 6]>> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .map(|record| {
            record.as_ref().map(|record| {
                [
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
                ]
            })
        })
        .collect()
}

fn fill_path_allocations(objects: &[PreparedSurfaceTypeObject]) -> Vec<Vec<Vec<usize>>> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .map(|record| {
            record
                .fill_surfaces
                .iter()
                .map(|surface| {
                    let expolygon = surface.as_parts().1;
                    std::iter::once(expolygon.contour())
                        .chain(expolygon.holes())
                        .map(|path| path.points().as_ptr() as usize)
                        .collect()
                })
                .collect()
        })
        .collect()
}

type FillContent = (u64, u16, u64, u16, Vec<Vec<(i64, i64)>>);

fn fill_content_without_kind(objects: &[PreparedSurfaceTypeObject]) -> Vec<FillContent> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
        .map(|surface| {
            let (_, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            let paths = std::iter::once(expolygon.contour())
                .chain(expolygon.holes())
                .map(|path| {
                    path.points()
                        .iter()
                        .map(|point| (point.x(), point.y()))
                        .collect()
                })
                .collect();
            (thickness.to_bits(), layers, angle.to_bits(), extra, paths)
        })
        .collect()
}

fn fill_kinds(objects: &[PreparedSurfaceTypeObject]) -> Vec<RegionSurfaceKind> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
        .map(|surface| surface.as_parts().0)
        .collect()
}
