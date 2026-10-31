//! "Arrange" on the current plate (the A3c addendum in
//! `docs/architecture/web-plater-move-rotate-arrange.md`): `ArrangeJob::prepare_partplate` and
//! the current-plate finalize at v2.5.0-beta `345efec64`, against the CLI global arrange of the
//! plate it slices (`--slice <n>`). Plate 1's object stays where it is; Native exports only
//! plate 2, so its instances are compared.

use ares_core::ArrangeScope;

use super::{Edit, MODEL, OBJECTS, PROJECT, assert_scope_arranged_like_native, settings};

/// An Ender-3 project with a cube on each of two plates, one filament.
const TWO_PLATES: &str = "tests/parity/custom-gcode-overview/two-plates.3mf";
/// The second plate, 0-based.
const PLATE_2: ArrangeScope = ArrangeScope::CurrentPlate(1);

/// Names the two cubes apart; both load as `cube10.stl`.
const NAMED: [Edit; 2] = [
    (
        OBJECTS,
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"2\">\n    <metadata key=\"name\" value=\"plate-1-cube\"/>",
    ),
    (
        OBJECTS,
        "<object id=\"3\">\n    <metadata key=\"name\" value=\"cube10.stl\"/>",
        "<object id=\"3\">\n    <metadata key=\"name\" value=\"plate-2-cube\"/>",
    ),
];
const OFF_CENTRE: Edit = (MODEL, " 374 110 5\"", " 300 60 5\"");
/// Two more instances of plate 2's cube, on plate 2.
const TWO_MORE: Edit = (
    MODEL,
    "</build>",
    "<item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 420 160 5\" printable=\"1\"/><item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 330 170 5\" printable=\"1\"/></build>",
);

#[test]
fn plater_arrange_centres_an_object_on_the_current_plate_like_native() {
    let edits = [NAMED[0], NAMED[1], OFF_CENTRE];
    assert_scope_arranged_like_native(
        "plate-centre",
        (TWO_PLATES, &edits),
        settings(false, false),
        &[],
        PLATE_2,
    );
}

#[test]
fn plater_arrange_packs_several_objects_on_the_current_plate_like_native() {
    let edits = [NAMED[0], NAMED[1], OFF_CENTRE, TWO_MORE];
    assert_scope_arranged_like_native(
        "plate-several",
        (TWO_PLATES, &edits),
        settings(false, false),
        &[],
        PLATE_2,
    );
}

#[test]
fn plater_arrange_rotates_objects_on_the_current_plate_like_native() {
    let turned = (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 374 110 5\"",
        "transform=\"0.866025404 0.5 0 -0.5 0.866025404 0 0 0 1 374 110 5\"",
    );
    let edits = [NAMED[0], NAMED[1], turned, TWO_MORE];
    assert_scope_arranged_like_native(
        "plate-rotated",
        (TWO_PLATES, &edits),
        settings(true, false),
        &[],
        PLATE_2,
    );
}

#[test]
fn plater_arrange_aligns_long_sides_to_y_on_the_current_plate_like_native() {
    let long = (
        MODEL,
        "transform=\"1 0 0 0 1 0 0 0 1 374 110 5\"",
        "transform=\"3 0 0 0 1 0 0 0 1 374 110 5\"",
    );
    let edits = [NAMED[0], NAMED[1], long, TWO_MORE];
    assert_scope_arranged_like_native(
        "plate-align-to-y",
        (TWO_PLATES, &edits),
        settings(false, true),
        &[],
        PLATE_2,
    );
    assert_scope_arranged_like_native(
        "plate-align-to-y-off",
        (TWO_PLATES, &edits),
        settings(false, false),
        &[],
        PLATE_2,
    );
}

#[test]
fn plater_arrange_keeps_clear_of_the_bed_exclude_area_on_the_current_plate_like_native() {
    let excluded = (
        PROJECT,
        "\"bed_exclude_area\": [\n\t\t\"0x0\"\n\t]",
        "\"bed_exclude_area\": [\n\t\t\"0x0\",\n\t\t\"120x0\",\n\t\t\"120x120\",\n\t\t\"0x120\"\n\t]",
    );
    let edits = [NAMED[0], NAMED[1], OFF_CENTRE, TWO_MORE, excluded];
    assert_scope_arranged_like_native(
        "plate-exclude-area",
        (TWO_PLATES, &edits),
        settings(false, false),
        &[],
        PLATE_2,
    );
}

#[test]
fn plater_arrange_spaces_a_by_object_plate_in_a_by_layer_project_like_native() {
    let by_object = (
        OBJECTS,
        "<metadata key=\"plater_id\" value=\"2\"/>",
        "<metadata key=\"plater_id\" value=\"2\"/>\n    <metadata key=\"print_sequence\" value=\"by object\"/>",
    );
    let edits = [NAMED[0], NAMED[1], OFF_CENTRE, TWO_MORE, by_object];
    assert_scope_arranged_like_native(
        "plate-by-object",
        (TWO_PLATES, &edits),
        settings(false, false),
        &[],
        PLATE_2,
    );
}
