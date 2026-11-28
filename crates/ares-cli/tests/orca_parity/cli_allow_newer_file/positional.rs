use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use super::{
    super::{export_3mf_sliced::invoke, export_3mf_sliced_rejection::rewrite_project},
    with_version,
};
use crate::{
    OrcaRunner, golden::compare_ordered_bytes_generator_only,
    native_wall_clock::result_comparison_bytes, runner::repo_root,
};
use serde_json::Value;

const SLICE: &[&str] = &["--slice", "1", "--outputdir", "out"];
const PLATE_REPORT: &str = "out/result.json";
const GCODE: &str = "out/plate_1.gcode";
const NEWER: &str = "2.6.0";

#[test]
fn direct_slice_rejects_newer_project_without_flag() {
    reject(&with_version(NEWER), SLICE, SLICE, (PLATE_REPORT, GCODE));
}

#[test]
fn direct_slice_rejects_newer_project_with_false_flag() {
    let argv = [&["--allow-newer-file=0"], SLICE].concat();
    reject(&with_version(NEWER), &argv, &argv, (PLATE_REPORT, GCODE));
}

#[test]
fn direct_slice_rejects_before_reading_replacement_presets() {
    let argv = [SLICE, &["--load-settings", "missing.json"]].concat();
    reject(&with_version(NEWER), &argv, &argv, (PLATE_REPORT, GCODE));
}

#[test]
fn direct_slice_all_rejects_newer_project() {
    let argv = ["--slice", "0", "--outputdir", "out"];
    reject(&with_version(NEWER), &argv, &argv, (PLATE_REPORT, GCODE));
}

#[test]
fn slice_subcommand_with_selection_rejects_newer_project() {
    reject(
        &with_version(NEWER),
        SLICE,
        &[&["slice"], SLICE].concat(),
        (PLATE_REPORT, GCODE),
    );
}

#[test]
fn unsliced_project_export_rejects_newer_project() {
    let argv = ["--export-3mf", "out/export.3mf"];
    reject(
        &with_version(NEWER),
        &argv,
        &argv,
        ("result.json", "out/export.3mf"),
    );
}

#[test]
fn settings_file_export_rejects_newer_project() {
    let argv = ["--export-settings", "out/settings.json"];
    reject(
        &with_version(NEWER),
        &argv,
        &argv,
        ("result.json", "out/settings.json"),
    );
}

#[test]
fn settings_stdout_export_rejects_before_writing_settings_json() {
    let argv = ["--export-settings", "-"];
    reject(
        &with_version(NEWER),
        &argv,
        &argv,
        ("result.json", "out/settings.json"),
    );
}

#[test]
fn single_stl_export_rejects_newer_project() {
    let native = ["--export-stl", "--outputdir", "out"];
    let ares = [&["slice"], native.as_slice()].concat();
    reject(
        &with_version(NEWER),
        &native,
        &ares,
        (PLATE_REPORT, "out/stl"),
    );
}

#[test]
fn multi_stl_export_rejects_newer_project() {
    let native = ["--export-stls", "out/meshes", "--outputdir", "out"];
    let ares = [&["slice"], native.as_slice()].concat();
    reject(
        &with_version(NEWER),
        &native,
        &ares,
        (PLATE_REPORT, "out/meshes"),
    );
}

#[test]
fn positional_assembly_rejects_newer_project() {
    let assemble = ["--assemble", "--arrange", "0"];
    let native = [&assemble, &["--slice", "0", "--outputdir", "out"][..]].concat();
    let ares = [&["slice"], assemble.as_slice(), &["-o", GCODE]].concat();
    reject(&with_version(NEWER), &native, &ares, (PLATE_REPORT, GCODE));
}

#[test]
fn application_fallback_rejects_newer_project_without_orca_tag() {
    reject(
        &without_orca_tag("BambuStudio-02.06.00.51"),
        SLICE,
        SLICE,
        (PLATE_REPORT, GCODE),
    );
}

#[test]
fn direct_slice_allows_newer_project_with_bare_flag() {
    accept_slice(&with_version(NEWER), Some("--allow-newer-file"));
}

#[test]
fn direct_slice_allows_newer_project_with_explicit_true() {
    accept_slice(&with_version(NEWER), Some("--allow-newer-file=1"));
}

#[test]
fn newer_patch_in_orca_tag_does_not_gate_newer_application() {
    // The checked-in synthetic input's BambuStudio Application is 2.6. The
    // parsed OrcaSlicer 2.5.99 takes precedence and has the same major/minor.
    accept_slice(&with_version("2.5.99"), None);
}

#[test]
fn flag_is_accepted_for_settings_stdout_without_slicing() {
    let argv = ["--export-settings", "-", "--allow-newer-file=1"];
    accept_settings_stdout(&with_version(NEWER), &argv);
}

