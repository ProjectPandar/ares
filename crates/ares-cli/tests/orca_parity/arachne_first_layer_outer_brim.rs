//! A positive outer brim puts the first Arachne exterior wall before inner walls.

use super::first_layer_object;

#[tokio::test]
async fn positive_outer_brim_prints_outer_wall_first() {
    let project = include_bytes!(
        "../../../../tests/parity/arachne_first_layer_outer_brim/outer_only_first_layer.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "arachne_first_layer_outer_brim",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:2909-2911 process_arachne first-layer brim wall order",
                "PerimeterGenerator.cpp:3204-3211 outer_walls_first_for_brim",
            ],
        ),
        "outer_only_first_layer",
        project,
        "c844cc32b42f635ed24e494c01492c70727d28c7bcbade92b19a7951512f3f1c",
        false,
    );
    let native_first_layer = first_layer_object(&reference.native);
    let native_wall_roles = native_first_layer.split_once(";TYPE:").unwrap().1;
    assert!(native_wall_roles.starts_with("Outer wall\n"));
    assert!(native_wall_roles.contains(";TYPE:Inner wall\n"));
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
        "Arachne first-layer wall order with an outer brim must match the runtime beta oracle: {:?}",
        reference.root,
    );
}
