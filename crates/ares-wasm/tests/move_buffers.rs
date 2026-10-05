use ares_core::{
    ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::SliceOutput;

#[test]
fn processed_buffers_preserve_ids_and_owned_data() {
    let vertex = MoveVertex {
        gcode_id: 16777217,
        layer_id: 4,
        internal_only: true,
        move_type: EMoveType::Extrude,
        position: [1.0, 2.0, 3.0],
        time: 7.0,
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(
        ProcessedGCode {
            gcode: b"G1 X1\n".to_vec(),
            result: GCodeProcessorResult {
                moves: vec![MoveVertex::default(), vertex],
                spiral_vase_mode: true,
            },
        },
        &["#FF0000".into()],
        &[],
    );
    assert_eq!(output.take_gcode(), b"G1 X1\n");
    assert!(output.take_gcode().is_empty());
    let mut moves = output.take_moves();
    assert_eq!(moves.count(), 2);
    assert_eq!(moves.take_ids(), [0, 0, 16777217, 4]);
    assert_eq!(&moves.take_values()[12..15], &[1.0, 2.0, 3.0]);
    assert_eq!(&moves.take_tags()[5..], &[10, 0, 0, 0, 1]);
    let mut vertices = output.take_vertices();
    assert_eq!(vertices.count(), 2);
    assert!(vertices.spiral_vase_mode());
    assert_eq!(vertices.take_ids(), [16777217, 4, 16777217, 4]);
    let values = vertices.take_values();
    assert_eq!(&values[..3], &[0.0; 3]);
    assert_eq!(&values[16..19], &[1.0, 2.0, 3.0]);
    assert_eq!(values[10], 0.0);
    assert_eq!(values[26], 7.0);
    assert_eq!(vertices.take_tool_colors(), [255, 48, 48]);
    assert_eq!(vertices.take_color_print_colors(), [255, 48, 48]);
    assert!(output.take_moves().take_ids().is_empty());
}
