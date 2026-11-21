use super::rewrite_archive;
use crate::{
    OrcaRunner, golden,
    presets::VendorProfiles,
    runner::{
        CaseInputs,
        application::{cli_command, settings, slice_project_plate_validated},
        native_api_cli::NativeCase,
        repo_root,
    },
};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Map, json};
use std::fs;

#[tokio::test]
async fn prusa_xl_ramming_drives_first_layer_footprint_like_native() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let project = export_xl_project(&runner);
    let case = NativeCase::slice(&runner, &project, "xl-ramming-footprint-");
    let api = slice_project_plate_validated(
        &project,
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(case.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI at {:?}", case.root);
    let cli_bytes = fs::read(cli_path).unwrap();
    assert!(api == cli_bytes, "API/CLI differ at {:?}", case.root);
    let comparison = golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), &api);
    fs::write(
        case.root.join("full-comparison.txt"),
        format!("{comparison:?}\n"),
    )
    .unwrap();

    let native_area = m555(&case.native);
    assert_eq!(native_area, "M555 X72.1869 Y173 W120.813 H75.2464");
    assert_eq!(
        m555(std::str::from_utf8(&api).unwrap()),
        native_area,
        "first-layer print area at {:?}",
        case.root
    );
    comparison.unwrap_or_else(|error| panic!("complete output at {:?}: {error}", case.root));
}

fn export_xl_project(runner: &OrcaRunner) -> Vec<u8> {
    let repo = repo_root();
    let presets =
        VendorProfiles::load(&repo.join("OrcaSlicer/resources/profiles"), "Prusa").unwrap();
    let machine = presets.machine("Prusa XL 5T 0.4 nozzle").unwrap();
    let process = presets.process("0.20mm Speed @Prusa XL 5T 0.4").unwrap();
    let mut filament = presets.filament("Generic PLA @Prusa XL").unwrap();
    // Materialize the false default so the exporter composes one entry per filament.
    filament
        .entry("filament_is_support")
        .or_insert_with(|| json!(["0"]));
    let dir = tempfile::tempdir_in(crate::artifacts::root_from_env().unwrap()).unwrap();
    let cube = repo.join("tests/parity/cube10.stl");
    let second_cube = dir.path().join("second-cube10.stl");
    fs::copy(&cube, &second_cube).unwrap();
    let export = runner
        .export_case_multi(
            &CaseInputs {
                label: "prusa-xl-multitool-ramming",
                machine: &machine,
                process: &process,
                filaments: &[filament.clone(), filament],
            },
            &Map::new(),
            &[&cube, &second_cube],
            &[1, 2],
        )
        .unwrap();
    let mut config = settings(&export.project).unwrap();
    // Native's flush matrix recomputation indexes these selectors by physical head.
    config.insert("extruder_type".into(), json!(vec!["Direct Drive"; 5]));
    config.insert("nozzle_volume_type".into(), json!(vec!["Standard"; 5]));
    config.insert("filament_colour".into(), json!(["#0060C0", "#FDBD10"]));
    config.insert("wipe_tower_x".into(), json!(["100"]));
    rewrite_archive(&export.project, |entry, _| {
        (entry == "Metadata/project_settings.config")
            .then(|| serde_json::to_string(&config).unwrap())
    })
}

fn m555(gcode: &str) -> &str {
    gcode
        .lines()
        .find(|line| line.starts_with("M555 "))
        .unwrap()
}
