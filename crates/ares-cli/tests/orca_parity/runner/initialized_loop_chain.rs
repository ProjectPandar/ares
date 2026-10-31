#[path = "initialized_loop_chain/model.rs"]
mod model;

use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, golden, presets::VendorProfiles};
use serde_json::Value;
use std::io::{Cursor, Read, Write};

#[test]
fn initialized_loop_chain_matches_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_LOOP_CHAIN_REFERENCE") {
        return;
    }
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    std::env::var("ORCA_REPLAY_CPU")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(
        runner.bin(),
        &repo_root()
            .join("scripts/orca-seeded-parity.sh")
            .canonicalize()
            .unwrap()
    );
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("three-posts-on-plate.stl");
    std::fs::write(&model, model::plate()).unwrap();
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
        ("wall_generator", "classic"),
        ("wall_loops", "2"),
        ("only_one_wall_top", "1"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("sparse_infill_density", "0%"),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
        ("enable_overhang_speed", "0"),
        ("slowdown_for_curled_perimeters", "0"),
        ("outer_wall_line_width", "0.42"),
        ("inner_wall_line_width", "0.45"),
        ("enable_arc_fitting", "0"),
        ("skirt_loops", "0"),
        ("seam_position", "nearest"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), Value::String(value.into()));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    for filament in &mut filaments {
        filament.insert("slow_down_layer_time".into(), Value::String("0".into()));
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "initialized-loop-chain/three-posts/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
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
            let build = xml.find("<build").unwrap();
            let start = build + xml[build..].find("transform=\"").unwrap() + "transform=\"".len();
            let end = start + xml[start..].find('"').unwrap();
            xml.replace_range(start..end, "1 0 0 0 1 0 0 0 1 110 110 0");
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
    let project_path = runner.work.join("loop-chain.3mf");
    std::fs::write(&project_path, &project).unwrap();
    for (key, value) in settings {
        application::verify_value(&project, key, &Value::String(value.into())).unwrap();
    }
    assert_eq!(
        application::settings(&project).unwrap()["slow_down_layer_time"],
        serde_json::json!(["0"])
    );
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let transform = loaded.objects()[0].instances()[0].transform();
    let origin = transform.transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
    let x = transform.transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0));
    assert_eq!((origin.x, origin.y), (110.0, 110.0));
    assert_eq!((x.x, x.y), (111.0, 110.0));
    let mut references = Vec::new();
    for index in 0..2 {
        let output_dir = runner.work.join(format!("loop-chain-{index}"));
        std::fs::create_dir_all(&output_dir).unwrap();
        let exported = ExportedCase {
            label: format!("initialized-loop-chain/three-posts/reference-{index}"),
            project: project.clone(),
            project_path: project_path.clone(),
            output_dir,
        };
        let sliced = runner.slice_case(&exported).unwrap();
        let text = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(text.contains("; total layer number: 2\n"));
        assert!(text.contains("; only_one_wall_top = 1\n"));
        assert!(!text.contains("\nM73 "));
        references.push(sliced);
    }
    assert_eq!(
        golden::normalize_one_generator_line(&references[0].reference, golden::GeneratorKind::Orca)
            .unwrap(),
        golden::normalize_one_generator_line(&references[1].reference, golden::GeneratorKind::Orca)
            .unwrap(),
        "independent reference replays must be stable",
    );
    let outcomes = references
        .iter()
        .map(parity::compare_case)
        .collect::<Vec<_>>();
    for outcome in outcomes {
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}
