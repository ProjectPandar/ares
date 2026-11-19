//! Compare the emitted old-nozzle flow on a stock dual-head Type1 tower.
use crate::{
    CaseInputs, OrcaRunner, golden,
    presets::VendorProfiles,
    runner::{application, native_api_cli::NativeCase, repo_root},
};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Map, json};
use std::fs;

fn nozzle_flow(section: &str) -> f64 {
    let (mut x, mut y) = (None, None);
    for line in section.lines() {
        let mut fields = line.split_ascii_whitespace();
        if fields.next() != Some("G1") {
            continue;
        }
        let (old_x, old_y) = (x, y);
        let mut e = 0.0;
        for field in fields.take_while(|field| !field.starts_with(';')) {
            match field.as_bytes()[0] {
                b'X' => x = Some(field[1..].parse::<f64>().unwrap()),
                b'Y' => y = Some(field[1..].parse::<f64>().unwrap()),
                b'E' => e = field[1..].parse::<f64>().unwrap(),
                _ => (),
            }
        }
        if let (Some(x), Some(y), Some(old_x), Some(old_y)) = (x, y, old_x, old_y) {
            let distance = (x - old_x).hypot(y - old_y);
            // Use a long row with known starting XY, not a short turn or the entry move.
            if e > 0.0 && distance > 5.0 {
                return e / distance;
            }
        }
    }
    panic!("no measurable nozzle-change extrusion in {section}");
}

#[test]
fn type1_nozzle_change_flow_uses_stock_x2d_width() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let repo = repo_root();
    let profiles =
        VendorProfiles::load(&repo.join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let selection = crate::select_printer(&profiles, "BBL", "Bambu Lab X2D 0.6 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("curr_bed_type".into(), json!("High Temp Plate"));
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), json!("1"));
    process.insert("wipe_tower_type".into(), json!("type1"));
    // The stock left-edge anchor is outside the second physical head’s reach.
    process.insert("wipe_tower_x".into(), json!("150"));
    process.insert("wipe_tower_y".into(), json!("200"));
    let filaments = [("1", "#F2754E"), ("2", "#0000FF")].map(|(head, colour)| {
        let mut filament = profiles.filament(&selection.filaments[0]).unwrap();
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("filament_map".into(), json!([head]));
        filament
    });
    let model = repo.join("tests/parity/cube10.stl");
    let exported = runner
        .export_case_multi(
            &CaseInputs {
                label: "type1-nozzle-change-x2d-06",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &[&model, &model],
            &[1, 2],
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    let settings = application::settings(&exported.project).unwrap();
    assert_eq!(settings["nozzle_diameter"], json!(["0.6", "0.6"]));
    assert_eq!(settings["filament_map"], json!(["1", "2"]));
    assert_eq!(settings["wipe_tower_type"], "type1");
    let case = NativeCase::slice(&runner, &exported.project, "type1-nozzle-change-width-");
    let replay = NativeCase::slice(&runner, &exported.project, "type1-nozzle-change-replay-");
    let normalize_native = |gcode: &str| {
        golden::normalize_one_generator_line(gcode.as_bytes(), golden::GeneratorKind::Orca).unwrap()
    };
    let native = normalize_native(&case.native);
    let reprinted = normalize_native(&replay.native);
    if let Some(diff) = golden::first_difference(&native, &reprinted) {
        panic!(
            "Native replay at {:?} and {:?}: {diff}",
            case.root, replay.root
        );
    }
    let markers = ("\n; NOZZLE_CHANGE_START ", "\n; NOZZLE_CHANGE_END ");
    let (_, tail) = case
        .native
        .split_once(markers.0)
        .expect("Native nozzle change");
    let (tag, rest) = tail.split_once('\n').unwrap();
    assert!(
        tag.ends_with(" ON0 NN1") || tag.ends_with(" ON1 NN0"),
        "physical head change: {tag}"
    );
    let expected = nozzle_flow(rest.split_once(markers.1).unwrap().0);

    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &exported.project,
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
    assert!(
        cli.status.success(),
        "CLI at {:?}: {}",
        case.root,
        String::from_utf8_lossy(&cli.stderr)
    );
    assert!(
        api == fs::read(cli_path).unwrap(),
        "API/CLI byte mismatch at {:?}",
        case.root
    );
    let (_, tail) = std::str::from_utf8(&api)
        .unwrap()
        .split_once(markers.0)
        .expect("Ares nozzle change");
    let (_, rest) = tail.split_once('\n').unwrap();
    let observed = nozzle_flow(rest.split_once(markers.1).unwrap().0);
    fs::write(
        case.root.join("nozzle-flow.json"),
        serde_json::to_vec_pretty(
            &json!({"native": expected, "ares": observed, "head_change": tag}),
        )
        .unwrap(),
    )
    .unwrap();
    // XY and E are rounded independently in G-code; compare E per mm within that precision.
    assert!(
        (expected - observed).abs() < 1e-4,
        "at {:?}: nozzle-change E/mm Native={expected:.7} Ares={observed:.7}",
        case.root
    );
}
