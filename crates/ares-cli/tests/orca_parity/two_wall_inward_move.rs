//! Beta GCode.cpp:8730-8736,8790-8794 and WipePathHelpers.cpp:886-926.

#[tokio::test]
async fn two_wall_layer_preserves_native_inward_destination() {
    let project = include_bytes!("../../../../tests/avoid_crossing_travel/two-walls.3mf");
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "two_wall_inward_move",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode/WipePathHelpers.cpp:886-926 wipe_on_loops_destination",
                "GCode.cpp:8730-8736 destination eligibility",
                "GCode.cpp:8790-8794 writer and logical cursor move",
            ],
        ),
        "existing_two_walls",
        project,
        "ff7c08c7dc1339fe38fc24d23b207bf6edd686c8263c0531d46c9c5372fd644a",
        false,
    );
    assert!(reference.native.contains("; wall_generator = classic\n"));
    assert!(reference.native.contains("; wall_loops = 2\n"));
    assert!(reference.native.contains("; wipe_on_loops = 1\n"));
    let native = target_layer(&reference.native);
    std::fs::write(reference.root.join("native.Z034.whole-layer.gcode"), native).unwrap();
    assert!(native.contains("G1 X164.855 Y179.87\n"));

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
    let api = target_layer(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.Z034.whole-layer.gcode"), api).unwrap();
    let difference = crate::golden::first_difference(native.as_bytes(), api.as_bytes())
        .map(|difference| difference.to_string());
    std::fs::write(
        reference.root.join("whole-layer-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "boundary": "entire Z0.34 layer; every ordered record retained",
            "native_bytes": native.len(), "api_bytes": api.len(),
            "first_difference": difference,
            "rust_entry": "ares_core::slice_project_plate_validated",
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        api, native,
        "the entire two-wall layer must match the runtime beta inward destination: {:?}",
        reference.root,
    );
}

fn target_layer(gcode: &str) -> &str {
    let start = gcode.find(";Z:0.34\n").unwrap();
    let layer = &gcode[start..];
    &layer[..layer.find(";LAYER_CHANGE\n").unwrap()]
}
