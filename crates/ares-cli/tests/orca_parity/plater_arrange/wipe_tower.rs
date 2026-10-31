//! "Arrange all" around the prime tower (the A3b addendum in
//! `docs/architecture/web-plater-move-rotate-arrange.md`): `ArrangeJob::prepare_wipe_tower` and
//! the plate's tower estimate at v2.5.0-beta `345efec64`, against the CLI global arrange's tower
//! (`OrcaSlicer.cpp:5729-5810`). The cases keep the stored tower positions valid and the tower
//! brim at 0. The tower cases put a 20 mm wide rectangle-wall tower in the bed's lower-left
//! corner, off the diagonal, at sizes and a position where the GUI's tower item (a box plus the
//! scaled position, `GLCanvas3D.cpp:6149-6185`) and the CLI's polygon (scaled float sums,
//! `PartPlate.cpp:4139-4152`) agree to the scaled unit.

use super::{
    Edit, MODEL, OBJECTS, PROJECT, SINGLE, assert_arranged_like_native,
    assert_arranged_like_native_with, settings,
};

/// Two 10 mm cubes on filaments 1 and 2 (both PLA at 60 °C) on an Ender-3; tower at (150, 150).
const TWO_FILAMENTS: &str = "tests/parity/print-order-as-obj-list/class-default.3mf";
/// One object whose parts print with filaments 1 and 2.
const TWO_COLOURS: &str = "tests/parity/interface-shells/two-region-prism.3mf";

const NO_TOWER_BRIM: Edit = (
    PROJECT,
    "\"prime_tower_brim_width\": \"3\"",
    "\"prime_tower_brim_width\": \"0\"",
);
/// A rectangle-wall tower (`WipeTowerEstimate.cpp:213-232`) of `prime_tower_width` 20 mm: two
/// filaments at 0.2 mm make it 20 x 11.25 mm, one filament 20 mm by the height's minimum depth.
const RECTANGLE_TOWER: [Edit; 2] = [
    (
        PROJECT,
        "\"wipe_tower_wall_type\": \"rib\"",
        "\"wipe_tower_wall_type\": \"rectangle\"",
    ),
    (
        PROJECT,
        "\"prime_tower_width\": \"60\"",
        "\"prime_tower_width\": \"20\"",
    ),
];
/// The tower purges the prime volume, not the single-extruder flush matrix
/// (`WipeTowerEstimate.cpp:141-146`): 45 mm³ make the 20 mm tower 11.25 mm deep.
const PRIME_VOLUME_PURGE: Edit = (
    PROJECT,
    "\"purge_in_prime_tower\": \"1\"",
    "\"purge_in_prime_tower\": \"0\"",
);
/// The tower stored at (1, 1) in the two-filament project.
const TWO_FILAMENTS_CORNER: [Edit; 2] = [
    (
        PROJECT,
        "\"wipe_tower_x\": [\"150\"]",
        "\"wipe_tower_x\": [\"1\"]",
    ),
    (
        PROJECT,
        "\"wipe_tower_y\": [\"150\"]",
        "\"wipe_tower_y\": [\"1\"]",
    ),
];
/// The second cube stretched to 15 x 10 mm, so no placement of it mirrors another.
const LONG_SECOND: Edit = (
    MODEL,
    "transform=\"1 0 0 0 1 0 0 0 1 110 104 5\"",
    "transform=\"1.5 0 0 0 1 0 0 0 1 110 104 5\"",
);
/// Names the two cubes apart; both load as `cube10.stl`.
const NAMED: [Edit; 2] = [
    (
        OBJECTS,
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"first-filament-cube\"/>",
    ),
    (
        OBJECTS,
        "<object id=\"4\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"4\">\n    <metadata key=\"name\" value=\"second-filament-cube\"/>",
    ),
];

#[test]
fn plater_arrange_keeps_a_two_colour_object_clear_of_the_prime_tower_like_native() {
    let edits = [
        NO_TOWER_BRIM,
        RECTANGLE_TOWER[0],
        RECTANGLE_TOWER[1],
        PRIME_VOLUME_PURGE,
        (
            PROJECT,
            "\"wipe_tower_x\": [\n    \"150\"\n  ]",
            "\"wipe_tower_x\": [\n    \"1\"\n  ]",
        ),
        (
            PROJECT,
            "\"wipe_tower_y\": [\n    \"150\"\n  ]",
            "\"wipe_tower_y\": [\n    \"1\"\n  ]",
        ),
    ];
    assert_arranged_like_native(
        "tower-two-colour",
        TWO_COLOURS,
        &edits,
        settings(false, false),
    );
}

