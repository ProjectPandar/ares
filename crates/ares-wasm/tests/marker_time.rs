//! Public WASM adapter seam: native processor CSV -> packed vertex timing.
//! Failure inventory: phantom/repeated-id conversion changes timing; fields
//! swap; f32 bits change in transport; taking display buffers drains timing.
use ares_core::{
    ExtrusionRole, ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::SliceOutput;

#[path = "../../../tests/support/marker_time.rs"]
pub mod marker_time_native;

#[test]
fn required_missing_native_cannot_pass_from_frozen_transport() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "marker_time_transport_preserves_native_bits_after_buffer_transfer",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_MARKER_TIME_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(
        !output.status.success(),
        "transport used frozen authority without Native"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_MARKER_TIME_BIN")
    );
}

#[test]
fn marker_time_transport_preserves_native_bits_after_buffer_transfer() {
    let Some(root) = marker_time_native::raw_fixture() else {
        return;
    };
    let moves_csv = std::fs::read_to_string(root.join("moves.csv")).unwrap();
    let dynamics_csv = std::fs::read_to_string(root.join("dynamics.csv")).unwrap();
    let dynamics: Vec<_> = dynamics_csv.lines().skip(1).collect();
    let moves = moves_csv
        .lines()
        .skip(1)
        .enumerate()
        .map(|(i, line)| {
            let fields: Vec<_> = line.split(',').collect();
            let uint = |i: usize| fields[i].parse::<u32>().unwrap();
            let float = |i: usize| fields[i].parse::<f32>().unwrap();
            // dynamics.csv rows are the same result.moves in the same order
            // (--dump-move-dynamics): per-move acceleration and jerk.
            let dynamics_fields: Vec<_> = dynamics[i].split(',').collect();
            MoveVertex {
                gcode_id: uint(0),
                move_type: match uint(1) {
                    0 => EMoveType::Noop,
                    3 => EMoveType::Seam,
                    4 => EMoveType::ToolChange,
                    8 => EMoveType::Travel,
                    10 => EMoveType::Extrude,
                    other => panic!("unexpected fixture type {other}"),
                },
                extrusion_role: match uint(2) {
                    0 => ExtrusionRole::None,
                    2 => ExtrusionRole::ExternalPerimeter,
                    4 => ExtrusionRole::InternalInfill,
                    other => panic!("unexpected fixture role {other}"),
                },
                extruder_id: uint(3) as u8,
                cp_color_id: uint(4) as u8,
                position: [float(5), float(6), float(7)],
                delta_extruder: float(8),
                feedrate: float(9),
                width: float(10),
                height: float(11),
                mm3_per_mm: float(12),
                travel_dist: float(13),
                fan_speed: float(14),
                temperature: float(15),
                time: float(16),
                layer_id: uint(17),
                internal_only: uint(18) != 0,
                acceleration: dynamics_fields[4].parse::<f32>().unwrap(),
                jerk: dynamics_fields[5].parse::<f32>().unwrap(),
                ..MoveVertex::default()
            }
        })
        .collect();
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves,
            extruder_colors: vec!["#F2754E".into()],
            ..Default::default()
        },
        print_statistics: Default::default(),
    });
    let vertices = output.take_vertices();
    vertices.values_range(0, vertices.count() as u32).unwrap();
    vertices.ids_range(0, vertices.count() as u32).unwrap();
    vertices.tags_range(0, vertices.count() as u32).unwrap();
    let records_tsv = std::fs::read_to_string(root.join("vertices.tsv")).unwrap();
    let records: Vec<_> = records_tsv.lines().collect();
    assert_eq!(vertices.count(), records.len());
    for line in records.into_iter().rev() {
        let fields: Vec<_> = line.split('\t').collect();
        let uint = |i: usize| fields[i].parse::<u32>().unwrap();
        let time = vertices.marker_time(uint(0)).unwrap();
        assert_eq!(
            time.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
            [uint(10), uint(8)]
        );
    }
}
