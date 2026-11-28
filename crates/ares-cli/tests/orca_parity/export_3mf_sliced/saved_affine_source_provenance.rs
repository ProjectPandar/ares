use std::{collections::BTreeMap, fs, path::Path};

use serde_json::{Value, json};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::saved_build_transforms::{MODEL, assert_build_item, prism_stl};
use super::saved_part_provenance::{KEYS, MODEL_SETTINGS, part_provenance, with_provenance};
use super::{SETTINGS, STL, comparator, invoke, write_inputs};
use crate::OrcaRunner;

const SOURCE: &str = "input/affine-witness.3mf";
const BASENAME: &str = "affine-witness.3mf";
const EXPORT: &str = "sliced.3mf";
const TILT: &str = "1 0 0 0 0 1 0 -1 0 128 128 5";
const EXPLICIT: [&str; 6] = ["original-prism.stl", "2", "4", "0.5", "-0.75", "1.25"];

#[test]
fn affine_part_without_source_fields_exports_with_fallback() {
    compare([None; 6], [BASENAME, "0", "0", "0", "0", "0"]);
}

#[test]
fn affine_part_missing_source_filename_keeps_set_companions() {
    compare(
        [None, Some("-1"), Some("9"), Some("0.125"), None, None],
        [BASENAME, "0", "9", "0.125", "0", "0"],
    );
}

#[test]
fn affine_part_empty_source_filename_keeps_set_companions() {
    compare(
        [Some(""), Some("6"), Some("-1"), None, None, Some("-0.25")],
        [BASENAME, "6", "0", "0", "0", "-0.25"],
    );
}

#[test]
fn affine_part_explicit_source_basename_retains_complete_provenance() {
    compare(EXPLICIT.map(Some), EXPLICIT);
}

#[test]
fn affine_part_explicit_nested_source_exports_its_basename() {
    let mut input = EXPLICIT.map(Some);
    input[0] = Some("source/nested/original-prism.stl");
    compare(input, EXPLICIT);
}

fn compare(input: [Option<&str>; 6], expected: [&str; 6]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let project = saved_project(&runner, &root.join("source/cwd"), input);
    let expected = KEYS
        .into_iter()
        .zip(expected.map(str::to_owned))
        .collect::<BTreeMap<_, _>>();

    let native = export(runner.bin(), &root.join("native/cwd"), &project, "Native");
    assert_sliced_part(&native, &expected, "Native");
    let ares = export(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        &root.join("ares/cwd"),
        &project,
        "Ares",
    );
    assert_sliced_part(&ares, &expected, "Ares");
    comparator::compare_members(native, ares);
}

fn saved_project(runner: &OrcaRunner, cwd: &Path, input: [Option<&str>; 6]) -> Vec<u8> {
    fs::create_dir_all(cwd).unwrap();
    write_inputs(cwd);
    fs::write(cwd.join(STL), prism_stl()).unwrap();
    let process = cwd.join("process.json");
    let mut settings: Value = serde_json::from_slice(&fs::read(&process).unwrap()).unwrap();
    settings["brim_type"] = json!("no_brim");
    fs::write(process, serde_json::to_vec(&settings).unwrap()).unwrap();
    let argv = [SETTINGS.as_slice(), &["--export-3mf", "generated.3mf", STL]].concat();
    assert_eq!(invoke(runner.bin(), cwd, &argv), Some(0));
    rewrite_project(
        &fs::read(cwd.join("generated.3mf")).unwrap(),
        |name, xml| match name {
            MODEL => {
                let item = xml.lines().find(|line| line.contains("<item ")).unwrap();
                let (_, attr) = item.split_once("transform=\"").unwrap();
                let (old, _) = attr.split_once('"').unwrap();
                xml.replace(item, &item.replace(old, TILT))
            }
            MODEL_SETTINGS => with_provenance(xml, input),
            _ => xml.to_owned(),
        },
    )
}

fn export(bin: &Path, cwd: &Path, project: &[u8], engine: &str) -> Vec<comparator::Member> {
    fs::create_dir_all(cwd.join("input")).unwrap();
    fs::write(cwd.join(SOURCE), project).unwrap();
    let argv = ["--slice", "1", "--export-3mf", EXPORT, SOURCE];
    assert_eq!(
        invoke(bin, cwd, &argv),
        Some(0),
        "{engine} affine provenance export in {cwd:?}"
    );
    comparator::read_members(&cwd.join(EXPORT))
}

fn assert_sliced_part(
    members: &[comparator::Member],
    expected: &BTreeMap<&str, String>,
    engine: &str,
) {
    let member = |name| {
        &members
            .iter()
            .find(|entry| entry.name == name)
            .unwrap()
            .bytes
    };
    assert_build_item(member(MODEL), TILT);
    assert_eq!(
        &part_provenance(member(MODEL_SETTINGS)),
        expected,
        "{engine} affine part source provenance"
    );
}
