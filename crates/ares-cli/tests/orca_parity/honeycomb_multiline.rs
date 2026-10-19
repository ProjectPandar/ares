//! Regular Honeycomb multiline RED; spec boundary pending intermediate-value localization.
//!
//! The target must fail strict Native/Ares equality until the Honeycomb
//! spacing/alignment owner is localized and ported. Controls must remain exact.

use super::*;
use crate::presets::VendorProfiles;
use crate::runner::CaseInputs;
use serde_json::{Map, Value};

struct CaseParameters<'a> {
    label: &'a str,
    multiline: u32,
    angle: u32,
    density: &'a str,
}

type CaseSetup = (
    OrcaRunner,
    Map<String, Value>,
    Map<String, Value>,
    Vec<Map<String, Value>>,
);

fn case(
    runner: &OrcaRunner,
    machine: &Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &[Map<String, Value>],
    CaseParameters {
        label,
        multiline,
        angle,
        density,
    }: CaseParameters<'_>,
) -> crate::ParityOutcome {
    process.insert("sparse_infill_pattern".into(), Value::from("honeycomb"));
    process.insert("fill_multiline".into(), Value::from(multiline.to_string()));
    process.insert("sparse_infill_angle".into(), Value::from(angle.to_string()));
    process.insert("sparse_infill_density".into(), Value::from(density));
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine,
                process,
                filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root()
                .join("tests/parity/honeycomb-multiline/honeycomb-multiline-target.stl"),
        )
        .unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["sparse_infill_pattern"], "honeycomb");
    assert_eq!(settings["fill_multiline"], multiline.to_string());
    assert_eq!(settings["sparse_infill_density"], density);
    crate::compare_exported_for(runner, &exported, Some("fill_multiline"))
}

fn setup() -> Option<CaseSetup> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    normalize_filament_defaults(&mut filaments);
    Some((runner, machine, process, filaments))
}

#[test]
fn honeycomb_multiline_ordered_bytes() {
    let Some((runner, machine, mut process, filaments)) = setup() else {
        return;
    };
    let outcome = case(
        &runner,
        &machine,
        &mut process,
        &filaments,
        CaseParameters {
            label: "honeycomb/multiline-2/angle-45/density-20-target",
            multiline: 2,
            angle: 45,
            density: "20%",
        },
    );
    assert_eq!(
        outcome.status, "PASS",
        "target divergence: {}",
        outcome.detail
    );
}

#[test]
fn honeycomb_multiline_controls_match_native() {
    let Some((runner, machine, mut process, filaments)) = setup() else {
        return;
    };
    for (label, multiline, angle, density) in [
        (
            "honeycomb/multiline-3/angle-45/density-20-control",
            3,
            45,
            "20%",
        ),
        (
            "honeycomb/multiline-2/angle-0/density-40-control",
            2,
            0,
            "40%",
        ),
        (
            "honeycomb/multiline-3/angle-0/density-40-control",
            3,
            0,
            "40%",
        ),
    ] {
        let outcome = case(
            &runner,
            &machine,
            &mut process,
            &filaments,
            CaseParameters {
                label,
                multiline,
                angle,
                density,
            },
        );
        assert_eq!(
            outcome.status, "PASS",
            "control {label}: {}",
            outcome.detail
        );
    }
}
