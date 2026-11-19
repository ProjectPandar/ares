use std::{
    fs,
    io::{Cursor, Write},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};

use crate::{
    CaseInputs, OrcaRunner,
    presets::VendorProfiles,
    runner::{application, native_api_cli::NativeCase, repo_root},
};

fn project(runner: &OrcaRunner, h2d: bool) -> Vec<u8> {
    let (vendor, machine, process, filament) = if h2d {
        (
            "BBL",
            "Bambu Lab H2D 0.4 nozzle",
            "0.20mm Standard @BBL H2D",
            "Bambu PLA Basic @BBL H2D",
        )
    } else {
        (
            "Snapmaker",
            "Snapmaker U1 (0.4 nozzle)",
            "0.20 Standard @Snapmaker U1 (0.4 nozzle)",
            "Generic PLA @U1 0.4 nozzle",
        )
    };
    let root = repo_root();
    let profiles =
        VendorProfiles::load(&root.join("OrcaSlicer/resources/profiles"), vendor).unwrap();
    let machine = profiles.machine(machine).unwrap();
    let process = profiles.process(process).unwrap();
    let filaments = ["#F2754E", "#0000FF"].map(|colour| {
        let mut profile = profiles.filament(filament).unwrap();
        profile.insert("filament_colour".into(), json!([colour]));
        profile.insert("filament_is_support".into(), json!(["0"]));
        profile
    });
    let inputs = CaseInputs {
        label: "type1-nozzle-acceleration",
        machine: &machine,
        process: &process,
        filaments: &filaments,
    };
    let cube = root.join("tests/parity/cube10.stl");
    let small = root.join("tests/parity/cube08.stl");
    let exported = if h2d {
        runner.export_case(&inputs, &Default::default(), &cube)
    } else {
        runner.export_case_multi(&inputs, &Default::default(), &[&cube, &small], &[1, 2])
    }
    .unwrap();
    let mut changes = json!({
        "enable_prime_tower": "1", "enable_arc_fitting": "0", "wipe_tower_type": "type1",
        "timelapse_type": "0", "farthest_point_timelapse": "0", "detect_wrapping": "0",
        "wipe_tower_sparse_layers_combination": "0", "filament_map_mode": "Manual", "filament_map": ["1", "2"],
    });
    if h2d {
        changes.as_object_mut().unwrap().extend(json!({
            "top_surface_filament_id": "2", "wipe_tower_x": ["105"], "wipe_tower_y": ["215"],
            "nozzle_volume_type": ["Standard", "Standard"], "extruder_type": ["Direct Drive", "Direct Drive"],
        }).as_object().unwrap().clone());
    } else {
        changes["curr_bed_type"] = json!("High Temp Plate");
        changes["wipe_tower_x"] = json!(["20"]);
        changes["wipe_tower_y"] = json!(["220"]);
        changes["initial_layer_acceleration"] = json!(["500", "500", "500", "500"]);
    }
    variant(&exported.project, changes)
}

fn variant(source: &[u8], changes: Value) -> Vec<u8> {
    let mut settings = application::settings(source).unwrap();
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

fn first_diff(native: &str, ares: &str) -> Value {
    let offset = native.bytes().zip(ares.bytes()).position(|(n, a)| n != a);
    let Some(offset) =
        offset.or_else(|| (native.len() != ares.len()).then(|| native.len().min(ares.len())))
    else {
        return Value::Null;
    };
    let line = native.as_bytes()[..offset]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count();
    let start = native.as_bytes()[..offset]
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |i| i + 1);
    json!({"offset": offset, "line": line + 1, "column": offset - start + 1,
        "native": native.split('\n').nth(line), "ares": ares.split('\n').nth(line),
        "native_bytes": native.len(), "ares_bytes": ares.len()})
}

fn before_extrusion<'a>(gcode: &'a str, markers: (&str, &str), prefix: &str) -> Vec<&'a str> {
    let (_, tail) = gcode
        .split_once(markers.0)
        .expect("physical nozzle-change start");
    let (span, _) = tail
        .split_once(markers.1)
        .expect("physical nozzle-change end");
    let mut lines = span.lines();
    let mut accelerations = Vec::new();
    for line in &mut lines {
        if line.starts_with("G1 ")
            && line
                .split_ascii_whitespace()
                .any(|word| word.starts_with('E'))
        {
            return accelerations;
        }
        if line.starts_with(prefix) {
            accelerations.push(line);
        }
    }
    panic!("no old-nozzle extrusion in {span}");
}

fn compare(
    runner: &OrcaRunner,
    project: &[u8],
    label: &str,
    (map, prefix): (&str, &str),
    expected: &str,
) -> (Vec<String>, Vec<String>) {
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
    let api = std::str::from_utf8(&api).unwrap();
    fs::write(
        case.root.join("whole-file-first-diff.json"),
        serde_json::to_vec_pretty(&json!({
            "api": first_diff(&case.native, api), "cli": first_diff(&case.native, &cli),
        }))
        .unwrap(),
    )
    .unwrap();
    let native = before_extrusion(
        &case.native,
        ("; NOZZLE_CHANGE_START ", "; NOZZLE_CHANGE_END"),
        prefix,
    );
    assert_eq!(native, [expected], "Native: {:?}", case.root);
    for text in [&case.native, api, &cli] {
        assert!(
            text.lines().any(|line| line == map),
            "physical map at {:?}",
            case.root
        );
    }
    let extract = |text: &str| {
        before_extrusion(
            text,
            ("; Nozzle change start\n", "; Nozzle change end"),
            prefix,
        )
        .into_iter()
        .map(str::to_owned)
        .collect()
    };
    (extract(api), extract(&cli))
}

#[test]
fn type1_nozzle_change_acceleration_u1() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base = project(&runner, false);
    let faster = variant(
        &base,
        json!({"initial_layer_acceleration": ["777", "777", "777", "777"]}),
    );
    let run = |project: &[u8], label, expected| {
        compare(
            &runner,
            project,
            label,
            ("; filament_nozzle_map = 0,1", "SET_VELOCITY_LIMIT ACCEL="),
            expected,
        )
    };
    let low = "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 MINIMUM_CRUISE_RATIO=0.5";
    let high = "SET_VELOCITY_LIMIT ACCEL=777 ACCEL_TO_DECEL=388.5 MINIMUM_CRUISE_RATIO=0.5";
    let actual = (
        run(&base, "type1-nozzle-u1-500-", low),
        run(&faster, "type1-nozzle-u1-777-", high),
    );
    assert_eq!(
        actual,
        (
            (vec![low.into()], vec![low.into()]),
            (vec![high.into()], vec![high.into()])
        ),
        "U1 first nozzle acceleration (API, CLI) for 500 and 777"
    );
}

#[test]
fn type1_nozzle_change_acceleration_h2d() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let expected = "M204 S8000";
    let actual = compare(
        &runner,
        &project(&runner, true),
        "type1-nozzle-h2d-",
        ("; filament_nozzle_map = 1,0", "M204 "),
        expected,
    );
    assert_eq!(
        actual,
        (vec![expected.into()], vec![expected.into()]),
        "H2D first nozzle acceleration (API, CLI)"
    );
}
