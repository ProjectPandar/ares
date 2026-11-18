use super::*;
use crate::runner::CaseInputs;

#[test]
fn initialized_short_line_matches_complete_internal_fill_output() {
    if !crate::runner::seeded_reference_enabled() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("wall_loops".into(), Value::String("0".into()));
    process.insert("top_shell_layers".into(), Value::String("0".into()));
    process.insert("enable_support".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    process.insert(
        "detect_narrow_internal_solid_infill".into(),
        Value::String("1".into()),
    );
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "initialized-short-line/internal-fill/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/arachne/short-branch.stl"),
        )
        .unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["enable_support"], "0");
    assert_eq!(settings["disable_m73"], "1");
    assert_eq!(settings["detect_narrow_internal_solid_infill"], "1");
    assert_eq!(settings["wall_loops"], "0");
    assert_eq!(settings["top_shell_layers"], "0");
    let sliced = runner.slice_case(&exported).unwrap();
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Internal solid infill")
    );
    assert!(
        !std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains("\nM73 ")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
