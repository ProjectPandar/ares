use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn lightning_preserves_complete_ordered_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_LIGHTNING_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("sparse_infill_pattern".into(), json!("lightning"));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "lightning/complete-output",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    runner::application::verify_value(
        &exported.project,
        "sparse_infill_pattern",
        &json!("lightning"),
    )
    .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&sliced);
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    let mut sparse = false;
    let mut relative_e = false;
    assert!(reference.lines().any(|line| {
        let instruction = line.split(';').next().unwrap();
        let words = instruction.split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            _ => {}
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        }
        relative_e
            && sparse
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    }));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
