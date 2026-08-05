use super::{prepared, rejects_alignment};
use crate::geometry::CoordinateScale;

macro_rules! prelude {
    ($prepared:expr) => {
        &mut ($prepared).predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
    };
}

#[test]
fn task22o24_retained_scale_mismatch_rejects_before_geometry() {
    let mut input = prepared();
    input.predecessor.scale = match input.predecessor.scale {
        CoordinateScale::Normal => CoordinateScale::LargeBed,
        CoordinateScale::LargeBed => CoordinateScale::Normal,
    };
    rejects_alignment(input);
}

#[test]
fn task22o24_every_outer_alignment_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects.pop();
    rejects_alignment(objects);
    let mut caches = prepared();
    caches.caches.pop();
    rejects_alignment(caches);
    let mut projections = prepared();
    projections.projections.pop();
    rejects_alignment(projections);
    let mut trims = prepared();
    trims.trims.pop();
    rejects_alignment(trims);
    let mut regularizations = prepared();
    regularizations.regularizations.pop();
    rejects_alignment(regularizations);
    let mut filters = prepared();
    filters.filters.pop();
    rejects_alignment(filters);
    let mut traversal = prepared();
    traversal.predecessor.objects.pop();
    rejects_alignment(traversal);
}

#[test]
fn task22o24_every_record_count_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects[0].records.pop();
    rejects_alignment(objects);
    let mut caches = prepared();
    caches.caches[0].records.pop();
    rejects_alignment(caches);
    let mut projections = prepared();
    projections.projections[0].records.pop();
    rejects_alignment(projections);
    let mut trims = prepared();
    trims.trims[0].records.pop();
    rejects_alignment(trims);
    let mut regularizations = prepared();
    regularizations.regularizations[0].records.pop();
    rejects_alignment(regularizations);
    let mut filters = prepared();
    filters.filters[0].records.pop();
    rejects_alignment(filters);
    let mut traversal = prepared();
    traversal.predecessor.objects[0].records.pop();
    rejects_alignment(traversal);
    let mut inputs = prepared();
    prelude!(&mut inputs).object.records.pop();
    rejects_alignment(inputs);
    let mut flows = prepared();
    prelude!(&mut flows).records.pop();
    rejects_alignment(flows);
    let mut plan = prepared();
    prelude!(&mut plan)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .layers
        .pop();
    rejects_alignment(plan);
    let mut lslices = prepared();
    prelude!(&mut lslices).object.object.as_parts_mut().1.pop();
    rejects_alignment(lslices);
}

#[test]
fn task22o24_every_slot_presence_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects[0].records[1] = None;
    rejects_alignment(objects);
    let mut caches = prepared();
    caches.caches[0].records[1] = None;
    rejects_alignment(caches);
    let mut projections = prepared();
    projections.projections[0].records[1] = None;
    rejects_alignment(projections);
    let mut trims = prepared();
    trims.trims[0].records[1] = None;
    rejects_alignment(trims);
    let mut regularizations = prepared();
    regularizations.regularizations[0].records[1] = None;
    rejects_alignment(regularizations);
    let mut filters = prepared();
    filters.filters[0].records[1] = None;
    rejects_alignment(filters);
    let mut traversal = prepared();
    traversal.predecessor.objects[0].records[1] = None;
    rejects_alignment(traversal);
    let mut inputs = prepared();
    prelude!(&mut inputs).object.records[1] = None;
    rejects_alignment(inputs);
    let mut flows = prepared();
    prelude!(&mut flows).records[1] = None;
    rejects_alignment(flows);
}

#[test]
fn task22o24_every_input_and_plan_identity_mismatch_rejects_before_geometry() {
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
        let mut input = prepared();
        mutate(prelude!(&mut input).object.records[0].as_mut().unwrap());
        rejects_alignment(input);
    }

    let mut source = prepared();
    prelude!(&mut source)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .source_object_index += 1;
    rejects_alignment(source);
    let mut transform = prepared();
    prelude!(&mut transform)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .transform_index += 1;
    rejects_alignment(transform);
    let mut regions = prepared();
    prelude!(&mut regions)
        .object
        .object
        .as_parts_mut()
        .0
        .regions
        .pop();
    rejects_alignment(regions);
}
