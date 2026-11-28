use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::{Value, json};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{STL, invoke, repo_root, write_inputs};
use crate::{OrcaRunner, golden::compare_ordered_bytes_generator_only};

const PROJECT: &str = "saved.3mf";
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

    let install = root.join("installation");
    let ares = install.join("bin/ares");
    fs::create_dir_all(ares.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ares"), &ares).unwrap();
    symlink(
        repo_root().join("OrcaSlicer/resources"),
        install.join("resources"),
    )
    .unwrap();
    let actual = slice(&ares, &root.join("ares/cwd"), &project);
    compare_ordered_bytes_generator_only(&expected, &actual)
        .unwrap_or_else(|error| panic!("{error}; {root:?}"));
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
