use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_locked_skin_width_drives_sparse_extrusion() {
    compare_width("skin_infill_line_width");
}

#[test]
fn orca_parity_locked_skeleton_width_drives_sparse_extrusion() {
    compare_width("skeleton_infill_line_width");
}

fn compare_width(key: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut prior_extrusions = None;
    for (wire, width) in [("0.3", 0.3), ("150%", 0.6)] {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        let mut process = profiles.process(&selection.process).unwrap();
        parity::normalize_process_defaults(&machine, &mut process);
        let mut filaments = selection
            .filaments
            .iter()
            .map(|name| profiles.filament(name).unwrap())
            .collect::<Vec<_>>();
        parity::normalize_filament_defaults(&mut filaments);
        let case = domains::OptionCase {
            label: wire.into(),
            value: Some(json!(wire)),
        };
        inject_case(&plan, &case, &mut machine, &mut process, &mut filaments);
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("locked-width/{key}/{wire}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        for (option, value) in [
            (key, wire),
            ("sparse_infill_pattern", "lockedzag"),
            ("sparse_infill_density", "40%"),
            ("use_relative_e_distances", "1"),
        ] {
            runner::application::verify_value(&exported.project, option, &json!(value)).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let mut sparse = false;
        let mut current_width = 0.0_f64;
        let mut extrusions = Vec::new();
        for line in std::str::from_utf8(&sliced.reference).unwrap().lines() {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            } else if let Some(value) = line.strip_prefix(";WIDTH:") {
                current_width = value.parse().unwrap();
            } else if sparse
                && (current_width - width).abs() < 1e-6
                && (line.starts_with("G1 X") || line.starts_with("G1 Y"))
                && line.split_whitespace().any(|word| {
                    word.strip_prefix('E')
                        .and_then(|value| value.parse::<f64>().ok())
                        .is_some_and(|value| value > 0.0)
                })
            {
                extrusions.push(line.to_owned());
            }
        }
        assert!(
            !extrusions.is_empty(),
            "{key}={wire} must drive sparse extrusion"
        );
        if let Some(prior) = prior_extrusions {
            assert_ne!(
                prior, extrusions,
                "width probes must change actual extrusion"
            );
        }
        prior_extrusions = Some(extrusions);
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
    }
}
