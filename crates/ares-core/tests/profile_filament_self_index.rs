#[path = "../../../tests/support/filament_self_index.rs"]
mod fixture;

#[tokio::test]
async fn profile_slice_switch_to_four_tools_reuses_first_self_index_for_missing_variant_ids() {
    let (input, catalog, selection) = fixture::inputs();
    // The printer switch's normalizeFilamentSlots supplies four per-nozzle matrices.
    let overrides = ares_core::ConfigOverrides::from_json(
        serde_json::json!({
            "flush_multiplier": "1,1,1,1",
            "flush_volumes_vector": "140,140,140,140,140,140,140,140",
            "flush_volumes_matrix": vec!["0"; 64].join(","),
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    let gcode = ares_core::slice_with_profiles(
        &input,
        ares_core::ProfileSliceRequest {
            filename: "four-tool-transition.3mf",
            catalog_json: &catalog,
            selection_json: &selection,
            plate: Some(1),
            overrides: &overrides,
            placement: ares_core::Placement::None,
        },
        ares_core::GenerationMetadata::deterministic(2026, 11, 15, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let directory =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("profile-filament-self-index");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("input.3mf"), input).unwrap();
    std::fs::write(directory.join("catalog.json"), catalog).unwrap();
    std::fs::write(directory.join("selection.json"), selection).unwrap();
    std::fs::write(directory.join("ares.gcode"), &gcode).unwrap();
    eprintln!("artifact: {}", directory.display());
    let gcode = String::from_utf8(gcode).unwrap();
    assert!(gcode.contains("G1 X"), "cube must produce motion");
    assert!(
        gcode
            .lines()
            .any(|line| line == "; filament_self_index = 1,3,1,1"),
        "indices 0 and 2 retain their IDs; indices 4 and 6 reuse the first ID (Config.hpp:676-680)"
    );
}
