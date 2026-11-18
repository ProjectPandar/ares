use super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn fully_clipped_loops_preserve_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("seam_gap", "50"),
        ("wipe_before_external_loop", "1"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "clipped-loop/seam-gap50/wipe-before-external",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    application::verify_value(&exported.project, "use_relative_e_distances", &json!("1")).unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&sliced);
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(
        native
            .lines()
            .any(|line| line.split_whitespace().next() == Some("M83"))
    );
    assert!(
        !native
            .lines()
            .any(|line| matches!(line, ";TYPE:Inner wall" | ";TYPE:Outer wall"))
    );
    assert!(native.contains(";TYPE:Internal solid infill"));
    let infill = native
        .split(";TYPE:Internal solid infill\n")
        .nth(1)
        .unwrap()
        .split(";TYPE:")
        .next()
        .unwrap();
    assert!(infill.lines().any(|line| {
        line.starts_with("G1 ")
            && line
                .split_whitespace()
                .any(|word| word.starts_with('X') || word.starts_with('Y'))
            && line.split_whitespace().any(|word| {
                word.strip_prefix('E')
                    .and_then(|value| value.parse::<f64>().ok())
                    .is_some_and(|value| value > 0.0)
            })
    }));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
