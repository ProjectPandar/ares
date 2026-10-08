//! A second PrintInstance of the same source object, not another ModelObject.
use super::ExportedCase;
use std::io::{Cursor, Read, Write};

/// Retargets the first build item onto the second object so the project
/// holds ONE print object with TWO instances (by-object M73 coverage:
/// upstream accumulates `instances().size() * temp_layer_count`,
/// `GCode.cpp:2505`).
pub(crate) fn second_instance_of_first_object(mut exported: ExportedCase) -> ExportedCase {
    let project = {
        let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..input.len() {
            let mut file = input.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if file.name() == "3D/3dmodel.model" {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replacen("objectid=\"2\"", "objectid=\"4\"", 1)
                    .into_bytes();
            } else if file.name() == "Metadata/model_settings.config" {
                let mut xml = String::from_utf8(bytes).unwrap();
                xml = xml.replacen(
                    "<metadata key=\"object_id\" value=\"2\"/>",
                    "<metadata key=\"object_id\" value=\"4\"/>",
                    1,
                );
                if let Some(start) = xml.find("<metadata key=\"object_id\" value=\"4\"/>") {
                    if let Some(offset) = xml[start..].find("<metadata key=\"instance_id\" value=\"0\"/>") {
                        let at = start + offset;
                        xml.replace_range(at..at + "<metadata key=\"instance_id\" value=\"0\"/>".len(),
                            "<metadata key=\"instance_id\" value=\"1\"/>");
                    }
                }
                xml = xml.replacen(
                    "<metadata key=\"identify_id\" value=\"45\"/>",
                    "<metadata key=\"identify_id\" value=\"145\"/>",
                    1,
                );
                bytes = xml.into_bytes();
            }
            output
                .start_file(file.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
        output.finish().unwrap().into_inner()
    };
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].instances().len(), 2);
    exported.project = project;
    exported
}

pub(crate) fn second_copy(mut exported: ExportedCase) -> ExportedCase {
    let project = {
        let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..input.len() {
            let mut file = input.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if file.name() == "3D/3dmodel.model" {
                let mut xml = String::from_utf8(bytes).unwrap();
                let start = xml.find("<build").unwrap();
                let end = start + xml[start..].find("</build>").unwrap() + "</build>".len();
                xml.replace_range(start..end, concat!(
                    "<build>",
                    "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 95 110 5\" printable=\"1\"/>",
                    "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 125 110 5\" printable=\"1\"/>",
                    "</build>"));
                bytes = xml.into_bytes();
            } else if file.name() == "Metadata/model_settings.config" {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace(
                        "</plate>",
                        concat!(
                            "<model_instance><metadata key=\"object_id\" value=\"2\"/>",
                            "<metadata key=\"instance_id\" value=\"1\"/>",
                            "<metadata key=\"identify_id\" value=\"902\"/></model_instance></plate>"
                        ),
                    )
                    .replace(
                        "</assemble>",
                        concat!(
                            "<assemble_item object_id=\"2\" instance_id=\"1\" ",
                            "transform=\"1 0 0 0 1 0 0 0 1 0 0 5\" offset=\"0 0 0\"/></assemble>"
                        ),
                    )
                    .into_bytes();
            }
            output
                .start_file(file.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
        output.finish().unwrap().into_inner()
    };
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].instances().len(), 2);
    for (instance, expected_x) in loaded.objects()[0].instances().iter().zip([95.0, 125.0]) {
        assert!(instance.printable());
        let origin = instance
            .transform()
            .transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
        assert_eq!((origin.x, origin.y), (expected_x, 110.0));
    }
    let stem = exported.project_path.file_stem().unwrap().to_string_lossy();
    exported.project_path = exported
        .project_path
        .with_file_name(format!("{stem}-two-copies.3mf"));
    exported.output_dir = exported.output_dir.join("two-copies");
    std::fs::create_dir_all(&exported.output_dir).unwrap();
    std::fs::write(&exported.project_path, &project).unwrap();
    exported.project = project;
    exported
}
