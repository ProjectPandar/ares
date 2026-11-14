//! Ares-only symmetric fixture: the instance overload must not center volumes
//! or drop a floating instance to the bed as the object overload does.
use ares_core::{load_project, plater::auto_orient_instance};
use std::io::{Cursor, Read, Write};
const CUBE: &[u8] =
    include_bytes!("../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf");

#[test]
fn instance_rotation_keeps_saved_translation_and_volume_frame() {
    let mut archive = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut root = String::new();
    archive
        .by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut root)
        .unwrap();
    let old = "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"";
    assert_eq!(root.matches(old).count(), 1);
    let root = root.replace(old, "transform=\"1 0 0 0 1 0 0 0 1 110 110 40\"");
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "3D/3dmodel.model" {
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(root.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    let input = writer.finish().unwrap().into_inner();
    let mut project = load_project(&input).unwrap();
    let object = &project.objects()[0];
    let instance = &object.instances()[0];
    let (object_id, instance_id) = (object.id(), instance.instance_id());
    let transform = instance.transform();
    let volumes = object.volumes().to_vec();
    let result = auto_orient_instance(&mut project, object_id, instance_id).unwrap();
    assert_eq!(result.direction, [0., 0., 1.]);
    assert_eq!(project.objects()[0].instances()[0].transform(), transform);
    assert_eq!(project.objects()[0].volumes(), volumes);
}

#[test]
fn core_instance_premultiplies_nonidentity_rotation_without_centering_or_drop() {
    let mut archive = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_owned();
        if !matches!(
            name.as_str(),
            "3D/3dmodel.model" | "3D/Objects/cube10.stl_1.model" | "Metadata/model_settings.config"
        ) {
            writer.raw_copy_file(entry).unwrap();
            continue;
        }
        let mut xml = String::new();
        entry.read_to_string(&mut xml).unwrap();
        match name.as_str() {
            "3D/3dmodel.model" => {
                xml = xml.replace(
                    "1 0 0 0 1 0 0 0 1 110 110 5",
                    "0 0 -0.75 -1.25 0 0 0 -1.5 0 110 110 40",
                );
                xml = xml.replace("</build>", r#"<item objectid="2" transform="0.5 0 0 0 0.5 0 0 0 0.5 150 110 30" printable="1" auto_drop="1"/></build>"#);
            }
            "3D/Objects/cube10.stl_1.model" => {
                let start = xml.find("<mesh>").unwrap();
                let end = xml.find("</mesh>").unwrap() + 7;
                xml.replace_range(
                    start..end,
                    r#"<mesh><vertices>
<vertex x="0" y="0" z="-5"/><vertex x="11" y="0" z="-5"/>
<vertex x="1" y="7" z="-5"/><vertex x="2" y="1" z="12"/>
</vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="1" v3="3"/>
<triangle v1="1" v2="2" v3="3"/><triangle v1="2" v2="0" v3="3"/></triangles></mesh>"#,
                );
            }
            "Metadata/model_settings.config" => {
                xml = xml.replace("</plate>", r#"<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="15"/></model_instance></plate>"#);
            }
            _ => unreachable!(),
        }
        writer
            .start_file(&name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(xml.as_bytes()).unwrap();
    }
    let input = writer.finish().unwrap().into_inner();
    let mut project = load_project(&input).unwrap();
    let object = &project.objects()[0];
    assert_eq!(object.instances().len(), 2);
    let first = &object.instances()[0];
    let (id, instance) = (object.id(), first.instance_id());
    let before = first.transform();
    let other = object.instances()[1].transform();
    let volumes = object.volumes().to_vec();
    let result = auto_orient_instance(&mut project, id, instance).unwrap();
    let after = project.objects()[0].instances()[0].transform();
    let a = after.to_column_major();
    let b = before.to_column_major();
    let root = std::env::var_os("ARES_CORE_ORIENT_ARTIFACT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("ares-core-auto-orientation"));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("core-instance-nonidentity.3mf"), input).unwrap();
    std::fs::write(root.join("core-instance-nonidentity.json"), serde_json::to_vec_pretty(&serde_json::json!({"scope":"core-only; no Native comparison", "source":"Orient.cpp:638-648; Model.hpp:1483-1487", "before":b,"after":a,"rotation":result.rotation_matrix,"angle":result.angle,"direction":result.direction})).unwrap()).unwrap();
    assert!(result.angle > 0.01);
    for row in 0..3 {
        assert_eq!(a[row + 12], b[row + 12]);
        for column in 0..3 {
            let expected: f64 = (0..3)
                .map(|k| result.rotation_matrix[row][k] * b[k + 4 * column])
                .sum();
            assert!(
                (a[row + 4 * column] - expected).abs() < 64. * f64::EPSILON,
                "row {row}, column {column}"
            );
        }
    }
    assert_eq!(project.objects()[0].instances()[1].transform(), other);
    assert_eq!(project.objects()[0].volumes(), volumes);
}
