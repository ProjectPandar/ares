use crate::{
    FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt, Percent, SliceError,
    project_slice::{
        layers::PlannedLayer,
        perimeters::{flow::resolve_external_perimeter_flow, types::Flow},
    },
};

#[test]
fn task22m_flow_uses_layer_id_for_initial_then_outer_bits() {
    let first = resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();
    let later = resolve_external_perimeter_flow(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();

    assert_eq!(first.nozzle_diameter.to_bits(), 0x3ecccccd);
    assert_eq!(first.height.to_bits(), 0x3e4ccccd);
    assert_eq!(first.width.to_bits(), 0x3f000000);
    assert_eq!(first.spacing.to_bits(), 0x3eea0658);
    assert_eq!(first.minimum_width().to_bits(), 0x3f75032c);
    assert_eq!(later.width.to_bits(), 0x3ed70a3d);
    assert_eq!(later.spacing.to_bits(), 0x3ec11094);
    assert_eq!(later.minimum_width().to_bits(), 0x3f4c0d68);
}

#[test]
fn task22m_flow_uses_each_planned_layer_height_bits() {
    let flow = resolve_external_perimeter_flow(
        &layer(2, 0.3),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();

    assert_eq!(flow.height.to_bits(), 0x3e99999a);
    assert_eq!(flow.width.to_bits(), 0x3ed70a3d);
    assert_eq!(flow.spacing.to_bits(), 0x3eb613c0);
    assert_eq!(flow.minimum_width().to_bits(), 0x3f468efe);
}

#[test]
fn task22m_flow_preserves_spacing_valid_volume_underflow() {
    let flow = resolve_external_perimeter_flow(
        &layer(0, 1e-30),
        FloatOrPercent::Float(1e-30),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();

    assert_eq!(flow.width.to_bits(), 0x0da24260);
    assert_eq!(flow.height.to_bits(), 0x0da24260);
    assert_eq!(flow.spacing.to_bits(), 0x0d7ee054);
    assert_eq!(flow.minimum_width().to_bits(), 0x0e10d945);
    assert_eq!(flow.mm3_per_mm.to_bits(), 0);
}

#[test]
fn task22m_flow_preserves_absolute_fallback_and_auto_bits() {
    let initial_override = resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Float(0.6),
        FloatOrPercent::Float(0.38),
        FloatOrPercent::Float(0.52),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();
    let initial_fallback = resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Float(0.0),
        FloatOrPercent::Float(0.38),
        FloatOrPercent::Float(0.52),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();
    let object_fallback = resolve_external_perimeter_flow(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(0.0),
        FloatOrPercent::Float(0.52),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();
    let automatic = resolve_external_perimeter_flow(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(-0.1),
        FloatOrPercent::Float(0.52),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();
    let automatic_zero = resolve_external_perimeter_flow(
        &layer(1, 0.2),
        FloatOrPercent::Float(0.5),
        FloatOrPercent::Float(0.0),
        FloatOrPercent::Float(0.0),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
    )
    .unwrap();

    assert_flow_bits(initial_override, 0x3f19999a, 0x3f0e9cc6, 0x3f941b30);
    assert_flow_bits(initial_fallback, 0x3ec28f5c, 0x3eac95b4, 0x3f379288);
    assert_flow_bits(object_fallback, 0x3f051eb8, 0x3ef443c8, 0x3f7f409c);
    assert_flow_bits(automatic, 0x3ee66667, 0x3ed06cbe, 0x3f5b6992);
    assert_flow_bits(automatic_zero, 0x3ee66667, 0x3ed06cbe, 0x3f5b6992);
}

#[test]
fn task22m_flow_converts_selected_nozzle_before_percent_math() {
    let flow = resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Percent(Percent(125.0)),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.6)]),
    )
    .unwrap();

    assert_eq!(flow.nozzle_diameter.to_bits(), 0x3f19999a);
    assert_flow_bits(flow, 0x3f400000, 0x3f35032c, 0x3fba8196);

    let precise_percent = resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Percent(Percent(137.123_456_789)),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(1),
        &OrcaFloats(vec![OrcaFloat(0.6)]),
    )
    .unwrap();
    assert_flow_bits(precise_percent, 0x3f529f24, 0x3f47a250, 0x3fcd20ba);
}

