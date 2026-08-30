use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloat, OrcaFloats, OrcaInt, Percent, RegionOptions,
    project_slice::{
        layers::PlannedLayer,
        perimeters::{
            flow::{resolve_external_perimeter_flow, resolve_perimeter_flows},
            types::PerimeterFlows,
        },
    },
};

use super::fixture::flow_options;

mod edge_cases;

type PackedExpectedFlow = (u128, u64);

const FIRST_FLOWS: [PackedExpectedFlow; 4] = [
    (0x3f000000_3e4ccccd_3eea0658_3ecccccd, 0x3fb76708c0000000),
    (0x3f000000_3e4ccccd_3eea0658_3ecccccd, 0x3fb76708c0000000),
    (0x3ecccccd_3e4ccccd_3eb6d324_3ecccccd, 0x3fb2485080000000),
    (0x3f000000_3e4ccccd_3eea0658_3ecccccd, 0x3fb76708c0000000),
];
const LATER_FLOWS: [PackedExpectedFlow; 4] = [
    (0x3ee66666_3e99999a_3ec56fe9_3ecccccd, 0x3fbd9d9640000000),
    (0x3ed70a3d_3e99999a_3eb613c0_3ecccccd, 0x3fbb4fc340000000),
    (0x3ecccccd_3e99999a_3eabd650_3ecccccd, 0x3fb9c68c20000000),
    (0x3ed70a3d_3e99999a_3eb613c0_3ecccccd, 0x3fbb4fc340000000),
];
const PERCENT_FLOWS: [PackedExpectedFlow; 4] = [
    (0x3f28f5c3_3e4ccccd_3f1df8ef_3f19999a, 0x3fbf982fc0000000),
    (0x3f400000_3e4ccccd_3f35032c_3f19999a, 0x3fc219eac0000000),
    (0x3ef5c290_3e4ccccd_3edfc8e8_3f19999a, 0x3fb660e400000000),
    (0x3f19999a_3e4ccccd_3f0e9cc6_3f19999a, 0x3fbc85c120000000),
];

#[test]
fn task22n_flow_record_preserves_fixed_f32_and_volume_bits() {
    let first = resolve(
        layer(0, 0.2),
        [
            FloatOrPercent::Float(0.5),
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.42),
        ],
        1,
        &[0.4],
    );
    assert_bits(
        first,
        0x3f000000_3e4ccccd_3eea0658_3ecccccd,
        0x3fb76708c0000000,
    );
    let later = resolve(
        layer(1, 0.3),
        [
            FloatOrPercent::Float(0.5),
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.42),
        ],
        1,
        &[0.4],
    );
    assert_bits(
        later,
        0x3ed70a3d_3e99999a_3eb613c0_3ecccccd,
        0x3fbb4fc340000000,
    );
}

#[test]
fn task22n_flow_record_percent_uses_selected_f32_nozzle() {
    let flow = resolve(
        layer(0, 0.2),
        [
            FloatOrPercent::Float(0.0),
            FloatOrPercent::Percent(Percent(125.0)),
            FloatOrPercent::Float(0.42),
        ],
        2,
        &[0.4, 0.6],
    );
    assert_bits(
        flow,
        0x3f400000_3e4ccccd_3f35032c_3f19999a,
        0x3fc219eac0000000,
    );
}

#[test]
fn task22n_flow_record_preserves_object_fallback_and_auto_bits() {
    let fallback = resolve(
        layer(0, 0.2),
        [
            FloatOrPercent::Float(0.0),
            FloatOrPercent::Float(0.0),
            FloatOrPercent::Float(0.52),
        ],
        1,
        &[0.4],
    );
    assert_bits(
        fallback,
        0x3f051eb8_3e4ccccd_3ef443c8_3ecccccd,
        0x3fb86d2da0000000,
    );
    let automatic = resolve(
        layer(0, 0.2),
        [
            FloatOrPercent::Float(0.0),
            FloatOrPercent::Float(-0.1),
            FloatOrPercent::Float(0.42),
        ],
        1,
        &[0.4],
    );
    assert_bits(
        automatic,
        0x3ee66667_3e4ccccd_3ed06cbe_3ecccccd,
        0x3fb4d7aca0000000,
    );
    let percent_zero = resolve(
        layer(0, 0.2),
        [
            FloatOrPercent::Percent(Percent(0.0)),
            FloatOrPercent::Percent(Percent(0.0)),
            FloatOrPercent::Percent(Percent(0.0)),
        ],
        1,
        &[0.4],
    );
    assert_bits(
        percent_zero,
        0x3ee66667_3e4ccccd_3ed06cbe_3ecccccd,
        0x3fb4d7aca0000000,
    );
}

#[test]
fn task22n_flow_canonical_increase_else_recomputes_width_and_spacing() {
    let (mut region, mut object) = flow_options();
    region.outer_wall_line_width = FloatOrPercent::Percent(Percent(1000.0));
    region.inner_wall_line_width = FloatOrPercent::Percent(Percent(1000.0));
    region.internal_solid_infill_line_width = FloatOrPercent::Percent(Percent(1000.0));
    region.bridge_line_width = FloatOrPercent::Float(0.0);
    region.bridge_flow = OrcaFloat(1.0000001);
    object.line_width = FloatOrPercent::Percent(Percent(1000.0));
    let flow = resolve_perimeter_flows(
        &layer(1, f64::from(f32::from_bits(0x4113a9f3))),
        FloatOrPercent::Float(0.0),
        &region,
        &object,
        &OrcaFloats(vec![OrcaFloat(f64::from(f32::from_bits(0x4253561c)))]),
    )
    .unwrap()
    .overhang_flow;

    assert_bits(
        flow,
        0x440415d1_4113a9f3_44039710_4253561c,
        0x40b2f9c660000000,
    );
}

