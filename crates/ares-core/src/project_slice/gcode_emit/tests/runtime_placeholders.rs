#[tokio::test]
async fn project_templates_share_orca_runtime_placeholders() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        ";======== X2D start gcode==========",
        ";RUNTIME [initial_extruder] [initial_tool] [year] [timestamp] [total_toolchanges] [first_layer_print_max] [bed_temperature] [min_vitrification_temperature] [adaptive_bed_mesh_min] [adaptive_bed_mesh_max] [bed_mesh_probe_count] [bed_mesh_algo] [outer_wall_volumetric_speed] [has_tpu_in_first_layer] [position] [printer_preset] [print_preset] [filament_preset] [is_extruder_used[0]] [is_extruder_used[5]]",  
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    let runtime = output
        .lines()
        .find(|line| line.starts_with(";RUNTIME "))
        .unwrap();

    assert!(runtime.starts_with(";RUNTIME 0 0 2026 20260716-010203 0 "));
    assert!(!runtime.contains(['[', ']']));
}
