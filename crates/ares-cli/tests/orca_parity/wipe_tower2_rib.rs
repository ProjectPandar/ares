//! The square rib tower of `WipeTower2::generate` (`WipeTower2.cpp:2485-2514`, v2.5.0-beta
//! `345efec64`) and its rib offset (`:2679-2684`) through `transform_wt2_pt`/`append_tcr2`
//! (`GCode.cpp:1003-1007, 1681`), the fake tower (`Print.cpp:5610-5612`) and `PrintExtents.cpp:152-158`
//! (`docs/architecture/wipe-tower2-rib-square.md`). A section oracle: each layer's `;TYPE:Prime
//! tower` extrusions must cover the box Native's do; the rest of the output is not compared.

use super::{RECTANGLE_TOWER, TWO_FILAMENTS, rewrite_project};
use crate::runner::{OrcaRunner, repo_root};

/// Per layer, the box `[min x, min y, max x, max y]` of the extruding moves in `;TYPE:Prime tower`.
fn tower_boxes(gcode: &[u8]) -> Vec<Option<[f64; 4]>> {
    let (mut at, mut tower, mut boxes) = ([0.0; 2], false, Vec::new());
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            boxes.push(None);
            continue;
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            tower = role.trim() == "Prime tower";
            continue;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        if !matches!(words.next(), Some("G1")) {
            continue;
        }
        let (mut moved, mut extruding) = (false, false);
        for word in words {
            let value = || word[1..].parse::<f64>().unwrap();
            match word.as_bytes()[0] {
                axis @ (b'X' | b'Y') => {
                    at[usize::from(axis == b'Y')] = value();
                    moved = true;
                }
                b'E' => extruding = value() > 0.0,
                _ => {}
            }
        }
        if let (true, true, true, Some(layer)) = (tower, moved, extruding, boxes.last_mut()) {
            let [x0, y0, x1, y1] = layer.get_or_insert([at[0], at[1], at[0], at[1]]);
            (*x0, *y0, *x1, *y1) = (x0.min(at[0]), y0.min(at[1]), x1.max(at[0]), y1.max(at[1]));
        }
    }
    assert!(
        boxes.iter().any(Option::is_some),
        "the G-code prints no prime tower"
    );
    boxes
}

/// Native and the CLI slicing plate 1 of `class-default.3mf` under `edits`.
pub(super) fn slice_both(label: &str, edits: &[(&str, &str)]) -> (Vec<u8>, Vec<u8>) {
    slice_fixture(TWO_FILAMENTS, label, edits)
}

/// Native and the CLI slicing plate 1 of the repo `fixture` under `edits`.
pub(super) fn slice_fixture(
    fixture: &str,
    label: &str,
    edits: &[(&str, &str)],
) -> (Vec<u8>, Vec<u8>) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0 slicing oracle");
    let project = rewrite_project(&std::fs::read(repo_root().join(fixture)).unwrap(), edits);
    let exported = runner
        .fixture_case(&format!("wipe-tower2-rib/{label}"), project)
        .unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    let output = exported.output_dir.join("ares.cli.gcode");
    let cli = crate::runner::application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{label}: CLI {cli:?}");
    (native.reference, std::fs::read(output).unwrap())
}

#[test]
fn rib_tower_prints_square_like_native() {
    let (native, ares) = slice_both("rib", &[]);
    assert_eq!(tower_boxes(&ares), tower_boxes(&native), "rib: tower boxes");
}

#[test]
fn rotated_rib_tower_carries_the_rib_offset_like_native() {
    let rotated = [(
        "\"wipe_tower_rotation_angle\": \"0\"",
        "\"wipe_tower_rotation_angle\": \"90\"",
    )];
    let (native, ares) = slice_both("rib-rotated", &rotated);
    assert_eq!(
        tower_boxes(&ares),
        tower_boxes(&native),
        "rib-rotated: tower boxes"
    );
}

#[test]
fn rectangle_tower_keeps_its_boxes() {
    let (native, ares) = slice_both("rectangle", &RECTANGLE_TOWER);
    assert_eq!(
        tower_boxes(&ares),
        tower_boxes(&native),
        "rectangle: tower boxes"
    );
}
