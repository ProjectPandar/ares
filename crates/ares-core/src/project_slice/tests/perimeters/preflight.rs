use crate::{
    FloatOrPercent, ObjectOptions, OrcaBool, OrcaFloat, OrcaFloats, Percent, RegionOptions,
    SliceError, geometry::CoordinateScale,
    project_slice::perimeters::preflight::preflight_perimeter_flows,
};

use super::fixture::{case, flow_options, snapshot, split};

#[test]
fn task22n_preflight_preserves_order_empty_slots_and_scale_independent_flows() {
    let (region, object) = flow_options();
    let (objects, resolved) = split(vec![
        case(
            0,
            region.clone(),
            object.clone(),
            &[(0.2, 2), (0.3, 1), (0.2, 0)],
            CoordinateScale::Normal,
        ),
        case(1, region, object, &[(0.2, 1)], CoordinateScale::LargeBed),
    ]);
    let prepared = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap();

    assert_eq!(prepared.len(), 2);
    assert_eq!(prepared[0].layers.len(), 3);
    assert!(prepared[0].layers[0].is_some());
    assert!(prepared[0].layers[1].is_some());
    assert!(prepared[0].layers[2].is_none());
    assert_eq!(prepared[1].layers.len(), 1);
    let normal = prepared[0].layers[0].unwrap();
    let large = prepared[1].layers[0].unwrap();
    assert_flow_bits(normal.perimeter_flow, large.perimeter_flow);
    assert_flow_bits(normal.ext_perimeter_flow, large.ext_perimeter_flow);
    assert_flow_bits(normal.overhang_flow, large.overhang_flow);
    assert_flow_bits(normal.solid_infill_flow, large.solid_infill_flow);
    assert_eq!(
        prepared[0].layers[1]
            .unwrap()
            .perimeter_flow
            .width
            .to_bits(),
        0x3ee66666
    );
}

#[test]
fn task22n_preflight_validates_every_config_before_resolving_any_flow() {
    let (mut first_region, first_object) = flow_options();
    first_region.inner_wall_line_width = FloatOrPercent::Float(0.01);
    let (mut second_region, second_object) = flow_options();
    second_region.bridge_flow = OrcaFloat(0.0);
    let (objects, resolved) = split(vec![
        case(
            0,
            first_region,
            first_object,
            &[(0.2, 1)],
            CoordinateScale::Normal,
        ),
        case(
            1,
            second_region,
            second_object,
            &[(0.2, 1)],
            CoordinateScale::Normal,
        ),
    ]);
    let before = snapshot(&objects, &resolved);

    let error = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned())
    );
    assert_eq!(snapshot(&objects, &resolved), before);
}

#[test]
fn task22n_preflight_validates_later_layer_before_resolving_earlier_objects() {
    let (region, object) = flow_options();
    let (objects, resolved) = split(vec![
        case(
            0,
            region.clone(),
            object.clone(),
            &[(0.2, 1)],
            CoordinateScale::Normal,
        ),
        case(
            1,
            region,
            object,
            &[(0.2, 1), (0.0, 1)],
            CoordinateScale::Normal,
        ),
    ]);
    let before = snapshot(&objects, &resolved);

    let error = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SliceError::InvalidInput("invalid Orca option layer_height".to_owned())
    );
    assert_eq!(snapshot(&objects, &resolved), before);
}

#[test]
fn task22n_preflight_validates_zero_layer_options_and_every_nozzle() {
    let (mut region, object) = flow_options();
    region.bridge_flow = OrcaFloat(0.0);
    let (objects, resolved) = split(vec![case(0, region, object, &[], CoordinateScale::Normal)]);
    let error = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]),
    )
    .unwrap_err();
    assert_eq!(
        error,
        SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned())
    );

    let (region, object) = flow_options();
    let (objects, resolved) = split(vec![case(
        0,
        region,
        object,
        &[(0.2, 1)],
        CoordinateScale::Normal,
    )]);
    let error = preflight_perimeter_flows(
        &objects,
        &resolved,
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(f64::NAN)]),
    )
    .unwrap_err();
    assert_eq!(
        error,
        SliceError::InvalidInput("invalid Orca option nozzle_diameter".to_owned())
    );
}