#[test]
fn plater_arrange_keeps_objects_of_two_filaments_clear_of_the_prime_tower_like_native() {
    let edits = [
        NAMED[0],
        NAMED[1],
        NO_TOWER_BRIM,
        RECTANGLE_TOWER[0],
        RECTANGLE_TOWER[1],
        PRIME_VOLUME_PURGE,
        TWO_FILAMENTS_CORNER[0],
        TWO_FILAMENTS_CORNER[1],
        LONG_SECOND,
    ];
    assert_arranged_like_native(
        "tower-two-filaments",
        TWO_FILAMENTS,
        &edits,
        settings(false, false),
    );
}

#[test]
fn plater_arrange_rotates_objects_around_the_prime_tower_like_native() {
    let turned = (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 110 116 5\"",
        "transform=\"1.299038106 0.75 0 -0.5 0.866025404 0 0 0 1 110 116 5\"",
    );
    let edits = [
        NAMED[0],
        NAMED[1],
        NO_TOWER_BRIM,
        RECTANGLE_TOWER[0],
        RECTANGLE_TOWER[1],
        PRIME_VOLUME_PURGE,
        TWO_FILAMENTS_CORNER[0],
        TWO_FILAMENTS_CORNER[1],
        LONG_SECOND,
        turned,
    ];
    assert_arranged_like_native(
        "tower-rotated",
        TWO_FILAMENTS,
        &edits,
        settings(true, false),
    );
}

#[test]
fn plater_arrange_overflows_two_filaments_onto_a_new_plate_with_its_tower_like_native() {
    // Five 70 mm instances beside the tower do not fit one 220 mm plate.
    let edits = [
        NAMED[0],
        NAMED[1],
        NO_TOWER_BRIM,
        RECTANGLE_TOWER[0],
        RECTANGLE_TOWER[1],
        PRIME_VOLUME_PURGE,
        TWO_FILAMENTS_CORNER[0],
        TWO_FILAMENTS_CORNER[1],
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 110 116 5\"",
            "7 0 0 0 7 0 0 0 1 110 116 5\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 110 104 5\"",
            "7 0 0 0 7 0 0 0 1 110 104 5\"",
        ),
        (
            MODEL,
            "</build>",
            "<item objectid=\"2\" transform=\"7 0 0 0 7 0 0 0 1 60 40 5\" printable=\"1\"/><item objectid=\"4\" transform=\"7 0 0 0 7 0 0 0 1 160 40 5\" printable=\"1\"/><item objectid=\"2\" transform=\"7 0 0 0 7 0 0 0 1 40 160 5\" printable=\"1\"/></build>",
        ),
    ];
    assert_arranged_like_native(
        "tower-overflow",
        TWO_FILAMENTS,
        &edits,
        settings(false, false),
    );
}

#[test]
fn plater_arrange_keeps_one_filament_clear_of_a_smooth_timelapse_tower_like_native() {
    // A 40 mm cube on one filament; the 20 x 12.6 mm tower right of the bed centre takes the
    // place the pile would be centred on, so the cube moves 10 mm left of it. The CLI only reads the
    // smooth timelapse with `--enable-timelapse` (`OrcaSlicer.cpp:4303-4306`); the GUI reads
    // `timelapse_type` itself (`ArrangeJob.cpp:457-459`).
    let edits = [
        NO_TOWER_BRIM,
        RECTANGLE_TOWER[0],
        RECTANGLE_TOWER[1],
        (
            MODEL,
            "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"",
            "transform=\"4 0 0 0 4 0 0 0 1 40 60 5\"",
        ),
        (
            PROJECT,
            "\"timelapse_type\": \"0\"",
            "\"timelapse_type\": \"1\"",
        ),
        (
            PROJECT,
            "\"wipe_tower_x\": [\n\t\t\"15\"\n\t]",
            "\"wipe_tower_x\": [\n\t\t\"120\"\n\t]",
        ),
        (
            PROJECT,
            "\"wipe_tower_y\": [\n\t\t\"220\"\n\t]",
            "\"wipe_tower_y\": [\n\t\t\"95\"\n\t]",
        ),
    ];
    assert_arranged_like_native_with(
        "tower-smooth-timelapse",
        SINGLE,
        &edits,
        settings(false, false),
        &["--enable-timelapse=1"],
    );
}

#[test]
fn plater_arrange_adds_no_prime_tower_to_a_by_object_print_like_native() {
    let by_object = (
        PROJECT,
        "\"print_sequence\": \"by layer\"",
        "\"print_sequence\": \"by object\"",
    );
    let edits = [NO_TOWER_BRIM, by_object];
    assert_arranged_like_native(
        "tower-by-object",
        TWO_COLOURS,
        &edits,
        settings(false, false),
    );
}

#[test]
fn plater_arrange_without_the_prime_tower_packs_two_filaments_like_native() {
    let disabled = (
        PROJECT,
        "\"enable_prime_tower\": \"1\"",
        "\"enable_prime_tower\": \"0\"",
    );
    let edits = [NAMED[0], NAMED[1], NO_TOWER_BRIM, disabled];
    assert_arranged_like_native(
        "tower-disabled",
        TWO_FILAMENTS,
        &edits,
        settings(false, false),
    );
}
