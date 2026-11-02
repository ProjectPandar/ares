//! Per-object skirts in the arrange (`docs/architecture/web-plater-arrange-object-skirt.md`):
//! `init_arrange_params` widens the by-object clearance by `Print::object_skirt_offset` of the
//! current plate's print (`ArrangeJob.cpp:989-1001`, `Print.cpp:5796-5817` at v2.5.0-beta
//! `345efec64`). The CLI never sets the offset and reads only `extruder_clearance_radius`, so for
//! parts taller than `nozzle_height` the GUI arrange with offset `o` is Native's arrange with the
//! radius raised by `2·o` (every reader sits behind `is_seq_print`, `Arrange.cpp:156-221`).

use super::arranged::Arranged;
use super::{
    BY_LAYER, CUBES, Edit, PROJECT, assert_arranged_like_native, edited_input, native_arranged,
    settings,
};
use ares_core::{ArrangeScope, PlaterSession, SliceError};

/// A one-loop per-object skirt, 10 layers tall: 10 x 0.36 mm (`max_layer_height`) exceeds
/// `nozzle_height` 2.5, so the offset is `skirt_distance` + the first-layer line width,
/// 2 + 0.5 = 2.5 (`Print.cpp:5809-5810`). The cubes are 4 mm tall, so no part is short.
const PER_OBJECT_SKIRT: [Edit; 5] = [
    (
        PROJECT,
        "\"skirt_type\": \"combined\"",
        "\"skirt_type\": \"perobject\"",
    ),
    (PROJECT, "\"skirt_loops\": \"2\"", "\"skirt_loops\": \"1\""),
    (
        PROJECT,
        "\"skirt_distance\": \"6\"",
        "\"skirt_distance\": \"2\"",
    ),
    (
        PROJECT,
        "\"skirt_height\": \"1\"",
        "\"skirt_height\": \"10\"",
    ),
    (
        PROJECT,
        "\"initial_layer_line_width\": \"0.42\"",
        "\"initial_layer_line_width\": \"0.5\"",
    ),
];
/// The clearance radius 47 widened by twice the 2.5 mm offset.
const RADIUS_52: Edit = (
    PROJECT,
    "\"extruder_clearance_radius\": \"47\"",
    "\"extruder_clearance_radius\": \"52\"",
);

fn with(edits: &[Edit], more: &[Edit]) -> Vec<Edit> {
    [edits, more].concat()
}

#[test]
fn plater_arrange_widens_the_by_object_clearance_by_the_per_object_skirt() {
    let input = edited_input("object-skirt", (CUBES, &PER_OBJECT_SKIRT));
    let widened = edited_input(
        "object-skirt-radius-52",
        (CUBES, &with(&PER_OBJECT_SKIRT, &[RADIUS_52])),
    );
    let native = native_arranged(
        "object-skirt-radius-52",
        &widened,
        settings(false, false),
        (&[], None),
    );
    let unwidened = native_arranged("object-skirt", &input, settings(false, false), (&[], None));
    assert_ne!(unwidened, native, "the 52 mm clearance arranges like 47 mm");
    let mut session = PlaterSession::load(input).unwrap();
    session
        .arrange(ArrangeScope::All, settings(false, false))
        .unwrap();
    assert_eq!(Arranged::from_session(&session, None), native);
}

#[test]
fn plater_arrange_on_the_current_plate_widens_the_clearance_by_the_per_object_skirt() {
    // Session-only: Native's `--slice` validates the plate it arranged, and its validation adds
    // the skirt offset (`Print.cpp:917, 928`) that its arrange never applies, so it rejects every
    // per-object-skirt plate (-63). The derivation of case 1 holds within the GUI path: the
    // offset equals a combined skirt (same `get_real_skirt_dist`, no offset) at radius 52.
    let arranged = |label: &str, edits: &[Edit]| {
        let mut session = PlaterSession::load(edited_input(label, (CUBES, edits))).unwrap();
        session
            .arrange(ArrangeScope::CurrentPlate(0), settings(false, false))
            .unwrap();
        Arranged::from_session(&session, Some(0))
    };
    let combined_52 = with(&PER_OBJECT_SKIRT[1..], &[RADIUS_52]);
    assert_eq!(
        arranged("object-skirt-plate", &PER_OBJECT_SKIRT),
        arranged("object-skirt-plate-combined-52", &combined_52)
    );
}

#[test]
fn plater_arrange_reads_the_skirt_offset_from_the_current_plate() {
    // An empty current plate's print has no objects, so its offset is 0 (`Print.cpp:5798-5799`):
    // the arrange is that of a combined skirt, which has the same skirt distance
    // (`get_real_skirt_dist`) and no offset.
    let arranged = |skirt_type: &'static str, current: usize| {
        let edits = with(
            &PER_OBJECT_SKIRT[1..],
            &[(PROJECT, "\"skirt_type\": \"combined\"", skirt_type)],
        );
        let mut session =
            PlaterSession::load(edited_input("object-skirt-current", (CUBES, &edits))).unwrap();
        session.add_plate().unwrap();
        session.select_plate(current).unwrap();
        session
            .arrange(ArrangeScope::All, settings(false, false))
            .unwrap();
        Arranged::from_session(&session, None)
    };
    assert_eq!(
        arranged("\"skirt_type\": \"perobject\"", 1),
        arranged("\"skirt_type\": \"combined\"", 0)
    );
}

#[test]
fn plater_arrange_by_layer_ignores_the_per_object_skirt_like_native() {
    assert_arranged_like_native(
        "object-skirt-by-layer",
        CUBES,
        &with(&PER_OBJECT_SKIRT, &[BY_LAYER]),
        settings(false, false),
    );
}

#[test]
fn plater_select_plate_tracks_the_current_plate() {
    let mut session = PlaterSession::load(edited_input(
        "current-plate",
        ("tests/parity/custom-gcode-overview/two-plates.3mf", &[]),
    ))
    .unwrap();
    // `Plater.cpp:10222, 10229`: plate 0 after loading.
    assert_eq!(session.current_plate(), 0);
    // Adding a plate does not select it (`PartPlateList::create_plate`).
    session.add_plate().unwrap();
    assert_eq!(session.current_plate(), 0);
    session.select_plate(2).unwrap();
    assert_eq!(session.current_plate(), 2);
    // `delete_plate` moves the current plate back past a deleted one before it
    // (`PartPlate.cpp:6809-6815`).
    assert!(session.delete_plate(0));
    assert_eq!(session.current_plate(), 1);
    // The plate's arrange icon selects its plate first.
    session
        .arrange(ArrangeScope::CurrentPlate(0), settings(false, false))
        .unwrap();
    assert_eq!(session.current_plate(), 0);
    assert!(matches!(
        session.select_plate(2),
        Err(SliceError::InvalidInput(_))
    ));
}
