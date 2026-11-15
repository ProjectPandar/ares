//! `GCodeProcessor::TimeMachine::calculate_time:630-646`: Normal block duration,
//! observed from pinned Native public streaming moves and real libvgcode conversion.
use crate::runner::{OrcaRunner, cache_identity, native_api_cli::NativeCase};
use ares_core::gcode_processor::{EMoveType, MoveVertex};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, process::Command};

const UPSTREAM: &str = "345efec64c7b95a05ce5cf96df0dde66a158835c";
const INPUT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/preheat-normal-move-durations/x2d-two-prisms.3mf"
));

#[test]
fn preheat_normal_move_durations_from_x2d_geometry() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let Some(probe) = crate::native_environment::binary("ARES_PREHEAT_MOVE_DURATIONS_BIN") else {
        return;
    };
    assert_eq!(
        cache_identity::raw_digest(INPUT),
        "3af196464a2e8315bdb9d84c6fdaa1eaf8062f1104bc5a60a940203d334ac87f"
    );
    let case = NativeCase::slice(&runner, INPUT, "preheat-normal-move-durations-");
    let root = &case.root;
    let native_path = root.join("native.json");
    let config_path = root.join("native-applied-config.json");
    let command = [
        "--preheat-duration-oracle".to_owned(),
        root.join("native.gcode").to_str().unwrap().to_owned(),
        native_path.to_str().unwrap().to_owned(),
        config_path.to_str().unwrap().to_owned(),
    ];
    let output = Command::new(&probe).args(&command).output().unwrap();
    fs::write(root.join("probe.stdout"), &output.stdout).unwrap();
    fs::write(root.join("probe.stderr"), &output.stderr).unwrap();
    fs::write(
        root.join("probe-command.json"),
        serde_json::to_vec_pretty(&json!({
            "binary": probe, "binary_sha256": cache_identity::raw_digest(&fs::read(&probe).unwrap()),
            "args": command, "exit": output.status.code(), "upstream": UPSTREAM,
            "input_sha256": cache_identity::raw_digest(INPUT),
            "gcode_sha256": cache_identity::raw_digest(case.native.as_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(output.status.success(), "Native probe at {root:?}");
    assert_eq!(
        fs::read(root.join("native.gcode")).unwrap(),
        case.native.as_bytes(),
        "Native read-only processing at {root:?}"
    );
    let native: Value = serde_json::from_slice(&fs::read(native_path).unwrap()).unwrap();
    assert_eq!(native["upstream"], UPSTREAM);
    let nozzles = native["nozzles"].as_array().unwrap();
    assert_eq!(nozzles.len(), 2, "Native grouping at {root:?}");
    assert_eq!(nozzles[0]["filament"], 0);
    assert_eq!(nozzles[0]["extruder_id"], 0);
    assert_eq!(nozzles[1]["filament"], 1);
    assert_eq!(nozzles[1]["extruder_id"], 1);
    let expected_moves = native["moves"].as_array().unwrap();
    for extruder in [0, 1] {
        assert!(
            expected_moves.iter().any(|v| {
                v["type"] == EMoveType::Extrude as u8
                    && v["role"] == ares_core::ExtrusionRole::Perimeter as u8
                    && v["extruder_id"] == extruder
                    && f32::from_bits(v["normal_time_bits"].as_u64().unwrap() as u32) > 0.0
            }),
            "Native physical head {extruder} must extrude model perimeters with positive Normal duration: {root:?}"
        );
    }
    let layers = expected_moves
        .iter()
        .filter(|v| v["type"] == EMoveType::Extrude as u8)
        .map(|v| v["layer_id"].as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(layers.len() >= 3, "Native layer transitions at {root:?}");
    let config = fs::read(config_path).unwrap();
    let settings: ares_core::ProjectSettings = serde_json::from_slice(&config).unwrap();
    let processed = ares_core::gcode_processor::process_gcode(&case.native, &settings, true, false);
    let moves = processed
        .result
        .moves
        .iter()
        .enumerate()
        .map(|(i, v)| move_row(i, v))
        .collect::<Vec<_>>();
    let converted = ares_vgcode::convert::from_processor_result(&processed.result);
    let vertices = converted
        .vertices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            json!({
                "index": i, "gcode_id": v.gcode_id, "type": v.move_type as u8,
                "role": v.role as u8, "extruder_id": v.extruder_id, "layer_id": v.layer_id,
                "position_bits": v.position.map(f32::to_bits),
                "normal_time_bits": v.times[0].to_bits(), "stealth_time_bits": v.times[1].to_bits(),
            })
        })
        .collect::<Vec<_>>();
    fs::write(root.join("processed.gcode"), &processed.gcode).unwrap();
    fs::write(
        root.join("actual.json"),
        serde_json::to_vec_pretty(&json!({
            "moves": moves, "vertices": vertices,
            "print_statistics": processed.result.print_statistics,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        identities(expected_moves),
        identities(&moves),
        "preceding ordered move-identity boundary at {root:?}; do not weaken or widen the duration slice"
    );
    let expected_vertices = native["vertices"].as_array().unwrap();
    assert_eq!(
        identities(expected_vertices),
        identities(&vertices),
        "preceding real-conversion vertex-identity boundary at {root:?}"
    );
    let duration_bits = |rows: &[Value]| {
        rows.iter()
            .map(|row| row["normal_time_bits"].as_u64().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        duration_bits(expected_moves),
        duration_bits(&moves),
        "every ordered public move's Native Normal duration bits, including zero/derived moves: {root:?}"
    );
    assert_eq!(
        duration_bits(expected_vertices),
        duration_bits(&vertices),
        "every real libvgcode Normal time, including zero-time phantom starts: {root:?}"
    );
    assert!(
        converted.vertices.iter().all(|v| v.times[1].to_bits() == 0),
        "Stealth remains deferred: {root:?}"
    );
}

fn move_row(index: usize, v: &MoveVertex) -> Value {
    json!({
        "index": index, "gcode_id": v.gcode_id, "type": v.move_type as u8,
        "role": v.extrusion_role as u8, "extruder_id": v.extruder_id, "layer_id": v.layer_id,
        "internal_only": v.internal_only, "position_bits": v.position.map(f32::to_bits),
        "normal_time_bits": v.time.to_bits(),
    })
}

fn identities(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            let mut row = row.clone();
            let fields = row.as_object_mut().unwrap();
            fields.remove("normal_time_bits");
            fields.remove("stealth_time_bits");
            row
        })
        .collect()
}
