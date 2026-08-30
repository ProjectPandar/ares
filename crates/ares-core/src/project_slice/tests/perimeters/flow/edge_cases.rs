use super::{PackedExpectedFlow, assert_bits, flow_options, four_flows, layer, unpack_fields};
use crate::project_slice::perimeters::flow::resolve_perimeter_flows;
use crate::{FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt, Percent, SliceError};

#[test]
fn task22n_flow_roles_resolve_overhang_before_solid_derived_error() {
    let (mut region, object) = flow_options();
    region.bridge_line_width = FloatOrPercent::Float(0.01);
    region.internal_solid_filament_id = OrcaInt(2);

    let error = resolve_perimeter_flows(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.0),
        &region,
        &object,
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(f64::MAX)]),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SliceError::InvalidInput("invalid external perimeter flow spacing".to_owned())
    );
}

#[test]
fn task22n_flow_roles_preserve_object_fallback_and_negative_auto_widths() {
    let (mut region, mut object) = flow_options();
    region.inner_wall_line_width = FloatOrPercent::Float(0.0);
    region.internal_solid_infill_line_width = FloatOrPercent::Float(0.0);
    object.line_width = FloatOrPercent::Float(0.52);
    let fallback = four_flows(layer(1, 0.2), FloatOrPercent::Float(0.0), &region, &object);
    for flow in [fallback.perimeter_flow, fallback.solid_infill_flow] {
        assert_bits(
            flow,
            0x3f051eb8_3e4ccccd_3ef443c8_3ecccccd,
            0x3fb86d2da0000000,
        );
    }

    region.inner_wall_line_width = FloatOrPercent::Float(-0.2);
    region.internal_solid_infill_line_width = FloatOrPercent::Float(-0.3);
    let automatic = four_flows(layer(1, 0.2), FloatOrPercent::Float(0.0), &region, &object);
    for flow in [automatic.perimeter_flow, automatic.solid_infill_flow] {
        assert_bits(
            flow,
            0x3ee66667_3e4ccccd_3ed06cbe_3ecccccd,
            0x3fb4d7aca0000000,
        );
    }
}

#[test]
fn task22n_flow_overhang_preserves_each_reachable_cross_section_branch() {
    let inputs = [
        ("grow-height", Some(100.0), 1.4, false),
        ("shrink-width", Some(100.0), 0.8, false),
        ("decrease-round", Some(100.0), 0.2, false),
        ("epsilon-noop", Some(100.0), 1.0005, false),
        ("nonthick-auto-width", None, 0.8, false),
        ("thick-configured", Some(120.0), 1.44, true),
        ("thick-auto", None, 0.64, true),
    ];
    let expected: [PackedExpectedFlow; 7] = [
        (0x3ed59710_3e8f5c2a_3eb6d324_3ecccccd, 0x3fb99870c0000000),
        (0x3ea83c2c_3e4ccccd_3e924284_3ecccccd, 0x3fad4080c0000000),
        (0x3d8a1779_3d8a1779_3eb6d324_3ecccccd, 0x3f6d4080a0000000),
        (0x3ecccccd_3e4ccccd_3eb6d324_3ecccccd, 0x3fb2485080000000),
        (0x3ebcb70d_3e4ccccd_3ea6bd64_3ecccccd, 0x3fb0ac8a00000000),
        (0x3f1374bd_3f1374bd_3f20418a_3ecccccd, 0x3fd0ad4840000000),
        (0x3ea3d70b_3ea3d70b_3ebd70a5_3ecccccd, 0x3fb496b7e0000000),
    ];

    for ((label, percent_width, bridge_flow, thick), (fields, mm3_per_mm)) in
        inputs.into_iter().zip(expected)
    {
        let (mut region, mut object) = flow_options();
        region.bridge_line_width = percent_width.map_or(FloatOrPercent::Float(0.0), |value| {
            FloatOrPercent::Percent(Percent(value))
        });
        region.bridge_flow = OrcaFloat(bridge_flow);
        object.thick_bridges = crate::OrcaBool(thick);
        let flow =
            four_flows(layer(1, 0.2), FloatOrPercent::Float(0.0), &region, &object).overhang_flow;

        assert_eq!(
            [
                flow.width.to_bits(),
                flow.height.to_bits(),
                flow.spacing.to_bits(),
                flow.nozzle_diameter.to_bits(),
            ],
            unpack_fields(fields),
            "{label}"
        );
        assert_eq!(flow.bridge, thick, "{label}");
        assert_eq!(flow.mm3_per_mm.to_bits(), mm3_per_mm, "{label}");
    }
}
