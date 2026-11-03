//! Prime tower positions through plate edits (the "Prime tower positions on plate edits" section
//! of `docs/architecture/web-plater-move-rotate-arrange.md`, v2.5.0-beta `345efec64`). A plate the
//! GUI adds takes `set_default_wipe_tower_pos_for_plate(new_index, true)` (`PartPlate.cpp:6644-6646,
//! 6339-6428`), a deleted plate's entry is erased (`:6770-6781`), and slicing reads the sliced
//! plate's entry (`OrcaSlicer.cpp:4580-4581`, `GCode.cpp:7270`). Native slices the project the GUI
//! exports for the session; the cases compare where the prime tower prints, not the whole file.

use ares_core::PlaterSession;

use super::super::plater_archive::{read_entry, rewrite_archive};
use super::{STRIDE, export, slice_session};
use crate::runner::{OrcaRunner, repo_root};

#[path = "wipe_tower2_rib.rs"]
mod wipe_tower2_rib;

const TWO_FILAMENTS: &str = "tests/parity/print-order-as-obj-list/class-default.3mf";
const PROJECT: &str = "Metadata/project_settings.config";
/// A 20 mm rectangle-wall tower purging the prime volume, without a brim: 20 x 11.25 mm for two
/// filaments (`WipeTowerEstimate.cpp:141-146, 213-232`).
const RECTANGLE_TOWER: [(&str, &str); 4] = [
    (
        "\"prime_tower_brim_width\": \"3\"",
        "\"prime_tower_brim_width\": \"0\"",
    ),
    (
        "\"wipe_tower_wall_type\": \"rib\"",
        "\"wipe_tower_wall_type\": \"rectangle\"",
    ),
    (
        "\"prime_tower_width\": \"60\"",
        "\"prime_tower_width\": \"20\"",
    ),
    (
        "\"purge_in_prime_tower\": \"1\"",
        "\"purge_in_prime_tower\": \"0\"",
    ),
];
/// The default of a new plate on the 220 mm i3 bed: `I3_WIPE_TOWER_DEFAULT_X_POS`/`_Y_POS` (0, 250)
/// (`PartPlate.cpp:147-148`) clamped by `WIPE_TOWER_AUTO_MARGIN` 15 (`libslic3r.h:103`) around
/// the 20 x 11.25 mm footprint (`:6398-6410`): x = 15, y = 220 - 11.25 - 15.
const NEW_PLATE_TOWER: [f64; 2] = [15.0, 193.75];

fn rewrite_project(input: &[u8], edits: &[(&str, &str)]) -> Vec<u8> {
    rewrite_archive(input, |name, text| {
        (name == PROJECT).then(|| {
            edits.iter().fold(text.to_owned(), |text, (from, to)| {
                assert!(text.contains(from), "{PROJECT} has no {from:?}");
                text.replace(from, to)
            })
        })
    })
}

/// Both cubes moved from plate 1 onto an added plate 2.
fn cubes_on_added_plate() -> PlaterSession {
    let input = std::fs::read(repo_root().join(TWO_FILAMENTS)).unwrap();
    let mut session = PlaterSession::load(rewrite_project(&input, &RECTANGLE_TOWER)).unwrap();
    session.add_plate().unwrap();
    for object in 0..2 {
        session
            .translate_instance(object, 0, [STRIDE, 0.0, 0.0])
            .unwrap();
    }
    assert!(session.plates()[0].instances.is_empty());
    assert_eq!(session.plates()[1].instances, vec![(0, 0), (1, 0)]);
    session
}

/// The lowest X and Y the extruding moves of the `;TYPE:Prime tower` sections reach.
fn tower_corner(gcode: &[u8]) -> [f64; 2] {
    let (mut at, mut tower, mut corner) = ([0.0; 2], false, [f64::INFINITY; 2]);
    for line in std::str::from_utf8(gcode).unwrap().lines() {
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
        if tower && moved && extruding {
            corner = [corner[0].min(at[0]), corner[1].min(at[1])];
        }
    }
    assert!(corner[0].is_finite(), "the G-code prints no prime tower");
    corner
}

/// The session's export stores `positions` (`wipe_tower_x`, `wipe_tower_y` as the GUI writes them),
/// Native slicing it places plate `plate`'s tower at `expected`, and so does the session.
fn assert_tower_like_native(
    label: &str,
    session: &PlaterSession,
    (plate, positions): (u32, [&[&str]; 2]),
    expected: [f64; 2],
) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0-beta slicing oracle");
    let oracle = export(session);
    let settings = read_entry(&oracle, PROJECT);
    let config: serde_json::Value = serde_json::from_str(&settings).unwrap();
    for (key, values) in ["wipe_tower_x", "wipe_tower_y"].into_iter().zip(positions) {
        assert_eq!(
            config[key],
            serde_json::json!(values),
            "{label}: exported {key}"
        );
    }
    let exported = runner
        .fixture_case(&format!("plater/{label}"), oracle)
        .unwrap();
    let native = runner.slice_selected_plate_case(&exported, plate).unwrap();
    assert_eq!(tower_corner(&native.reference), expected, "{label}: Native");
    assert_eq!(
        tower_corner(&slice_session(session, plate)),
        expected,
        "{label}: session"
    );
}

