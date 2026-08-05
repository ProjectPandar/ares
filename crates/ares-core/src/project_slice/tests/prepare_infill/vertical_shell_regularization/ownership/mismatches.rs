use crate::project_slice::{
    prepare_infill::{vertical_shell_regularization, vertical_shell_trimming},
    tests::support::KsrArchive,
};

use super::super::fixture;

#[test]
fn task22o22_every_outer_vector_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects.pop();
    rejects(objects);
    let mut caches = prepared();
    caches.caches.pop();
    rejects(caches);
    let mut projections = prepared();
    projections.projections.pop();
    rejects(projections);
    let mut trims = prepared();
    trims.trims.pop();
    rejects(trims);
    let mut classic = prepared();
    classic.predecessor.objects.pop();
    rejects(classic);
}

#[test]
fn task22o22_every_record_count_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects[0].records.pop();
    rejects(objects);
    let mut caches = prepared();
    caches.caches[0].records.pop();
    rejects(caches);
    let mut projections = prepared();
    projections.projections[0].records.pop();
    rejects(projections);
    let mut trims = prepared();
    trims.trims[0].records.pop();
    rejects(trims);
    let mut inputs = prepared();
    inputs.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records
        .pop();
    rejects(inputs);
    let mut flows = prepared();
    flows.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .pop();
    rejects(flows);
    let mut plan = prepared();
    plan.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .layers
        .pop();
    rejects(plan);
    let mut lslices = prepared();
    lslices.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .1
        .pop();
    rejects(lslices);
}

#[test]
fn task22o22_every_per_slot_presence_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects[0].records[1] = None;
    rejects(objects);
    let mut caches = prepared();
    caches.caches[0].records[1] = None;
    rejects(caches);
    let mut projections = prepared();
    projections.projections[0].records[1] = None;
    rejects(projections);
    let mut trims = prepared();
    trims.trims[0].records[1] = None;
    rejects(trims);
    let mut inputs = prepared();
    inputs.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[1] = None;
    rejects(inputs);
    let mut flows = prepared();
    flows.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records[1] = None;
    rejects(flows);
}

#[test]
fn task22o22_every_input_identity_mismatch_rejects_before_geometry() {
    for mutate in [
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.source_object_index += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.transform_index += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.planned_layer_index += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.layer_id += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.region_id += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.compatible_region_ids[0] += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.current.layer_index += 1
        },
        |input: &mut crate::project_slice::perimeters::types::PerimeterInputRecord| {
            input.current.region_index += 1
        },
    ] {
        let mut prepared = prepared();
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
        rejects(prepared);
    }
}

#[test]
fn task22o22_plan_source_and_transform_identity_mismatches_reject_before_geometry() {
    let mut source = prepared();
    source.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .source_object_index += 1;
    rejects(source);

    let mut transform = prepared();
    transform.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .transform_index += 1;
    rejects(transform);
}

fn prepared() -> vertical_shell_trimming::PreparedPostVerticalShellTrim {
    fixture::prepare_o21(KsrArchive::new().bytes())
}

fn rejects(prepared: vertical_shell_trimming::PreparedPostVerticalShellTrim) {
    vertical_shell_regularization::reset_geometry_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = vertical_shell_regularization::prepare(prepared);
        }))
        .is_err()
    );
    assert!(vertical_shell_regularization::geometry_events().is_empty());
}
