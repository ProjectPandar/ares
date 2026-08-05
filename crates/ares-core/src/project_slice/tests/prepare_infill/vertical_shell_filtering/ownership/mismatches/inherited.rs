use super::{prepared, rejects};

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
fn task22o23_every_inherited_record_count_mismatch_rejects_before_geometry() {
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
    let mut regularizations = prepared();
    regularizations.regularizations[0].records.pop();
    rejects(regularizations);
    let mut traversal = prepared();
    traversal.predecessor.objects[0].records.pop();
    rejects(traversal);
    let mut inputs = prepared();
    prelude!(&mut inputs).object.records.pop();
    rejects(inputs);
    let mut flows = prepared();
    prelude!(&mut flows).records.pop();
    rejects(flows);
    let mut plan = prepared();
    prelude!(&mut plan)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .layers
        .pop();
    rejects(plan);
    let mut lslices = prepared();
    prelude!(&mut lslices).object.object.as_parts_mut().1.pop();
    rejects(lslices);
}

#[test]
fn task22o23_every_slot_presence_mismatch_rejects_before_geometry() {
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
    let mut regularizations = prepared();
    regularizations.regularizations[0].records[1] = None;
    rejects(regularizations);
    let mut traversal = prepared();
    traversal.predecessor.objects[0].records[1] = None;
    rejects(traversal);
    let mut inputs = prepared();
    prelude!(&mut inputs).object.records[1] = None;
    rejects(inputs);
    let mut flows = prepared();
    prelude!(&mut flows).records[1] = None;
    rejects(flows);
}

#[test]
fn task22o23_every_input_and_plan_identity_mismatch_rejects_before_geometry() {
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
        rejects(input);
    }

    let mut source = prepared();
    prelude!(&mut source)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .source_object_index += 1;
    rejects(source);
    let mut transform = prepared();
    prelude!(&mut transform)
        .object
        .object
        .as_parts_mut()
        .0
        .plan
        .transform_index += 1;
    rejects(transform);
    let mut regions = prepared();
    prelude!(&mut regions)
        .object
        .object
        .as_parts_mut()
        .0
        .regions
        .pop();
    rejects(regions);
}
