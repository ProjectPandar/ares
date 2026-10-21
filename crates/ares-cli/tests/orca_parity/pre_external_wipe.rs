//! Upstream pre-external force-no-extrusion path public Native/API/CLI streams.

#[test]
fn qualifying_prewipe_retraction() {
    check(
        "single-wipe-threshold02",
        include_bytes!("../../../../tests/parity/pre-external-wipe/single-wipe-threshold02.3mf"),
        "ba4ce2095ec47e5f171ab8f1fd4be9af976e36f30a41c33dfd9c52b4be1254d7",
        false,
    );
}

#[test]
fn disabled_prewipe_control() {
    check(
        "single-off-threshold02",
        include_bytes!("../../../../tests/parity/pre-external-wipe/single-off-threshold02.3mf"),
        "ca9e648a6249b33973f7685923e65e3bf8222b9d115dfce5075a56c064d6b1d8",
        false,
    );
}

#[test]
fn original_single_control() {
    check(
        "single",
        include_bytes!("../../../../tests/parity/interlocking-base/single.3mf"),
        "1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651",
        false,
    );
}

#[test]
fn default_beam_control() {
    check(
        "default-beam1",
        include_bytes!("../../../../tests/parity/interlocking-base/two-extruder-beam1.3mf"),
        "04c7d2a58b7b21909b867542faad149a6b9d0b1bb976eab3fec57b7b60ff082c",
        true,
    );
}

fn check(label: &str, project: &[u8], sha: &str, two_tools: bool) {
    crate::runner::native_stream_case::check(
        (
            "pre-external-wipe",
            &[
                "GCode.cpp:extrude_loop:5884-5888",
                "GCode.cpp:extrude_path:6109/_extrude:6378/travel_to:7440/retract:7654",
            ],
        ),
        label,
        project,
        sha,
        (two_tools, |_, _| {}),
    );
}
