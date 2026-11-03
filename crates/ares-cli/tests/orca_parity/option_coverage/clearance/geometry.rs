//! Print.cpp:613-930: fixed two-object clearance consumer witnesses.

use crate::runner::{self, CaseInputs, OrcaRunner};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[test]
fn g2_sequential_consumer_boundaries() {
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
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: "g2/sequential-base",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &[&model, &model],
            &[],
        )
        .unwrap();
    let root = crate::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    for (key, wire, centers, heights, accepted) in [
        (
            "extruder_clearance_height_to_lid",
            "10",
            [(60., 60.), (60., 90.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_height_to_lid",
            "10.00001",
            [(60., 60.), (60., 90.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_height_to_lid",
            "9.99999",
            [(60., 60.), (60., 90.)],
            [10., 10.],
            false,
        ),
        (
            "extruder_clearance_height_to_rod",
            "10",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_height_to_rod",
            "10.00001",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_height_to_rod",
            "9.99999",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            false,
        ),
        (
            "extruder_clearance_radius",
            "20.19999",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_radius",
            "20.2",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            true,
        ),
        (
            "extruder_clearance_radius",
            "20.20001",
            [(60., 60.), (90., 60.)],
            [10., 10.],
            false,
        ),
        (
            "nozzle_height",
            "10.00001",
            [(60., 60.), (75., 60.)],
            [10., 10.],
            true,
        ),
        (
            "nozzle_height",
            "10",
            [(60., 60.), (75., 60.)],
            [10., 10.],
            false,
        ),
        (
            "nozzle_height",
            "9.99999",
            [(60., 60.), (75., 60.)],
            [10., 10.],
            false,
        ),
        (
            "extruder_clearance_height_to_lid",
            "10",
            [(60., 60.), (60., 90.)],
            [10., 20.],
            true,
        ),
        (
            "extruder_clearance_height_to_lid",
            "10",
            [(60., 60.), (60., 90.)],
            [20., 10.],
            false,
        ),
    ] {
        let label = format!("g2/geometry/{key}/{wire}/{heights:?}");
        let directory = tempfile::Builder::new()
            .prefix("g2-geometry-")
            .tempdir_in(&root)
            .unwrap()
            .keep();
        let project = scene(&exported.project, key, wire, centers, heights);
        let path = directory.join("input.3mf");
        crate::artifacts::write(&path, &project).unwrap();
        let native = runner
            .sequential_clearance_reference(&path, accepted)
            .unwrap();
        let result: serde_json::Value = serde_json::from_slice(&native.result_json).unwrap();
        assert_eq!(result["return_code"], if accepted { 0 } else { -63 });
        crate::artifacts::write(&directory.join("native.stdout"), &native.stdout).unwrap();
        crate::artifacts::write(&directory.join("native.stderr"), &native.stderr).unwrap();
        crate::artifacts::write(&directory.join("native.result.json"), &native.result_json)
            .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let actual = runtime.block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
        if accepted {
            let actual = actual.unwrap();
            crate::artifacts::write(&directory.join("ares.gcode"), &actual).unwrap();
            let reference = native.gcode.as_ref().unwrap();
            crate::artifacts::write(&directory.join("native.gcode"), reference).unwrap();
            crate::golden::compare_ordered_bytes_generator_only(reference, &actual)
                .unwrap_or_else(|difference| panic!("{label}: {difference}"));
        } else {
            let error = actual.unwrap_err();
            crate::artifacts::write(
                &directory.join("ares.validation.txt"),
                format!("{error:?}\n").as_bytes(),
            )
            .unwrap();
            let ares_core::SliceError::SequentialPrintCollision { message, .. } = error else {
                panic!("{label}: wrong public rejection {error:?}");
            };
            assert_eq!(native.stderr, format!("{message}\n").as_bytes(), "{label}");
            assert!(native.gcode.is_none());
            let gcode = directory.join("plate_1.gcode");
            let cli = crate::runner::application::cli_command()
                .args(["slice", "-o"])
                .arg(&gcode)
                .arg(&path)
                .output()
                .unwrap();
            crate::artifacts::write(&directory.join("ares.stdout"), &cli.stdout).unwrap();
            crate::artifacts::write(&directory.join("ares.stderr"), &cli.stderr).unwrap();
            crate::artifacts::write(
                &directory.join("ares.exit"),
                format!("{:?}\n", cli.status.code()).as_bytes(),
            )
            .unwrap();
            assert_eq!(cli.status.code(), Some(if cfg!(unix) { 193 } else { -63 }));
            assert_eq!(cli.stdout, native.stdout);
            assert_eq!(cli.stderr, native.stderr);
            assert_eq!(
                crate::native_wall_clock::result_comparison_bytes(
                    &std::fs::read(directory.join("result.json")).unwrap()
                ),
                crate::native_wall_clock::result_comparison_bytes(&native.result_json)
            );
            assert!(!gcode.exists());
        }
        proofs.push(json!({"label":label,"key":key,"requested":wire,"centers":centers,"heights":heights,"accepted":accepted,"Native_receipt":native.directory.join("receipt.json"),"input":path}));
    }
    crate::artifacts::overwrite(
        &root.join("g2-sequential-consumer-proof.json"),
        &serde_json::to_vec_pretty(&proofs).unwrap(),
    )
    .unwrap();
}

pub(in crate::option_coverage) fn scene(
    input: &[u8],
    key: &str,
    wire: &str,
    centers: [(f64, f64); 2],
    heights: [f64; 2],
) -> Vec<u8> {
    let loaded = ares_core::load_project(input).unwrap();
    assert_eq!(loaded.objects().len(), 2);
    let transforms = loaded
        .objects()
        .iter()
        .enumerate()
        .map(|(index, object)| {
            let volume = &object.volumes()[0];
            let points = volume
                .mesh()
                .vertices()
                .iter()
                .map(|point| volume.transform().transform_point(*point))
                .collect::<Vec<_>>();
            let min = [0, 1, 2].map(|axis| {
                points
                    .iter()
                    .map(|p| [p.x, p.y, p.z][axis])
                    .fold(f64::INFINITY, f64::min)
            });
            let max = [0, 1, 2].map(|axis| {
                points
                    .iter()
                    .map(|p| [p.x, p.y, p.z][axis])
                    .fold(f64::NEG_INFINITY, f64::max)
            });
            assert_eq!(
                [max[0] - min[0], max[1] - min[1], max[2] - min[2]],
                [10., 10., 10.]
            );
            let scale = heights[index] / 10.;
            format!(
                "1 0 0 0 1 0 0 0 {scale} {} {} {}",
                centers[index].0 - (min[0] + max[0]) / 2.,
                centers[index].1 - (min[1] + max[1]) / 2.,
                -min[2] * scale
            )
        })
        .collect::<Vec<_>>();
    let mut input = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let attribute = regex::Regex::new(r#"transform="[^"]*""#).unwrap();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in [
                ("print_sequence", "by object"),
                ("timelapse_type", "0"),
                ("enable_wrapping_detection", "0"),
                ("skirt_loops", "0"),
                ("draft_shield", "disabled"),
                ("enable_prime_tower", "0"),
                ("raft_layers", "0"),
                ("enable_support", "0"),
                ("disable_m73", "1"),
                (
                    "extruder_clearance_radius",
                    if key == "nozzle_height" { "20" } else { "10" },
                ),
                ("extruder_clearance_height_to_rod", "40"),
                (
                    "extruder_clearance_height_to_lid",
                    if key == "printable_height" {
                        "10"
                    } else {
                        "120"
                    },
                ),
                ("nozzle_height", "2.5"),
            ] {
                settings[key] = json!(value);
            }
            settings[key] = json!(wire);
            bytes = serde_json::to_vec(&settings).unwrap();
        } else if file.name() == "3D/3dmodel.model" {
            let mut xml = String::from_utf8(bytes).unwrap();
            let start = xml.find("<build").unwrap();
            let end = start + xml[start..].find("</build>").unwrap();
            let mut item = 0;
            let build = attribute
                .replace_all(&xml[start..end], |_: &regex::Captures<'_>| {
                    let transform = format!("transform=\"{}\"", transforms[item]);
                    item += 1;
                    transform
                })
                .into_owned();
            assert_eq!(item, 2);
            xml.replace_range(start..end, &build);
            bytes = xml.into_bytes();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let result = output.finish().unwrap().into_inner();
    let loaded = ares_core::load_project(&result).unwrap();
    for (index, object) in loaded.objects().iter().enumerate() {
        assert_eq!(object.instances().len(), 1);
        let volume = &object.volumes()[0];
        let transform = object.instances()[0].transform().then(volume.transform());
        let points = volume
            .mesh()
            .vertices()
            .iter()
            .map(|point| transform.transform_point(*point))
            .collect::<Vec<_>>();
        let min = [0, 1, 2].map(|axis| {
            points
                .iter()
                .map(|p| [p.x, p.y, p.z][axis])
                .fold(f64::INFINITY, f64::min)
        });
        let max = [0, 1, 2].map(|axis| {
            points
                .iter()
                .map(|p| [p.x, p.y, p.z][axis])
                .fold(f64::NEG_INFINITY, f64::max)
        });
        assert_eq!(
            [
                (min[0] + max[0]) / 2.,
                (min[1] + max[1]) / 2.,
                min[2],
                max[2]
            ],
            [centers[index].0, centers[index].1, 0., heights[index]]
        );
    }
    runner::application::verify_value(&result, "nozzle_diameter", &json!(["0.4"])).unwrap();
    result
}
