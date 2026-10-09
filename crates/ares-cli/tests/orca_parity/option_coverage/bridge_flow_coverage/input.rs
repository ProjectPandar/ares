use super::super::*;
use crate::runner::CaseInputs;
use serde_json::json;

pub(in super::super) fn export(
    runner: &OrcaRunner,
    key: &str,
    wire: &str,
    thick: bool,
    label: &str,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("nozzle_diameter".into(), json!(["0.4"]));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (field, value) in [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("enable_prime_tower", "0"),
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
    let thick_key = if key == "bridge_flow" {
        "thick_bridges"
    } else {
        "thick_internal_bridges"
    };
    process.insert(thick_key.into(), json!(if thick { "1" } else { "0" }));
    process.insert(key.into(), json!(wire));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    assert_eq!(filaments.len(), 1);
    filaments[0].insert("filament_flow_ratio".into(), json!(["1"]));
    filaments[0].insert("filament_diameter".into(), json!(["1.75"]));
    let fixture = if key == "bridge_flow" {
        "overhang.stl"
    } else {
        "cube10.stl"
    };
    let exported = runner.export_case(
        &CaseInputs {
            label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &parity::smoke_overrides(),
        &runner::repo_root().join("tests/parity").join(fixture),
    )?;
    for field in ["nozzle_diameter", "use_relative_e_distances"] {
        runner::application::verify_value(&exported.project, field, &machine[field])?;
    }
    for field in [
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
    Ok(exported)
}

pub(in super::super) fn metadata(key: &str, thick: bool) -> Value {
    json!({
        "scope":"finite 0.5/1/1.5/2 requests; single 0.4mm nozzle and filament; not all positive ratios",
        "source":"LayerRegion.cpp:31-58; Fill/Fill.cpp:904-958; GCode.cpp:6467-6513",
        "fixture":if key == "bridge_flow" {"overhang.stl"} else {"cube10.stl"},
        "thick_bridges":if key == "bridge_flow" && !thick {"0"} else {"1"},
        "thick_internal_bridges":if key == "internal_bridge_flow" && !thick {"0"} else {"1"},
        "layer_height":"0.2","initial_layer_print_height":"0.2",
        "filament_flow_ratio":["1"],"filament_diameter":["1.75"],
        "use_relative_e_distances":"1","print_flow_ratio":"1","set_other_flow_ratios":"0",
        "support":"disabled","raft_layers":"0","prime_tower":"disabled",
        "sparse_infill_pattern":"rectilinear","sparse_infill_density":"15%",
        "top_shell_layers":"3","bottom_shell_layers":"3",
        "top_shell_thickness":"0","bottom_shell_thickness":"0",
        "other_bridge_ratio":"1",
        "deferred":["arbitrary bridge geometries", "mixed nozzles/filaments", "support flows",
            "object/region ranges", "APA", "all ratio combinations", "minimum positive float"]
    })
}