#[test]
fn task22n_flow_roles_preserve_first_and_later_bits() {
    let (region, object) = flow_options();
    let first = four_flows(layer(0, 0.2), FloatOrPercent::Float(0.5), &region, &object);
    assert_four_bits(first, FIRST_FLOWS);

    let later = four_flows(layer(1, 0.3), FloatOrPercent::Float(0.5), &region, &object);
    assert_four_bits(later, LATER_FLOWS);
}

#[test]
fn task22n_flow_roles_resolve_percent_widths_against_direct_second_nozzle() {
    let (mut region, object) = flow_options();
    region.outer_wall_line_width = FloatOrPercent::Percent(Percent(125.0));
    region.inner_wall_line_width = FloatOrPercent::Percent(Percent(110.0));
    region.internal_solid_infill_line_width = FloatOrPercent::Percent(Percent(100.0));
    region.bridge_line_width = FloatOrPercent::Percent(Percent(80.0));
    region.outer_wall_filament_id = OrcaInt(2);
    region.inner_wall_filament_id = OrcaInt(2);
    region.internal_solid_filament_id = OrcaInt(2);
    let flows = resolve_perimeter_flows(
        &layer(0, 0.2),
        FloatOrPercent::Float(0.0),
        &region,
        &object,
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]),
    )
    .unwrap();

    assert_four_bits(flows, PERCENT_FLOWS);
}

#[test]
fn task22n_flow_roles_use_each_selector_and_element_zero_fallback() {
    let (mut region, object) = flow_options();
    region.outer_wall_filament_id = OrcaInt(2);
    region.inner_wall_filament_id = OrcaInt(1);
    region.internal_solid_filament_id = OrcaInt(2);
    let nozzles = OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]);
    let mixed = resolve_perimeter_flows(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.0),
        &region,
        &object,
        &nozzles,
    )
    .unwrap();
    assert_eq!(mixed.perimeter_flow.nozzle_diameter.to_bits(), 0x3ecccccd);
    assert_eq!(
        mixed.ext_perimeter_flow.nozzle_diameter.to_bits(),
        0x3f19999a
    );
    assert_eq!(mixed.overhang_flow.nozzle_diameter.to_bits(), 0x3ecccccd);
    assert_eq!(
        mixed.solid_infill_flow.nozzle_diameter.to_bits(),
        0x3f19999a
    );

    for selector in [0, -1, 3] {
        region.outer_wall_filament_id = OrcaInt(selector);
        region.inner_wall_filament_id = OrcaInt(selector);
        region.internal_solid_filament_id = OrcaInt(selector);
        let fallback = resolve_perimeter_flows(
            &layer(1, 0.2),
            FloatOrPercent::Float(0.0),
            &region,
            &object,
            &nozzles,
        )
        .unwrap();
        for flow in [
            fallback.perimeter_flow,
            fallback.ext_perimeter_flow,
            fallback.overhang_flow,
            fallback.solid_infill_flow,
        ] {
            assert_eq!(flow.nozzle_diameter.to_bits(), 0x3ecccccd);
        }
    }
}

fn four_flows(
    layer: PlannedLayer,
    initial: FloatOrPercent,
    region: &RegionOptions,
    object: &ObjectOptions,
) -> PerimeterFlows {
    resolve_perimeter_flows(
        &layer,
        initial,
        region,
        object,
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap()
}

fn assert_four_bits(flows: PerimeterFlows, expected: [PackedExpectedFlow; 4]) {
    for (flow, (fields, mm3_per_mm)) in [
        flows.perimeter_flow,
        flows.ext_perimeter_flow,
        flows.overhang_flow,
        flows.solid_infill_flow,
    ]
    .into_iter()
    .zip(expected)
    {
        assert_bits(flow, fields, mm3_per_mm);
    }
}

fn unpack_fields(fields: u128) -> [u32; 4] {
    [
        (fields >> 96) as u32,
        (fields >> 64) as u32,
        (fields >> 32) as u32,
        fields as u32,
    ]
}

fn resolve(
    layer: PlannedLayer,
    [initial, outer, object]: [FloatOrPercent; 3],
    selector: i32,
    nozzles: &[f64],
) -> crate::project_slice::perimeters::types::Flow {
    resolve_external_perimeter_flow(
        &layer,
        initial,
        outer,
        object,
        OrcaInt(selector),
        &OrcaFloats(nozzles.iter().copied().map(OrcaFloat).collect()),
    )
    .unwrap()
}

fn assert_bits(flow: crate::project_slice::perimeters::types::Flow, fields: u128, mm3_per_mm: u64) {
    let fields = unpack_fields(fields);
    assert_eq!(flow.width.to_bits(), fields[0]);
    assert_eq!(flow.height.to_bits(), fields[1]);
    assert_eq!(flow.spacing.to_bits(), fields[2]);
    assert_eq!(flow.nozzle_diameter.to_bits(), fields[3]);
    assert!(!flow.bridge);
    assert_eq!(flow.mm3_per_mm.to_bits(), mm3_per_mm);
}

fn layer(id: usize, height: f64) -> PlannedLayer {
    PlannedLayer {
        id,
        height,
        print_z: height,
        slice_z: height * 0.5,
    }
}
