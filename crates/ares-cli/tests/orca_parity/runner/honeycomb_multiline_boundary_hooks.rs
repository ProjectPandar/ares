//! Beta FillBase.cpp:2014-2046: remaining-endpoint hooks toward multiline copies.
use crate::runner::native_stream_case::qualify;

#[test]
fn first_sparse_layer_boundary_hooks_match_native() {
    let bytes = include_bytes!(
        "../../../../../tests/parity/honeycomb-multiline-boundary-hooks/honeycomb.3mf"
    );
    let (reference, _) = qualify(
        (
            "honeycomb-multiline-boundary-hooks",
            &["Fill/FillBase.cpp:2014-2046"],
        ),
        "honeycomb",
        bytes,
        "e1d22463fc2b151ed1043f837c2833432c6ce3724a12361bbb011fef1b8ca0c3",
        false,
    );
    assert!(
        reference
            .native
            .contains("; sparse_infill_pattern = honeycomb")
    );
    assert!(reference.native.contains("; fill_multiline = 2"));
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
    let expected = sparse_layer_six(&reference.native);
    assert!(!expected.is_empty());
    assert_eq!(
        sparse_layer_six(std::str::from_utf8(&api).unwrap()),
        expected
    );
}

fn sparse_layer_six(gcode: &str) -> Vec<&str> {
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
