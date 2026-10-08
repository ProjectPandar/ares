use super::outputs;

#[test]
fn two_nozzle_overhang_vertex_preserves_native_seam_and_role_order() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let first_loop = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        let layer = text.split_once(";Z:1.64\n").unwrap().1;
        let layer = layer.split(";LAYER_CHANGE\n").next().unwrap();
        let approach = layer.split_once("M204 T10000\n").unwrap().1;
        approach
            .split_once("G1 X166.355 Y104.355 F18000\n")
            .unwrap()
            .0
            .to_owned()
    };
    let expected = first_loop(&reference);
    assert_eq!(expected.matches(";TYPE:Overhang wall\n").count(), 2);
    assert!(expected.contains(";TYPE:Inner wall\n"));
    assert_eq!(first_loop(&actual), expected);
}
