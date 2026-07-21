use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats,
    geometry::{CoordinateScale, ExPolygon},
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        compensation::{PostCompensationPrintObject, apply_project_compensation},
        perimeters::{
            context::prepare_perimeter_contexts,
            preflight::preflight_perimeter_flows,
            types::{Flow, PerimeterInputRecord, PostPerimeterInputPrintObject},
        },
        region_slices::RegionSurface,
    },
};

use super::{
    super::support::{resolved_object, transform},
    fixture::{Case, case, flow_options, snapshot, split},
};

#[test]
fn task22n_context_preserves_slots_identity_adjacency_flows_and_owned_state() {
    let (region, object) = flow_options();
    let (objects, resolved) = split(vec![
        case(
            10,
            region.clone(),
            object.clone(),
            &[],
            CoordinateScale::Normal,
        ),
        case(
            11,
            region.clone(),
            object.clone(),
            &[(0.2, 1)],
            CoordinateScale::Normal,
        ),
        case(
            12,
            region,
            object,
            &[(0.2, 2), (0.3, 1), (0.4, 0)],
            CoordinateScale::LargeBed,
        ),
    ]);
    let before = snapshot(&objects, &resolved);
    let before_metadata = surface_metadata(&objects);
    let outputs = prepare_objects(objects, &resolved, false);

    assert!(outputs[0].as_parts().1.is_empty());
    let one = outputs[1].as_parts().1[0].as_ref().unwrap();
    assert_identity(one, [11, 0, 0, 0, 0]);
    assert_eq!(one.compatible_region_ids, [0]);
    assert_eq!(one.current.region_index, 0);
    assert_eq!(one.current.layer_index, 0);
    assert_eq!(outputs[1].current_surfaces(one).len(), 1);
    assert!(outputs[1].lower_slices(one).is_none());
    assert!(outputs[1].upper_slices(one).is_none());
    assert!(outputs[1].upper_same_region_surfaces(one).is_none());

    let wrapper = &outputs[2];
    let records = wrapper.as_parts().1;
    assert_eq!(
        records.iter().map(Option::is_some).collect::<Vec<_>>(),
        [true, true, false]
    );
    let first = records[0].as_ref().unwrap();
    let later = records[1].as_ref().unwrap();
    assert_identity(first, [12, 0, 0, 0, 0]);
    assert_identity(later, [12, 0, 1, 1, 0]);
    assert_eq!(
        (first.lower_layer_index, first.upper_layer_index),
        (None, Some(1))
    );
    assert_eq!(
        (later.lower_layer_index, later.upper_layer_index),
        (Some(0), Some(2))
    );
    assert_eq!(first.upper_same_region.unwrap().layer_index, 1);
    assert_eq!(later.upper_same_region.unwrap().layer_index, 2);
    assert_eq!(
        [first.layer_height.to_bits(), first.slice_z.to_bits()],
        [0x3fc999999999999a, 0x3fb999999999999a]
    );
    assert_eq!(
        [later.layer_height.to_bits(), later.slice_z.to_bits()],
        [0x3fd3333333333333, 0x3fd6666666666666]
    );

    assert_flow(
        first.perimeter_flow,
        [0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd],
        0x3fb76708c0000000,
    );
    assert_flow(
        first.ext_perimeter_flow,
        [0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd],
        0x3fb76708c0000000,
    );
    assert_flow(
        first.overhang_flow,
        [0x3ecccccd, 0x3e4ccccd, 0x3eb6d324, 0x3ecccccd],
        0x3fb2485080000000,
    );
    assert_flow(
        first.solid_infill_flow,
        [0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd],
        0x3fb76708c0000000,
    );
    assert_flow(
        later.ext_perimeter_flow,
        [0x3ed70a3d, 0x3e99999a, 0x3eb613c0, 0x3ecccccd],
        0x3fbb4fc340000000,
    );

    let (owned, _) = wrapper.as_parts();
    let (post_region, lslices) = owned.as_parts();
    let (_, _, regions) = post_region.as_parts();
    let layers = regions[0].as_parts().2;
    assert_surfaces(wrapper.current_surfaces(first), layers[0].surfaces());
    assert_surfaces(
        wrapper.upper_same_region_surfaces(first).unwrap(),
        layers[1].surfaces(),
    );
    assert_eq!(wrapper.upper_slices(first).unwrap(), lslices[1]);
    assert_eq!(wrapper.lower_slices(later).unwrap(), lslices[0]);
    assert!(
        wrapper
            .upper_slices(later)
            .is_some_and(<[ExPolygon]>::is_empty)
    );
    assert!(
        wrapper
            .upper_same_region_surfaces(later)
            .is_some_and(<[RegionSurface]>::is_empty)
    );

    let restored = outputs
        .into_iter()
        .map(|output| output.into_parts().0)
        .collect::<Vec<_>>();
    assert_eq!(snapshot(&restored, &resolved), before);
    assert_eq!(surface_metadata(&restored), before_metadata);
}

