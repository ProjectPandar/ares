//! The arrange on the shared bed of several nozzles (`docs/architecture/web-plater-arrange-shared-bed.md`):
//! `get_bed_shape(config, use_share = true)` returns `get_shared_poly` of `extruder_printable_area`
//! (`PrintConfig.cpp:13820-13864` at v2.5.0-beta `345efec64`). The GUI job and the CLI global arrange
//! both read it (`ArrangeJob.cpp:741`, `OrcaSlicer.cpp:5514, 5989`), so the cases compare with Native.

use super::{
    Edit, MODEL, PROJECT, assert_arranged_like_native, assert_scope_arranged_like_native,
    edited_input, settings,
};
use ares_core::{ArrangeScope, PlaterSession, SliceError};

/// A Bambu Lab X2D: nozzle areas X 0..256 and X 20.5..256 on a 256 mm bed, one object on
/// filament 1.
const X2D: &str = "tests/parity/generated-printer-ids/input.3mf";
/// The object moved off the shared bed's centre (138.25, 128), so an arrange of it shows.
const MOVED: Edit = (
    MODEL,
    "transform=\"1 0 0 0 1 0 0 0 1 138.25 128 5\"",
    "transform=\"1 0 0 0 1 0 0 0 1 60 60 5\"",
);

#[test]
fn plater_arrange_packs_into_the_shared_bed_of_two_nozzles_like_native() {
    assert_arranged_like_native("shared-bed", X2D, &[MOVED], settings(false, false));
}

#[test]
fn plater_arrange_packs_several_instances_into_the_shared_bed_like_native() {
    let more = (
        MODEL,
        "</build>",
        "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 200 60 5\" printable=\"1\"/><item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 60 200 5\" printable=\"1\"/></build>",
    );
    assert_arranged_like_native(
        "shared-bed-several",
        X2D,
        &[MOVED, more],
        settings(false, false),
    );
}

#[test]
fn plater_arrange_on_the_current_plate_packs_into_the_shared_bed_like_native() {
    assert_scope_arranged_like_native(
        "shared-bed-current-plate",
        (X2D, &[MOVED]),
        settings(false, false),
        &[],
        ArrangeScope::CurrentPlate(0),
    );
}

#[test]
fn plater_arrange_refuses_nozzle_areas_that_share_no_area() {
    // `get_shared_poly` indexes the first polygon of an empty intersection, undefined behaviour
    // upstream, so there is no Native case.
    let disjoint = (
        PROJECT,
        "\"20.5x0,256x0,256x256,20.5x256\"",
        "\"300x0,556x0,556x256,300x256\"",
    );
    let input = edited_input("disjoint-nozzle-areas", (X2D, &[MOVED, disjoint]));
    let mut session = PlaterSession::load(input).unwrap();
    let result = session.arrange(ArrangeScope::All, settings(false, false));
    assert!(
        matches!(&result, Err(SliceError::InvalidInput(message)) if message == "the nozzle areas share no printable area"),
        "{result:?}"
    );
}
