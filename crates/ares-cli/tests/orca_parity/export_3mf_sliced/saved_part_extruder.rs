use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::{Value, json};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{STL, comparator, invoke, repo_root, write_inputs};
use crate::{OrcaRunner, golden::compare_ordered_bytes_generator_only};

const PROJECT: &str = "saved.3mf";
const EXPORT: &str = "export.3mf";
const SETTINGS: &str = "Metadata/model_settings.config";
const SLICE: &[&str] = &["--slice", "1", "--outputdir", "out", PROJECT];

#[test]
fn plain_slice_uses_object_slot_one() {
    compare(Some(1), None, "1", "0");
}

#[test]
fn plain_slice_uses_object_slot_two() {
    compare(Some(2), None, "2", "1");
}

#[test]
fn plain_slice_ignores_a_lone_part_selector_without_an_object_selector() {
    compare(None, Some(2), "1", "0");
}

#[test]
fn plain_slice_prefers_the_saved_object_selector_over_its_lone_part() {
    compare(Some(2), Some(1), "2", "1");
}

#[test]
fn plain_slice_defaults_an_out_of_range_saved_object_slot() {
    compare(Some(3), None, "1", "0");
}

#[test]
fn saved_export_writes_object_slot_one() {
    compare_export(1, "1");
}

#[test]
fn saved_export_preserves_object_slot_two() {
    compare_export(2, "2");
}

#[test]
fn saved_export_defaults_an_out_of_range_object_slot() {
    compare_export(3, "1");
}

fn compare_export(object: u8, slot: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let project = saved_project(&runner, &root.join("source/cwd"), Some(object), None);
    let expected = export_metadata(runner.bin(), &root.join("native/cwd"), &project);
    let config: ExportedConfig = quick_xml::de::from_str(&expected).unwrap();
    let [object] = config.objects.as_slice() else {
        panic!("expected one object in Native export in {root:?}");
    };
    let [part] = object.parts.as_slice() else {
        panic!("expected one part in Native export in {root:?}");
    };
    let selectors = object
        .metadata
        .iter()
        .filter(|entry| entry.key == "extruder")
        .map(|entry| entry.value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(selectors, [slot], "unexpected Native export in {root:?}");
    assert!(part.metadata.iter().all(|entry| entry.key != "extruder"));
    let actual = export_metadata(&installed_ares(&root), &root.join("ares/cwd"), &project);
    assert_eq!(actual, expected, "{SETTINGS} differs in {root:?}");
}

#[derive(Deserialize)]
struct ExportedConfig {
    #[serde(rename = "object")]
    objects: Vec<ExportedObject>,
}

#[derive(Deserialize)]
struct ExportedObject {
    metadata: Vec<ExportedMetadata>,
    #[serde(rename = "part")]
    parts: Vec<ExportedPart>,
}

#[derive(Deserialize)]
struct ExportedPart {
    metadata: Vec<ExportedMetadata>,
}

#[derive(Deserialize)]
struct ExportedMetadata {
    #[serde(rename = "@key")]
    key: String,
    #[serde(rename = "@value")]
    value: String,
}

fn export_metadata(bin: &Path, cwd: &Path, project: &[u8]) -> String {
    fs::create_dir_all(cwd).unwrap();
    fs::write(cwd.join(PROJECT), project).unwrap();
    let args = ["--slice", "1", "--export-3mf", EXPORT, PROJECT];
    assert_eq!(invoke(bin, cwd, &args), Some(0), "export in {cwd:?}");
    comparator::read_members(&cwd.join(EXPORT))
        .into_iter()
        .find(|member| member.name == SETTINGS)
        .map(|member| String::from_utf8(member.bytes).unwrap())
        .unwrap_or_else(|| panic!("missing {SETTINGS} in {cwd:?}"))
}

fn compare(object: Option<u8>, part: Option<u8>, slot: &str, virtual_tool: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let project = saved_project(&runner, &root.join("source/cwd"), object, part);
    let expected = slice(runner.bin(), &root.join("native/cwd"), &project);
    let text = std::str::from_utf8(&expected).unwrap();
    for (prefix, value) in [("; filament: ", slot), (";VT", virtual_tool)] {
        let markers = text
            .lines()
            .filter_map(|line| line.strip_prefix(prefix))
            .collect::<Vec<_>>();
        assert_eq!(markers, [value], "unexpected Native selector in {root:?}");
    }

    let actual = slice(&installed_ares(&root), &root.join("ares/cwd"), &project);
    compare_ordered_bytes_generator_only(&expected, &actual)
        .unwrap_or_else(|error| panic!("{error}; {root:?}"));
}

fn installed_ares(root: &Path) -> PathBuf {
    let install = root.join("installation");
    let ares = install.join("bin/ares");
    fs::create_dir_all(ares.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ares"), &ares).unwrap();
    symlink(
        repo_root().join("OrcaSlicer/resources"),
        install.join("resources"),
    )
    .unwrap();
    ares
}

fn saved_project(runner: &OrcaRunner, cwd: &Path, object: Option<u8>, part: Option<u8>) -> Vec<u8> {
    fs::create_dir_all(cwd).unwrap();
    write_inputs(cwd);
    let process = cwd.join("process.json");
    let mut config: Value = serde_json::from_slice(&fs::read(&process).unwrap()).unwrap();
    config["brim_type"] = json!("no_brim");
    config["enable_prime_tower"] = json!("0");
    fs::write(process, serde_json::to_vec(&config).unwrap()).unwrap();
    let argv = [
        "--load-settings",
        "machine.json;process.json",
        "--load-filaments",
        "filament.json;filament.json",
        "--export-3mf",
        PROJECT,
        STL,
    ];
    assert_eq!(invoke(runner.bin(), cwd, &argv), Some(0));
    let archive = fs::read(cwd.join(PROJECT)).unwrap();
    rewrite_project(&archive, |member, text| match member {
        "3D/3dmodel.model" => {
            let saved_transform = "1 0 0 0 1 0 0 0 1 128 128 5";
            assert_eq!(text.matches(saved_transform).count(), 1);
            text.replace(saved_transform, "1 0 0 0 1 0 0 0 1 128 128 1")
        }
        "Metadata/model_settings.config" => {
            let original = "    <metadata key=\"extruder\" value=\"1\"/>\n";
            let part_start = "    <part id=\"1\" subtype=\"normal_part\">\n";
            assert_eq!(text.matches(original).count(), 1);
            assert_eq!(text.matches(part_start).count(), 1);
            let object = object.map_or_else(String::new, |id| {
                format!("    <metadata key=\"extruder\" value=\"{id}\"/>\n")
            });
            let mut text = text.replace(original, &object);
            if let Some(id) = part {
                text = text.replace(
                    part_start,
                    &format!("{part_start}      <metadata key=\"extruder\" value=\"{id}\"/>\n"),
                );
            }
            text
        }
        "Metadata/project_settings.config" => {
            let config: Value = serde_json::from_str(text).unwrap();
            let slots = config["filament_settings_id"].as_array().unwrap();
            assert_eq!(slots.len(), 2);
            assert_eq!(slots[0], slots[1]);
            text.to_owned()
        }
        _ => text.to_owned(),
    })
}

fn slice(bin: &Path, cwd: &Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir_all(cwd.join("out")).unwrap();
    fs::write(cwd.join(PROJECT), project).unwrap();
    assert_eq!(invoke(bin, cwd, SLICE), Some(0), "slice failed in {cwd:?}");
    fs::read(cwd.join("out/plate_1.gcode")).unwrap()
}