#[test]
fn task22n_context_empty_middle_keeps_adjacent_empty_slots() {
    let (region, object) = flow_options();
    let outputs = prepare_cases(
        vec![case(
            20,
            region,
            object,
            &[(0.2, 1), (0.2, 0), (0.2, 2)],
            CoordinateScale::Normal,
        )],
        false,
    );
    let wrapper = &outputs[0];
    let records = wrapper.as_parts().1;
    assert_eq!(
        records.iter().map(Option::is_some).collect::<Vec<_>>(),
        [true, false, true]
    );
    let first = records[0].as_ref().unwrap();
    let last = records[2].as_ref().unwrap();
    assert_eq!(first.upper_layer_index, Some(1));
    assert!(
        wrapper
            .upper_slices(first)
            .is_some_and(<[ExPolygon]>::is_empty)
    );
    assert!(
        wrapper
            .upper_same_region_surfaces(first)
            .is_some_and(<[RegionSurface]>::is_empty)
    );
    assert_eq!(last.lower_layer_index, Some(1));
    assert!(
        wrapper
            .lower_slices(last)
            .is_some_and(<[ExPolygon]>::is_empty)
    );
}

#[test]
fn task22n_context_resolvers_preserve_complete_ordered_surface_collections() {
    let (region, object) = flow_options();
    let outputs = prepare_cases(
        vec![case(
            21,
            region,
            object,
            &[(0.2, 2), (0.2, 2), (0.2, 1)],
            CoordinateScale::Normal,
        )],
        false,
    );
    let wrapper = &outputs[0];
    let records = wrapper.as_parts().1;
    let (owned, _) = wrapper.as_parts();
    let (post_region, lslices) = owned.as_parts();
    let layers = post_region.as_parts().2[0].as_parts().2;

    for (layer_index, record) in records.iter().enumerate() {
        let record = record.as_ref().unwrap();
        assert_surfaces(
            wrapper.current_surfaces(record),
            layers[layer_index].surfaces(),
        );
        if let Some(upper) = record.upper_layer_index {
            assert_eq!(wrapper.upper_slices(record).unwrap(), lslices[upper]);
            assert_surfaces(
                wrapper.upper_same_region_surfaces(record).unwrap(),
                layers[upper].surfaces(),
            );
        }
    }
}

#[test]
fn task22n_context_alignment_and_occurrences_use_matching_transform_columns() {
    let (mut disabled_region, object) = flow_options();
    disabled_region.align_infill_direction_to_model = OrcaBool(false);
    let mut disabled = case(
        30,
        disabled_region,
        object,
        &[(0.2, 1)],
        CoordinateScale::Normal,
    );
    disabled.resolved.print_objects[0].transform = transform("0 1 0 -1 0 0 0 0 1 7 8 9");

    let (mut signed_region, object) = flow_options();
    signed_region.align_infill_direction_to_model = OrcaBool(true);
    let mut signed = case(
        31,
        signed_region,
        object,
        &[(0.2, 1)],
        CoordinateScale::Normal,
    );
    signed.resolved.print_objects[0].transform = transform("1 -0 0 0 1 0 0 0 1 7 8 9");
    let outputs = prepare_cases(vec![disabled, signed], false);
    assert_eq!(
        outputs[0].as_parts().1[0]
            .as_ref()
            .unwrap()
            .model_rotation_rad
            .to_bits(),
        0
    );
    assert_eq!(
        outputs[1].as_parts().1[0]
            .as_ref()
            .unwrap()
            .model_rotation_rad
            .to_bits(),
        0x8000000000000000
    );

    let occurrences = occurrence_outputs();
    let first = occurrences[0].as_parts().1[0].as_ref().unwrap();
    let second = occurrences[1].as_parts().1[0].as_ref().unwrap();
    assert_eq!([first.source_object_index, first.transform_index], [200, 0]);
    assert_eq!(
        [second.source_object_index, second.transform_index],
        [200, 1]
    );
    assert_eq!(first.model_rotation_rad.to_bits(), 0);
    assert_eq!(second.model_rotation_rad.to_bits(), 0x400921fb54442d18);
}

