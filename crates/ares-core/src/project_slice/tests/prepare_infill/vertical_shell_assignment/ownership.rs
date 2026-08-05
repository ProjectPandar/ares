use crate::project_slice::{prepare_infill::vertical_shell_assignment, tests::support::KsrArchive};

#[test]
fn task22o24_success_moves_the_exact_o23_graph_and_only_assigns_fill_surfaces() {
    let input = super::fixture::prepare_o23(KsrArchive::new().bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let objects = input.objects.as_ptr();
    let caches = input.caches.as_ptr();
    let projections = input.projections.as_ptr();
    let trims = input.trims.as_ptr();
    let regularizations = input.regularizations.as_ptr();
    let filters = input.filters.as_ptr();
    let object_records = input
        .objects
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();
    let record_fields = record_field_allocations(&input.objects);
    let cache_records = input
        .caches
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();
    let projection_records = input
        .projections
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();
    let trim_records = input
        .trims
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();
    let regularization_records = input
        .regularizations
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();
    let filter_records = input
        .filters
        .iter()
        .map(|object| object.records.as_ptr() as usize)
        .collect::<Vec<_>>();

    let output = vertical_shell_assignment::prepare(input).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(output.objects.as_ptr(), objects);
    assert_eq!(output.caches.as_ptr(), caches);
    assert_eq!(output.projections.as_ptr(), projections);
    assert_eq!(output.trims.as_ptr(), trims);
    assert_eq!(output.regularizations.as_ptr(), regularizations);
    assert_eq!(output.filters.as_ptr(), filters);
    assert_eq!(
        output
            .objects
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        object_records
    );
    assert_eq!(record_field_allocations(&output.objects), record_fields);
    assert_eq!(
        output
            .caches
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        cache_records
    );
    assert_eq!(
        output
            .projections
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        projection_records
    );
    assert_eq!(
        output
            .trims
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        trim_records
    );
    assert_eq!(
        output
            .regularizations
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        regularization_records
    );
    assert_eq!(
        output
            .filters
            .iter()
            .map(|object| object.records.as_ptr() as usize)
            .collect::<Vec<_>>(),
        filter_records
    );
    vertical_shell_assignment::dispose(output);
}

fn record_field_allocations(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> Vec<Option<[usize; 5]>> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .map(|record| {
            record.as_ref().map(|record| {
                [
                    record.perimeters.as_ptr() as usize,
                    record.thin_fills.as_ptr() as usize,
                    record.slices.as_ptr() as usize,
                    record.fill_expolygons.as_ptr() as usize,
                    record.fill_no_overlap_expolygons.as_ptr() as usize,
                ]
            })
        })
        .collect()
}
