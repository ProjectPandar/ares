//! Public profile-entry tracer. Native CLI input is independently flattened;
//! raw catalog input retains every inherited fragment for the Rust entry.

#[path = "native_user_presets.rs"]
pub(super) mod native_user_presets;

#[path = "loaded_config_acceptance.rs"]
mod loaded_config_acceptance;

#[path = "loaded_config_probes.rs"]
mod loaded_config_probes;

#[path = "loaded_config_validation.rs"]
mod loaded_config_validation;

#[path = "loaded_config_ranges.rs"]
mod loaded_config_ranges;

#[path = "plater_stl_session.rs"]
mod plater_stl_session;

#[path = "preset_scalar_array_slice.rs"]
mod preset_scalar_array_slice;
#[path = "preset_scalar_loading.rs"]
mod preset_scalar_loading;
#[path = "preset_vector_loading.rs"]
mod preset_vector_loading;

#[path = "../plater_archive.rs"]
mod plater_archive;

// The plater session's public slice entries against Native `--slice <plate>`.
#[path = "../plater_session.rs"]
mod plater_session;

// The plater session's arrange against the v2.5.0-beta Native global arrange.
#[path = "../plater_arrange.rs"]
mod plater_arrange;

use std::path::Path;

use serde_json::{Value, json};

use super::{OrcaRunner, stages};

#[test]
fn profile_entry_stl_load_settings() {
    run_profile_entry(&[]);
}

#[test]
fn profile_entry_cli_config_presence() {
    for upward in [json!([]), json!(["another printer"])] {
        run_profile_entry(&[
            ("machine", "upward_compatible_machine", upward),
            ("machine", "thumbnails", json!("48x48/PNG,300x300/PNG")),
            (
                "process",
                "compatible_printers_condition",
                json!("nozzle_diameter[0] == 0.4"),
            ),
            (
                "filament",
                "compatible_printers_condition",
                json!("printer_model != \"unknown\""),
            ),
        ]);
    }
}

