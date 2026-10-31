use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[path = "normal_support_output/angle.rs"]
mod angle;
#[path = "normal_support_output/cooling.rs"]
mod cooling;
#[path = "normal_support_output/dependencies.rs"]
mod dependencies;
#[path = "normal_support_output/interface_fan.rs"]
mod interface_fan;
#[path = "normal_support_output/style.rs"]
mod style;

#[test]
fn normal_support_boolean_values_preserve_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_NORMAL_SUPPORT_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let support_settings = [
        ("support_type", "normal(auto)"),
        ("support_style", "grid"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ];
    for (key, value) in support_settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let mut outcomes = Vec::new();
    for enabled in [false, true] {
        let value = if enabled { "1" } else { "0" };
        process.insert("enable_support".into(), json!(value));
        let label = format!("normal-support/enable_support/{enabled}");
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, "enable_support", &json!(value))
            .unwrap();
        for (key, value) in support_settings {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let active_witness = if enabled {
            has_support_extrusion(reference)
        } else {
            !reference
                .lines()
                .any(|line| line.starts_with(";TYPE:Support"))
        };
        outcomes.push((label, outcome, active_witness));
    }
    let failures = outcomes
        .into_iter()
        .filter_map(|(label, outcome, witness)| {
            (outcome.status != "PASS" || !witness)
                .then(|| format!("{label}: {} (active witness={witness})", outcome.detail))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

pub(crate) fn has_support_extrusion(reference: &str) -> bool {
    let mut support = false;
    let mut relative_e = false;
    reference.lines().any(|line| {
        let instruction = line.split(';').next().unwrap();
        let words = instruction.split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            _ => {}
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            support = matches!(role, "Support" | "Support interface");
        }
        relative_e
            && support
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    })
}
