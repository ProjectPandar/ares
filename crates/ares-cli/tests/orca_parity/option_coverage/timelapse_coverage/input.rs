use super::super::*;
use crate::runner::CaseInputs;
use serde_json::json;

pub(super) const TEMPLATE: &str = "{if timelapse_type == 0};W6_TL_TRAD layer={layer_num} z={layer_z} max={max_layer_z}\n{else};W6_TL_SMOOTH layer={layer_num} z={layer_z} max={max_layer_z}\n{endif}";

type Settings = (
    Map<String, Value>,
    Map<String, Value>,
    Vec<Map<String, Value>>,
);

fn settings() -> Settings {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    assert_eq!(filaments.len(), 1);
    (machine, process, filaments)
}

pub(in super::super) fn export(
    runner: &OrcaRunner,
    mode: &Value,
    active: bool,
    label: &str,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let (mut machine, mut process, filaments) = settings();
    machine.insert("nozzle_diameter".into(), json!(["0.4"]));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    machine.insert(
        "time_lapse_gcode".into(),
        json!(if active { TEMPLATE } else { "" }),
    );
    for (key, value) in [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
        ("spiral_mode", "0"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
    ] {
        process.insert(key.into(), json!(value));
    }
    process.insert("timelapse_type".into(), mode.clone());
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
    for key in [
        "nozzle_diameter",
        "use_relative_e_distances",
        "time_lapse_gcode",
        "printer_structure",
        "printer_model",
        "printable_area",
    ] {
        runner::application::verify_value(&exported.project, key, &machine[key])?;
    }
    for key in [
        "timelapse_type",
        "layer_height",
        "initial_layer_print_height",
        "enable_prime_tower",
        "print_sequence",
        "spiral_mode",
        "enable_support",
        "raft_layers",
    ] {
        runner::application::verify_value(&exported.project, key, &process[key])?;
    }
    Ok(exported)
}
