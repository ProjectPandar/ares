use super::outputs;
use serde_json::json;

#[test]
fn auto_lift_preserves_native_travel_lifts() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[("z_hop_types", json!(["Auto Lift"]))]);
    // libslic3r/GCode.cpp:10552-10567,10647-10653,10681-10688 selects
    // normal, slope or spiral lifts conditionally; Auto does not require arcs.
    let z_moves = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .split_once(";LAYER_CHANGE\n")
            .unwrap()
            .1
            .lines()
            .filter_map(|line| {
                let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
                if !matches!(words.next(), Some("G0" | "G1" | "G2" | "G3")) {
                    return None;
                }
                let z = words
                    .find_map(|word| word.strip_prefix('Z'))?
                    .parse::<f64>()
                    .unwrap();
                Some((line.to_owned(), z))
            })
            .collect::<Vec<_>>()
    };
    let expected = z_moves(&reference);
    assert!(!expected.is_empty());
    assert!(
        expected
            .windows(3)
            .any(|motion| { motion[1].1 > motion[0].1 && motion[2].1 < motion[1].1 })
    );
    assert_eq!(z_moves(&actual), expected);
}
