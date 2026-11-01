//! Beta Arachne simplify uses coordinate epsilon, not the old 5um early-out.

#[tokio::test]
async fn narrow_neck_uses_native_collinear_vertex_tolerance() {
    let project = include_bytes!(
        "../../../../tests/parity/validation/arachne-open-wall-order/nearest-control-neck0.25-r0.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_collinear_vertex_tolerance",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "Arachne/utils/ExtrusionLine.hpp:34-40 colinear_vertex_tolerance",
                "Arachne/utils/ExtrusionLine.cpp:139-144 simplify early-out",
                "Arachne/WallToolPaths.cpp:176-179 polygon simplify early-out",
            ],
        ),
        "existing_nearest_neck_r0",
        project,
        "ce5caae069fdf344382fc395f1ec85f576623bd33fbe483706a9b980705bcc41",
        false,
    );
    let native = super::first_layer_object(&reference.native);
    std::fs::write(
        reference.root.join("native.first_layer_object.gcode"),
        native,
    )
    .unwrap();
    assert!(native.contains("G1 X107.696 Y108.831 E.07746\n"));
    assert!(native.contains("M73 P64 R0\n"));
    assert!(native.contains(";WIPE_START\n"));

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
    let api = super::first_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.first_layer_object.gcode"), api).unwrap();
    assert_eq!(
        api, native,
        "the whole first-layer neck object must match the runtime beta Arachne geometry: {:?}",
        reference.root,
    );
}