fn prepare_cases(cases: Vec<Case>, spiral_mode: bool) -> Vec<PostPerimeterInputPrintObject> {
    let (objects, resolved) = split(cases);
    prepare_objects(objects, &resolved, spiral_mode)
}

fn prepare_objects(
    objects: Vec<PostCompensationPrintObject>,
    resolved: &[ResolvedProjectObject],
    spiral_mode: bool,
) -> Vec<PostPerimeterInputPrintObject> {
    let flows = preflight_perimeter_flows(
        &objects,
        resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap();
    prepare_perimeter_contexts(objects, flows, resolved, spiral_mode)
}

fn occurrence_outputs() -> Vec<PostPerimeterInputPrintObject> {
    let (mut region, mut object) = flow_options();
    region.align_infill_direction_to_model = OrcaBool(true);
    object.elefant_foot_compensation = OrcaFloat(0.0);
    let first = case(
        200,
        region.clone(),
        object.clone(),
        &[(0.2, 1)],
        CoordinateScale::Normal,
    );
    let second = case(
        200,
        region,
        object.clone(),
        &[(0.2, 1)],
        CoordinateScale::Normal,
    );
    let (first, _) = first.object.into_parts();
    let (mut second, _) = second.object.into_parts();
    second.plan.transform_index = 1;
    let mut resolved = resolved_object(
        200,
        &[
            transform("1 0 0 0 1 0 0 0 1 0 0 0"),
            transform("-1 0 0 0 -1 0 0 0 1 0 0 0"),
        ],
    );
    resolved.object = object;
    let objects = apply_project_compensation(
        vec![first, second],
        std::slice::from_ref(&resolved),
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
        CoordinateScale::Normal,
    )
    .unwrap();
    prepare_objects(objects, std::slice::from_ref(&resolved), false)
}

fn assert_identity(record: &PerimeterInputRecord, expected: [usize; 5]) {
    assert_eq!(
        [
            record.source_object_index,
            record.transform_index,
            record.planned_layer_index,
            record.layer_id,
            record.region_id
        ],
        expected
    );
}

fn assert_flow(flow: Flow, fields: [u32; 4], mm3_per_mm: u64) {
    assert_eq!(
        [
            flow.width.to_bits(),
            flow.height.to_bits(),
            flow.spacing.to_bits(),
            flow.nozzle_diameter.to_bits()
        ],
        fields
    );
    assert!(!flow.bridge);
    assert_eq!(flow.mm3_per_mm.to_bits(), mm3_per_mm);
}

fn assert_surfaces(actual: &[RegionSurface], expected: &[RegionSurface]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        let (ak, ap, at, atl, ab, ae) = actual.as_parts();
        let (ek, ep, et, etl, eb, ee) = expected.as_parts();
        assert_eq!(
            (ak, ap, at.to_bits(), atl, ab.to_bits(), ae),
            (ek, ep, et.to_bits(), etl, eb.to_bits(), ee)
        );
    }
}

fn surface_metadata(objects: &[PostCompensationPrintObject]) -> Vec<[u64; 4]> {
    objects
        .iter()
        .flat_map(|object| object.as_parts().0.as_parts().2)
        .flat_map(|region| region.as_parts().2)
        .flat_map(|layer| layer.surfaces())
        .map(|surface| {
            let (_, _, thickness, layers, bridge, extra) = surface.as_parts();
            [
                thickness.to_bits(),
                u64::from(layers),
                bridge.to_bits(),
                u64::from(extra),
            ]
        })
        .collect()
}