#[test]
fn task22n_preflight_reports_each_nonfinite_width_by_owning_option() {
    let keys = [
        "initial_layer_line_width",
        "outer_wall_line_width",
        "inner_wall_line_width",
        "internal_solid_infill_line_width",
        "line_width",
        "bridge_line_width",
    ];
    for (index, key) in keys.into_iter().enumerate() {
        let (mut region, mut object) = flow_options();
        let mut initial = FloatOrPercent::Float(0.5);
        let invalid = if index % 2 == 0 {
            FloatOrPercent::Percent(Percent(f64::NAN))
        } else {
            FloatOrPercent::Float(f64::NAN)
        };
        match key {
            "initial_layer_line_width" => initial = invalid,
            "outer_wall_line_width" => region.outer_wall_line_width = invalid,
            "inner_wall_line_width" => region.inner_wall_line_width = invalid,
            "internal_solid_infill_line_width" => {
                region.internal_solid_infill_line_width = invalid;
            }
            "line_width" => object.line_width = invalid,
            "bridge_line_width" => region.bridge_line_width = invalid,
            _ => unreachable!(),
        }
        assert_eq!(
            preflight_error(initial, region, object, &[(0.2, 1)], valid_nozzles()),
            SliceError::InvalidInput(format!("invalid Orca option {key}"))
        );
    }
}

#[test]
fn task22n_preflight_rejects_every_invalid_bridge_ratio_in_both_modes() {
    for thick in [false, true] {
        for ratio in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let (mut region, mut object) = flow_options();
            region.bridge_flow = OrcaFloat(ratio);
            object.thick_bridges = OrcaBool(thick);
            assert_eq!(
                preflight_error(
                    FloatOrPercent::Float(0.5),
                    region,
                    object,
                    &[(0.2, 1)],
                    valid_nozzles(),
                ),
                SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned())
            );
        }
    }
}

#[test]
fn task22n_preflight_rejects_underflowed_flow_volumes_before_consuming_state() {
    for thick in [false, true] {
        let (mut region, mut object) = flow_options();
        region.bridge_flow = OrcaFloat(f64::MIN_POSITIVE);
        object.thick_bridges = OrcaBool(thick);
        assert_eq!(
            preflight_error(
                FloatOrPercent::Float(0.5),
                region,
                object,
                &[(0.2, 1)],
                valid_nozzles(),
            ),
            SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned())
        );
    }

    let (region, object) = flow_options();
    assert_eq!(
        preflight_error(
            FloatOrPercent::Float(1e-30),
            region,
            object,
            &[(1e-30, 1)],
            valid_nozzles(),
        ),
        SliceError::InvalidInput("invalid external perimeter flow volume".to_owned())
    );
}

#[test]
fn task22n_preflight_rejects_every_invalid_layer_height_before_flow_resolution() {
    for height in [0.0, -0.2, f64::NAN, f64::INFINITY, f64::MIN_POSITIVE] {
        let (region, object) = flow_options();
        assert_eq!(
            preflight_error(
                FloatOrPercent::Float(0.5),
                region,
                object,
                &[(height, 1)],
                valid_nozzles(),
            ),
            SliceError::InvalidInput("invalid Orca option layer_height".to_owned())
        );
    }
}

fn preflight_error(
    initial: FloatOrPercent,
    region: RegionOptions,
    object: ObjectOptions,
    layers: &[(f64, usize)],
    nozzles: OrcaFloats,
) -> SliceError {
    let (objects, resolved) = split(vec![case(
        0,
        region,
        object,
        layers,
        CoordinateScale::Normal,
    )]);
    let before = snapshot(&objects, &resolved);
    let error = preflight_perimeter_flows(&objects, &resolved, initial, &nozzles).unwrap_err();
    assert_eq!(
        format!("{:?}", snapshot(&objects, &resolved)),
        format!("{before:?}")
    );
    error
}

fn valid_nozzles() -> OrcaFloats {
    OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)])
}

fn assert_flow_bits(
    left: crate::project_slice::perimeters::types::Flow,
    right: crate::project_slice::perimeters::types::Flow,
) {
    assert_eq!(left.width.to_bits(), right.width.to_bits());
    assert_eq!(left.height.to_bits(), right.height.to_bits());
    assert_eq!(left.spacing.to_bits(), right.spacing.to_bits());
    assert_eq!(
        left.nozzle_diameter.to_bits(),
        right.nozzle_diameter.to_bits()
    );
    assert_eq!(left.bridge, right.bridge);
    assert_eq!(left.mm3_per_mm.to_bits(), right.mm3_per_mm.to_bits());
}
