use super::outputs;
use serde_json::json;

#[test]
fn two_nozzle_auto_lift_preserves_native_overhang_spirals() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[("z_hop_types", json!(["Auto Lift"]))]);
    let spirals = |bytes: &[u8]| {
        let text = std::str::from_utf8(bytes).unwrap();
        let layer = text.split(";Z:10.28\n").nth(1).unwrap();
        layer
            .split(";LAYER_CHANGE\n")
            .next()
            .unwrap()
            .lines()
            .filter(|line| line.starts_with("G3 Z"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = spirals(&reference);
    assert!(expected.len() >= 4);
    assert_eq!(spirals(&actual), expected);
}
