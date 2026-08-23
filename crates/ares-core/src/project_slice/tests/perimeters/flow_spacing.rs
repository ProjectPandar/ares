use crate::project_slice::perimeters::{flow::with_spacing, types::Flow};

#[test]
fn task22o220_with_spacing_groups_source_float_subtraction_first() {
    let flow = Flow {
        width: f32::from_bits(0x3ed7_0a3d),
        height: f32::from_bits(0x3e4c_cccd),
        spacing: f32::from_bits(0x3ec1_1094),
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.0,
    };

    let adjusted = with_spacing(flow, f32::from_bits(0x3ec1_1080));

    assert_eq!(adjusted.width.to_bits(), 0x3ed7_0a29);
    assert_eq!((adjusted.mm3_per_mm as f32).to_bits(), 0x3d9a_739a);
}
