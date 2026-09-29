use super::PressureEqualizerPass;

fn pass() -> PressureEqualizerPass {
    PressureEqualizerPass::new(100.0, 2.0, false, &[1.75], false)
}

/// A layer of two extrusion lines at very different feedrates: the pass
/// re-emits them with the marker structure and (with the look-back
/// window) may lower the fast line's rates.
#[test]
fn pass_rewrites_layer_with_markers() {
    let mut pass = pass();
    let layer = ";_EXTRUSION_ROLE:1\n\
                 G1 F300;_EXTRUDE_SET_SPEED\n\
                 G1 X10 Y0 E1\n\
                 G1 X20 Y0 E2\n\
                 ;_EXTRUDE_END\n";

    let out = pass.process_layer(layer);

    // First call returns None (the previous-layer buffering).
    assert!(out.is_none());
    let out = pass.flush().expect("buffered layer");
    assert!(out.contains(";_EXTRUDE_END"));
    assert!(out.contains(";_EXTRUDE_SET_SPEED"));
    // Both extrusion lines survive the rewrite.
    assert!(out.lines().filter(|l| l.starts_with("G1 X")).count() >= 2);
}

/// Marker-only lines are filtered from the output (they carry state,
/// not motion).
#[test]
fn role_markers_are_filtered() {
    let mut pass = pass();
    let layer = ";_EXTRUSION_ROLE:2\n\
                 G1 F600;_EXTRUDE_SET_SPEED\n\
                 G1 X5 Y0 E1\n\
                 ;_EXTRUDE_END\n";

    pass.process_layer(layer);
    let out = pass.flush().expect("buffered layer");

    assert!(!out.contains(";_EXTRUSION_ROLE:"));
}

/// Non-extruding layers pass through unchanged (modulo markers).
#[test]
fn travel_only_layer_passes_through() {
    let mut pass = pass();
    let layer = "G1 X10 Y0 F9000\nG1 X0 Y0 F9000\n";

    pass.process_layer(layer);
    let out = pass.flush().expect("buffered layer");

    assert_eq!(out, "G1 X10 Y0 F9000\nG1 X0 Y0 F9000\n");
}

/// Two extrusion lines separated by a sub-3mm travel are treated as one
/// continuous segment (the small-gap bridge,
/// `PressureEqualizer.cpp:136-152`).
#[test]
fn small_travel_gap_bridges_segments() {
    let mut pass = pass();
    let layer = ";_EXTRUSION_ROLE:1\n\
                 G1 F300;_EXTRUDE_SET_SPEED\n\
                 G1 X10 Y0 E1\n\
                 G1 X11 Y0 F9000\n\
                 G1 X20 Y0 E2\n\
                 ;_EXTRUDE_END\n";

    pass.process_layer(layer);
    let out = pass.flush().expect("buffered layer");

    // The travel survives verbatim; the extrusions were re-emitted.
    assert!(out.contains("G1 X11 Y0 F9000"));
    assert!(out.lines().filter(|l| l.starts_with("G1 X")).count() >= 3);
}

/// The previous layer stays in the line buffer while the next one is
/// processed (`PressureEqualizer.cpp:174-200`), so an extrusion segment
/// spans the layer change: the next layer's fast wall ramps up from the
/// previous layer's slow tail and is split into segments.
#[test]
fn segment_spans_the_layer_change() {
    let mut pass = pass();
    let slow = ";_EXTRUSION_ROLE:1\n\
                G1 F300;_EXTRUDE_SET_SPEED\n\
                G1 X10 Y0 E0.5\n\
                G1 X10 Y10 E1\n\
                G1 X0 Y10 E1.5\n\
                G1 X0 Y1 E1.95\n\
                ;_EXTRUDE_END\n";
    let fast = "G1 X1 Y1 F9000\n\
                ;_EXTRUSION_ROLE:1\n\
                G1 F6000;_EXTRUDE_SET_SPEED\n\
                G1 X31 Y1 E3.45\n\
                G1 X31 Y31 E4.95\n\
                G1 X1 Y31 E6.45\n\
                ;_EXTRUDE_END\n";

    assert!(pass.process_layer(slow).is_none());
    pass.process_layer(fast).expect("first layer");
    let out = pass.flush().expect("second layer");

    let extrusions = out
        .lines()
        .filter(|l| l.starts_with("G1 X") && l.contains('E'));
    assert!(extrusions.count() > 3, "{out}");
}
