mod options;

use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, Percent, ProcessFuzzySkinType,
    ProcessPerimeterGenerator, ProjectVolumeType, load_project,
    project::effective_config::resolve_bounded_project_config,
    project_slice::{task22n_browser_input_oracle, task22n_browser_oracle},
};

use super::super::oracle::{NFrame, NRecord, WireFlow, parse_n};
use crate::project_slice::tests::support::ksr_project;

const INITIAL: ([u32; 4], u64) = (
    [0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd],
    0x3fb76708c0000000,
);
const PERIMETER: ([u32; 4], u64) = (
    [0x3ee66666, 0x3e4ccccd, 0x3ed06cbe, 0x3ecccccd],
    0x3fb4d7aca0000000,
);
const EXTERNAL: ([u32; 4], u64) = (
    [0x3ed70a3d, 0x3e4ccccd, 0x3ec11094, 0x3ecccccd],
    0x3fb34e7540000000,
);
const OVERHANG: ([u32; 4], u64) = (
    [0x3ecccccd, 0x3e4ccccd, 0x3eb6d324, 0x3ecccccd],
    0x3fb2485080000000,
);

#[test]
fn task22n_ksr_inventory_freezes_loaded_and_effective_options() {
    let project = load_project(ksr_project()).unwrap();
    let raw = project.settings();
    assert_eq!(
        (
            raw.process.object.wall_generator,
            raw.process.region.wall_loops,
            raw.process.print.spiral_mode,
            raw.process.print.initial_layer_line_width,
        ),
        (
            ProcessPerimeterGenerator::Classic,
            OrcaInt(2),
            OrcaBool(false),
            FloatOrPercent::Float(0.5),
        )
    );
    assert_eq!(
        [
            raw.process.region.outer_wall_line_width,
            raw.process.region.inner_wall_line_width,
            raw.process.region.internal_solid_infill_line_width,
        ],
        [
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.45),
            FloatOrPercent::Float(0.42),
        ]
    );
    assert_eq!(
        [
            raw.process.region.outer_wall_filament_id,
            raw.process.region.inner_wall_filament_id,
            raw.process.region.internal_solid_filament_id,
        ],
        [OrcaInt(0); 3]
    );
    assert_eq!(
        raw.project.print.nozzle_diameter,
        OrcaFloats(vec![OrcaFloat(0.4); 2])
    );
    assert_eq!(
        (
            raw.process.region.bridge_line_width,
            raw.process.region.bridge_flow,
            raw.process.object.thick_bridges,
            raw.process.region.align_infill_direction_to_model,
        ),
        (
            FloatOrPercent::Percent(Percent(100.0)),
            OrcaFloat(1.0),
            OrcaBool(false),
            OrcaBool(false),
        )
    );
    assert_eq!(raw.process.object.interlocking_beam, OrcaBool(false));
    assert_eq!(
        raw.process.object.mmu_segmented_region_max_width,
        OrcaFloat(0.0)
    );
    assert_eq!(
        raw.process.object.mmu_segmented_region_interlocking_depth,
        OrcaFloat(0.0)
    );
    assert_eq!(
        raw.process.region.fuzzy_skin,
        ProcessFuzzySkinType::Disabled
    );

    let resolved = resolve_bounded_project_config(&project).unwrap();
    let [object] = resolved.objects.as_slice() else {
        panic!("one resolved object")
    };
    let [candidate] = object.layer_candidates.as_slice() else {
        panic!("one candidate")
    };
    let [part] = candidate.model_parts.as_slice() else {
        panic!("one model part")
    };
    let effective = &part.region;
    assert_eq!(
        object.object.wall_generator,
        ProcessPerimeterGenerator::Classic
    );
    assert_eq!(effective.wall_loops, OrcaInt(2));
    assert_eq!(
        [
            effective.outer_wall_line_width,
            effective.inner_wall_line_width,
            effective.internal_solid_infill_line_width,
        ],
        [
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.45),
            FloatOrPercent::Float(0.42),
        ]
    );
    assert_eq!(
        [
            effective.outer_wall_filament_id,
            effective.inner_wall_filament_id,
            effective.internal_solid_filament_id,
        ],
        [OrcaInt(1); 3]
    );
    assert_eq!(
        effective.bridge_line_width,
        FloatOrPercent::Percent(Percent(100.0))
    );
    assert_eq!(effective.bridge_flow, OrcaFloat(1.0));
    assert_eq!(effective.align_infill_direction_to_model, OrcaBool(false));
    assert_eq!(effective.fuzzy_skin, ProcessFuzzySkinType::Disabled);

    let [source] = project.objects() else {
        panic!("one project object")
    };
    let [volume] = source.volumes() else {
        panic!("one project volume")
    };
    assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
    assert_eq!(
        [
            volume.mesh().vertices().len(),
            volume.mesh().triangles().len()
        ],
        [6_109, 12_234]
    );
    assert!(
        !source
            .volumes()
            .iter()
            .any(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
    );
}

