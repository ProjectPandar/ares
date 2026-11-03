//! Beta FillBase.cpp:1809–1897: choose the contour order maximizing complete joins.
use crate::runner::native_stream_case::qualify;

#[test]
fn first_crosshatch_sparse_contour_order_matches_native() {
    let bytes =
        include_bytes!("../../../../../tests/parity/crosshatch-contour-arc-order/crosshatch.3mf");
    let (reference, _) = qualify(
        (
            "crosshatch-contour-arc-order",
            &["Fill/FillBase.cpp:1809-1897"],
        ),
        "crosshatch",
        bytes,
        "8305d6fcd9f2f9f04199cc17070a4b167b419ab9b1e8d5ffef213b23d44d11d6",
        false,
    );
    assert!(
        reference
            .native
            .contains("; sparse_infill_pattern = crosshatch")
    );
    assert!(reference.native.contains("; fill_multiline = 1"));
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let expected = first_sparse_layer(&reference.native);
    assert!(!expected.is_empty());
    assert_eq!(
        first_sparse_layer(std::str::from_utf8(&api).unwrap()),
        expected
    );
}

fn first_sparse_layer(gcode: &str) -> Vec<&str> {
    let mut layer = 0;
    let mut sparse = false;
    gcode
        .lines()
        .filter(|line| {
            if *line == ";LAYER_CHANGE" {
                layer += 1;
                sparse = false;
            }
            if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            }
            if *line == ";WIPE_START" {
                sparse = false;
            }
            layer == 6
                && sparse
                && line.starts_with("G1 ")
                && line.contains(" E")
                && (line.contains(" X") || line.contains(" Y"))
        })
        .collect()
}
