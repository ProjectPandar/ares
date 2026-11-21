use std::{
    fs,
    io::{Cursor, Write},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;

use crate::{
    CaseInputs, OrcaRunner,
    presets::VendorProfiles,
    runner::{application, native_api_cli::NativeCase, repo_root},
};

fn project(runner: &OrcaRunner) -> Vec<u8> {
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let machine = profiles.machine("Bambu Lab H2D 0.4 nozzle").unwrap();
    let process = profiles.process("0.20mm Standard @BBL H2D").unwrap();
    let filaments = ["#F2754E", "#0000FF"].map(|colour| {
        let mut profile = profiles.filament("Bambu PLA Basic @BBL H2D").unwrap();
        profile.insert("filament_colour".into(), json!([colour]));
        profile.insert("filament_is_support".into(), json!(["0"]));
        profile
    });
    let inputs = CaseInputs {
        label: "h2d-nozzle-row-spacing",
        machine: &machine,
        process: &process,
        filaments: &filaments,
    };
    let mut stl = include_bytes!("../../../../../tests/parity/cube10.stl").to_vec();
    for triangle in stl[84..].as_chunks_mut::<50>().0 {
        for vertex in triangle[12..48].as_chunks_mut::<12>().0 {
            for (axis, scale) in vertex
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip([0.2, 0.2, 0.04])
            {
                let value = f32::from_le_bytes(*axis) * scale;
                *axis = value.to_le_bytes();
            }
        }
    }
    let scratch = tempfile::tempdir_in(crate::artifacts::root_from_env().unwrap()).unwrap();
    let model = scratch.path().join("cuboid.stl");
    fs::write(&model, stl).unwrap();
    runner
        .export_case(&inputs, &Default::default(), &model)
        .unwrap()
        .project
}

fn variant(source: &[u8], gap: &str) -> Vec<u8> {
    let mut settings = application::settings(source).unwrap();
    let changes = json!({
        "enable_prime_tower": "1", "enable_arc_fitting": "0", "wipe_tower_wall_type": "rib",
        "prime_tower_infill_gap": gap, "wipe_tower_x": ["105"], "wipe_tower_y": ["215"],
        "timelapse_type": "0", "farthest_point_timelapse": "0", "detect_wrapping": "0",
        "wipe_tower_sparse_layers_combination": "0", "filament_map_mode": "Manual", "filament_map": ["1", "2"],
        "top_surface_filament_id": "2", "nozzle_volume_type": ["Standard", "Standard"],
        "extruder_type": ["Direct Drive", "Direct Drive"],
    });
    settings.extend(changes.as_object().unwrap().clone());
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let entry = archive.by_index(i).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}

fn connector_lengths(gcode: &str, (start, end): (&str, &str)) -> Vec<i32> {
    assert!(
        gcode
            .lines()
            .any(|line| line == "; filament_nozzle_map = 1,0")
    );
    let (before, tail) = gcode.split_once(start).expect("nozzle-change start");
    let (span, after) = tail.split_once(end).expect("nozzle-change end");
    assert!(!after.contains(start) && after.lines().any(|line| line == "T1"));
    let layers: Vec<_> = before
        .lines()
        .filter_map(|l| l.strip_prefix("; Z_HEIGHT: "))
        .collect();
    assert_eq!(layers, ["0.2", "0.4"]);
    let mut position = None;
    let mut rows = Vec::new();
    let mut connectors = Vec::new();
    for line in span.lines().filter(|line| line.starts_with("G1 ")) {
        let motion = line.split(';').next().unwrap();
        let get = |axis| {
            motion.split_ascii_whitespace().find_map(|word| {
                word.strip_prefix(axis)
                    .map(|value| value.parse::<f64>().unwrap())
            })
        };
        let (x, y) = (get('X'), get('Y'));
        if get('E').unwrap_or_default() <= 0.0 || (x.is_none() && y.is_none()) {
            continue;
        }
        // G-code XY is in millimetres with at most three decimals.
        let micron = |mm: f64| (mm * 1000.0).round() as i32;
        let (x, y) = (x.map(micron), y.map(micron));
        if let Some((last_x, last_y)) = position {
            let (x, y) = (x.unwrap_or(last_x), y.unwrap_or(last_y));
            if x == last_x {
                assert_ne!(y, last_y);
                connectors.push((y - last_y).abs());
            } else {
                assert_eq!(y, last_y);
                rows.push(y);
            }
            position = Some((x, y));
        } else {
            rows.push(y.unwrap());
            position = Some((x.unwrap(), y.unwrap()));
        }
    }
    assert_eq!(
        rows.windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .collect::<Vec<_>>(),
        connectors
    );
    assert_eq!(rows.len(), connectors.len() + 1);
    connectors
}

fn pitches(runner: &OrcaRunner, project: &[u8], label: &str) -> [Vec<i32>; 3] {
    let case = NativeCase::slice(runner, project, label);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(case.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(case.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "CLI at {:?}: {:?}",
        case.root,
        cli.status
    );
    let cli = fs::read_to_string(cli_path).unwrap();
    let ares_markers = ("; Nozzle change start\n", "; Nozzle change end");
    let actual = [
        connector_lengths(
            &case.native,
            (
                "; NOZZLE_CHANGE_START OF0 NF1 ON1 NN0\n",
                "; NOZZLE_CHANGE_END OF0 NF1 ON1 NN0",
            ),
        ),
        connector_lengths(std::str::from_utf8(&api).unwrap(), ares_markers),
        connector_lengths(&cli, ares_markers),
    ];
    fs::write(
        case.root.join("nozzle-row-spacing.json"),
        serde_json::to_vec_pretty(&json!({
            "native_um": actual[0], "api_um": actual[1], "cli_um": actual[2],
        }))
        .unwrap(),
    )
    .unwrap();
    actual
}

#[test]
fn type1_later_nozzle_change_rows_match_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = project(&runner);
    let spaced = pitches(&runner, &variant(&source, "150%"), "h2d-nozzle-gap150-");
    let dense = pitches(&runner, &variant(&source, "100%"), "h2d-nozzle-gap100-");
    for (actual, expected) in [(spaced, vec![1250; 2]), (dense, vec![1000; 3])] {
        let [native, api, cli] = actual;
        assert_eq!(native, expected, "Native nozzle-change connectors in µm");
        assert_eq!(
            [api, cli],
            [native.clone(), native],
            "API and CLI nozzle-change connectors in µm"
        );
    }
}
