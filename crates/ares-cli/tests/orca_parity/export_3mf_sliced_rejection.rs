use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde_json::{Value, json};
use zip::ZipArchive;

use super::export_3mf_sliced::{invoke, write_saved_input};
use crate::OrcaRunner;

#[test]
fn cli_sliced_3mf_rejects_multiobject_saved_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = runner.work_dir().join("source/cwd");
    let cwd = runner.work_dir().join("ares/cwd");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(cwd.join("out")).unwrap();
    write_saved_input(&runner, &source);
    let input = multiobject_project(&fs::read(source.join("saved.3mf")).unwrap(), true);
    let project = ares_core::load_project(&input).unwrap();
    assert_eq!(
        project
            .objects()
            .iter()
            .map(|object| object.volumes().len())
            .collect::<Vec<_>>(),
        [2, 1]
    );
    assert_eq!(project.plates().len(), 1);
    fs::write(cwd.join("saved.3mf"), &input).unwrap();

    let argv = ["--slice", "1", "--export-3mf", "export.3mf", "saved.3mf"];
    assert_eq!(
        invoke(Path::new(env!("CARGO_BIN_EXE_ares")), &cwd, &argv, false),
        Some(243)
    );
    let stderr = fs::read_to_string(cwd.parent().unwrap().join("stderr.log")).unwrap();
    assert!(
        stderr.starts_with("Project export to export.3mf failed\nFailed exporting 3mf files. ")
    );
    assert!(stderr.ends_with("sliced export requires one saved object\n"));
    let mut expected = vec!["out", "saved.3mf"];
    if cfg!(target_os = "linux") {
        let report: Value =
            serde_json::from_slice(&fs::read(cwd.join("result.json")).unwrap()).unwrap();
        assert_eq!(report["return_code"], -13);
        assert_eq!(report["error_string"], "Failed exporting 3mf files.");
        assert_eq!(report["plate_index"], 0);
        assert_eq!(report["warnings"], json!([]));
        expected.insert(1, "result.json");
    }
    let mut names: Vec<_> = fs::read_dir(&cwd)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names.sort_unstable();
    assert_eq!(names, expected);
    assert_eq!(fs::read_dir(cwd.join("out")).unwrap().count(), 0);
}

pub(super) fn multiobject_project(input: &[u8], additional_part: bool) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(input)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).unwrap();
        let name = member.name().to_owned();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        let object_variant = |object: &str| {
            object
                .replace("id=\"2\"", "id=\"3\"")
                .replace("00000001-", "00000002-")
        };
        let patched = match name.as_str() {
            "3D/3dmodel.model" => {
                let object = xml_element(xml, "object", " ");
                let component = xml
                    .lines()
                    .find(|line| line.contains("<component "))
                    .unwrap();
                let part = component
                    .replace("objectid=\"1\"", "objectid=\"4\"")
                    .replace("00010000-", "00040000-")
                    .replace(" 0 0 0\"", " 16 0 0\"");
                let second = object_variant(object)
                    .replace("objectid=\"1\"", "objectid=\"5\"")
                    .replace("00010000-", "00050000-");
                let item = xml.lines().find(|line| line.contains("<item ")).unwrap();
                let second_item = item
                    .replace("objectid=\"2\"", "objectid=\"3\"")
                    .replace("00000002-", "00000003-")
                    .replace(" 128 128 5\"", " 176 128 5\"");
                let first = if additional_part {
                    object.replace(component, &format!("{component}\n{part}"))
                } else {
                    object.to_owned()
                };
                xml.replace(object, &format!("{first}\n{second}"))
                    .replace(item, &format!("{item}\n{second_item}"))
            }
            "3D/Objects/3mf-export.stl_1.model" => {
                let mesh = xml_element(xml, "object", " ");
                let mut meshes = mesh.to_owned();
                for id in [4, 5].into_iter().skip(usize::from(!additional_part)) {
                    meshes.push_str(
                        &mesh
                            .replace("id=\"1\"", &format!("id=\"{id}\""))
                            .replace("00010000-", &format!("{id:04}0000-")),
                    );
                }
                xml.replace(mesh, &meshes)
            }
            "Metadata/model_settings.config" => {
                let object = xml_element(xml, "object", " ");
                let part = xml_element(object, "part", " ");
                let second_part = part.replace("id=\"1\"", "id=\"4\"").replace(
                    "key=\"source_volume_id\" value=\"0\"",
                    "key=\"source_volume_id\" value=\"1\"",
                );
                let second_object = object_variant(object)
                    .replace("id=\"1\"", "id=\"5\"")
                    .replace("3mf-export.stl", "second-cube.stl");
                let instance = xml_element(xml, "model_instance", ">");
                let second_instance = instance.replace(
                    "key=\"object_id\" value=\"2\"",
                    "key=\"object_id\" value=\"3\"",
                );
                let first = if additional_part {
                    object.replace(part, &format!("{part}\n{second_part}"))
                } else {
                    object.to_owned()
                };
                xml.replace(object, &format!("{first}\n{second_object}"))
                    .replace(instance, &format!("{instance}\n{second_instance}"))
            }
            _ => xml.to_owned(),
        };
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(patched.as_bytes()).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn xml_element<'a>(xml: &'a str, tag: &str, after_tag: &str) -> &'a str {
    let start = xml.find(&format!("<{tag}{after_tag}")).unwrap();
    let close = format!("</{tag}>");
    let end = xml[start..].find(&close).unwrap() + start + close.len();
    &xml[start..end]
}
