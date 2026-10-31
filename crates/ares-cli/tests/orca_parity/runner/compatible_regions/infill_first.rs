use super::super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root, volume_fixture};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn modifier_infill_first_preserves_region_wall_phases() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_PERIMETERS_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
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
    for (key, value) in [
        ("enable_support", "0"),
        ("sparse_infill_density", "15%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("is_infill_first", "0"),
    ] {
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
                label: "compatible-regions/modifier-infill-first",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    application::verify_value(&exported.project, "is_infill_first", &json!("0")).unwrap();
    let baseline = runner.slice_case(&exported).unwrap();
    let baseline_outcome = parity::compare_case(&baseline);
    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("is_infill_first", "1")],
    );
    let project_path = runner.work.join("modifier-infill-first.3mf");
    let output_dir = runner.work.join("modifier-infill-first-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let modified = runner
        .slice_case(&ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let role_sequence = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .filter(|line| line.starts_with(";TYPE:") || *line == ";LAYER_CHANGE")
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_ne!(
        role_sequence(&baseline.reference),
        role_sequence(&modified.reference)
    );
    let outcome = parity::compare_case(&modified);
    assert_eq!(
        (baseline_outcome.status, outcome.status),
        ("PASS", "PASS"),
        "baseline: {}; modifier: {}",
        baseline_outcome.detail,
        outcome.detail
    );
}
