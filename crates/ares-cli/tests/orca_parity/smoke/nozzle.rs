use super::*;
use serde_json::json;

#[test]
fn nozzle_ceiling_seams() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    machine.insert("nozzle_diameter".into(), json!(["100"]));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let overrides = parity::smoke_overrides();
    for seam in ["aligned", "nearest"] {
        process.insert("seam_position".into(), json!(seam));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("nozzle-ceiling/{seam}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &overrides,
                &cube_model(),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, "nozzle_diameter", &json!(["100"]))
            .unwrap();
        runner::application::verify_value(&exported.project, "seam_position", &json!(seam))
            .unwrap();
        let outcome = parity::compare_exported(&runner, &exported);
        assert_eq!(outcome.status, "PASS", "{seam}: {}", outcome.detail);
    }
}
