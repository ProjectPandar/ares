//! Public WASM adapter seam: native processor CSV -> packed vertex timing.
//! Failure inventory: phantom/repeated-id conversion changes timing; fields
//! swap; f32 bits change in transport; taking display buffers drains timing.
use ares_core::{
    ExtrusionRole, ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::SliceOutput;

#[test]
fn marker_time_transport_preserves_native_bits_after_buffer_transfer() {
    let moves = include_str!("../../../tests/parity/marker-time/moves.csv")
        .lines()
        .skip(1)
        .map(|line| {
            let fields: Vec<_> = line.split(',').collect();
            let uint = |i: usize| fields[i].parse::<u32>().unwrap();
            let float = |i: usize| fields[i].parse::<f32>().unwrap();
            MoveVertex {
                gcode_id: uint(0),
                move_type: match uint(1) {
                    0 => EMoveType::Noop,
                    3 => EMoveType::Seam,
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
            }
        })
        .collect();
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves,
            extruder_colors: vec!["#F2754E".into()],
            ..Default::default()
        },
    });
    let mut vertices = output.take_vertices();
    vertices.take_values();
    vertices.take_ids();
    vertices.take_tags();
    let records: Vec<_> = include_str!("../../../tests/parity/marker-time/vertices.tsv")
        .lines()
        .collect();
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
