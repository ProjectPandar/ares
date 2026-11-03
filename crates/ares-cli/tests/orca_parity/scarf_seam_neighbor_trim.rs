//! Beta `ExtrusionEntity.cpp:444-484`: retain the seam and drop its tiny neighbor.

#[tokio::test]
async fn scarf_keeps_fitted_seam_without_its_micrometre_neighbor() {
    let project = include_bytes!(
        "../../../../tests/parity/scarf_seam_neighbor_trim/rear_low_motion_peak_scarf.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "scarf_seam_neighbor_trim",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "ExtrusionEntity.cpp:444-484 trim_seam_ends",
                "ExtrusionEntity.cpp:516-521 sloped-loop trim and length clamp",
            ],
        ),
        "rear_low_motion_peak_scarf",
        project,
        "e28178fae28e3abbe4ab6403997e74e29570cb0b63f5c1f4c3c92ff9fbb4308a",
        false,
    );
    let native = below_gap_object(&reference.native);
    std::fs::write(reference.root.join("native.below_gap_object.gcode"), native).unwrap();
    assert!(
        native.contains("X107.05 Y115.43"),
        "the fitted seam must remain"
    );
    assert!(
        !native.contains("X107.029 Y115.431"),
        "Native must remove the micrometre neighbor next to the retained seam"
    );

    let api = crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 31, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api = below_gap_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.below_gap_object.gcode"), api).unwrap();
    assert_eq!(
        api, native,
        "the complete scarf object must match the runtime beta seam-neighbor trim oracle: {:?}",
        reference.root,
    );
}

fn below_gap_object(gcode: &str) -> &str {
    gcode
        .split_once(";LAYER_CHANGE\n;Z:4.4\n")
        .unwrap()
        .1
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
