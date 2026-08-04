mod snapshots;

use crate::project_slice::{
    prepare_infill::vertical_shell_projection,
    tests::{prepare_infill::fill_surfaces::ownership::allocation_snapshot, support::KsrArchive},
};

use self::snapshots::{
    cache_point_buffers, cache_snapshot, lslice_point_buffers, predecessor_snapshot,
};
use super::fixture;

#[test]
fn task22o20_moves_every_o19_allocation_and_projection_paths_are_fresh() {
    let input = fixture::prepare_o19(KsrArchive::new().bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let objects = allocation_snapshot(&input.objects);
    let caches = cache_snapshot(&input.caches);
    let source = predecessor_snapshot(&input.predecessor);
    let cache_points = cache_point_buffers(&input.caches);
    let source_points = lslice_point_buffers(&input.predecessor);
    let output = vertical_shell_projection::prepare(input).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(allocation_snapshot(&output.objects), objects);
    assert_eq!(cache_snapshot(&output.caches), caches);
    assert_eq!(predecessor_snapshot(&output.predecessor), source);
    for projection in output.projections[0].records.iter().flatten() {
        for path in projection.shell.iter().chain(&projection.holes) {
            let points = path.points().as_ptr() as usize;
            assert!(!cache_points.contains(&points));
            assert!(!source_points.contains(&points));
        }
    }
}

#[test]
fn task22o20_current_none_stays_none_without_shifting_neighbors() {
    let mut input = fixture::prepare_o19(KsrArchive::new().bytes());
    input.objects[0].records[1] = None;
    input.caches[0].records[1] = None;
    let prelude = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[1] = None;
    prelude.records[1] = None;
    let output = vertical_shell_projection::prepare(input).unwrap();
    assert!(output.projections[0].records[0].is_some());
    assert!(output.projections[0].records[1].is_none());
    assert!(output.projections[0].records[2].is_some());
}

#[test]
fn task22o20_alignment_and_identity_fail_before_any_geometry() {
    let mut count = fixture::prepare_o19(KsrArchive::new().bytes());
    count.caches[0].records.pop();
    assert_preflight_rejects(count);

    let mut identity = fixture::prepare_o19(KsrArchive::new().bytes());
    let prelude = &mut identity.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[0]
        .as_mut()
        .unwrap()
        .planned_layer_index = 1;
    assert_preflight_rejects(identity);
}

#[test]
fn task22o20_outer_object_and_surface_record_counts_fail_before_geometry() {
    let mut outer = fixture::prepare_o19(KsrArchive::new().bytes());
    outer.caches.pop();
    assert_preflight_rejects(outer);

    let mut records = fixture::prepare_o19(KsrArchive::new().bytes());
    records.objects[0].records.pop();
    assert_preflight_rejects(records);
}

#[test]
fn task22o20_input_and_prelude_record_counts_fail_before_geometry() {
    let mut inputs = fixture::prepare_o19(KsrArchive::new().bytes());
    inputs.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records
        .pop();
    assert_preflight_rejects(inputs);

    let mut flows = fixture::prepare_o19(KsrArchive::new().bytes());
    flows.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .pop();
    assert_preflight_rejects(flows);
}

#[test]
fn task22o20_slot_presence_mismatch_fails_before_geometry() {
    let mut input = fixture::prepare_o19(KsrArchive::new().bytes());
    input.caches[0].records[1] = None;
    assert_preflight_rejects(input);
}

#[test]
fn task22o20_plan_and_lslice_counts_fail_before_geometry() {
    let mut plan = fixture::prepare_o19(KsrArchive::new().bytes());
    let prelude = &mut plan.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.object.as_parts_mut().0.plan.layers.pop();
    assert_preflight_rejects(plan);

    let mut lslices = fixture::prepare_o19(KsrArchive::new().bytes());
    let prelude = &mut lslices.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.object.as_parts_mut().1.pop();
    assert_preflight_rejects(lslices);
}

#[test]
fn task22o20_source_and_transform_identities_fail_before_geometry() {
    reject_input_mutation(|input| input.source_object_index += 1);
    reject_input_mutation(|input| input.transform_index += 1);
}

#[test]
fn task22o20_region_and_compatibility_identities_fail_before_geometry() {
    reject_input_mutation(|input| input.region_id += 1);
    reject_input_mutation(|input| input.compatible_region_ids[0] += 1);
}

#[test]
fn task22o20_layer_and_current_identities_fail_before_geometry() {
    reject_input_mutation(|input| input.layer_id += 1);
    reject_input_mutation(|input| input.current.layer_index += 1);
    reject_input_mutation(|input| input.current.region_index += 1);
}

fn reject_input_mutation(
    mutate: impl FnOnce(&mut crate::project_slice::perimeters::types::PerimeterInputRecord),
) {
    let mut prepared = fixture::prepare_o19(KsrArchive::new().bytes());
    let input = prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[0]
        .as_mut()
        .unwrap();
    mutate(input);
    assert_preflight_rejects(prepared);
}

fn assert_preflight_rejects(
    prepared: crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
) {
    vertical_shell_projection::reset_geometry_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = vertical_shell_projection::prepare(prepared);
        }))
        .is_err()
    );
    assert!(vertical_shell_projection::geometry_events().is_empty());
}
