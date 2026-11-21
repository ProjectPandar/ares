//! `SeamPlacer.cpp:1581`: the concave-corner seam gate compares
//! `local_ccw_angle < -EPSILON` with the shared `EPSILON = 1e-4`
//! double literal (`libslic3r.h:52`), not the f32 machine epsilon.

use crate::runner::native_stream_case::{check, qualify};

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/seam-position-random/x1c-seam-position-random.3mf");
const SHA: &str = "11c3ec4b666ec546f899debb200fcddb5e18e97221a6a7f46694459c4e499534";

#[test]
fn seam_position_random_concave_epsilon_matches_complete_output() {
    // Seeded-only oracle: two official replays of the single-object KSR
    // project disagree on the uninitialized `; printing object` id, so
    // `native_stream_case::qualify` would panic with Native
    // nondeterminism in official mode (same qualification as
    // `initialized_object_id.rs`).
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    check(
        (
            "seam-position-random",
            &["libslic3r.h:52", "SeamPlacer.cpp:1581"],
        ),
        "x1c-seam-position-random",
        PROJECT,
        SHA,
        (false, |_, _| {}),
    );
}

#[test]
fn farthest_timelapse_matches_native_before_closing_arc() {
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    // Without the preset catalog this project does not exercise the inline hook.
    assert!(
        crate::runner::application::project_supply()
            .unwrap()
            .is_some()
    );
    let (native, _) = qualify(
        (
            "farthest-timelapse",
            &["GCode.cpp:6390-6435", "GCode.cpp:7259-7262"],
        ),
        "x1c-seam-position-random",
        PROJECT,
        SHA,
        false,
    );
    let ares = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(native.root.join("ares.api.gcode"), &ares).unwrap();

    fn snapshot_window(gcode: &str) -> &str {
        let (_, layer) = gcode
            .split_once("M991 S0 P44 ;notify layer change\n")
            .unwrap();
        let (layer, _) = layer
            .split_once("M991 S0 P45 ;notify layer change\n")
            .unwrap();
        let (_, window) = layer.split_once("G1 X169.894 Y141.957 E2.02179\n").unwrap();
        let (window, _) = window.split_once("; FEATURE: Outer wall\n").unwrap();
        assert_eq!(window.matches("; SKIPTYPE: timelapse\n").count(), 1);
        assert!(window.contains("G3 X168.873 Y143.299 I-1.382 J.008 E.06016\n"));
        window
    }

    let native_window = snapshot_window(&native.native);
    assert!(native_window.starts_with(";========Date"));
    assert_eq!(
        snapshot_window(std::str::from_utf8(&ares).unwrap()),
        native_window,
        "snapshot placement between the Inner wall G1 and its closing G3: {:?}",
        native.root,
    );
}
