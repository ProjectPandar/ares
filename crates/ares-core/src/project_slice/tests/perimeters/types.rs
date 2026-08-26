use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, ProcessPerimeterGenerator,
    geometry::CoordinateScale,
    project_slice::perimeters::{
        context::prepare_perimeter_contexts,
        preflight::preflight_perimeter_flows,
        types::{Flow, PerimeterDispatch, PerimeterFlows, PreparedObjectFlows},
    },
};

use super::fixture::{Case, case, flow_options, split};

#[test]
fn task22n_flow_record_equality_matches_fixed_value_identity() {
    let baseline = flow(0.35, 0.08);
    let derived_fields_differ = flow(0.36, 0.09);

    assert_eq!(baseline, derived_fields_differ);
    assert_ne!(
        baseline,
        Flow {
            width: 0.41,
            ..baseline
        }
    );
    assert_ne!(
        baseline,
        Flow {
            height: 0.21,
            ..baseline
        }
    );
    assert_ne!(
        baseline,
        Flow {
            nozzle_diameter: 0.6,
            ..baseline
        }
    );
    assert_ne!(
        baseline,
        Flow {
            bridge: true,
            ..baseline
        }
    );
}

#[test]
fn task22n_dispatch_preserves_spiral_thresholds_scalars_and_exhaustive_selection() {
    let arachne = dispatch_case(100, ProcessPerimeterGenerator::Arachne, 3);
    let classic = dispatch_case(101, ProcessPerimeterGenerator::Classic, 2);
    let (objects, resolved) = split(vec![arachne, classic]);
    let flows = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap();
    let outputs = prepare_perimeter_contexts(objects, flows, &resolved, true);

    let records = outputs[0]
        .as_parts()
        .1
        .iter()
        .map(|slot| slot.as_ref().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        records
            .iter()
            .map(|record| record.spiral_mode)
            .collect::<Vec<_>>(),
        [false, true, true]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.dispatch)
            .collect::<Vec<_>>(),
        [
            PerimeterDispatch::Arachne,
            PerimeterDispatch::Classic,
            PerimeterDispatch::Classic
        ]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.model_rotation_rad.to_bits())
            .collect::<Vec<_>>(),
        [0x3ff921fb54442d18; 3]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.layer_height.to_bits())
            .collect::<Vec<_>>(),
        [0x3fc999999999999a, 0x3fc999999999999a, 0x3fc999999999999c,]
    );
    assert!(outputs[1].as_parts().1.iter().all(|slot| {
        slot.as_ref()
            .is_some_and(|record| record.dispatch == PerimeterDispatch::Classic)
    }));

    let arachne = dispatch_case(102, ProcessPerimeterGenerator::Arachne, 3);
    let (objects, resolved) = split(vec![arachne]);
    let flows = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap();
    let outputs = prepare_perimeter_contexts(objects, flows, &resolved, false);
    assert!(
        outputs[0]
            .as_parts()
            .1
            .iter()
            .all(|slot| slot.as_ref().is_some_and(
                |record| !record.spiral_mode && record.dispatch == PerimeterDispatch::Arachne
            ))
    );
}

#[test]
fn task22n_context_maps_each_flow_role_and_layer_slot_without_aliasing() {
    let (region, object) = flow_options();
    let Case { object, resolved } = case(
        103,
        region,
        object,
        &[(0.2, 1), (0.2, 1)],
        CoordinateScale::Normal,
    );
    let first = tagged_flows(1.0);
    let second = tagged_flows(11.0);
    let outputs = prepare_perimeter_contexts(
        vec![object],
        vec![PreparedObjectFlows {
            layers: vec![Some(first), Some(second)],
        }],
        std::slice::from_ref(&resolved),
        false,
    );

    for (record, expected) in outputs[0]
        .as_parts()
        .1
        .iter()
        .map(|slot| slot.as_ref().unwrap())
        .zip([first, second])
    {
        for (actual, expected) in [
            record.perimeter_flow,
            record.ext_perimeter_flow,
            record.overhang_flow,
            record.solid_infill_flow,
        ]
        .into_iter()
        .zip([
            expected.perimeter_flow,
            expected.ext_perimeter_flow,
            expected.overhang_flow,
            expected.solid_infill_flow,
        ]) {
            assert_exact_flow(actual, expected);
        }
    }
}

