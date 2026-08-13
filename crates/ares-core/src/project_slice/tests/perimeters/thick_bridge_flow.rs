use crate::{
    FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt, Percent, ProjectSettings, RegionOptions,
    SliceError,
    project_slice::perimeters::{flow::resolve_thick_solid_infill_bridge_flow, types::Flow},
};

#[test]
fn task22o48_default_thick_solid_bridge_flow_matches_ksr_bits_without_mutation() {
    let region = region();
    let nozzles = nozzles(&[0.4]);
    let before = (region.clone(), nozzles.clone());

    let first = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();
    let second = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();

    assert_eq!(flow_bits(first), flow_bits(second));
    assert_flow(
        first,
        [0x3ecc_cccd, 0x3ecc_cccd, 0x3ee6_6667, 0x3ecc_cccd],
        0x3fc0_15bf_a000_0000,
    );
    assert_eq!((region, nozzles), before);
}

#[test]
fn task22o48_percent_width_uses_selected_solid_nozzle_and_ratio_cast_order() {
    let mut region = region();
    region.internal_solid_filament_id = OrcaInt(2);
    region.bridge_line_width = FloatOrPercent::Percent(Percent(50.0));
    region.bridge_flow = OrcaFloat(0.25);

    let nozzles = nozzles(&[0.4, 0.6]);
    let before = (region.clone(), nozzles.clone());
    let flow = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();

    assert_flow(
        flow,
        [0x3e19_999a, 0x3e19_999a, 0x3e4c_cccd, 0x3f19_999a],
        0x3f92_1877_a000_0000,
    );
    assert_eq!((region, nozzles), before);
}

#[test]
fn task22o48_nonbinary_percent_width_uses_f64_percent_evaluation() {
    let mut region = region();
    region.internal_solid_filament_id = OrcaInt(2);
    region.bridge_line_width = FloatOrPercent::Percent(Percent(33.3));
    region.bridge_flow = OrcaFloat(1.0);
    let nozzles = nozzles(&[0.4, 0.6]);
    let before = (region.clone(), nozzles.clone());

    let flow = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();

    assert_flow(
        flow,
        [0x3e4c_9860, 0x3e4c_9860, 0x3e7f_cb93, 0x3f19_999a],
        0x3fa0_0d84_8000_0000,
    );
    assert_eq!((region, nozzles), before);
}

#[test]
fn task22o48_absolute_width_casts_before_f64_sqrt_factor_is_cast_to_f32() {
    let mut region = region();
    region.bridge_line_width = FloatOrPercent::Float(0.45);
    region.bridge_flow = OrcaFloat(0.1);

    let flow = resolve_thick_solid_infill_bridge_flow(&region, &nozzles(&[0.4])).unwrap();

    assert_flow(
        flow,
        [0x3e11_b7be, 0x3e11_b7be, 0x3e44_eaf1, 0x3ecc_cccd],
        0x3f90_4938_2000_0000,
    );
}

#[test]
fn task22o48_zero_width_and_selector_fallback_use_the_selected_or_first_nozzle() {
    for selector in [2, 0, -1, 3] {
        let mut region = region();
        region.internal_solid_filament_id = OrcaInt(selector);
        region.bridge_line_width = FloatOrPercent::Float(0.0);
        let nozzles = nozzles(&[0.4, 0.6]);
        let before = (region.clone(), nozzles.clone());

        let flow = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();

        if selector == 2 {
            assert_flow(
                flow,
                [0x3f19_999a, 0x3f19_999a, 0x3f26_6667, 0x3f19_999a],
                0x3fd2_1877_a000_0000,
            );
        } else {
            assert_flow(
                flow,
                [0x3ecc_cccd, 0x3ecc_cccd, 0x3ee6_6667, 0x3ecc_cccd],
                0x3fc0_15bf_a000_0000,
            );
        }
        assert_eq!((region, nozzles), before);
    }
}

#[test]
fn task22o48_missing_or_invalid_selected_nozzle_uses_existing_error_without_mutation() {
    for (selector, values) in [
        (1, vec![]),
        (1, vec![0.0]),
        (1, vec![-0.4]),
        (1, vec![f64::NAN]),
        (1, vec![f64::INFINITY]),
        (1, vec![f64::MIN_POSITIVE]),
        (2, vec![0.4, f64::NAN]),
    ] {
        let mut region = region();
        region.internal_solid_filament_id = OrcaInt(selector);
        let nozzles = nozzles(&values);
        let before_region = region.clone();
        let before_nozzles = nozzle_bits(&nozzles);

        assert_eq!(
            resolve_thick_solid_infill_bridge_flow(&region, &nozzles),
            Err(SliceError::InvalidInput(
                "invalid Orca option nozzle_diameter".to_owned()
            ))
        );
        assert_eq!(region, before_region);
        assert_eq!(nozzle_bits(&nozzles), before_nozzles);
    }
}

#[test]
fn task22o48_nonpositive_ratio_preserves_diameter_and_infinite_ratio_errors_atomically() {
    for ratio in [0.0, -1.0] {
        let mut region = region();
        region.bridge_flow = OrcaFloat(ratio);
        let nozzles = nozzles(&[0.4]);
        let before = (region.clone(), nozzles.clone());

        let flow = resolve_thick_solid_infill_bridge_flow(&region, &nozzles).unwrap();

        assert_flow(
            flow,
            [0x3ecc_cccd, 0x3ecc_cccd, 0x3ee6_6667, 0x3ecc_cccd],
            0x3fc0_15bf_a000_0000,
        );
        assert_eq!((region, nozzles), before);
    }

    let mut region = region();
    region.bridge_flow = OrcaFloat(f64::INFINITY);
    let nozzles = nozzles(&[0.4]);
    let before = (region.clone(), nozzles.clone());
    assert_eq!(
        resolve_thick_solid_infill_bridge_flow(&region, &nozzles),
        Err(SliceError::InvalidInput(
            "invalid Orca option bridge_flow".to_owned()
        ))
    );
    assert_eq!((region, nozzles), before);
}

fn region() -> RegionOptions {
    RegionOptions::from_base(&ProjectSettings::default().process.region)
}

fn nozzles(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn assert_flow(flow: Flow, fields: [u32; 4], volume: u64) {
    assert_eq!(flow.width.to_bits(), fields[0]);
    assert_eq!(flow.height.to_bits(), fields[1]);
    assert_eq!(flow.spacing.to_bits(), fields[2]);
    assert_eq!(flow.nozzle_diameter.to_bits(), fields[3]);
    assert!(flow.bridge);
    assert_eq!(flow.mm3_per_mm.to_bits(), volume);
}

fn nozzle_bits(nozzles: &OrcaFloats) -> Vec<u64> {
    nozzles.0.iter().map(|value| value.0.to_bits()).collect()
}

fn flow_bits(flow: Flow) -> (u32, u32, u32, u32, bool, u64) {
    (
        flow.width.to_bits(),
        flow.height.to_bits(),
        flow.spacing.to_bits(),
        flow.nozzle_diameter.to_bits(),
        flow.bridge,
        flow.mm3_per_mm.to_bits(),
    )
}