#[test]
fn task22n_ksr_inventory_exposes_complete_layer_contexts() {
    let m = task22n_browser_input_oracle(ksr_project()).unwrap();
    let n = task22n_browser_oracle(ksr_project()).unwrap();
    assert_eq!(&n[16..16 + m.len()], m);
    let frame = parse_n(&n).unwrap();
    let [(before, slices)] = frame.predecessor.as_slice() else {
        panic!("one M object")
    };
    assert_eq!(
        [
            before.source_object_index,
            before.transform_index,
            before.planned_layer_count
        ],
        [0, 0, 460]
    );
    assert_eq!([before.retained_layers.len(), slices.len()], [460; 2]);
    assert!(
        before
            .retained_layers
            .iter()
            .all(|layer| matches!(layer.regions.as_slice(), [region] if region.id == 0))
    );
    let [object] = frame.objects.as_slice() else {
        panic!("one N object")
    };
    assert_eq!(
        [object.source, object.transform, object.planned],
        [0, 0, 460]
    );
    assert_eq!(
        (object.slots.len(), object.slots.iter().flatten().count()),
        (460, 460)
    );
    for (index, record) in object.slots.iter().map(Option::as_ref).enumerate() {
        let record = record.unwrap();
        let upper = (index + 1 < 460).then_some(index + 1);
        assert_eq!(
            [
                record.source,
                record.transform,
                record.planned,
                record.layer,
                record.region
            ],
            [0, 0, index as u64, index as u64, 0]
        );
        assert_eq!(
            (
                record.compatible.as_slice(),
                record.current,
                record.lower,
                record.upper,
                record.upper_same
            ),
            (
                [0].as_slice(),
                [0, index],
                index.checked_sub(1),
                upper,
                upper.map(|layer| [0, layer])
            )
        );
        assert_eq!(
            (record.spiral, record.rotation, record.dispatch),
            (false, 0, 0)
        );
        assert_ksr_flows(record, index == 0);
    }
    for (index, counts) in [
        (0, [6, 0, 6, 6]),
        (1, [6; 4]),
        (229, [1; 4]),
        (459, [9, 9, 0, 0]),
    ] {
        assert_eq!(
            surface_counts(object.slots[index].as_ref().unwrap()),
            counts
        );
    }
}

#[test]
fn task22n_ksr_inventory_is_repeatable_and_requires_exact_eof() {
    let first = task22n_browser_oracle(ksr_project()).unwrap();
    assert_eq!(task22n_browser_oracle(ksr_project()).unwrap(), first);
    assert!(parse_n(&first).is_ok());
    let mut trailing = first;
    trailing.push(0);
    assert!(parse_n(&trailing).is_err());
}

pub(super) fn assert_n_geometry_matches_predecessor(frame: &NFrame) {
    assert_eq!(frame.objects.len(), frame.predecessor.len());
    for (object, (predecessor, slices)) in frame.objects.iter().zip(&frame.predecessor) {
        assert_eq!(
            [object.source, object.transform, object.planned],
            [
                predecessor.source_object_index,
                predecessor.transform_index,
                predecessor.planned_layer_count,
            ]
        );
        assert_eq!(object.slots.len(), object.planned as usize);
        for (planned, record) in object
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|record| (index, record)))
        {
            assert_eq!(
                (record.source, record.transform, record.planned),
                (object.source, object.transform, planned as u64)
            );
            assert_eq!(
                record.current_surfaces,
                predecessor.retained_layers[record.current[1]].regions[record.current[0]].surfaces
            );
            assert_eq!(
                record.lower_slices.as_ref(),
                record.lower.map(|layer| &slices[layer])
            );
            assert_eq!(
                record.upper_slices.as_ref(),
                record.upper.map(|layer| &slices[layer])
            );
            assert_eq!(
                record.upper_same_surfaces.as_ref(),
                record.upper_same.map(|[region, layer]| {
                    &predecessor.retained_layers[layer].regions[region].surfaces
                })
            );
        }
    }
}

pub(super) fn assert_noncontext_record(before: &NRecord, after: &NRecord) {
    assert_eq!(
        (
            [
                before.source,
                before.transform,
                before.planned,
                before.layer,
                before.region,
            ],
            &before.compatible,
            before.current,
            before.lower,
            before.upper,
            before.upper_same,
            before.height,
            before.slice_z,
        ),
        (
            [
                after.source,
                after.transform,
                after.planned,
                after.layer,
                after.region,
            ],
            &after.compatible,
            after.current,
            after.lower,
            after.upper,
            after.upper_same,
            after.height,
            after.slice_z,
        )
    );
    for (before, after) in before.flows.iter().zip(after.flows) {
        assert_eq!(
            (before.fields, before.bridge, before.mm3_per_mm),
            (after.fields, after.bridge, after.mm3_per_mm)
        );
    }
}

fn assert_ksr_flows(record: &NRecord, first: bool) {
    let (perimeter, external) = if first {
        (INITIAL, INITIAL)
    } else {
        (PERIMETER, EXTERNAL)
    };
    for (flow, expected) in record
        .flows
        .iter()
        .zip([perimeter, external, OVERHANG, external])
    {
        assert_flow(*flow, expected);
    }
}

fn assert_flow(flow: WireFlow, expected: ([u32; 4], u64)) {
    assert_eq!(
        (flow.fields, flow.bridge, flow.mm3_per_mm),
        (expected.0, false, expected.1)
    );
}

fn surface_counts(record: &NRecord) -> [usize; 4] {
    [
        record.current_surfaces.len(),
        record.lower_slices.as_ref().map_or(0, Vec::len),
        record.upper_slices.as_ref().map_or(0, Vec::len),
        record.upper_same_surfaces.as_ref().map_or(0, Vec::len),
    ]
}
