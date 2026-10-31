//! Runtime island visits retain the first object-layer's complete move order.

#[tokio::test]
async fn nested_island_visits_follow_runtime_orca_order() {
    let project = include_bytes!(
        "../../../../../tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "island-visit-order",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode.cpp:7238-7362 process_layer per-filament IslandOrderNode/InstanceVisit producer",
                "GCode.cpp:7680-7720 process_layer explicit visit island consumption",
                "Polygon.cpp:154-169 centroid",
            ],
        ),
        "nested-hole-island",
        project,
        "f91fae699cd1410003fe4c0d59ce2602c0fa1039e709c11d195641e5488da74c",
        false,
    );
    let native = first_printed_object_layer(&reference.native);
    assert!(native.contains(";TYPE:Inner wall\n"));
    assert!(native.contains(";TYPE:Outer wall\n"));
    std::fs::write(
        reference.root.join("native.first_object_layer.gcode"),
        native,
    )
    .unwrap();

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
    let api_layer = first_printed_object_layer(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.first_object_layer.gcode"),
        api_layer,
    )
    .unwrap();
    assert_eq!(
        api_layer, native,
        "The complete first object-layer, including its initial travel, must match runtime beta island visits: {:?}",
        reference.root,
    );
}

fn first_printed_object_layer(gcode: &str) -> &str {
    gcode
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
