//! Arachne top-layer one-wall reduction requires an actual top shell.

#[tokio::test]
async fn arachne_top_layer_keeps_inner_wall_without_top_shell() {
    let project = include_bytes!(
        "../../../../tests/parity/arachne_top_layer_requires_top_shell/without_top_shell.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_top_layer_requires_top_shell",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:619-622 has_top_shell_layers",
                "PerimeterGenerator.cpp:2681,2713-2716,2735-2739 process_arachne top-wall gate",
            ],
        ),
        "without_top_shell",
        project,
        "0cec246033f07815c005989c44a6fd27a8253aa9fe0c94952c06e43da4c27560",
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
        "Arachne top-layer object output without a top shell must match the runtime beta oracle: {:?}",
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
