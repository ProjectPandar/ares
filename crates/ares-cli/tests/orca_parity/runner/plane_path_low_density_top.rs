//! Beta FillPlanePath.cpp:93-113 aligns sparse roles, not low-density top surfaces.
use crate::runner::native_stream_case::qualify;

#[test]
fn low_density_hilbert_top_surface_extrusions_match_native() {
    let (reference, _) = qualify(
        (
            "plane-path-low-density-top",
            &["Fill/FillPlanePath.cpp:93-113"],
        ),
        "hilbert",
        include_bytes!("../../../../../tests/parity/plane-path-low-density-top/hilbert.3mf"),
        "95c3914db5975363a0dae71e078368baf1ac183558c6605b6d79f6e35ec3dd4b",
        false,
    );
    assert!(
        reference
            .native
            .contains("; top_surface_pattern = hilbertcurve")
    );
    assert!(reference.native.contains("; top_surface_density = 60%"));
    let bytes =
        include_bytes!("../../../../../tests/parity/plane-path-low-density-top/hilbert.3mf");
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
    let expected = top_extrusions(&reference.native);
    assert!(!expected.is_empty());
    assert_eq!(top_extrusions(std::str::from_utf8(&api).unwrap()), expected);
}

// Same role/G1/XY/E observation as compatible_regions/top_pattern.rs::observe.
// Stop at wipe: retraction moves are not surface extrusion paths.
fn top_extrusions(gcode: &str) -> Vec<&str> {
    let mut top = false;
    gcode
        .lines()
        .filter(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                top = role == "Top surface";
            }
            if *line == ";WIPE_START" {
                top = false;
            }
            top && line.starts_with("G1 ")
                && line.contains(" E")
                && (line.contains(" X") || line.contains(" Y"))
        })
        .collect()
}
