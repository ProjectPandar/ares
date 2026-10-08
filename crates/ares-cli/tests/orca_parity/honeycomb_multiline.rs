use super::*;

#[test]
fn honeycomb_multiline_ordered_bytes() {
    if !crate::runner::reference_enabled("ARES_PARITY_HONEYCOMB_MULTILINE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("live oracle required");
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection = select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    normalize_filament_defaults(&mut filaments);
    let mut failed = Vec::new();
    for multiline in [2, 3, 4] {
        let label = format!("honeycomb/multiline/{multiline}");
        let mut overrides = smoke_overrides();
        overrides.insert("sparse_infill_pattern".into(), Value::from("honeycomb"));
        let requested = Value::from(multiline.to_string());
        process.insert("fill_multiline".into(), requested.clone());
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &overrides,
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let settings = runner::application::settings(&exported.project).unwrap();
        assert_eq!(settings["fill_multiline"], requested);
        assert_eq!(settings["sparse_infill_pattern"], "honeycomb");
        let outcome = compare_exported_for(&runner, &exported, Some("fill_multiline"));
        eprintln!("{label}: {} {}", outcome.status, outcome.detail);
        if outcome.status != "PASS" {
            failed.push(label);
        }
    }
    assert!(failed.is_empty(), "divergent honeycomb bundles: {failed:?}");
}
