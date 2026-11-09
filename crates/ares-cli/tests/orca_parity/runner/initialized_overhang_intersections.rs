#[path = "initialized_overhang_intersections/model.rs"]
mod model;

use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::Value;
use std::io::{Cursor, Read, Write};

#[test]
fn initialized_overhang_intersections_match_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_OVERHANG_INTERSECTION_REFERENCE") {
        return;
    }
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("floating-intersections.stl");
    std::fs::write(&model, model::loft()).unwrap();
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
    let vector_settings = [
        ("enable_overhang_speed", serde_json::json!(["1"])),
        ("slowdown_for_curled_perimeters", serde_json::json!(["0"])),
        ("outer_wall_speed", serde_json::json!(["200"])),
    ];
    let settings = [
        ("wall_loops", "1"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("sparse_infill_density", "0%"),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
        ("outer_wall_line_width", "0.42"),
        ("bridge_speed", "18"),
        ("overhang_1_4_speed", "50"),
        ("overhang_2_4_speed", "30"),
        ("overhang_3_4_speed", "20"),
        ("overhang_4_4_speed", "15"),
        ("enable_arc_fitting", "0"),
        ("skirt_loops", "0"),
    ];
    for (key, value) in &vector_settings {
        process.insert((*key).into(), value.clone());
    }
    for (key, value) in settings {
        process.insert(key.into(), Value::String(value.into()));
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
                label: "initialized-overhang/floating-intersections/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
        )
        .unwrap();

    // Bind ordinary input placement before either slicer consumes the project.
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/3dmodel.model" {
            let mut xml = String::from_utf8(bytes).unwrap();
            let build = xml.find("<build").unwrap();
            let start = build + xml[build..].find("transform=\"").unwrap() + "transform=\"".len();
            let end = start + xml[start..].find('"').unwrap();
            xml.replace_range(start..end, "1 0 0 0 1 0 0 0 1 110 110.000105 0");
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
    let project = output.finish().unwrap().into_inner();
    let project_path = runner.work.join("floating-intersections.3mf");
    let output_dir = runner.work.join("floating-intersections");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let exported = ExportedCase {
        label: exported.label,
        project,
        project_path,
        output_dir,
    };
    for (key, value) in &vector_settings {
        application::verify_value(&exported.project, key, value).unwrap();
    }
    for (key, value) in settings {
        // PrintConfig.cpp:1991–2054 declares these speeds as vector options.
        let expected = match key {
            "bridge_speed" | "overhang_1_4_speed" | "overhang_2_4_speed" | "overhang_3_4_speed"
            | "overhang_4_4_speed" => serde_json::json!([value]),
            _ => Value::String(value.into()),
        };
        application::verify_value(&exported.project, key, &expected).unwrap();
    }
    let loaded = ares_core::load_project(&exported.project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let transform = loaded.objects()[0].instances()[0].transform();
    let origin = transform.transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
    let x = transform.transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0));
    assert_eq!((origin.x, origin.y), (110.0, 110.000105));
    assert_eq!((x.x, x.y), (111.0, 110.000105));
    let sliced = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(reference.contains("; total layer number: 2\n"));
    assert!(!reference.contains("\nM73 "));
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
