use super::outputs;

#[test]
fn two_nozzle_overhang_vertex_preserves_native_seam_and_role_order() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let layer_index = std::str::from_utf8(&reference)
        .unwrap()
        .split(";LAYER_CHANGE\n")
        .skip(1)
        .position(|layer| {
            layer.contains(";TYPE:Overhang wall\n") && layer.contains(";TYPE:Inner wall\n")
        })
        .unwrap();
    let layer = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .split(";LAYER_CHANGE\n")
            .nth(layer_index + 1)
            .unwrap()
            .to_owned()
    };
    assert_eq!(layer(&actual), layer(&reference));
}
