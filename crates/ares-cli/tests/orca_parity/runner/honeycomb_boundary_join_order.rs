//! Beta FillBase.cpp:1809-1897: contour arc orders maximizing complete joins.
use crate::runner::native_stream_case::qualify;

#[test]
fn second_layer_sparse_boundary_joins_match_native() {
    let bytes =
        include_bytes!("../../../../../tests/parity/honeycomb-boundary-join-order/honeycomb.3mf");
    let (reference, _) = qualify(
        (
            "honeycomb-boundary-join-order",
            &["Fill/FillBase.cpp:1809-1897"],
        ),
        "honeycomb",
        bytes,
        "d88cb59ce2a810545e8de2a1e5ba890227305485f4bdd5501a54bda049636f58",
        false,
    );
    assert!(
        reference
            .native
            .contains("; sparse_infill_pattern = honeycomb")
    );
    assert!(reference.native.contains("; fill_multiline = 1"));
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let expected = sparse_layer_two(&reference.native);
    assert!(!expected.is_empty());
    assert_eq!(
        sparse_layer_two(std::str::from_utf8(&api).unwrap()),
        expected
    );
}

fn sparse_layer_two(gcode: &str) -> Vec<&str> {
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
            layer == 2
                && sparse
                && line.starts_with("G1 ")
                && line.contains(" E")
                && (line.contains(" X") || line.contains(" Y"))
        })
        .collect()
}
