//! The arrange over locked plates (`docs/architecture/web-plater-plate-lock.md`, OrcaSlicer
//! v2.5.0-beta `345efec64`). Native `--arrange` keeps a plate locked in the 3MF
//! (`OrcaSlicer.cpp:4701, 5701-5716`), so case 1 compares with it. Native never locks a plate for its
//! print sequence, and its tower loop does not skip locked plates (`:5784-5809`), so the GUI-only
//! paths are session cases, derived from case 1's Native result and from the tower's footprint.

use super::arranged::Arranged;
use super::wipe_tower::{NO_TOWER_BRIM, PRIME_VOLUME_PURGE, RECTANGLE_TOWER, TWO_FILAMENTS};
use super::{Edit, MODEL, OBJECTS, PROJECT, edited_input, native_arranged, settings};
use ares_core::{ArrangeScope, PlaterSession, SliceError};

/// One 10 mm cube on each of two plates of an Ender-3, at (110, 110) and (374, 110).
const TWO_PLATES: &str = "tests/parity/custom-gcode-overview/two-plates.3mf";
/// Plate 1's cube moved off the plate centre, so an arrange of it shows.
const MOVED: Edit = (
    MODEL,
    "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"",
    "transform=\"1 0 0 0 1 0 0 0 1 40 40 5\"",
);
const PLATE_ONE: &str = "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>";
/// Plate 1 locked in the 3MF (`bbs_3mf.cpp:4637-4640`).
const LOCKED: Edit = (
    OBJECTS,
    PLATE_ONE,
    "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"true\"/>",
);
/// `prepare_partplate`'s message for a locked plate (`ArrangeJob.cpp:548-552`).
const LOCKED_PLATE_MESSAGE: &str = "This plate is locked.\nCannot auto-arrange on this plate.";

#[test]
fn plater_arrange_leaves_a_locked_plate_alone_like_native() {
    super::assert_arranged_like_native(
        "locked-plate",
        TWO_PLATES,
        &[MOVED, LOCKED],
        settings(false, false),
    );
}

#[test]
fn plater_arrange_locks_a_plate_with_its_own_print_sequence() {
    // `prepare_all` locks an unlocked plate whose print sequence is not the global one
    // (`ArrangeJob.cpp:346-354`), so the arrange is case 1's, and unlocks it after `finalize`
    // (`:953-955`).
    let own_sequence = (
        OBJECTS,
        PLATE_ONE,
        "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"print_sequence\" value=\"by object\"/>",
    );
    let locked = edited_input("locked-plate-reference", (TWO_PLATES, &[MOVED, LOCKED]));
    let native = native_arranged(
        "locked-plate-reference",
        &locked,
        settings(false, false),
        (&[], None),
    );
    let input = edited_input("own-sequence", (TWO_PLATES, &[MOVED, own_sequence]));
    let mut session = PlaterSession::load(input).unwrap();
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    assert_eq!(Arranged::from_session(&session, None), native);
    assert!(!session.is_plate_locked(0).unwrap());
}

/// The two-filament cubes moved onto a second plate, the first plate locked and empty. Plate 2's
/// 20 x 11.25 mm rectangle tower (`wipe_tower.rs`) is stored at (100, 100), in the plate's middle.
const TOWER_ON_SECOND_PLATE: [Edit; 8] = [
    (
        "Metadata/filament_sequence.json",
        "\"sequence\":[]}}",
        "\"sequence\":[]},\"plate_2\":{\"nozzle_sequence\":[],\"optimal_assignment\":[],\"sequence\":[]}}",
    ),
    (
        OBJECTS,
        "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"\"/>\n",
        "<metadata key=\"plater_id\" value=\"1\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"true\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"\"/>\n  </plate>\n  <plate>\n    <metadata key=\"plater_id\" value=\"2\"/>\n    <metadata key=\"plater_name\" value=\"\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    <metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>\n    <metadata key=\"gcode_file\" value=\"\"/>\n",
    ),
    (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 110 116 5\"",
        "transform=\"1 0 0 0 1 0 0 0 1 374 116 5\"",
    ),
    (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 110 104 5\"",
        "transform=\"1 0 0 0 1 0 0 0 1 374 104 5\"",
    ),
    (
        PROJECT,
        "\"wipe_tower_x\": [\"150\"]",
        "\"wipe_tower_x\": [\"1\", \"100\"]",
    ),
    (
        PROJECT,
        "\"wipe_tower_y\": [\"150\"]",
        "\"wipe_tower_y\": [\"1\", \"100\"]",
    ),
    NO_TOWER_BRIM,
    PRIME_VOLUME_PURGE,
];

#[test]
fn plater_arrange_keeps_clear_of_an_unlocked_plate_tower_past_a_locked_plate() {
    // `prepare_wipe_tower` skips the locked plate and numbers the beds over the unlocked ones
    // (`ArrangeJob.cpp:504-526`): bed 0 is plate 2, with plate 2's tower. Packing both cubes around
    // the middle of plate 2 keeps them clear of that tower's box.
    let edits = [TOWER_ON_SECOND_PLATE.as_slice(), &RECTANGLE_TOWER].concat();
    let mut session =
        PlaterSession::load(edited_input("locked-plate-tower", (TWO_FILAMENTS, &edits))).unwrap();
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    let plates = session.plates();
    assert_eq!(plates[1].instances, vec![(0, 0), (1, 0)]);
    let [ox, oy] = plates[1].origin;
    let (tower_min, tower_max) = ([ox + 100.0, oy + 100.0], [ox + 120.0, oy + 111.25]);
    for object in 0..2 {
        let matrix = session
            .instance_transform(object, 0)
            .unwrap()
            .to_column_major();
        let (x, y) = (matrix[12], matrix[13]);
        let overlaps = x - 5.0 < tower_max[0]
            && x + 5.0 > tower_min[0]
            && y - 5.0 < tower_max[1]
            && y + 5.0 > tower_min[1];
        assert!(
            !overlaps,
            "cube {object} at ({x}, {y}) overlaps plate 2's tower"
        );
    }
}

#[test]
fn plater_arrange_on_a_locked_plate_moves_nothing() {
    let input = edited_input("locked-current-plate", (TWO_PLATES, &[MOVED, LOCKED]));
    let mut session = PlaterSession::load(input).unwrap();
    let transforms = |session: &PlaterSession| {
        [0, 1].map(|object| {
            session
                .instance_transform(object, 0)
                .unwrap()
                .to_column_major()
        })
    };
    let before = transforms(&session);
    let result = session.arrange(ArrangeScope::CurrentPlate(0), settings(false, false));
    assert!(
        matches!(&result, Err(SliceError::InvalidInput(message)) if message == LOCKED_PLATE_MESSAGE),
        "{result:?}"
    );
    assert_eq!(transforms(&session), before);
}

#[test]
fn plater_lock_plate_toggles_the_plate_lock() {
    let input = edited_input("lock-plate", (TWO_PLATES, &[LOCKED]));
    let mut session = PlaterSession::load(input).unwrap();
    assert!(session.is_plate_locked(0).unwrap());
    assert!(!session.is_plate_locked(1).unwrap());
    session.lock_plate(1, true).unwrap();
    session.lock_plate(0, false).unwrap();
    assert!(!session.is_plate_locked(0).unwrap());
    assert!(session.is_plate_locked(1).unwrap());
    assert!(matches!(
        session.lock_plate(2, true),
        Err(SliceError::InvalidInput(_))
    ));
}
