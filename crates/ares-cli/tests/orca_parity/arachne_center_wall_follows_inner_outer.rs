//! Arachne center walls honor the selected inner-to-outer order.

#[tokio::test]
async fn arachne_inner_outer_prints_center_line_before_outer_loop() {
    let project = include_bytes!(
        "../../../../tests/parity/arachne_center_wall_follows_inner_outer/inner_before_outer.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_center_wall_follows_inner_outer",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "Arachne/WallToolPaths.cpp:810-818,885-899 getRegionOrder",
                "PerimeterGenerator.cpp:2918-3000 process_arachne ordering consumer",
            ],
        ),
        "inner_before_outer",
        project,
        "03b49a4e5c048cd27ae1079216d3dcbdfe2f8740f790a4f073a369892269d78c",
        false,
    );
    let native_first_layer = first_layer_object(&reference.native);
    let native_wall_roles = native_first_layer.split_once(";TYPE:").unwrap().1;
    assert!(native_wall_roles.starts_with("Inner wall\n"));
    assert!(native_wall_roles.contains(";TYPE:Outer wall\n"));
    let native_inner_wall = native_wall_roles
        .split_once(";TYPE:Outer wall\n")
        .unwrap()
        .0;
    assert_eq!(native_inner_wall.matches("\nG1 X110 Y").count(), 1);
    std::fs::write(
        reference.root.join("native.first_layer_object.gcode"),
        native_first_layer,
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
    let api_first_layer = first_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.first_layer_object.gcode"),
        api_first_layer,
    )
    .unwrap();
    assert_eq!(
        api_first_layer, native_first_layer,
        "center-wall inner-to-outer ordering must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

fn first_layer_object(gcode: &str) -> &str {
    gcode
        .split(";LAYER_CHANGE\n")
        .nth(1)
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
