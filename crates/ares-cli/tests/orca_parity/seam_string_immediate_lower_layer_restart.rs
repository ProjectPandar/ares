//! Rear fitting retains the layer immediately below an upward continuation gap.

#[tokio::test]
async fn back_seam_fits_layer_immediately_below_upward_gap() {
    let project = include_bytes!(
        "../../../../tests/parity/seam_string_immediate_lower_layer_restart/rear_low_motion_peak.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "seam_string_immediate_lower_layer_restart",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode/SeamPlacer.cpp:1865-1869 find_seam_string upward-miss reversal",
                "GCode/SeamPlacer.cpp:1969-1982 alternative-start sampling",
            ],
        ),
        "rear_low_motion_peak",
        project,
        "be46fa7a81ee17aef5019dc3e341b1264e7261c551a60cbce91849842521c02e",
        false,
    );
    let native = immediately_below_gap_object(&reference.native);
    assert!(!native.contains(";TYPE:Overhang wall\n"));
    std::fs::write(reference.root.join("native.below_gap_object.gcode"), native).unwrap();

    let api = ares_core::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 31, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api = immediately_below_gap_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.below_gap_object.gcode"), api).unwrap();
    assert_eq!(
        api, native,
        "Back seam fitting immediately below the upward gap must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

fn immediately_below_gap_object(gcode: &str) -> &str {
    gcode
        .split_once(";LAYER_CHANGE\n;Z:4.4\n")
        .unwrap()
        .1
        .split_once("; printing object ")
        .unwrap()
        .1
        .split_once('\n')
        .unwrap()
        .1
        .split_once("; stop printing object ")
        .unwrap()
        .0
}
