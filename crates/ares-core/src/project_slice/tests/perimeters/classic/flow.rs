use crate::project_slice::perimeters::{classic::prelude::lower_sample_offsets, types::Flow};

fn external_flow() -> Flow {
    Flow {
        width: 0.42,
        height: 0.2,
        spacing: 0.377_079_6,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: f64::from(f32::from_bits(0x3d9a_72b4)),
    }
}

#[test]
fn task22o1_smaller_external_flow_matches_fixed_flow_arithmetic() {
    let flow = external_flow()
        .with_width(f32::from_bits(0x3ec1_cd6a))
        .unwrap();

    assert_eq!(flow.width.to_bits(), 0x3ec1_cd6a);
    assert_eq!(flow.spacing.to_bits(), 0x3eab_d3c2);
    assert_eq!(flow.mm3_per_mm.to_bits(), 0x3fb1_2ec6_8000_0000);
}

#[test]
fn task22o1_smaller_external_flow_rejects_invalid_reconstruction() {
    for width in [0.0, f32::NAN, f32::INFINITY] {
        assert!(external_flow().with_width(width).is_err());
    }
}

#[test]
fn task22o1_lower_support_samples_preserve_mixed_precision() {
    let samples = lower_sample_offsets(0.45, 0.4);
    assert_eq!(samples.map(f32::to_bits), [0xbe3a_e147, 0x3e4c_cccd]);
}

#[test]
fn task22o2_zero_infill_width_uses_source_auto_width() {
    assert_eq!(
        Flow::auto_infill_width(0.4).to_bits(),
        f64::from(1.125_f32 * 0.4_f32).to_bits()
    );
}
