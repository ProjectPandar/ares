mod mismatches;
pub(in crate::project_slice::tests::prepare_infill) mod snapshots;

use crate::project_slice::{
    prepare_infill::vertical_shell_regularization,
    tests::{prepare_infill::fill_surfaces::ownership::allocation_snapshot, support::KsrArchive},
};

use self::snapshots::{
    all_predecessor_points, cache_snapshot, projection_snapshot, regularization_allocations,
    regularization_point_buffers, trim_snapshot,
};
use super::super::vertical_shell_projection::predecessor_snapshot;
use super::fixture;

#[test]
fn task22o22_moves_complete_o21_graph_and_regularized_geometry_is_fresh() {
    let input = fixture::prepare_o21(KsrArchive::new().bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let classic = predecessor_snapshot(&input.predecessor);
    let objects = allocation_snapshot(&input.objects);
    let caches = cache_snapshot(&input.caches);
    let projections = projection_snapshot(&input.projections);
    let trims = trim_snapshot(&input.trims);
    let predecessor_points = all_predecessor_points(&input);
    let predecessor_allocations = classic
        .iter()
        .chain(&objects)
        .chain(&caches)
        .chain(&projections)
        .chain(&trims)
        .copied()
        .collect::<Vec<_>>();

    let output = vertical_shell_regularization::prepare(input).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(predecessor_snapshot(&output.predecessor), classic);
    assert_eq!(allocation_snapshot(&output.objects), objects);
    assert_eq!(cache_snapshot(&output.caches), caches);
    assert_eq!(projection_snapshot(&output.projections), projections);
    assert_eq!(trim_snapshot(&output.trims), trims);
    for allocation in regularization_allocations(&output.regularizations) {
        assert!(!predecessor_allocations.contains(&allocation));
    }
    for point_buffer in regularization_point_buffers(&output.regularizations) {
        assert!(!predecessor_points.contains(&point_buffer));
    }
}

#[test]
fn task22o22_current_none_stays_none_without_shifting_neighbors() {
    let mut input = fixture::prepare_o21(KsrArchive::new().bytes());
    input.objects[0].records[1] = None;
    input.caches[0].records[1] = None;
    input.projections[0].records[1] = None;
    input.trims[0].records[1] = None;
    let prelude = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[1] = None;
    prelude.records[1] = None;
    vertical_shell_regularization::reset_geometry_hooks();
    let output = vertical_shell_regularization::prepare(input).unwrap();
    assert!(output.regularizations[0].records[0].is_some());
    assert!(output.regularizations[0].records[1].is_none());
    assert!(output.regularizations[0].records[2].is_some());
}
