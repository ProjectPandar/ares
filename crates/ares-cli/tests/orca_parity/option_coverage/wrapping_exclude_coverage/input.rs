use super::super::*;
use crate::runner::CaseInputs;
use serde_json::json;

pub(in super::super) fn export(
    runner: &OrcaRunner,
    points: &Value,
    enabled: bool,
    label: &str,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("nozzle_diameter".into(), json!(["0.4"]));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    machine.insert("wrapping_exclude_area".into(), points.clone());
    machine.insert(
        "wrapping_detection_gcode".into(),
        json!(";W6_WRAP {layer_num} {layer_z} {max_layer_z}"),
    );
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (field, value) in [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
        ("print_flow_ratio", "1"),
        ("set_other_flow_ratios", "0"),
        ("bridge_flow", "1"),
        ("internal_bridge_flow", "1"),
        ("thick_bridges", "1"),
        ("thick_internal_bridges", "1"),
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_density", "15%"),
        ("top_shell_layers", "3"),
        ("bottom_shell_layers", "3"),
        ("top_shell_thickness", "0"),
        ("bottom_shell_thickness", "0"),
    ] {
        process.insert(field.into(), json!(value));
    }
    process.insert(
        "enable_wrapping_detection".into(),
        json!(if enabled { "1" } else { "0" }),
    );
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    assert_eq!(filaments.len(), 1);
    filaments[0].insert("filament_flow_ratio".into(), json!(["1"]));
    filaments[0].insert("filament_diameter".into(), json!(["1.75"]));

    let exported = runner.export_case(
        &CaseInputs {
            label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &parity::smoke_overrides(),
        &runner::repo_root().join("tests/parity/cube10.stl"),
    )?;
    for field in [
        "nozzle_diameter",
        "use_relative_e_distances",
        "wrapping_exclude_area",
        "wrapping_detection_gcode",
    ] {
        runner::application::verify_value(&exported.project, field, &machine[field])?;
    }
    for field in [
        "print_sequence",
        "enable_wrapping_detection",
        "layer_height",
        "initial_layer_print_height",
        "enable_support",
        "raft_layers",
        "enable_prime_tower",
        "print_flow_ratio",
        "set_other_flow_ratios",
        "bridge_flow",
        "internal_bridge_flow",
        "thick_bridges",
        "thick_internal_bridges",
        "sparse_infill_pattern",
        "sparse_infill_density",
        "top_shell_layers",
        "bottom_shell_layers",
        "top_shell_thickness",
        "bottom_shell_thickness",
    ] {
        runner::application::verify_value(&exported.project, field, &process[field])?;
    }
    for field in ["filament_flow_ratio", "filament_diameter"] {
        runner::application::verify_value(&exported.project, field, &filaments[0][field])?;
    }
    assert_eq!(
        runner::application::settings(&exported.project).unwrap()["wrapping_exclude_area"],
        *points
    );
    Ok(exported)
}

pub(in super::super) fn metadata() -> Value {
    json!({"scope":"finite empty/one/two/off-object triangle/rectangle; single nozzle/filament no tower by_layer",
        "source":"PrintConfig.cpp:4120-4125; Config.hpp:1487-1550; Print.cpp:932-979,1304-1310; GCode.cpp:5215-5236,5302-5305",
        "fixture":"cube10.stl", "enable_wrapping_detection":"1",
        "wrapping_detection_gcode":";W6_WRAP {layer_num} {layer_z} {max_layer_z}",
        "layer_height":"0.2","initial_layer_print_height":"0.2",
        "nozzle_diameter":["0.4"],"filament_flow_ratio":["1"],"filament_diameter":["1.75"],
        "support":"disabled","raft_layers":"0","enable_prime_tower":"0","print_sequence":"by layer",
        "deferred":["towers","SEMM","multi-nozzle/filament","by_object","arbitrary polygon or transforms","touching/epsilon","detection motion","tool placeholders","APA"]})
}
