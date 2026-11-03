//! The Type2 tower approach through the wall opening: `WipeTowerIntegration::travel_to_tower_gap`
//! (`GCode.cpp:1029-1069`, v2.5.0-beta `345efec64`) called from `append_tcr2` (`:1731`)
//! (`docs/architecture/wipe-tower2-gap-travel.md`). A section oracle: each toolchange's approach
//! must match Native's; the rest of the output is not compared.

use super::wipe_tower2_rib::slice_both;

/// Per `; CP TOOLCHANGE START`, the approach section: the XY end points of the moves before it,
/// back to the last XY move that carries an `E` word.
fn approach_sections(gcode: &[u8]) -> Vec<Vec<(String, String)>> {
    let (mut at, mut approach, mut sections) =
        ([String::new(), String::new()], Vec::new(), Vec::new());
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
            match word.as_bytes()[0] {
                axis @ (b'X' | b'Y') => {
                    at[usize::from(axis == b'Y')] =
                        format!("{:.3}", word[1..].parse::<f64>().unwrap());
                    moved = true;
                }
                b'E' => extrudes = true,
                _ => {}
            }
        }
        match (moved, extrudes) {
            (true, true) => approach.clear(),
            (true, false) => approach.push((at[0].clone(), at[1].clone())),
            _ => {}
        }
    }
    assert!(!sections.is_empty(), "the G-code has no tower toolchange");
    sections
}

fn assert_approaches_match(label: &str, edits: &[(&str, &str)]) {
    let (native, ares) = slice_both(label, edits);
    assert_eq!(
        approach_sections(&ares),
        approach_sections(&native),
        "{label}: tower approach sections"
    );
}

#[test]
fn rib_tower_is_entered_through_the_wall_opening_like_native() {
    assert_approaches_match("gap-travel-rib", &[]);
}

#[test]
fn rectangle_tower_is_entered_through_the_wall_opening_like_native() {
    assert_approaches_match(
        "gap-travel-rectangle",
        &[
            (
                "\"wipe_tower_wall_type\": \"rib\"",
                "\"wipe_tower_wall_type\": \"rectangle\"",
            ),
            (
                "\"prime_tower_width\": \"60\"",
                "\"prime_tower_width\": \"20\"",
            ),
        ],
    );
}

#[test]
fn rotated_tower_is_entered_through_the_wall_opening_like_native() {
    assert_approaches_match(
        "gap-travel-rotated",
        &[(
            "\"wipe_tower_rotation_angle\": \"0\"",
            "\"wipe_tower_rotation_angle\": \"90\"",
        )],
    );
}

#[test]
fn tower_without_skip_points_is_approached_directly_like_native() {
    assert_approaches_match(
        "gap-travel-off",
        &[(
            "\"prime_tower_skip_points\": \"1\"",
            "\"prime_tower_skip_points\": \"0\"",
        )],
    );
}