#[test]
fn plater_added_plate_slices_its_default_prime_tower_like_native() {
    let session = cubes_on_added_plate();
    assert_tower_like_native(
        "tower-added-plate",
        &session,
        (2, [&["150", "15"], &["150", "193.75"]]),
        NEW_PLATE_TOWER,
    );
}

#[test]
fn plater_deleted_plate_drops_its_prime_tower_position_like_native() {
    let mut session = cubes_on_added_plate();
    assert!(session.delete_plate(0));
    assert_eq!(session.plates().len(), 1);
    assert_eq!(session.plates()[0].instances, vec![(0, 0), (1, 0)]);
    assert_tower_like_native(
        "tower-deleted-plate",
        &session,
        (1, [&["15"], &["193.75"]]),
        NEW_PLATE_TOWER,
    );
}

/// One cube on each of two plates of an Ender-3; the project stores one tower position.
const TWO_PLATES: &str = "tests/parity/custom-gcode-overview/two-plates.3mf";
/// The refusal for the undefined behaviour of `docs/architecture/web-plater-tower-position-hole.md`.
const TOWER_POSITION_HOLE: &str = "no prime tower position is left for a new plate: deleting a plate emptied the list (PartPlate.cpp:6777-6780), and creating a plate reads its first entry (PartPlate.cpp:6344)";

fn assert_tower_position_hole(result: Result<(), ares_core::SliceError>) {
    assert!(
        matches!(&result, Err(ares_core::SliceError::InvalidInput(message)) if message == TOWER_POSITION_HOLE),
        "{result:?}"
    );
}

#[test]
fn plater_add_plate_refuses_an_emptied_tower_position_list() {
    let input = std::fs::read(repo_root().join(TWO_PLATES)).unwrap();
    let mut session = PlaterSession::load(input).unwrap();
    // `delete_plate` erases plate 0's entry, the only one (`PartPlate.cpp:6777-6780`).
    assert!(session.delete_plate(0));
    assert_eq!(session.tower_positions(), (Vec::new(), Vec::new()));
    assert_tower_position_hole(session.add_plate());
    assert_eq!(session.plates().len(), 1);
}

#[test]
fn plater_arrange_refuses_a_new_plate_on_an_emptied_tower_position_list() {
    // Both cubes scaled to 150 x 150 mm, so no two share the 220 mm bed and the arrange needs a
    // second plate (`postprocess_bed_index_for_selected`, `PartPlate.cpp:7833-7879`).
    let input = rewrite_archive(
        &std::fs::read(repo_root().join(TWO_PLATES)).unwrap(),
        |name, text| {
            (name == "3D/3dmodel.model").then(|| {
                ["110", "374"].iter().fold(text.to_owned(), |text, x| {
                    let from = format!("transform=\"1 0 0 0 1 0 0 0 1 {x} 110 5\"");
                    assert!(text.contains(&from), "no {from}");
                    text.replace(
                        &from,
                        &format!("transform=\"15 0 0 0 15 0 0 0 1 {x} 110 5\""),
                    )
                })
            })
        },
    );
    let mut session = PlaterSession::load(input).unwrap();
    // Plate 0's cube moves to the unprintable area and stays arrangeable.
    assert!(session.delete_plate(0));
    let transforms = |session: &PlaterSession| {
        [0, 1].map(|object| {
            session
                .instance_transform(object, 0)
                .unwrap()
                .to_column_major()
        })
    };
    let before = transforms(&session);
    let settings = ares_core::ArrangeSettings {
        distance: 0.0,
        enable_rotation: false,
        allow_multi_materials_on_same_plate: true,
        avoid_extrusion_cali_region: false,
        align_to_y_axis: false,
    };
    assert_tower_position_hole(session.arrange(ares_core::ArrangeScope::All, settings));
    assert_eq!(transforms(&session), before);
    assert_eq!(session.plates().len(), 1);
}

/// The config block's `wipe_tower_x`/`wipe_tower_y` lines: the fixed-precision entry of the sliced
/// plate (`get_at(print.get_plate_index())`, `GCode.cpp:8342-8346`), then the ordinary
/// serialization.
fn tower_config_lines(gcode: &[u8]) -> Vec<String> {
    std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .filter(|line| {
            line.starts_with("; wipe_tower_x = ") || line.starts_with("; wipe_tower_y = ")
        })
        .map(str::to_owned)
        .collect()
}

#[test]
fn plater_added_plate_echoes_its_own_tower_position_like_native() {
    let session = cubes_on_added_plate();
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0-beta slicing oracle");
    let exported = runner
        .fixture_case("plater/tower-echo-added-plate", export(&session))
        .unwrap();
    let native = tower_config_lines(
        &runner
            .slice_selected_plate_case(&exported, 2)
            .unwrap()
            .reference,
    );
    assert!(
        native.contains(&"; wipe_tower_x = 15.000".to_owned()),
        "Native: {native:?}"
    );
    assert_eq!(tower_config_lines(&slice_session(&session, 2)), native);
}
