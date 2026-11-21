use super::*;
use crate::runner::CaseInputs;

#[path = "initialized_bridge_direction/rehash.rs"]
mod rehash;

#[path = "initialized_bridge_direction/tilted.rs"]
mod tilted;

#[test]
fn initialized_equal_cost_bridge_matches_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_support".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("wall_loops".into(), Value::String("0".into()));
    process.insert(
        "detect_narrow_internal_solid_infill".into(),
        Value::String("0".into()),
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
                label: "initialized-bridge/equal-cost/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    for (key, value) in [
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
        ("wall_loops", "0"),
        ("detect_narrow_internal_solid_infill", "0"),
    ] {
        runner::application::verify_value(&exported.project, key, &Value::String(value.into()))
            .unwrap();
    }
    let sliced = runner.slice_case(&exported).unwrap();
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Bridge\n")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
