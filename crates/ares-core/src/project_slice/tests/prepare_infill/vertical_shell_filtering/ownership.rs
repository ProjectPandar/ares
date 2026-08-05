mod mismatches;

use crate::project_slice::{
    prepare_infill::vertical_shell_filtering,
    tests::{
        prepare_infill::{
            fill_surfaces::ownership::allocation_snapshot,
            vertical_shell_regularization::ownership::snapshots::{
                cache_snapshot, projection_snapshot, regularization_allocations, trim_snapshot,
            },
        },
        support::KsrArchive,
    },
};

#[test]
fn task22o23_moves_exact_o22_graph_and_creates_fresh_survivor_storage() {
    let input = super::fixture::prepare_o22(KsrArchive::new().bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let classic = super::super::vertical_shell_projection::predecessor_snapshot(&input.predecessor);
    let object_allocations = allocation_snapshot(&input.objects);
    let objects = input.objects.as_ptr();
    let caches = input.caches.as_ptr();
    let projections = input.projections.as_ptr();
    let trims = input.trims.as_ptr();
    let regularizations = input.regularizations.as_ptr();
    let cache_allocations = cache_snapshot(&input.caches);
    let projection_allocations = projection_snapshot(&input.projections);
    let trim_allocations = trim_snapshot(&input.trims);
    let regularization_snapshot = regularization_allocations(&input.regularizations);
    let source_points = input
        .regularizations
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.regularized_shell)
        .flat_map(|expolygon| {
            std::iter::once(expolygon.contour().points().as_ptr())
                .chain(expolygon.holes().iter().map(|hole| hole.points().as_ptr()))
        })
        .collect::<Vec<_>>();

    let output = vertical_shell_filtering::prepare(input).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        super::super::vertical_shell_projection::predecessor_snapshot(&output.predecessor),
        classic
    );
    assert_eq!(allocation_snapshot(&output.objects), object_allocations);
    assert_eq!(output.objects.as_ptr(), objects);
    assert_eq!(output.caches.as_ptr(), caches);
    assert_eq!(output.projections.as_ptr(), projections);
    assert_eq!(output.trims.as_ptr(), trims);
    assert_eq!(output.regularizations.as_ptr(), regularizations);
    assert_eq!(cache_snapshot(&output.caches), cache_allocations);
    assert_eq!(
        projection_snapshot(&output.projections),
        projection_allocations
    );
    assert_eq!(trim_snapshot(&output.trims), trim_allocations);
    assert_eq!(
        regularization_allocations(&output.regularizations),
        regularization_snapshot
    );
    let retained_allocations = classic
        .iter()
        .chain(&object_allocations)
        .chain(&cache_allocations)
        .chain(&projection_allocations)
        .chain(&trim_allocations)
        .chain(&regularization_snapshot)
        .copied()
        .collect::<Vec<_>>();
    for allocation in filter_allocations(&output.filters) {
        assert!(!retained_allocations.contains(&allocation));
    }
    for points in output
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.filtered_shell)
        .flat_map(|expolygon| {
            std::iter::once(expolygon.contour().points().as_ptr())
                .chain(expolygon.holes().iter().map(|hole| hole.points().as_ptr()))
        })
    {
        assert!(!source_points.contains(&points));
    }
    vertical_shell_filtering::dispose(output);
}

fn filter_allocations(
    objects: &[vertical_shell_filtering::types::VerticalShellTinyFilterObject],
) -> Vec<usize> {
    let mut allocations = vec![objects.as_ptr() as usize];
    for object in objects {
        allocations.push(object.records.as_ptr() as usize);
        for filter in object.records.iter().flatten() {
            if !filter.filtered_shell.is_empty() {
                allocations.push(filter.filtered_shell.as_ptr() as usize);
            }
            for expolygon in &filter.filtered_shell {
                append_expolygon_allocations(&mut allocations, expolygon);
            }
        }
    }
    allocations
}

fn append_expolygon_allocations(
    allocations: &mut Vec<usize>,
    expolygon: &crate::geometry::ExPolygon,
) {
    allocations.push(expolygon.contour().points().as_ptr() as usize);
    if !expolygon.holes().is_empty() {
        allocations.push(expolygon.holes().as_ptr() as usize);
    }
    allocations.extend(
        expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().as_ptr() as usize),
    );
}

#[test]
fn task22o23_aligned_none_stays_none_without_changing_lslice_neighbors() {
    let mut input = super::fixture::prepare_o22(KsrArchive::new().bytes());
    input.objects[0].records[1] = None;
    input.caches[0].records[1] = None;
    input.projections[0].records[1] = None;
    input.trims[0].records[1] = None;
    input.regularizations[0].records[1] = None;
    let traversal = &mut input.predecessor.objects[0];
    traversal.records[1] = None;
    let prelude = &mut traversal.predecessor.predecessor.predecessor.predecessor;
    prelude.object.records[1] = None;
    prelude.records[1] = None;
    let output = vertical_shell_filtering::prepare(input).unwrap();
    assert!(output.filters[0].records[0].is_some());
    assert!(output.filters[0].records[1].is_none());
    assert!(output.filters[0].records[2].is_some());
    vertical_shell_filtering::dispose(output);
}
