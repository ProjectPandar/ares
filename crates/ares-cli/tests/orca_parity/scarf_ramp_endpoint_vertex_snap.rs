//! OrcaSlicer 2.5.0 `ExtrusionEntity.cpp:493-519`: keep a straight ramp's exact length.

#[tokio::test]
async fn scarf_ramp_keeps_exact_length_without_a_short_flat_stub() {
    let project = include_bytes!("../../../../tests/parity/scarf-default-threshold/boundary.3mf");
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "scarf_ramp_endpoint_vertex_snap",
            &[
                "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19",
                "ExtrusionEntity.cpp:493-519 split_at_slope_end",
                "ExtrusionEntity.cpp:592-600 sloped-loop ramp split",
            ],
        ),
        "existing_boundary",
        project,
        "1646ba992cc58685074abd95e009d17f85f88e190379912201d9d9f88399f4e5",
        false,
    );
    let native_object = second_layer_object(&reference.native);
    assert!(native_object.contains("G1 X107.5 Y105 Z.25 "));
    assert!(native_object.contains("G1 F3600\n"));
    assert!(native_object.contains("G1 F7200\n"));
    std::fs::write(
        reference.root.join("native.second_layer_object.gcode"),
        native_object,
    )
    .unwrap();

    let api = crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 30, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api_object = second_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.second_layer_object.gcode"),
        api_object,
    )
    .unwrap();
    assert_eq!(
        api_object, native_object,
        "the complete scarf object must match the runtime exact-length oracle: {:?}",
        reference.root,
    );
}

fn second_layer_object(gcode: &str) -> &str {
    gcode
        .split(";LAYER_CHANGE\n")
        .nth(2)
        .unwrap()
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
