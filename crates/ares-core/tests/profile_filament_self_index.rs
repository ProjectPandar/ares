#[path = "../../../tests/support/filament_self_index.rs"]
mod fixture;

#[tokio::test]
async fn direct_bbl_project_without_catalog_returns_missing_bundle_error() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let result = ares_core::slice_project_plate(
        input,
        ares_core::GenerationMetadata::deterministic(2026, 11, 15, 0, 0, 0),
        Some(1),
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await;
    assert!(
        result.is_err(),
        "direct BBL projects require a system preset bundle"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("system preset bundle")
    );
}

#[tokio::test]
async fn direct_bbl_project_with_catalog_slices_successfully() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let catalog = ares_core::PresetCatalogHandle::new();
    ares_core::slice_project_plate_with_catalog(
        input,
        ares_core::GenerationMetadata::deterministic(2026, 11, 15, 0, 0, 0),
        (Some(1), &ares_core::ConfigOverrides::default()),
        Some(&catalog),
        ares_core::SlicerContext::new(0),
    )
    .await
    .expect("a supplied system bundle permits direct BBL slicing");
}

#[tokio::test]
async fn direct_bbl_missing_filament_preset_keeps_slot_without_print_warning() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let catalog = ares_core::PresetCatalogHandle::new();
    let mut upward_report = Vec::new();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let output = ares_core::slice_cli_project_plate_validated_with_catalog(
        input,
        ares_core::CliProjectSliceRequest {
            plate: Some(1),
            overrides: &ares_core::ConfigOverrides::default(),
            record_moves: false,
            orient: false,
            upward: ares_core::CliUpwardMachine {
                source: None,
                input: ares_core::UpwardMachineInput::Embedded,
                report: &mut upward_report,
            },
            sliced_plate: &mut sliced_plate,
        },
        ares_core::GenerationMetadata::deterministic(2026, 11, 15, 0, 0, 0),
        Some(&catalog),
        ares_core::SlicerContext::new(0),
    )
    .await
    .expect("missing system filament presets preserve the direct project slot");
    assert!(sliced_plate.warnings.is_empty());
    let gcode = std::str::from_utf8(&output.gcode).unwrap();
    for setting in [
        "filament_density = 1.24",
        "filament_flow_ratio = 0.98",
        "filament_diameter = 1.75",
    ] {
        assert!(gcode.contains(&format!("\n; {setting}\n")));
    }
    let direct = ares_core::slice_project_plate_with_catalog(
        input,
        ares_core::GenerationMetadata::deterministic(2026, 11, 15, 0, 0, 0),
        (Some(1), &ares_core::ConfigOverrides::default()),
        Some(&catalog),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert_eq!(output.gcode, direct, "complete direct/CLI-aware output");
}

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
            .any(|line| line == "; filament_self_index = 1,2,3,4"),
        "CLI variant refresh layout appends each filament's complete variant segment and rebuilds self_index from the selected variant counts (OrcaSlicer.cpp:3510-3545, 3639-3701, 3753-3759, 3823-3845)"
    );
}
