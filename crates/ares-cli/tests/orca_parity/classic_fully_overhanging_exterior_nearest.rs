//! Fully overhanging classic outer loops retain exterior seam identity.

#[tokio::test]
async fn nearest_keeps_exterior_identity_for_fully_overhanging_walls() {
    let project = include_bytes!(
        "../../../../tests/parity/classic_fully_overhanging_exterior_nearest/stepped_cap.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "classic_fully_overhanging_exterior_nearest",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode/SeamPlacer.cpp:487-501 extract_perimeter_polygons",
                "GCode/SeamPlacer.cpp:495-496 fully overhanging outer loop inset identity",
            ],
        ),
        "stepped_cap",
        project,
        "3a0312948617d5e8f8cbb65cb56a1a4388b34b044bd65e3ead9e325949353f00",
        false,
    );
    let native = first_cap_layer_object(&reference.native);
    assert!(native.contains(";TYPE:Overhang wall\n"));
    assert!(!native.contains(";TYPE:Outer wall\n"));
    assert!(!native.contains(";TYPE:Inner wall\n"));
    std::fs::write(reference.root.join("native.first_cap_object.gcode"), native).unwrap();

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
    let api = first_cap_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.first_cap_object.gcode"), api).unwrap();
    assert_eq!(
        api, native,
        "Nearest seam output for a fully overhanging classic exterior must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

fn first_cap_layer_object(gcode: &str) -> &str {
    gcode
        .split_once(";LAYER_CHANGE\n;Z:1\n")
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
