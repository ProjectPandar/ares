//! Arachne first-layer one-wall reduction requires a bottom shell.

#[tokio::test]
async fn arachne_first_layer_keeps_inner_wall_without_bottom_shell() {
    let project = include_bytes!(
        "../../../../tests/parity/arachne_first_layer_requires_bottom_shell/without_bottom_shell.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_first_layer_requires_bottom_shell",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:628-631 has_bottom_shell_layers",
                "PerimeterGenerator.cpp:2682,2708-2711 process_arachne first-layer wall gate",
            ],
        ),
        "without_bottom_shell",
        project,
        "9bcc8c766c122534de2229fe5426221502312f75d7f7a36e67c8096c38e6b801",
        false,
    );
    let native_first_layer = first_layer_object(&reference.native);
    let native_wall_roles = native_first_layer.split_once(";TYPE:").unwrap().1;
    assert!(native_wall_roles.starts_with("Inner wall\n"));
    assert!(native_wall_roles.contains(";TYPE:Outer wall\n"));
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
        "Arachne first-layer walls without a bottom shell must match the runtime beta oracle: {:?}",
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
