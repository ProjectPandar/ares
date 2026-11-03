//! The Type2 gap wall: `WipeTower2::compute_wall_skip_points` (`WipeTower2.cpp:2448-2463`, v2.5.0-beta
//! `345efec64`) and the gapped wall of `generate_support_rib_wall` (`:2663-2676`) through
//! `construct_gap_for_skip_points` (`WipeTower.cpp:695-708`), gated by `use_gap_wall` (`:1029-1033`)
//! (`docs/architecture/wipe-tower2-gap-wall.md`). A section oracle: each layer's tower wall must
//! match Native's; the rest of the output is not compared.

use super::wipe_tower2_rib::slice_both;

/// Per layer, the tower-wall section: the `;TYPE:Prime tower` XY moves outside
/// `; CP TOOLCHANGE START` .. `; CP TOOLCHANGE END` and before `; CP EMPTY GRID START`, each as
/// its end point and whether it extrudes.
fn wall_sections(gcode: &[u8]) -> Vec<Vec<(String, String, bool)>> {
    let (mut at, mut sections) = ([String::new(), String::new()], Vec::new());
    let (mut tower, mut toolchange, mut grid) = (false, false, false);
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        match line {
            ";LAYER_CHANGE" => {
                sections.push(Vec::new());
                grid = false;
                continue;
            }
            "; CP TOOLCHANGE START" => toolchange = true,
            "; CP TOOLCHANGE END" => toolchange = false,
            "; CP EMPTY GRID START" => grid = true,
            _ => {}
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            tower = role.trim() == "Prime tower";
            continue;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        if !matches!(words.next(), Some("G0" | "G1" | "G2" | "G3")) {
            continue;
        }
        let (mut moved, mut extruding) = (false, false);
        for word in words {
            match word.as_bytes()[0] {
                axis @ (b'X' | b'Y') => {
                    at[usize::from(axis == b'Y')] =
                        format!("{:.3}", word[1..].parse::<f64>().unwrap());
                    moved = true;
                }
                b'E' => extruding = word[1..].parse::<f64>().unwrap() > 0.0,
                _ => {}
            }
        }
        if let (true, true, false, false, Some(layer)) =
            (tower, moved, toolchange, grid, sections.last_mut())
        {
            layer.push((at[0].clone(), at[1].clone(), extruding));
        }
    }
    assert!(
        sections.iter().any(|layer| !layer.is_empty()),
        "the G-code prints no tower wall"
    );
    sections
}

const NO_SKIP_POINTS: (&str, &str) = (
    "\"prime_tower_skip_points\": \"1\"",
    "\"prime_tower_skip_points\": \"0\"",
);

#[test]
fn rib_wall_opens_at_the_toolchange_entries_like_native() {
    let (native, ares) = slice_both("gap-wall-rib", &[]);
    assert_eq!(
        wall_sections(&ares),
        wall_sections(&native),
        "gap-wall-rib: tower wall sections"
    );
}

#[test]
fn rectangle_wall_opens_at_the_toolchange_entries_like_native() {
    let rectangle = [
        (
            "\"wipe_tower_wall_type\": \"rib\"",
            "\"wipe_tower_wall_type\": \"rectangle\"",
        ),
        (
            "\"prime_tower_width\": \"60\"",
            "\"prime_tower_width\": \"20\"",
        ),
    ];
    let (native, ares) = slice_both("gap-wall-rectangle", &rectangle);
    assert_eq!(
        wall_sections(&ares),
        wall_sections(&native),
        "gap-wall-rectangle: tower wall sections"
    );
}

#[test]
fn wall_without_skip_points_stays_closed_like_native() {
    let (native, ares) = slice_both("gap-wall-off", &[NO_SKIP_POINTS]);
    assert_eq!(
        wall_sections(&ares),
        wall_sections(&native),
        "gap-wall-off: tower wall sections"
    );
}