#[test]
fn task22n_dispatch_spiral_requires_each_gate_and_exact_epsilon_boundary() {
    let thickness = 0.25_f64 + 1e-4;
    let boundary = thickness - 1e-4;
    let below = f64::from_bits(boundary.to_bits() - 1);
    let cases = vec![
        spiral_gate_case(104, 0, 0.5, 0.2),
        spiral_gate_case(105, 1, 0.0, 0.2),
        spiral_gate_case(106, 0, thickness, boundary),
        spiral_gate_case(107, 0, thickness, below),
    ];
    let (objects, resolved) = split(cases);
    let flows = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap();
    let outputs = prepare_perimeter_contexts(objects, flows, &resolved, true);
    let records = outputs
        .iter()
        .map(|output| output.as_parts().1[0].as_ref().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        records
            .iter()
            .map(|record| record.spiral_mode)
            .collect::<Vec<_>>(),
        [false, false, true, false]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.dispatch)
            .collect::<Vec<_>>(),
        [
            PerimeterDispatch::Arachne,
            PerimeterDispatch::Arachne,
            PerimeterDispatch::Classic,
            PerimeterDispatch::Arachne,
        ]
    );
}

fn dispatch_case(
    source: usize,
    generator: ProcessPerimeterGenerator,
    layer_count: usize,
) -> super::fixture::Case {
    let (mut region, mut object) = flow_options();
    region.align_infill_direction_to_model = OrcaBool(true);
    region.bottom_shell_layers = OrcaInt(1);
    region.bottom_shell_thickness = OrcaFloat(0.3);
    object.wall_generator = generator;
    let heights = [0.2, 0.2, f64::from_bits(0x3fc999999999999c)];
    let layers = heights[..layer_count]
        .iter()
        .map(|height| (*height, 1))
        .collect::<Vec<_>>();
    let mut output = case(source, region, object, &layers, CoordinateScale::Normal);
    output.resolved.print_objects[0].transform =
        crate::Transform3d::parse_3mf("0 1 0 -1 0 0 0 0 1 17 18 19").unwrap();
    output
}

fn spiral_gate_case(
    source: usize,
    bottom_shell_layers: i32,
    bottom_shell_thickness: f64,
    height: f64,
) -> Case {
    let (mut region, mut object) = flow_options();
    region.bottom_shell_layers = OrcaInt(bottom_shell_layers);
    region.bottom_shell_thickness = OrcaFloat(bottom_shell_thickness);
    object.wall_generator = ProcessPerimeterGenerator::Arachne;
    case(
        source,
        region,
        object,
        &[(height, 1)],
        CoordinateScale::Normal,
    )
}

fn tagged_flows(seed: f32) -> PerimeterFlows {
    PerimeterFlows {
        perimeter_flow: tagged_flow(seed, false),
        ext_perimeter_flow: tagged_flow(seed + 1.0, true),
        overhang_flow: tagged_flow(seed + 2.0, false),
        solid_infill_flow: tagged_flow(seed + 3.0, true),
    }
}

fn tagged_flow(seed: f32, bridge: bool) -> Flow {
    Flow {
        width: seed,
        height: seed + 0.1,
        spacing: seed + 0.2,
        nozzle_diameter: seed + 0.3,
        bridge,
        mm3_per_mm: f64::from(seed) + 0.4,
    }
}

fn assert_exact_flow(actual: Flow, expected: Flow) {
    assert_eq!(
        [
            actual.width.to_bits(),
            actual.height.to_bits(),
            actual.spacing.to_bits(),
            actual.nozzle_diameter.to_bits(),
        ],
        [
            expected.width.to_bits(),
            expected.height.to_bits(),
            expected.spacing.to_bits(),
            expected.nozzle_diameter.to_bits(),
        ]
    );
    assert_eq!(actual.bridge, expected.bridge);
    assert_eq!(actual.mm3_per_mm.to_bits(), expected.mm3_per_mm.to_bits());
}

fn flow(spacing: f32, mm3_per_mm: f64) -> Flow {
    Flow {
        width: 0.4,
        height: 0.2,
        spacing,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm,
    }
}
