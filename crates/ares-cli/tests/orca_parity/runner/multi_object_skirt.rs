use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[test]
fn orca_parity_unequal_height_objects_preserve_complete_skirt_output() {
    if std::env::var("ARES_PARITY_MULTIOBJECT_SKIRT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("skirt_loops", "1"),
        ("skirt_height", "99"),
        ("draft_shield", "disabled"),
        ("print_sequence", "by layer"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "multi-object-skirt/unequal-heights/by-layer",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/3dmodel.model" {
            let mut xml = String::from_utf8(bytes).unwrap();
            let start = xml.find("<object id=\"2\"").unwrap();
            let end = start + xml[start..].find("</object>").unwrap() + "</object>".len();
            let original = &xml[start..end];
            let tall = original
                .replacen("id=\"2\"", "id=\"3\"", 1)
                .replace("00000001-", "00000003-")
                .replace("00010000-", "00020000-")
                .replace(
                    "/3D/Objects/cube10.stl_1.model",
                    "/3D/Objects/cube10-tall.model",
                );
            let short = original.replace(
                "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
                "transform=\"1 0 0 0 1 0 0 0 0.4 0 0 0\"",
            );
            xml.replace_range(start..end, &format!("{short}\n{tall}"));
            let start = xml.find("<build").unwrap();
            let end = start + xml[start..].find("</build>").unwrap() + "</build>".len();
            xml.replace_range(start..end, concat!(
                "<build>",
                "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 95 110 2\" printable=\"1\"/>",
                "<item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 125 110 5\" printable=\"1\"/>",
                "</build>",
            ));
            bytes = xml.into_bytes();
        } else if file.name() == "3D/_rels/3dmodel.model.rels" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "</Relationships>",
                    concat!(
                        "<Relationship Target=\"/3D/Objects/cube10-tall.model\" Id=\"rel-2\" ",
                        "Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>",
                        "</Relationships>",
                    ),
                )
                .into_bytes();
        } else if file.name() == "Metadata/model_settings.config" {
            let mut xml = String::from_utf8(bytes).unwrap();
            let start = xml.find("<object id=\"2\"").unwrap();
            let end = start + xml[start..].find("</object>").unwrap() + "</object>".len();
            let tall = xml[start..end].replacen("id=\"2\"", "id=\"3\"", 1);
            xml.insert_str(end, &tall);
            xml = xml.replace(
                "</plate>",
                concat!(
                    "<model_instance>",
                    "<metadata key=\"object_id\" value=\"3\"/>",
                    "<metadata key=\"instance_id\" value=\"0\"/>",
                    "<metadata key=\"identify_id\" value=\"15\"/>",
                    "</model_instance></plate>",
                ),
            );
            bytes = xml.into_bytes();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let mut tall_mesh = Vec::new();
    input
        .by_name("3D/Objects/cube10.stl_1.model")
        .unwrap()
        .read_to_end(&mut tall_mesh)
        .unwrap();
    output
        .start_file(
            "3D/Objects/cube10-tall.model",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    output.write_all(&tall_mesh).unwrap();
    let project = output.finish().unwrap().into_inner();
    let project_path = runner.work.join("unequal-height-objects.3mf");
    let output_dir = runner.work.join("unequal-height-objects-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    for (key, value) in settings {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 2);
    assert_ne!(
        loaded.objects()[0].volumes()[0].source_model_path(),
        loaded.objects()[1].volumes()[0].source_model_path()
    );
    for (object, (expected_height, expected_x)) in
        loaded.objects().iter().zip([(4.0, 95.0), (10.0, 125.0)])
    {
        let volume = &object.volumes()[0];
        assert_eq!(volume.id(), 1);
        let instance = object.instances()[0].transform();
        let origin = instance.transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
        assert!((origin.x - expected_x).abs() < 1e-6);
        assert!((origin.y - 110.0).abs() < 1e-6);
        let heights = volume
            .mesh()
            .vertices()
            .iter()
            .map(|&point| {
                instance
                    .transform_point(volume.transform().transform_point(point))
                    .z
            })
            .collect::<Vec<_>>();
        let min = heights.iter().copied().fold(f64::INFINITY, f64::min);
        let max = heights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((min - 0.0).abs() < 1e-6);
        assert!((max - expected_height).abs() < 1e-6);
    }
    let sliced = runner
        .slice_case(&ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = parity::compare_case(&sliced);
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    let layers = reference.split(";LAYER_CHANGE").skip(1).collect::<Vec<_>>();
    assert_eq!(layers.len(), 50);
    assert_eq!(
        layers
            .iter()
            .filter(|layer| layer.contains(";TYPE:Skirt\n"))
            .count(),
        50
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
