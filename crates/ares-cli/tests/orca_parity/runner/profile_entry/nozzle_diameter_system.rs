use super::{native_user_presets, placed_cube, vendor, write_json};
use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root, stages};
use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};
use std::fs;

#[tokio::test]
async fn h2d_system_preset_nozzle_diameters_match_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = repo_root().join("OrcaSlicer/resources/profiles");
    let vendors = ["OrcaFilamentLibrary", "BBL"];
    let catalog = json!({
        "vendors": vendors.map(|name| vendor(&profiles, name)),
        "imports": [],
    });
    let selection = json!({
        "machine": "Bambu Lab H2D 0.4 nozzle",
        "process": "0.20mm Standard @BBL H2D",
        "filaments": ["Bambu PLA Basic @BBL H2D"],
    });
    write_json(&runner.work.join("catalog.json"), &catalog);
    write_json(&runner.work.join("selection.json"), &selection);
    let source: Value = serde_json::from_slice(
        &fs::read(
            profiles.join("BBL/machine/Bambu Lab H2D 0.4 nozzle template machine_start_gcode.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let template = source["machine_start_gcode"].as_str().unwrap();
    assert_eq!(template.matches("nozzle_diameter_at_nozzle_id").count(), 13);
    let datadir = native_user_presets::system(&runner.work, &profiles, &vendors);
    let system = datadir.join("system/BBL");
    let machine = system.join("machine/Bambu Lab H2D 0.4 nozzle.json");
    let process = system.join("process/0.20mm Standard @BBL H2D.json");
    let filament = system.join("filament/Bambu PLA Basic @BBL H2D.json");
    let stl = placed_cube(&fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap());
    let stl_path = runner.work.join("cube10.stl");
    fs::write(&stl_path, &stl).unwrap();
    let project_path = runner.work.join("system.3mf");
    stages::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            datadir.to_string_lossy().into_owned(),
            "--load-settings".to_owned(),
            format!("{};{}", machine.display(), process.display()),
            "--load-filaments".to_owned(),
            filament.to_string_lossy().into_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--export-3mf".to_owned(),
            project_path.to_string_lossy().into_owned(),
            stl_path.to_string_lossy().into_owned(),
        ],
        stages::Stage::Export,
        &runner.work,
    )
    .unwrap();
    let project = fs::read(&project_path).unwrap();
    let case = NativeCase::slice(&runner, &project, "h2d-system-nozzle-array-");
    assert!(case.native.contains(";===== machine: H2D "));
    for prefix in ["M620.10 A0 ", "M620.10 A1 "] {
        assert!(
            case.native
                .lines()
                .any(|line| line.starts_with(prefix) && line.contains(" H0.4 "))
        );
    }
    assert!(case.native.contains("M1015.4 S1 K1 H0.4 "));

    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let request = || ProfileSliceRequest {
        filename: "input.3mf",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let composed = ares_core::profile_settings(&project, request())
        .await
        .unwrap();
    let composed = serde_json::to_value(composed).unwrap();
    write_json(&case.root.join("composition-settings.json"), &composed);
    assert_eq!(
        composed["machine_start_gcode"],
        source["machine_start_gcode"]
    );
    let api = ares_core::slice_with_profiles_processed_validated(
        &project,
        request(),
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .map(|processed| processed.gcode);
    if let Ok(bytes) = &api {
        fs::write(case.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "--project-preset-catalog"])
        .arg(runner.work.join("catalog.json"))
        .arg("-o")
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(case.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(case.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&cli_path);
    let api_comparison = api.as_ref().ok().map(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), bytes)
    });
    let cli_comparison = cli_bytes.as_ref().ok().map(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), bytes)
    });
    write_json(
        &case.root.join("result.json"),
        &json!({
            "selection": selection,
            "system_template_references": 13,
            "native_premises": "PASS",
            "native_sha256": crate::runner::native_api_cli::sha(case.native.as_bytes()),
            "input_sha256": crate::runner::native_api_cli::sha(&project),
            "api_emission_error": api.as_ref().err().map(|error| format!("{error:?}")),
            "api_comparison": api_comparison,
            "cli_code": cli.status.code(),
            "cli_output_error": cli_bytes.as_ref().err().map(ToString::to_string),
            "cli_comparison": cli_comparison,
            "strict_complete_output_required": true,
        }),
    );
    let api = api.unwrap_or_else(|error| panic!("API at {:?}: {error:?}", case.root));
    assert!(cli.status.success(), "CLI at {:?}", case.root);
    let cli = cli_bytes.unwrap();
    crate::golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), &api).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(case.native.as_bytes(), &cli).unwrap();
    assert_eq!(api, cli, "raw API/CLI equality");
}
