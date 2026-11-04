//! The lift type of the Type2 tower approach retract: `WipeTowerIntegration::append_tcr2`
//! (`GCode.cpp:1717-1728`, v2.5.0-beta `345efec64`) retracts with the configured `z_hop_types`,
//! `Auto` read as Spiral (`docs/architecture/wipe-tower2-tower-lift.md`). A section oracle: each
//! toolchange's approach moves, with Z, must match Native's; the rest of the output is not compared.

use super::wipe_tower2_rib::slice_both;

/// Per `; CP TOOLCHANGE START`, the approach section: the X/Y/Z end points of the moves before
/// it, back to the last axis move that carries an `E` word.
fn approach_sections(gcode: &[u8]) -> Vec<Vec<[String; 3]>> {
    let (mut at, mut approach, mut sections) = (
        [String::new(), String::new(), String::new()],
        Vec::new(),
        Vec::new(),
    );
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        if line == "; CP TOOLCHANGE START" {
            sections.push(std::mem::take(&mut approach));
            continue;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        if !matches!(words.next(), Some("G0" | "G1" | "G2" | "G3")) {
            continue;
        }
        let (mut moved, mut extrudes) = (false, false);
        for word in words {
            let axis = match word.as_bytes()[0] {
                b'X' => 0,
                b'Y' => 1,
                b'Z' => 2,
                b'E' => {
                    extrudes = true;
                    continue;
                }
                _ => continue,
            };
            at[axis] = format!("{:.3}", word[1..].parse::<f64>().unwrap());
            moved = true;
        }
        match (moved, extrudes) {
            (true, true) => approach.clear(),
            (true, false) => approach.push(at.clone()),
            _ => {}
        }
    }
    assert!(!sections.is_empty(), "the G-code has no tower toolchange");
    sections
}

fn assert_approaches_match(label: &str, lift: &str) {
    let edit = format!("\"z_hop_types\": [\"{lift}\"]");
    let (native, ares) = slice_both(label, &[("\"z_hop_types\": [\"Normal Lift\"]", &edit)]);
    assert_eq!(
        approach_sections(&ares),
        approach_sections(&native),
        "{label}: tower approach sections"
    );
}

#[test]
fn slope_lift_rises_along_the_tower_approach_like_native() {
    assert_approaches_match("tower-lift-slope", "Slope Lift");
}

#[test]
fn spiral_lift_spirals_before_the_tower_approach_like_native() {
    assert_approaches_match("tower-lift-spiral", "Spiral Lift");
}

#[test]
fn auto_lift_spirals_before_the_tower_approach_like_native() {
    assert_approaches_match("tower-lift-auto", "Auto Lift");
}

#[test]
fn normal_lift_stands_before_the_tower_approach_like_native() {
    assert_approaches_match("tower-lift-normal", "Normal Lift");
}
