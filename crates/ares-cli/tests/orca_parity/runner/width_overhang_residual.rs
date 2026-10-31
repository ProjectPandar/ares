//! Full first overhang object-layer block, without weakening the full-stream case.

#[tokio::test]
async fn coincident_width_overhang_first_affected_object_layer_matches_runtime_orca() {
    let project = include_bytes!(
        "../../../../../tests/parity/clipper-wrapper-geometry/coincident-width-overhang-intersections.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "width-overhang-residual",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "PerimeterGenerator.cpp:320-389 clip_extrusion and traverse_extrusions",
                "ClipperZUtils.cpp:48-80 clip_zpaths",
                "GCode/ExtrusionProcessor.hpp:50-355 estimate_points_properties",
            ],
        ),
        "coincident-width-overhang-intersections",
        project,
        "7688c5eaac8bb610fae4e92863474c85262d9f83f64737ba17f5574c6b635866",
        false,
    );
    let native_blocks = object_blocks(&reference.native);
    let ordinal = native_blocks
        .iter()
        .position(|block| block.contains(";TYPE:Overhang wall\n"))
        .expect("the immutable input activates Native overhang emission");
    let native = native_blocks[ordinal];
    std::fs::write(
        reference.root.join("native.affected_object_layer.gcode"),
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
    let api_blocks = object_blocks(std::str::from_utf8(&api).unwrap());
    let actual = api_blocks[ordinal];
    std::fs::write(
        reference.root.join("ares.affected_object_layer.gcode"),
        actual,
    )
    .unwrap();
    std::fs::write(
        reference.root.join("affected_object_layer.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "native_selected_ordinal": ordinal,
            "native_object_blocks": native_blocks.len(),
            "api_object_blocks": api_blocks.len(),
            "api_selection": "same ordinal, no API feature search",
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        actual, native,
        "The entire same-ordinal first overhang object layer, including initial travel and all records, must match runtime Native: {:?}",
        reference.root,
    );
}

fn object_blocks(gcode: &str) -> Vec<&str> {
    gcode
        .split("; printing object ")
        .skip(1)
        .map(|tail| {
            tail.split_once('\n')
                .unwrap()
                .1
                .split_once("; stop printing object ")
                .unwrap()
                .0
        })
        .collect()
}
