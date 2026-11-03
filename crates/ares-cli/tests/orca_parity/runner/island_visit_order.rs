//! Runtime island visits retain the first object-layer's complete move order.

#[tokio::test]
async fn nested_island_visits_follow_runtime_orca_order() {
    let project = include_bytes!(
        "../../../../../tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf"
    );
    compare_first_object_layer(
        "nested-hole-island",
        project,
        "f91fae699cd1410003fe4c0d59ce2602c0fa1039e709c11d195641e5488da74c",
        1,
    )
    .await;
}

#[tokio::test]
async fn four_island_visits_follow_runtime_orca_closed_tour() {
    compare_first_object_layer(
        "four-disjoint-prisms",
        include_bytes!("../../../../../tests/parity/island-visit-order/four-disjoint-prisms.3mf"),
        "1fc8527ace719c91329124259b93767329793e71e41438bfb0e5b2d07bab8be2",
        4,
    )
    .await;
}

async fn compare_first_object_layer(
    case: &str,
    project: &[u8],
    input_sha256: &str,
    minimum_wall_groups: usize,
) {
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "island-visit-order",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode.cpp:7238-7362 process_layer per-filament IslandOrderNode/InstanceVisit producer",
                "GCode.cpp:7680-7720 process_layer explicit visit island consumption",
                "Polygon.cpp:154-169 centroid",
                "ShortestPath.cpp:1624-1650 chain_points_with_postprocessing default tour",
                "GCode/OrderingStrategies.cpp:230-266 tsp_exact_cycle closed-cycle DP",
            ],
        ),
        case,
        project,
        input_sha256,
        false,
    );
    let native = first_printed_object_layer(&reference.native);
    let inner_groups = native.matches(";TYPE:Inner wall\n").count();
    let outer_groups = native.matches(";TYPE:Outer wall\n").count();
    assert!(inner_groups >= minimum_wall_groups);
    assert!(outer_groups >= minimum_wall_groups);
    std::fs::write(
        reference.root.join("native.island_activation.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "minimum_wall_groups": minimum_wall_groups,
            "native_inner_wall_groups": inner_groups,
            "native_outer_wall_groups": outer_groups,
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        reference.root.join("native.first_object_layer.gcode"),
        native,
    )
    .unwrap();

    let api = crate::runner::application::slice_project_plate_validated(
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
