//! Classic top-layer one-wall reduction requires an actual top shell.

#[tokio::test]
async fn classic_top_layer_keeps_inner_wall_without_top_shell() {
    let project = include_bytes!(
        "../../../../tests/parity/top_layer_one_wall_requires_top_shell/classic_without_top_shell.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "top_layer_one_wall_requires_top_shell",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:619-622 has_top_shell_layers",
                "PerimeterGenerator.cpp:1540-1559 process_classic top-wall gate",
            ],
        ),
        "classic_without_top_shell",
        project,
        "b0b375adc810bc7a5049239645b6e56dd51c739f58cebe4f0971080ab979d515",
        false,
    );
    let native_last_layer = last_layer_object(&reference.native);
    assert!(native_last_layer.contains(";TYPE:Inner wall\n"));
    assert!(native_last_layer.contains(";TYPE:Outer wall\n"));
    std::fs::write(
        reference.root.join("native.last_layer_object.gcode"),
        native_last_layer,
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
    let api_last_layer = last_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(
        reference.root.join("ares.last_layer_object.gcode"),
        api_last_layer,
    )
    .unwrap();
    assert_eq!(
        api_last_layer, native_last_layer,
        "top-layer object output without a top shell must match the runtime beta oracle: {:?}",
        reference.root,
    );
}

fn last_layer_object(gcode: &str) -> &str {
    gcode
        .rsplit(";LAYER_CHANGE\n")
        .next()
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