#[test]
fn foreign_application_is_not_gated_by_its_version() {
    accept_settings_stdout(
        &without_orca_tag("OtherSlicer-99.0.0"),
        &["--export-settings", "-"],
    );
}

fn without_orca_tag(application: &str) -> Vec<u8> {
    rewrite_project(&with_version(NEWER), |name, xml| {
        if name != "3D/3dmodel.model" {
            return xml.to_owned();
        }
        let orca = r#"<metadata name="OrcaSlicer">2.6.0</metadata>"#;
        let app = r#"<metadata name="Application">BambuStudio-02.06.00.51</metadata>"#;
        assert_eq!(xml.matches(orca).count(), 1);
        assert_eq!(xml.matches(app).count(), 1);
        xml.replace(orca, "").replace(
            app,
            &format!(r#"<metadata name="Application">{application}</metadata>"#),
        )
    })
}

fn reject(
    project: &[u8],
    native_args: &[&str],
    ares_args: &[&str],
    (report, product): (&str, &str),
) {
    let Some([native, ares]) = pair(project, native_args, ares_args) else {
        return;
    };
    assert_eq!(native.status, Some(232), "{:?}", native.cwd);
    assert_eq!(ares.status, native.status, "{:?}", ares.cwd);
    assert!(native.capture("stderr.log").is_empty(), "{:?}", native.cwd);
    compare_diagnostics(&native, &ares, report);
    for side in [&native, &ares] {
        assert!(
            !side.cwd.join(product).exists(),
            "unexpected artifact in {:?}",
            side.cwd
        );
    }
}

fn accept_slice(project: &[u8], flag: Option<&str>) {
    let args: Vec<_> = SLICE.iter().copied().chain(flag).collect();
    let Some([native, ares]) = pair(project, &args, &args) else {
        return;
    };
    assert_eq!(native.status, Some(0), "{:?}", native.cwd);
    assert_eq!(ares.status, native.status, "{:?}", ares.cwd);
    compare_diagnostics(&native, &ares, PLATE_REPORT);
    compare_ordered_bytes_generator_only(&native.file(GCODE), &ares.file(GCODE))
        .unwrap_or_else(|error| panic!("{:?}: {error}", ares.cwd));
}

fn accept_settings_stdout(project: &[u8], argv: &[&str]) {
    let Some([native, ares]) = pair(project, argv, argv) else {
        return;
    };
    assert_eq!(native.status, Some(0), "{:?}", native.cwd);
    assert_eq!(ares.status, native.status, "{:?}", ares.cwd);
    for side in [&native, &ares] {
        let settings: Value = serde_json::from_slice(&side.capture("stdout.log"))
            .unwrap_or_else(|error| panic!("{:?}: {error}", side.cwd));
        assert!(settings.is_object(), "{:?}", side.cwd);
    }
}

fn compare_diagnostics(native: &Outcome, ares: &Outcome, report: &str) {
    for name in ["stdout.log", "stderr.log"] {
        assert_eq!(
            ares.capture(name),
            native.capture(name),
            "{name}: {:?}",
            ares.cwd
        );
    }
    let parsed = |side: &Outcome| -> Value {
        serde_json::from_slice(&result_comparison_bytes(&side.file(report))).unwrap()
    };
    assert_eq!(parsed(ares), parsed(native), "{report}: {:?}", ares.cwd);
}

struct Outcome {
    cwd: PathBuf,
    status: Option<i32>,
}

impl Outcome {
    fn file(&self, name: &str) -> Vec<u8> {
        read(&self.cwd.join(name))
    }

    fn capture(&self, name: &str) -> Vec<u8> {
        read(&self.cwd.parent().unwrap().join(name))
    }
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

fn pair(project: &[u8], native_args: &[&str], ares_args: &[&str]) -> Option<[Outcome; 2]> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let root = runner.work_dir();
    let install = root.join("install");
    fs::create_dir_all(install.join("bin")).unwrap();
    symlink(
        repo_root().join("OrcaSlicer/resources"),
        install.join("resources"),
    )
    .unwrap();
    let bin = install.join("bin/ares");
    let compiled = env!("CARGO_BIN_EXE_ares");
    if fs::hard_link(compiled, &bin).is_err() {
        fs::copy(compiled, &bin).unwrap();
    }
    Some(
        [
            (runner.bin(), "native", native_args),
            (&bin, "ares", ares_args),
        ]
        .map(|(bin, side, args)| {
            let cwd = root.join(side).join("cwd");
            fs::create_dir_all(cwd.join("out")).unwrap();
            fs::write(cwd.join("input.3mf"), project).unwrap();
            let args = [args, &["input.3mf"]].concat();
            let status = invoke(bin, &cwd, &args);
            Outcome { cwd, status }
        }),
    )
}
