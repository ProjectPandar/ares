//! The first-layer-scan calibration region in the arrange
//! (`docs/architecture/web-plater-arrange-calibration-region.md`): for a Bambu Lab vendor printer
//! with "avoid extrusion calibration region" on and `scan_first_layer` set, `ArrangeJob::process`
//! adds the (18, 0)-(240, 15) mm box on every bed (`ArrangeJob.cpp:732-734`,
//! `PartPlate.cpp:7799-7829` at v2.5.0-beta `345efec64`). GUI-only: the CLI global arrange never
//! adds it, so the cases are properties of the box and the arrange's centring, with controls.

use super::{Edit, MODEL, PROJECT, edited_input, settings};
use crate::runner::{profile_entry, repo_root};
use ares_core::{ArrangeScope, ArrangeSettings, PlaterSession, PresetCatalogHandle};

/// A Bambu Lab X2D (shared bed X 20.5..256, Y 0..256) with one 10 mm cube at (138.25, 128).
const X2D: &str = "tests/parity/generated-printer-ids/input.3mf";
/// The cube scaled to 230 mm deep: centred at Y 128 it spans Y 13..243, into the region's Y 0..15
/// within its X 18..240.
const DEEP: Edit = (
    MODEL,
    "transform=\"1 0 0 0 1 0 0 0 1 138.25 128 5\"",
    "transform=\"1 0 0 0 23 0 0 0 1 138.25 128 5\"",
);
const HALF_DEPTH: f64 = 115.0;
const SCAN_FIRST_LAYER: Edit = (
    PROJECT,
    "\"scan_first_layer\": \"0\"",
    "\"scan_first_layer\": \"1\"",
);
/// The region's back edge.
const REGION_MAX_Y: f64 = 15.0;

/// The deep-cube X2D, scanning its first layer when `scan`, with BBL installed when `bbl`.
fn session(scan: bool, bbl: bool) -> PlaterSession {
    let edits: &[Edit] = if scan {
        &[DEEP, SCAN_FIRST_LAYER]
    } else {
        &[DEEP]
    };
    let mut session =
        PlaterSession::load(edited_input("calibration-region", (X2D, edits))).unwrap();
    if bbl {
        let profiles = repo_root().join("OrcaSlicer/resources/profiles");
        let mut installed = PresetCatalogHandle::new();
        let vendor = profile_entry::vendor(&profiles, "BBL");
        installed
            .add_vendor(&serde_json::to_vec(&vendor).unwrap())
            .unwrap();
        session.set_installed_vendors(&installed);
    }
    session
}

/// Arranges `scope` with "avoid extrusion calibration region" set to `avoid`, then the cube's front
/// edge, checking it stays on plate 1.
fn front_edge(mut session: PlaterSession, scope: ArrangeScope, avoid: bool) -> f64 {
    let settings = ArrangeSettings {
        avoid_extrusion_cali_region: avoid,
        ..settings(false, false)
    };
    session.arrange(scope, settings).unwrap();
    assert!(session.plates()[0].instances.contains(&(0, 0)));
    session.instance_transform(0, 0).unwrap().to_column_major()[13] - HALF_DEPTH
}

#[test]
fn plater_arrange_keeps_clear_of_the_first_layer_scan_region() {
    let avoided = front_edge(session(true, true), ArrangeScope::All, true);
    assert!(
        avoided >= REGION_MAX_Y,
        "the front edge at Y {avoided} reaches into the region"
    );
    // Controls: without "avoid", or without the first-layer scan, there is no region and the cube
    // stays centred, reaching into the box.
    for (scan, avoid) in [(true, false), (false, true)] {
        let centred = front_edge(session(scan, true), ArrangeScope::All, avoid);
        assert!(
            centred < REGION_MAX_Y,
            "scan {scan}, avoid {avoid}: front edge at Y {centred}"
        );
    }
}

#[test]
fn plater_arrange_adds_no_scan_region_without_a_bambu_lab_vendor() {
    // No vendor installed: the vendor type is Unknown, so `is_bbl_vendor` is false
    // (`PresetBundle.hpp:374`) and the cube arranges as without the region.
    let unknown = front_edge(session(true, false), ArrangeScope::All, true);
    let control = front_edge(session(true, true), ArrangeScope::All, false);
    assert_eq!(unknown, control);
}

#[test]
fn plater_arrange_on_the_current_plate_keeps_clear_of_the_first_layer_scan_region() {
    let avoided = front_edge(session(true, true), ArrangeScope::CurrentPlate(0), true);
    assert!(
        avoided >= REGION_MAX_Y,
        "the front edge at Y {avoided} reaches into the region"
    );
}
