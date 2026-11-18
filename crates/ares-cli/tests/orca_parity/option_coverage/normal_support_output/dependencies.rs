use super::*;

#[test]
fn support_flow_and_gap_dependencies_preserve_complete_raft_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    let contexts = [
        ("zero-top-gap", vec![("support_top_z_distance", "0")]),
        (
            "automatic-width",
            vec![
                ("support_line_width", "0"),
                ("line_width", "0"),
                ("initial_layer_line_width", "0"),
            ],
        ),
        (
            "first-layer-support-width",
            vec![
                ("support_line_width", "0.6"),
                ("initial_layer_line_width", "0"),
            ],
        ),
        (
            "synchronized-gap-rounding",
            vec![
                ("independent_support_layer_height", "0"),
                ("raft_contact_distance", "0.099999"),
            ],
        ),
        ("contact-auto", vec![("support_interface_pattern", "auto")]),
        (
            "contact-rectilinear",
            vec![("support_interface_pattern", "rectilinear")],
        ),
        (
            "contact-concentric",
            vec![("support_interface_pattern", "concentric")],
        ),
        (
            "contact-rectilinear-interlaced",
            vec![("support_interface_pattern", "rectilinear_interlaced")],
        ),
        ("contact-grid", vec![("support_interface_pattern", "grid")]),
        (
            "automatic-width-without-skirt",
            vec![
                ("support_line_width", "0"),
                ("line_width", "0"),
                ("initial_layer_line_width", "0"),
                ("skirt_loops", "0"),
            ],
        ),
    ];
    let mut failures = Vec::new();
    let mut automatic_width = None;
    for (label, overrides) in contexts {
        let mut candidate = process.clone();
        let settings = [
            ("enable_support", "0"),
            ("raft_layers", "4"),
            ("support_top_z_distance", "0.15"),
            ("raft_contact_distance", "0.15"),
            ("independent_support_layer_height", "1"),
        ]
        .into_iter()
        .chain(overrides)
        .collect::<Vec<_>>();
        for &(key, value) in &settings {
            candidate.insert(key.into(), json!(value));
        }
        let label = format!("normal-support/dependency/{label}");
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &candidate,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        for key in settings.iter().map(|&(key, _)| key) {
            runner::application::verify_value(&exported.project, key, &candidate[key]).unwrap();
        }
        let sliced = match runner.slice_case(&exported) {
            Ok(sliced) => sliced,
            Err(error) => {
                failures.push(format!("{label}: {error:?}"));
                continue;
            }
        };
        let outcome = parity::compare_case(&sliced);
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        let active = has_support_extrusion(reference);
        if outcome.status != "PASS" || !active {
            failures.push(format!("{label}: {} (support={active})", outcome.detail));
        }
        if label == "normal-support/dependency/automatic-width" {
            automatic_width = Some(sliced);
        }
    }
    if let Some(mut repeated) = automatic_width {
        repeated.label = "normal-support/dependency/automatic-width-after-other-jobs".into();
        let outcome = parity::compare_case(&repeated);
        if outcome.status != "PASS" {
            failures.push(format!("{}: {}", repeated.label, outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
