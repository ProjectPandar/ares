use std::{fs, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SlicedPlate, SlicerContext, slice_with_load_settings,
};
use serde_json::{Value, json};

use super::{preset_files, stages_run};
use crate::{
    golden,
    runner::{OrcaRunner, application, placed_cube, repo_root},
};

const TEMPLATE: &str = r"; load-settings first\n; load-settings literal \\n\n; load-settings last";
const EXPECTED: &str = "; load-settings first\n; load-settings literal \\n\n; load-settings last\n";

#[tokio::test]
async fn stock_stl_matches_native() {
    compare_stl(false).await;
}

#[tokio::test]
async fn minimal_stl_preserves_literal_backslash_and_decodes_newlines() {
    compare_stl(true).await;
}

#[tokio::test]
async fn stock_3mf_matches_native() {
    compare_saved_project().await;
}

fn prepare(work: &Path, minimal: bool) -> (Vec<u8>, Vec<String>) {
    let presets = preset_files(work, "Creality", "Creality Ender-3 V3 SE 0.4 nozzle");
    let mut machine: Value = serde_json::from_slice(&fs::read(&presets[0]).unwrap()).unwrap();
    assert!(
        machine["file_start_gcode"]
            .as_str()
            .unwrap()
            .contains(r"\n;TIME:")
    );
    if minimal {
        machine["file_start_gcode"] = json!(TEMPLATE);
        fs::write(&presets[0], serde_json::to_vec_pretty(&machine).unwrap()).unwrap();
    }
    let stl = placed_cube(&fs::read(repo_root().join("tests/parity/cube10.stl")).unwrap());
    fs::write(work.join("cube10.stl"), &stl).unwrap();
    (stl, presets)
}

async fn compare_stl(minimal: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("scalar-string-")
        .tempdir()
        .unwrap()
        .keep();
    let work = directory.as_path();
    let (stl, presets) = prepare(work, minimal);
    let settings = presets[..2].join(";");
    let filaments = presets[2..].join(";");
    let load = ["--load-settings", &settings, "--load-filaments", &filaments];
    stages_run(
        &runner,
        work,
        &[
            &load[..],
            &[
                "--arrange",
                "0",
                "--slice",
                "0",
                "--outputdir",
                "native",
                "cube10.stl",
            ],
        ]
        .concat(),
    );
    let reference = fs::read(work.join("native/plate_1.gcode")).unwrap();
    let texts: Vec<_> = presets
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    let files: Vec<_> = presets
        .iter()
        .zip(&texts)
        .map(|(label, json)| LoadSettingsFile { label, json })
        .collect();
    let api = slice_with_load_settings(
        &stl,
        LoadSettingsSliceRequest {
            filename: "cube10.stl",
            catalog_json: None,
            uptodate: false,
            settings: &files[..2],
            filaments: &files[2..],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            orient: false,
            sliced_plate: &mut SlicedPlate::default(),
        },
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let args = [&load[..], &["-o", "ares.gcode", "cube10.stl"]].concat();
    compare(work, &reference, &api, &args, minimal);
}

async fn compare_saved_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("scalar-string-")
        .tempdir()
        .unwrap()
        .keep();
    let work = directory.as_path();
    let (_, presets) = prepare(work, false);
    stages_run(
        &runner,
        work,
        &[
            "--load-settings",
            &presets[..2].join(";"),
            "--load-filaments",
            &presets[2..].join(";"),
            "--arrange",
            "0",
            "--export-3mf",
            "saved.3mf",
            "cube10.stl",
        ],
    );
    let project = fs::read(work.join("saved.3mf")).unwrap();
    stages_run(
        &runner,
        work,
        &["--slice", "0", "--outputdir", "native", "saved.3mf"],
    );
    let reference = fs::read(work.join("native/plate_1.gcode")).unwrap();
    let api = application::slice_project(
        &project,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    compare(
        work,
        &reference,
        &api,
        &["-o", "ares.gcode", "saved.3mf"],
        false,
    );
}

fn compare(work: &Path, reference: &[u8], api: &[u8], args: &[&str], minimal: bool) {
    let output = application::cli_command()
        .current_dir(work)
        .arg("slice")
        .args(args)
        .output()
        .unwrap();
    fs::write(work.join("api.gcode"), api).unwrap();
    fs::write(work.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(work.join("ares.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "CLI {}: {} ({work:?})",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let cli = fs::read(work.join("ares.gcode")).unwrap();
    let header = std::str::from_utf8(reference).unwrap();
    if minimal {
        assert!(
            header.starts_with(EXPECTED),
            "missing Native escape witness ({work:?})"
        );
    } else {
        assert!(
            header.starts_with(";FLAVOR:Marlin\n;TIME:")
                && header.contains("\n;Filament used:")
                && header.contains("\n;Layer height:"),
            "missing Native stock header ({work:?})"
        );
    }
    for stream in [reference, api, &cli] {
        // This witness contains no other branded generator line for the comparator to normalize.
        assert_eq!(
            stream
                .split(|b| *b == b'\n')
                .filter(|line| line.starts_with(b"; generated by "))
                .count(),
            1
        );
    }
    assert!(
        api == cli,
        "API vs CLI ({work:?}): {:?}",
        golden::first_difference(api, &cli).map(|difference| difference.to_string())
    );
    if let Err(error) = golden::compare_ordered_bytes_generator_only(reference, api) {
        panic!("complete Native vs API/CLI ({work:?}): {error}");
    }
}
