//! BuildVolume.cpp:377-421 and Print.cpp:833-930,1354-1396.
//! Fixed scene-EPS and tall-last witnesses; no new fixture pipeline.

use crate::runner::{self, CaseInputs, OrcaRunner};
use serde_json::json;

#[test]
fn printable_height_scene_and_tall_last() {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let model = runner::repo_root().join("tests/parity/cube10.stl");
    let root = crate::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    for (wire, code) in [("10", 0), ("10.0002", 0), ("9.99995", 0), ("9.9998", -50)] {
        let mut machine = machine.clone();
        machine.insert("printable_height".to_owned(), json!(wire));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("printable-height/scene/{wire}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &model,
            )
            .unwrap();
        let loaded = ares_core::load_project(&exported.project).unwrap();
        assert_eq!(loaded.objects().len(), 1);
        let object = &loaded.objects()[0];
        assert_eq!(object.instances().len(), 1);
        assert_eq!(object.volumes().len(), 1);
        let volume = &object.volumes()[0];
        let matrix = object.instances()[0].transform().then(volume.transform());
        let points = volume
            .mesh()
            .vertices()
            .iter()
            .map(|p| matrix.transform_point(*p))
            .collect::<Vec<_>>();
        let min_z = points.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
        let max_z = points.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
        assert_eq!([min_z, max_z], [0., 10.]);
        proofs.push(compare(&runner, &exported.project, "scene", wire, code));
    }
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: "printable-height/g2-tall-last-base",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &[&model, &model],
            &[],
        )
        .unwrap();
    for (wire, code) in [("20", 0), ("19.99995", -63)] {
        let project = super::super::clearance::geometry::scene(
            &exported.project,
            "printable_height",
            wire,
            [(60., 60.), (60., 90.)],
            [10., 20.],
        );
        for (key, value) in [
            ("print_sequence", "by object"),
            ("extruder_clearance_radius", "10"),
            ("extruder_clearance_height_to_rod", "40"),
            ("extruder_clearance_height_to_lid", "10"),
            ("nozzle_height", "2.5"),
        ] {
            runner::application::verify_value(&project, key, &json!(value)).unwrap();
        }
        proofs.push(compare(&runner, &project, "g2-tall-last", wire, code));
    }
    crate::artifacts::overwrite(
        &root.join("printable-height-scene-tall-last.json"),
        &serde_json::to_vec_pretty(&proofs).unwrap(),
    )
    .unwrap();
}

fn compare(
    runner: &OrcaRunner,
    project: &[u8],
    scope: &str,
    wire: &str,
    code: i32,
) -> serde_json::Value {
    runner::application::verify_value(project, "printable_height", &json!(wire)).unwrap();
    runner::application::verify_value(project, "nozzle_diameter", &json!(["0.4"])).unwrap();
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("printable-height-consumer-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    crate::artifacts::write(&input, project).unwrap();
    let native = match code {
        0 => runner.sequential_clearance_reference(&input, true),
        -63 => runner.sequential_clearance_reference(&input, false),
        -50 => runner.early_cli_reference(&input, runner::EarlyExit::EmptyPlate),
        _ => unreachable!(),
    }
    .unwrap();
    for (name, bytes) in [
        ("native.stdout", &native.stdout),
        ("native.stderr", &native.stderr),
        ("native.result.json", &native.result_json),
    ] {
        crate::artifacts::write(&directory.join(name), bytes).unwrap();
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let public = match &actual {
        Ok(gcode) => {
            crate::artifacts::write(&directory.join("ares.validated.gcode"), gcode).unwrap();
            json!({"accepted_bytes":gcode.len()})
        }
        Err(error) => json!({"error":format!("{error:?}")}),
    };
    let output = directory.join("plate_1.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    for (name, bytes) in [("ares.stdout", &cli.stdout), ("ares.stderr", &cli.stderr)] {
        crate::artifacts::write(&directory.join(name), bytes).unwrap();
    }
    let proof = json!({"scope":scope,"wire":wire,"Native_code":code,"Ares_public":public,"Ares_exit":cli.status.code(),"input":input,"Native_receipt":native.directory.join("receipt.json"),"artifacts":directory});
    crate::artifacts::write(
        &directory.join("proof.json"),
        &serde_json::to_vec_pretty(&proof).unwrap(),
    )
    .unwrap();
    if code == 0 {
        let actual = actual.unwrap();
        let reference = native.gcode.as_ref().unwrap();
        crate::artifacts::write(&directory.join("native.gcode"), reference).unwrap();
        crate::golden::compare_ordered_bytes_generator_only(reference, &actual).unwrap();
        assert!(cli.status.success());
        crate::golden::compare_ordered_bytes_generator_only(
            reference,
            &std::fs::read(&output).unwrap(),
        )
        .unwrap();
    } else {
        let error = actual.unwrap_err();
        match (code, error) {
            (-50, ares_core::SliceError::NoSuitableObjects(_)) => {}
            (-63, ares_core::SliceError::SequentialPrintCollision { message, .. }) => {
                assert_eq!(native.stderr, format!("{message}\n").as_bytes());
            }
            (_, error) => panic!("{scope}/{wire}: wrong public rejection {error:?}"),
        }
        assert_eq!(
            cli.status.code(),
            Some(if cfg!(unix) { (code + 256) % 256 } else { code })
        );
        assert_eq!(cli.stdout, native.stdout);
        assert_eq!(cli.stderr, native.stderr);
        assert_eq!(
            crate::native_wall_clock::result_comparison_bytes(
                &std::fs::read(directory.join("result.json")).unwrap()
            ),
            crate::native_wall_clock::result_comparison_bytes(&native.result_json)
        );
        assert!(!output.exists());
    }
    proof
}
