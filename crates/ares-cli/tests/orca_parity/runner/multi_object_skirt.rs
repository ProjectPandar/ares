use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[test]
fn unequal_height_objects_preserve_complete_skirt_output() {
    run("1", "combined", "99", "disabled", 50);
}

#[test]
fn zero_loop_per_object_shield_preserves_finite_hull_and_complete_output() {
    run("0", "perobject", "1", "enabled", 0);
}

#[test]
fn no_skirt_per_object_preserves_complete_output() {
    run("0", "perobject", "0", "disabled", 0);
}

fn run(loops: &str, skirt_type: &str, height: &str, shield: &str, skirt_layers: usize) {
    if !crate::runner::reference_enabled("ARES_PARITY_MULTIOBJECT_SKIRT_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let tall_x = if loops == "0" && height == "1" {
        109.0
    } else {
        125.0
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    if loops == "0" {
        let start = machine["machine_start_gcode"].as_str().unwrap().to_owned();
        machine.insert("machine_start_gcode".into(), json!(start + "\n; footprint_min = {first_layer_print_min[0]},{first_layer_print_min[1]}\n; footprint_max = {first_layer_print_max[0]},{first_layer_print_max[1]}"));
    }
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("skirt_loops", loops),
        ("skirt_type", skirt_type),
        ("skirt_height", height),
        ("draft_shield", shield),
        ("print_sequence", "by layer"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    if loops == "0" {
        process.insert("min_skirt_length".into(), json!("20"));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let label = if loops == "0" {
        format!("multi-object-skirt/zero/{skirt_type}/{height}/{shield}")
    } else {
        "multi-object-skirt/unequal-heights/by-layer".to_owned()
    };
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
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
            let mut tall = original
                .replacen("id=\"2\"", "id=\"3\"", 1)
                .replace("00000001-", "00000003-")
                .replace("00010000-", "00020000-")
                .replace(
                    "/3D/Objects/cube10.stl_1.model",
                    "/3D/Objects/cube10-tall.model",
                );
            if loops == "0" {
                tall = tall.replace("objectid=\"1\"", "objectid=\"4\"");
            }
            let short = original.replace(
                "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
                "transform=\"1 0 0 0 1 0 0 0 0.4 0 0 0\"",
            );
            xml.replace_range(start..end, &format!("{short}\n{tall}"));
            let start = xml.find("<build").unwrap();
            let end = start + xml[start..].find("</build>").unwrap() + "</build>".len();
            let build = concat!(
                "<build>",
                "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 95 110 2\" printable=\"1\"/>",
                "<item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 125 110 5\" printable=\"1\"/>",
                "</build>",
            )
            .replace("125 110 5", &format!("{tall_x} 110 5"));
            xml.replace_range(start..end, &build);
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
            let mut tall = xml[start..end].replacen("id=\"2\"", "id=\"3\"", 1);
            if loops == "0" {
                tall = tall.replace("<part id=\"1\"", "<part id=\"4\"").replace(
                    "key=\"source_file\" value=\"cube10.stl\"",
                    "key=\"source_file\" value=\"cube10-tall.stl\"",
                );
            }
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
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
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
    if loops == "0" {
        let mut mesh = String::from_utf8(tall_mesh)
            .unwrap()
            .replace("<object id=\"1\"", "<object id=\"4\"")
            .replace("00010000-", "00020000-");
        for (x, y) in [(5, 5), (-5, -5), (5, -5), (-5, 5)] {
            mesh = mesh.replace(
                &format!("<vertex x=\"{x}\" y=\"{y}\" z=\"5\"/>"),
                &format!("<vertex x=\"{}\" y=\"{}\" z=\"5\"/>", x * 2, y * 2),
            );
        }
        tall_mesh = mesh.into_bytes();
    }
    output
        .start_file(
            "3D/Objects/cube10-tall.model",
            zip::write::SimpleFileOptions::DEFAULT,
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
    if loops == "0" {
        application::verify_value(&project, "min_skirt_length", &json!("20")).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 2);
    assert_ne!(
        loaded.objects()[0].volumes()[0].source_model_path(),
        loaded.objects()[1].volumes()[0].source_model_path()
    );
    for (object, (expected_height, expected_x)) in
        loaded.objects().iter().zip([(4.0, 95.0), (10.0, tall_x)])
    {
        let volume = &object.volumes()[0];
        let expected_id = if loops == "0" && expected_height == 10.0 {
            4
        } else {
            1
        };
        assert_eq!(volume.id(), expected_id);
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
        if loops == "0" && expected_height == 10.0 {
            let xs = volume.mesh().vertices().iter().map(|point| point.x);
            let min = xs.clone().fold(f64::INFINITY, f64::min);
            let max = xs.fold(f64::NEG_INFINITY, f64::max);
            assert!((max - min - 20.0).abs() < 1e-6);
        }
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
        skirt_layers
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
