#[path = "initialized_loop_chain/model.rs"]
mod model;

use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::{self as parity, golden, presets::VendorProfiles};
use serde_json::Value;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

/// SHA-256 of the seed-replay executable the seeded reference must report against
/// when the environment does not pin `ARES_ORCA_SEEDED_NATIVE_SHA256`. See
/// `docs/architecture/seeded-oracle-identity-assertion-spec.md`.
const SEEDED_NATIVE_SHA256: &str =
    "84a7780653be3b46fe909e43ae98b3576907c561e8ff337c208ff97c2b23a84a";

/// The reference must come from the seed replay, not from the official release: the
/// launcher has to route through `ORCA_REPLAY_BIN`, it has to be the launcher the
/// environment declares, and the native binary compared against has to be that build.
#[track_caller]
fn assert_seeded_oracle(runner: &OrcaRunner) {
    let launcher = std::fs::read_to_string(runner.bin()).unwrap_or_else(|error| {
        panic!(
            "seed launcher {:?} is not a readable wrapper: {error}",
            runner.bin()
        )
    });
    assert!(
        launcher.contains("setarch -R \"$ORCA_REPLAY_BIN\""),
        "seed launcher {:?} does not run \"$ORCA_REPLAY_BIN\" under setarch",
        runner.bin()
    );
    if let Some(declared) = std::env::var_os("ARES_ORCA_SEEDED_BIN") {
        let declared = PathBuf::from(declared).canonicalize().unwrap();
        assert_eq!(
            runner.bin(),
            &declared,
            "ARES_ORCA_BIN is not the launcher ARES_ORCA_SEEDED_BIN declares"
        );
    }
    let native = std::env::var_os("ARES_ORCA_NATIVE_BIN")
        .expect("the seeded lane must name ARES_ORCA_NATIVE_BIN");
    let expected = std::env::var("ARES_ORCA_SEEDED_NATIVE_SHA256")
        .unwrap_or_else(|_| SEEDED_NATIVE_SHA256.to_owned());
    assert_eq!(
        crate::runner::native_api_cli::sha(&std::fs::read(&native).unwrap()),
        expected,
        "ARES_ORCA_NATIVE_BIN {native:?} is not the seed-replay build"
    );
}

#[test]
fn initialized_loop_chain_matches_complete_output() {
    if !crate::runner::seeded_reference_enabled(Some("ARES_PARITY_LOOP_CHAIN_REFERENCE")) {
        return;
    }
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    std::env::var("ORCA_REPLAY_CPU")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_seeded_oracle(&runner);
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
        // `PrintConfig.cpp:1959,1967`: both keys are `coBools`, so the exported project keeps
        // one value per extruder instead of the scalar the CLI override carries.
        let requested = match key {
            "enable_overhang_speed" | "slowdown_for_curled_perimeters" => {
                Value::Array(vec![Value::String(value.into())])
            }
            _ => Value::String(value.into()),
        };
        application::verify_value(&project, key, &requested).unwrap();
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