#[test]
fn task22m_flow_selector_is_direct_unmapped_and_falls_back() {
    let selected_second = selected_percent_flow(2);
    assert_eq!(selected_second.nozzle_diameter.to_bits(), 0x3f19999a);
    assert_flow_bits(selected_second, 0x3f400000, 0x3f35032c, 0x3fba8196);

    for selector in [1, 0, -1, i32::MIN, 3] {
        let fallback = selected_percent_flow(selector);
        assert_eq!(fallback.nozzle_diameter.to_bits(), 0x3ecccccd);
        assert_flow_bits(fallback, 0x3f000000, 0x3eea0658, 0x3f75032c);
    }
}

#[test]
fn task22m_flow_rejects_percent_zero_and_negative() {
    for (outer, object) in [
        (
            FloatOrPercent::Float(0.0),
            FloatOrPercent::Percent(Percent(0.0)),
        ),
        (
            FloatOrPercent::Percent(Percent(-25.0)),
            FloatOrPercent::Float(0.52),
        ),
    ] {
        let error = resolve_external_perimeter_flow(
            &layer(1, 0.2),
            FloatOrPercent::Float(0.5),
            outer,
            object,
            OrcaInt(1),
            &OrcaFloats(vec![OrcaFloat(0.4)]),
        )
        .unwrap_err();

        assert_eq!(
            error,
            SliceError::InvalidInput("invalid external perimeter flow spacing".to_owned())
        );
    }
}

#[test]
fn task22m_flow_rejects_invalid_nozzle_height_and_spacing() {
    for diameters in [
        vec![],
        vec![OrcaFloat(0.0)],
        vec![OrcaFloat(-0.4)],
        vec![OrcaFloat(f64::NAN)],
        vec![OrcaFloat(f64::MAX)],
    ] {
        let error = resolve_external_perimeter_flow(
            &layer(0, 0.2),
            FloatOrPercent::Float(0.5),
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.4),
            OrcaInt(1),
            &OrcaFloats(diameters),
        )
        .unwrap_err();
        assert_invalid(error, "invalid Orca option nozzle_diameter");
    }

    for height in [0.0, -0.2, f64::NAN, f64::INFINITY, f64::MIN_POSITIVE] {
        let error = resolve_external_perimeter_flow(
            &layer(0, height),
            FloatOrPercent::Float(0.5),
            FloatOrPercent::Float(0.42),
            FloatOrPercent::Float(0.4),
            OrcaInt(1),
            &OrcaFloats(vec![OrcaFloat(0.4)]),
        )
        .unwrap_err();
        assert_invalid(error, "invalid Orca option layer_height");
    }

    for width in [
        FloatOrPercent::Float(0.01),
        FloatOrPercent::Float(f64::NAN),
        FloatOrPercent::Percent(Percent(f64::MAX)),
    ] {
        let error = resolve_external_perimeter_flow(
            &layer(1, 0.2),
            FloatOrPercent::Float(0.5),
            width,
            FloatOrPercent::Float(0.4),
            OrcaInt(1),
            &OrcaFloats(vec![OrcaFloat(0.4)]),
        )
        .unwrap_err();
        assert_invalid(error, "invalid external perimeter flow spacing");
    }
}

fn selected_percent_flow(selector: i32) -> Flow {
    resolve_external_perimeter_flow(
        &layer(0, 0.2),
        FloatOrPercent::Percent(Percent(125.0)),
        FloatOrPercent::Float(0.42),
        FloatOrPercent::Float(0.4),
        OrcaInt(selector),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]),
    )
    .unwrap()
}

fn assert_flow_bits(flow: Flow, width: u32, spacing: u32, minimum_width: u32) {
    assert_eq!(flow.width.to_bits(), width);
    assert_eq!(flow.spacing.to_bits(), spacing);
    assert_eq!(flow.minimum_width().to_bits(), minimum_width);
}

fn assert_invalid(error: SliceError, expected: &str) {
    assert_eq!(error, SliceError::InvalidInput(expected.to_owned()));
}

fn layer(id: usize, height: f64) -> PlannedLayer {
    PlannedLayer {
        id,
        height,
        print_z: height,
        slice_z: height * 0.5,
    }
}
