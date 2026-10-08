//! `print_sequence="by object"` port slice S1 RED
//! ([print-sequence-by-object-slice-plan.md](../../../docs/architecture/print-sequence-by-object-slice-plan.md)):
//! two objects must print sequentially — the native layer count is the
//! per-object sum and the whole file must match Ares. Both halves fail
//! today: Ares prints by layer (50/62 instead of 100/124).

use crate::runner::{OrcaRunner, repo_root};
use crate::{self as parity, CaseInputs, smoke_overrides};
use serde_json::Value;

fn by_object_case(runner: &OrcaRunner, vendor: &str, printer: &str, label: &str) {
    let profiles = crate::presets::VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        vendor,
    )
    .unwrap();
    let selection = parity::select_printer(&profiles, vendor, printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("print_sequence".into(), Value::String("by object".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let filament = profiles.filament(&selection.filaments[0]).unwrap();
    let filaments = vec![filament];
    let model = repo_root().join("tests/parity/cube10.stl");
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &[&model, &model],
            &[],
        )
        .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    let native_layers: u32 = native
        .lines()
        .find_map(|line| {
            line.strip_prefix("; total layer number: ")
                .and_then(|count| count.parse().ok())
        })
        .unwrap();
    assert_eq!(
        native_layers % 2,
        0,
        "{label}: the sequential layer count must be the per-object sum"
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
}

#[test]
fn orca_parity_print_sequence_by_object_ender3_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    by_object_case(
        &runner,
        "Creality",
        "Creality Ender-3 0.4 nozzle",
        "by-object/ender3",
    );
}

#[test]
fn orca_parity_print_sequence_by_object_kobra3_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    by_object_case(
        &runner,
        "Anycubic",
        "Anycubic Kobra 3 0.4 nozzle",
        "by-object/kobra3",
    );
}
