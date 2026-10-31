//! Complete object-layer blocks; the original full-stream case stays strict.

#[tokio::test]
async fn coincident_width_overhang_first_affected_object_layer_matches_runtime_orca() {
    compare_object_layer(Selection::FirstOverhang).await;
}

#[tokio::test]
async fn last_supported_object_layer_preserves_arachne_collinear_corner() {
    compare_object_layer(Selection::LastSupported).await;
}

#[derive(Clone, Copy)]
enum Selection {
    FirstOverhang,
    LastSupported,
}

async fn compare_object_layer(selection: Selection) {
    let project = include_bytes!(
        "../../../../../tests/parity/clipper-wrapper-geometry/coincident-width-overhang-intersections.3mf"
    );
    let label = match selection {
        Selection::FirstOverhang => "width-overhang-residual",
        Selection::LastSupported => "arachne-collinear-simplification",
    };
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            label,
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "Arachne/utils/ExtrusionLine.hpp:40 colinear_vertex_tolerance",
                "Arachne/utils/ExtrusionLine.cpp:143-149 simplify collinear predicates",
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
    let first_overhang = native_blocks
        .iter()
        .position(|block| block.contains(";TYPE:Overhang wall\n"))
        .expect("the immutable input activates Native overhang emission");
    let ordinal = match selection {
        Selection::FirstOverhang => first_overhang,
        Selection::LastSupported => {
            let last = native_blocks.len() - 1;
            assert!(last > first_overhang);
            assert!(!native_blocks[last].contains(";TYPE:Overhang wall\n"));
            last
        }
    };
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
            "selection": label,
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
        "The entire same-ordinal object layer, including initial travel and all records, must match runtime Native: {:?}",
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
