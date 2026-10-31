//! Closed Arachne outer loops retain exterior seam identity through overhangs.

use super::first_cap_layer_object;

#[tokio::test]
async fn nearest_retains_exterior_identity_for_fully_overhanging_arachne_walls() {
    let project = include_bytes!(
        "../../../../tests/parity/arachne_fully_overhanging_exterior_nearest/stepped_cap.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_fully_overhanging_exterior_nearest",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:565-566 closed Arachne loop inset transfer",
                "GCode/SeamPlacer.cpp:487-497 fully overhanging exterior identity",
            ],
        ),
        "stepped_cap",
        project,
        "31909cf497033d59292683c0224ccc6c6806a368f766a1bac04747e8c64ea64e",
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
        "Nearest seam output for a fully overhanging Arachne exterior must match the runtime beta oracle: {:?}",
        reference.root,
    );
}
