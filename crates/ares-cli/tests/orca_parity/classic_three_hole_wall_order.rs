//! Three sibling hole walls follow the beta runtime's shortest chain.

use super::first_layer_object;

#[tokio::test]
async fn three_holes_follow_runtime_shortest_chain() {
    let project =
        include_bytes!("../../../../tests/parity/classic_three_hole_wall_order/three_holes.3mf");
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "classic_three_hole_wall_order",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "ShortestPath.cpp:103-137,164-176 exact small-chain dispatch",
                "ShortestPath.cpp:1429-1475 chain_extrusion_entities",
                "PerimeterGenerator.cpp:280-315 sibling loop traversal from zero",
            ],
        ),
        "three_holes",
        project,
        "cee9fb9113a3d5fc024eb3fc4f2fddd1ac84ff9708606773fbd65690a0bd70ac",
        false,
    );
    let native_first_layer = first_layer_object(&reference.native);
    assert!(native_first_layer.contains(";TYPE:Outer wall\n"));
    assert!(!native_first_layer.contains(";TYPE:Inner wall\n"));
    assert!(!native_first_layer.contains(";TYPE:Overhang wall\n"));
    let first_x: f64 = native_first_layer
        .lines()
        .find_map(|line| line.strip_prefix("G1 X"))
        .unwrap()
        .split_ascii_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(!(105.0..=115.0).contains(&first_x));
    std::fs::write(
        reference.root.join("native.first_layer_object.gcode"),
        native_first_layer,
    )
    .unwrap();

    let api = ares_core::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 30, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api_first_layer = first_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.first_layer_object.gcode"),
        api_first_layer,
    )
    .unwrap();
    assert_eq!(
        api_first_layer, native_first_layer,
        "Classic sibling-hole order must match the runtime beta oracle: {:?}",
        reference.root,
    );
}
