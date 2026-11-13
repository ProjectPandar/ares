use super::super::super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

pub(super) fn outputs(
    start: &str,
    end: &str,
    aliases: bool,
    machine_overrides: &[(&str, serde_json::Value)],
) -> (Vec<u8>, Vec<u8>) {
    configured(
        start,
        end,
        aliases,
        Overrides {
            machine: machine_overrides,
            ..Default::default()
        },
    )
}

#[derive(Default)]
pub(super) struct Overrides<'a> {
    pub(super) machine: &'a [(&'a str, serde_json::Value)],
    pub(super) process: &'a [(&'a str, &'a str)],
    pub(super) filaments: &'a [(&'a str, [&'a str; 2])],
}

pub(super) fn configured(
    start: &str,
    end: &str,
    aliases: bool,
    overrides: Overrides<'_>,
) -> (Vec<u8>, Vec<u8>) {
    let (_, reference, actual) = configured_project(start, end, aliases, overrides);
    (reference, actual)
}

/// The sliced project with its Native and Ares outputs.
pub(super) fn configured_project(
    start: &str,
    end: &str,
    aliases: bool,
    overrides: Overrides<'_>,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let machine_overrides = overrides.machine;
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Snapmaker",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Snapmaker", "Snapmaker J1 (0.4 nozzle)").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("filament_map".into(), json!(["1", "2"]));
    for (key, value) in machine_overrides {
        machine.insert((*key).into(), value.clone());
    }
    if aliases {
        machine.insert("enable_long_retraction_when_cut".into(), json!("2"));
        machine.insert("retraction_distances_when_cut".into(), json!(["10", "18"]));
        machine.insert("long_retractions_when_cut".into(), json!(["0", "1"]));
    }
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_filament_id", "2"),
        ("internal_solid_filament_id", "2"),
        ("top_surface_filament_id", "2"),
        ("bottom_surface_filament_id", "2"),
    ] {
        process.insert(key.into(), json!(value));
    }
    for &(key, value) in overrides.process {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("filament_start_gcode".into(), json!([start]));
        filament.insert("filament_end_gcode".into(), json!([end]));
    }
    for &(key, values) in overrides.filaments {
        for (filament, value) in filaments.iter_mut().zip(values) {
            filament.insert(key.into(), json!([value]));
        }
    }
    if aliases {
        for (filament, (distance, enabled)) in filaments.iter_mut().zip([("3", "0"), ("7", "1")]) {
            filament.insert("retraction_distances_when_ec".into(), json!([distance]));
            filament.insert("long_retractions_when_ec".into(), json!([enabled]));
        }
        for (filament, (distance, enabled)) in filaments.iter_mut().zip([("10", "0"), ("18", "1")])
        {
            filament.insert(
                "filament_retraction_distances_when_cut".into(),
                json!([distance]),
            );
            filament.insert(
                "filament_long_retractions_when_cut".into(),
                json!([enabled]),
            );
        }
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/printing-tool-template-lifecycle",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    application::verify_value(
        &exported.project,
        "filament_start_gcode",
        &json!([start, start]),
    )
    .unwrap();
    application::verify_value(&exported.project, "filament_end_gcode", &json!([end, end])).unwrap();
    for (key, value) in machine_overrides {
        application::verify_value(&exported.project, key, value).unwrap();
    }
    for &(key, value) in overrides.process {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    for &(key, values) in overrides.filaments {
        application::verify_value(&exported.project, key, &json!(values)).unwrap();
    }
    if aliases {
        for (key, expected) in [
            ("enable_long_retraction_when_cut", json!("2")),
            ("retraction_distances_when_cut", json!(["10", "18"])),
            ("long_retractions_when_cut", json!(["0", "1"])),
            (
                "filament_retraction_distances_when_cut",
                json!(["10", "18"]),
            ),
            ("filament_long_retractions_when_cut", json!(["0", "1"])),
            ("retraction_distances_when_ec", json!(["3", "7"])),
            ("long_retractions_when_ec", json!(["0", "1"])),
        ] {
            application::verify_value(&exported.project, key, &expected).unwrap();
        }
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let complete = parity::compare_case(&sliced);
    eprintln!("complete output: {}: {}", complete.status, complete.detail);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime
        .block_on(crate::runner::application::slice_project(
            &sliced.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    (sliced.project, sliced.reference, actual)
}
