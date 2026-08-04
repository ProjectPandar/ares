mod mismatches;
mod snapshots;

use crate::project_slice::{
    prepare_infill::vertical_shell_trimming,
    tests::{prepare_infill::fill_surfaces::ownership::allocation_snapshot, support::KsrArchive},
};

use self::snapshots::{
    cache_snapshot, predecessor_geometry_point_buffers, predecessor_snapshot, projection_snapshot,
};
use super::fixture;

#[test]
fn task22o21_moves_complete_o20_graph_and_trim_paths_are_fresh() {
    let input = fixture::prepare_o20(KsrArchive::new().bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let classic = predecessor_snapshot(&input.predecessor);
    let objects = allocation_snapshot(&input.objects);
    let caches = cache_snapshot(&input.caches);
    let projections = projection_snapshot(&input.projections);
    let predecessor_points = predecessor_geometry_point_buffers(
        &input.predecessor,
        &input.objects,
        &input.caches,
        &input.projections,
    );
    let output = vertical_shell_trimming::prepare(input).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(predecessor_snapshot(&output.predecessor), classic);
    assert_eq!(allocation_snapshot(&output.objects), objects);
    assert_eq!(cache_snapshot(&output.caches), caches);
    assert_eq!(projection_snapshot(&output.projections), projections);
    for path in output
        .trims
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|trim| &trim.shell)
    {
        assert!(!predecessor_points.contains(&(path.points().as_ptr() as usize)));
    }
}

#[test]
fn task22o21_current_none_stays_none_without_shifting_neighbors() {
    let mut input = fixture::prepare_o20(KsrArchive::new().bytes());
    input.objects[0].records[1] = None;
    input.caches[0].records[1] = None;
    input.projections[0].records[1] = None;
    let prelude = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[1] = None;
    prelude.records[1] = None;
    vertical_shell_trimming::reset_geometry_hooks();
    let output = vertical_shell_trimming::prepare(input).unwrap();
    assert!(output.trims[0].records[0].is_some());
    assert!(output.trims[0].records[1].is_none());
    assert!(output.trims[0].records[2].is_some());
}