fn run_profile_entry(overrides: &[(&str, &str, Value)]) {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine_name = "LH Stinger 0.4 nozzle";
    let process_name = profiles.compatible_process(machine_name).unwrap();
    let filament_name = profiles.compatible_filament(machine_name).unwrap();
    let mut catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "LH")],
        "imports": [],
    });
    let selection = json!({
        "machine": machine_name, "process": process_name, "filaments": [filament_name],
    });
    write_json(&runner.work.join("selection.json"), &selection);
    for (kind, name, mut fields) in [
        (
            "machine",
            machine_name,
            profiles.machine(machine_name).unwrap(),
        ),
        (
            "process",
            &process_name,
            profiles.process(&process_name).unwrap(),
        ),
        (
            "filament",
            &filament_name,
            profiles.filament(&filament_name).unwrap(),
        ),
    ] {
        // The CLI does not resolve separate parent files. Preserve the selected
        // parent name after the test-only native flattening removes it.
        let selected = catalog["vendors"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .flat_map(|vendor| vendor["files"].as_object_mut().unwrap().values_mut())
            .find(|profile| profile["type"] == kind && profile["name"] == name)
            .unwrap();
        if let Some(inherits) = selected.get("inherits") {
            fields.insert("inherits".to_owned(), inherits.clone());
        }
        for (_, key, value) in overrides.iter().filter(|(target, _, _)| *target == kind) {
            selected
                .as_object_mut()
                .unwrap()
                .insert((*key).to_owned(), value.clone());
            fields.insert((*key).to_owned(), value.clone());
        }
        write_json(
            &runner.work.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    write_json(&runner.work.join("catalog.json"), &catalog);
    let input = placed_cube(&std::fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    std::fs::write(runner.work.join("cube10.stl"), &input).unwrap();
    let first = native(&runner, "reference-first");
    let repeat = native(&runner, "reference-repeat");
    let normalize = |bytes: &[u8]| {
        crate::golden::normalize_one_generator_line(bytes, crate::golden::GeneratorKind::Orca)
            .unwrap()
    };
    assert_eq!(
        normalize(&first),
        normalize(&repeat),
        "native full-output repeat"
    );

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "cube10.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ares_core::ConfigOverrides::default(),
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&first, &actual);
    write_json(
        &runner.work.join("result.json"),
        &json!({
            "selection": selection,
            "native_repeat": "PASS",
            "rust_entry": "ares_core::slice_with_profiles",
            "evidence": crate::artifacts::EVIDENCE,
            "status": if comparison.is_ok() { "PASS" } else { "DIVERGENT" },
            "difference": comparison.as_ref().err().map(ToString::to_string),
        }),
    );
    comparison.unwrap();
}

pub(crate) fn placed_cube(source: &[u8]) -> Vec<u8> {
    // Both slicers receive the identical pre-positioned STL; arrangement is
    // outside the profile-entry contract. LH's bed center is (118, 138).
    let mut bytes = source.to_vec();
    for facet in bytes[84..].as_chunks_mut::<50>().0 {
        for vertex in facet[12..48].as_chunks_mut::<12>().0 {
            for (coordinate, shift) in vertex
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip([118.0_f32, 138.0, 5.0])
            {
                let value = f32::from_le_bytes(*coordinate);
                coordinate.copy_from_slice(&(value + shift).to_le_bytes());
            }
        }
    }
    bytes
}

fn native(runner: &OrcaRunner, output: &str) -> Vec<u8> {
    let output = runner.work.join(output);
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            super::join_paths(
                [
                    runner.work.join("machine.json"),
                    runner.work.join("process.json"),
                ]
                .iter(),
            ),
            "--load-filaments".to_owned(),
            runner
                .work
                .join("filament.json")
                .to_str()
                .unwrap()
                .to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("cube10.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    std::fs::read(output.join("plate_1.gcode")).unwrap()
}

pub(super) fn vendor(root: &Path, name: &str) -> Value {
    let index: Value =
        serde_json::from_slice(&std::fs::read(root.join(format!("{name}.json"))).unwrap()).unwrap();
    let files = [
        "machine_model_list",
        "machine_list",
        "process_list",
        "filament_list",
    ]
    .into_iter()
    .flat_map(|kind| index[kind].as_array().into_iter().flatten())
    .map(|entry| {
        let path = entry["sub_path"].as_str().unwrap();
        let value: Value =
            serde_json::from_slice(&std::fs::read(root.join(name).join(path)).unwrap()).unwrap();
        (path.to_owned(), value)
    })
    .collect::<serde_json::Map<_, _>>();
    json!({"index": index, "files": files})
}

pub(super) fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// `--center` placement parity: the CLI's `--center` transform is not
/// exposed as an option in this Orca build (`PrintConfig.cpp:10775` is
/// commented out), so the reference pre-positions the cube with its bbox
/// center at the bed center and grounded, while the Rust entry slices the
/// raw cube with `Placement::Center` (`OrcaSlicer.cpp:4376-4392`,
/// `Model.cpp:704-722`); identical world geometry must give identical
/// G-code.
#[test]
fn profile_entry_stl_center_placement() {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine_name = "LH Stinger 0.4 nozzle";
    let process_name = profiles.compatible_process(machine_name).unwrap();
    let filament_name = profiles.compatible_filament(machine_name).unwrap();
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "LH")],
        "imports": [],
    });
    let selection = json!({
        "machine": machine_name, "process": process_name, "filaments": [filament_name],
    });
    for (kind, fields) in [
        ("machine", profiles.machine(machine_name).unwrap()),
        ("process", profiles.process(&process_name).unwrap()),
        ("filament", profiles.filament(&filament_name).unwrap()),
    ] {
        let mut preset = fields;
        preset.insert("type".to_owned(), json!(kind));
        preset.insert("from".to_owned(), json!("system"));
        preset.insert("instantiation".to_owned(), json!("true"));
        write_json(
            &runner.work.join(format!("center-{kind}.json")),
            &Value::Object(preset),
        );
    }
    // The native reference receives the cube pre-positioned exactly where
    // `Placement::Center` must land it: bbox center at the LH bed center
    // (118,138), grounded at z=0.
    let raw = std::fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    let mut positioned = raw.clone();
    for facet in positioned[84..].as_chunks_mut::<50>().0 {
        for vertex in facet[12..48].as_chunks_mut::<12>().0 {
            for (coordinate, shift) in vertex
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip([118.0_f32, 138.0, 0.0])
            {
                let value = f32::from_le_bytes(*coordinate);
                *coordinate = (value + shift).to_le_bytes();
            }
        }
    }
    std::fs::write(runner.work.join("cube10.stl"), &positioned).unwrap();

    let settings = super::join_paths(
        [
            runner.work.join("center-machine.json"),
            runner.work.join("center-process.json"),
        ]
        .iter(),
    );
    let output = runner.work.join("center-reference");
    std::fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            settings,
            "--load-filaments".to_owned(),
            runner
                .work
                .join("center-filament.json")
                .to_str()
                .unwrap()
                .to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("cube10.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let reference = std::fs::read(output.join("plate_1.gcode")).unwrap();

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &raw,
            ares_core::ProfileSliceRequest {
                filename: "cube10.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ares_core::ConfigOverrides::default(),
                placement: ares_core::Placement::Center,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(runner.work.join("center-ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
