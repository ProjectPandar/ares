//! `print_sequence="by object"` port slice S1 RED
//! ([print-sequence-by-object-slice-plan.md](../../../docs/architecture/print-sequence-by-object-slice-plan.md)):
//! two objects must print sequentially — the native layer count is the
//! per-object sum and the whole file must match Ares. Both halves fail
//! today: Ares prints by layer (50/62 instead of 100/124).

use crate::runner::{OrcaRunner, repo_root};
use crate::{self as parity, CaseInputs, smoke_overrides};
use serde_json::Value;

fn by_object_case(runner: &OrcaRunner, vendor: &str, printer: &str, label: &str) {
    by_object_case_impl(runner, vendor, printer, label, false);
}

fn by_object_case_impl(
    runner: &OrcaRunner,
    vendor: &str,
    printer: &str,
    label: &str,
    second_instance: bool,
) {
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
    let exported = if second_instance {
        super::object_copy_second_print_instance::second_instance_of_first_object(exported)
    } else {
        exported
    };
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
fn print_sequence_by_object_single_object_two_instances_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // RED: one print object with two instances under `by object`. Upstream
    // iterates print object INSTANCES sequentially (GCode.cpp:3279 loop),
    // counts `instances().size() * temp_layer_count` (GCode.cpp:2505), and
    // runs the full inter-instance transition per copy (GCode.cpp:3304
    // gate, :3326-3327 hop/origin, :3333-3339 temps). Ares currently merges
    // both copies into one chunk: the header layer count says 62 vs native
    // 124 and the body is missing the inter-instance transition.
    by_object_case_impl(
        &runner,
        "Creality",
        "Creality Ender-3 0.4 nozzle",
        "by-object/ender3-one-object-two-instances",
        true,
    );
}

#[test]
fn print_sequence_by_object_ender3_matches_native() {
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
fn print_sequence_by_object_kobra3_matches_native() {
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
