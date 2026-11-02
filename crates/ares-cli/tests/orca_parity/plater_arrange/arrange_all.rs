//! "Arrange all" cases against the official v2.5.0-beta CLI global arrange (see `plater_arrange.rs`).

use super::{
    BY_LAYER, CUBES, Edit, MODEL, NAMED, PROJECT, SINGLE, TURNED_AND_LONG,
    assert_arranged_like_native, settings,
};

#[test]
fn plater_arrange_centres_a_single_object_like_native() {
    let off_centre = (MODEL, " 110 110 5\"", " 40 60 5\"");
    assert_arranged_like_native("single", SINGLE, &[off_centre], settings(false, false));
}

#[test]
fn plater_arrange_packs_several_objects_with_and_without_rotation_like_native() {
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        TURNED_AND_LONG[0],
        TURNED_AND_LONG[1],
    ];
    assert_arranged_like_native("several", CUBES, &edits, settings(false, false));
    assert_arranged_like_native("several-rotated", CUBES, &edits, settings(true, false));
}

#[test]
fn plater_arrange_overflows_onto_a_new_plate_like_native() {
    let scaled = |position| -> Edit {
        match position {
            50 => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 50 110 2\"",
                "8 0 0 0 8 0 0 0 1 50 110 2\"",
            ),
            125 => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 125 110 2\"",
                "8 0 0 0 8 0 0 0 1 125 110 2\"",
            ),
            _ => (
                MODEL,
                "1 0 0 0 1 0 0 0 1 200 110 2\"",
                "8 0 0 0 8 0 0 0 1 200 110 2\"",
            ),
        }
    };
    // Two more 80 mm instances of the first cube: five do not fit one 220 mm plate.
    let more = (
        MODEL,
        "</build>",
        "<item objectid=\"2\" transform=\"8 0 0 0 8 0 0 0 1 60 40 2\" printable=\"1\"/><item objectid=\"2\" transform=\"8 0 0 0 8 0 0 0 1 160 40 2\" printable=\"1\"/></build>",
    );
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        scaled(50),
        scaled(125),
        scaled(200),
        more,
    ];
    assert_arranged_like_native("overflow", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_aligns_long_sides_to_y_on_an_i3_printer_like_native() {
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        TURNED_AND_LONG[0],
        TURNED_AND_LONG[1],
    ];
    assert_arranged_like_native("align-to-y", CUBES, &edits, settings(false, true));
    assert_arranged_like_native("align-to-y-off", CUBES, &edits, settings(false, false));
}

/// The three instances scaled to 50 mm.
const FIFTY_MM: [Edit; 3] = [
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 50 110 2\"",
        "5 0 0 0 5 0 0 0 1 50 110 2\"",
    ),
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 125 110 2\"",
        "5 0 0 0 5 0 0 0 1 125 110 2\"",
    ),
    (
        MODEL,
        "1 0 0 0 1 0 0 0 1 200 110 2\"",
        "5 0 0 0 5 0 0 0 1 200 110 2\"",
    ),
];

#[test]
fn plater_arrange_keeps_clear_of_the_bed_exclude_area_like_native() {
    let excluded = (
        PROJECT,
        "\"bed_exclude_area\": [\n    \"0x0\"\n  ]",
        "\"bed_exclude_area\": [\n    \"0x0\",\n    \"120x0\",\n    \"120x120\",\n    \"0x120\"\n  ]",
    );
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        FIFTY_MM[0],
        FIFTY_MM[1],
        FIFTY_MM[2],
        excluded,
    ];
    assert_arranged_like_native("bed-exclude-area", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_spaces_a_by_object_print_for_the_extruder_clearance_like_native() {
    assert_arranged_like_native("by-object", CUBES, &NAMED, settings(false, false));
}

/// A 32-gon of radius 110 mm about the bed centre; the bed shrink makes it a polygon bin.
const ROUND_BED: Edit = (
    PROJECT,
    "\"printable_area\": [\n    \"0x0\",\n    \"220x0\",\n    \"220x220\",\n    \"0x220\"\n  ]",
    r#""printable_area": [
    "220.000000x110.000000",
    "217.886381x131.459935",
    "211.626749x152.095178",
    "201.461657x171.112726",
    "187.781746x187.781746",
    "171.112726x201.461657",
    "152.095178x211.626749",
    "131.459935x217.886381",
    "110.000000x220.000000",
    "88.540065x217.886381",
    "67.904822x211.626749",
    "48.887274x201.461657",
    "32.218254x187.781746",
    "18.538343x171.112726",
    "8.373251x152.095178",
    "2.113619x131.459935",
    "0.000000x110.000000",
    "2.113619x88.540065",
    "8.373251x67.904822",
    "18.538343x48.887274",
    "32.218254x32.218254",
    "48.887274x18.538343",
    "67.904822x8.373251",
    "88.540065x2.113619",
    "110.000000x0.000000",
    "131.459935x2.113619",
    "152.095178x8.373251",
    "171.112726x18.538343",
    "187.781746x32.218254",
    "201.461657x48.887274",
    "211.626749x67.904822",
    "217.886381x88.540065"
  ]"#,
);

#[test]
fn plater_arrange_fits_a_round_bed_like_native() {
    // Four 70 mm instances: a square bed holds them, the round one only two per plate.
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        ROUND_BED,
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 50 110 2\"",
            "7 0 0 0 7 0 0 0 1 50 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 125 110 2\"",
            "7 0 0 0 7 0 0 0 1 125 110 2\"",
        ),
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 200 110 2\"",
            "7 0 0 0 7 0 0 0 1 200 110 2\"",
        ),
        (
            MODEL,
            "</build>",
            "<item objectid=\"2\" transform=\"7 0 0 0 7 0 0 0 1 60 40 2\" printable=\"1\"/></build>",
        ),
    ];
    assert_arranged_like_native("round-bed", CUBES, &edits, settings(false, false));
}

#[test]
fn plater_arrange_moves_an_unprintable_instance_past_the_last_plate_like_native() {
    // The unprintable instance sits at the bed centre, where the CLI's own arrange of it
    // (`OrcaSlicer.cpp:6012`; the GUI job does not arrange it) leaves it.
    let edits = [
        NAMED[0],
        NAMED[1],
        BY_LAYER,
        (
            MODEL,
            "1 0 0 0 1 0 0 0 1 50 110 2\"",
            "1 0 0 0 1 0 0 0 1 60 40 2\"",
        ),
        (
            MODEL,
            "transform=\"1 0 0 0 1 0 0 0 1 125 110 2\" printable=\"1\"",
            "transform=\"1 0 0 0 1 0 0 0 1 110 110 2\" printable=\"0\"",
        ),
    ];
    assert_arranged_like_native("unprintable", CUBES, &edits, settings(false, false));
}
