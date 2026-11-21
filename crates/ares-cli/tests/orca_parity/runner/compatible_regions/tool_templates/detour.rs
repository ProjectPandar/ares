use super::outputs;

#[test]
fn two_nozzle_thin_corner_travel_preserves_native_detour() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    let travel = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        let layer = text.split(";Z:0.84\n").nth(1).unwrap();
        let layer = layer.split(";LAYER_CHANGE\n").next().unwrap();
        let approach = layer.split("M204 T10000\n").nth(1).unwrap();
        approach
            .split("G1 Z.84\n")
            .next()
            .unwrap()
            .lines()
            .filter(|line| line.starts_with("G1 X"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = travel(&reference);
    assert!(expected.len() >= 3);
    assert_eq!(travel(&actual), expected);
}
