//! Classic first-layer wall reduction requires an actual bottom shell.

#[tokio::test]
async fn classic_first_layer_keeps_inner_wall_without_bottom_shell() {
    let project = include_bytes!(
        "../../../../tests/parity/first_layer_one_wall_requires_bottom_shell/classic_without_bottom_shell.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "first_layer_one_wall_requires_bottom_shell",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:628-631 has_bottom_shell_layers",
                "PerimeterGenerator.cpp:1540-1556 process_classic first-layer wall gate",
            ],
        ),
        "classic_without_bottom_shell",
        project,
        "0e5d3396a4cdebafd9ea2671353ed87e7576c46f924591f14af70510baaa38f4",
        false,
    );
    let native_walls = first_layer_walls(&reference.native);
    assert!(native_walls.starts_with("Inner wall\n"));
    assert!(native_walls.contains(";TYPE:Outer wall\n"));
    std::fs::write(
        reference.root.join("native.first_layer_walls.gcode"),
        native_walls,
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
    let text = std::str::from_utf8(&api).unwrap();
    let api_walls = first_layer_walls(text);
    std::fs::write(
        reference.root.join("ares.first_layer_walls.gcode"),
        api_walls,
    )
    .unwrap();
    assert_eq!(
        api_walls, native_walls,
        "first-layer inner and outer walls must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

#[tokio::test]
async fn classic_first_layer_keeps_one_wall_with_bottom_shell() {
    let project = include_bytes!(
        "../../../../tests/parity/first_layer_one_wall_requires_bottom_shell/classic_with_bottom_shell.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "first_layer_one_wall_requires_bottom_shell",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:628-631 has_bottom_shell_layers",
                "PerimeterGenerator.cpp:1540-1556 process_classic first-layer wall gate",
            ],
        ),
        "classic_with_bottom_shell",
        project,
        "43d81e9ad79795c4ccb1eef8b8307fa80999bcbc704df5f468dd027515b6572d",
        false,
    );
    let native_walls = first_layer_outer_wall_and_wipe(&reference.native);
    assert!(native_walls.starts_with("Outer wall\n"));
    assert!(!native_walls.contains(";TYPE:Inner wall\n"));
    std::fs::write(
        reference.root.join("native.first_layer_walls.gcode"),
        native_walls,
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
    let api_walls = first_layer_outer_wall_and_wipe(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.first_layer_walls.gcode"),
        api_walls,
    )
    .unwrap();
    assert_eq!(
        api_walls, native_walls,
        "first-layer one-wall reduction with a bottom shell must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

fn first_layer_outer_wall_and_wipe(gcode: &str) -> &str {
    let walls = first_layer_walls(gcode);
    let end = walls.find(";WIPE_END\n").unwrap() + ";WIPE_END\n".len();
    &walls[..end]
}

fn first_layer_walls(gcode: &str) -> &str {
    gcode
        .split(";LAYER_CHANGE\n")
        .nth(1)
        .unwrap()
        .split_once(";TYPE:")
        .unwrap()
        .1
        .split_once("; stop printing object ")
        .unwrap()
        .0
}
