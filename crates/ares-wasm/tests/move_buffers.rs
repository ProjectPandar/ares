use ares_core::{
    ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::SliceOutput;

#[test]
fn processed_buffers_preserve_ids_and_owned_data() {
    let vertex = MoveVertex {
        gcode_id: 16777217,
        // libvgcode `Layers::update` requires layer ids to start at 0.
        layer_id: 0,
        internal_only: true,
        move_type: EMoveType::Extrude,
        position: [1.0, 2.0, 3.0],
        time: 7.0,
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: b"G1 X1\n".to_vec(),
        result: GCodeProcessorResult {
            moves: vec![MoveVertex::default(), vertex].into(),
            spiral_vase_mode: true,
            extruder_colors: vec!["#FF0000".into()],
            ..Default::default()
        },
        print_statistics: Default::default(),
    });
    assert_eq!(output.take_gcode(), b"G1 X1\n");
    assert!(output.take_gcode().is_empty());
    let mut moves = output.take_moves();
    assert_eq!(moves.count(), 2);
    assert_eq!(moves.take_ids(), [0, 0, 16777217, 0]);
    assert_eq!(&moves.take_values()[12..15], &[1.0, 2.0, 3.0]);
    assert_eq!(&moves.take_tags()[5..], &[10, 0, 0, 0, 1]);
    let mut vertices = output.take_vertices();
    let legend: serde_json::Value =
        serde_json::from_str(&vertices.filament_legend().unwrap()).unwrap();
    assert_eq!(legend["colorPrint"]["visible"], false);
    assert_eq!(legend["colorPrint"]["partialUsage"]["status"], "available");
    assert_eq!(
        legend["colorPrint"]["extruders"][0]["segments"][0]["color"],
        serde_json::json!([255, 48, 48])
    );
    assert_eq!(vertices.count(), 2);
    assert!(vertices.spiral_vase_mode());
    assert_eq!(
        vertices.ids_range(0, vertices.count() as u32).unwrap(),
        [16777217, 0, 16777217, 0]
    );
    let values = vertices.values_range(0, vertices.count() as u32).unwrap();
    assert_eq!(&values[..3], &[0.0; 3]);
    assert_eq!(&values[16..19], &[1.0, 2.0, 3.0]);
    assert_eq!(values[10], 0.0);
    assert_eq!(values[26], 7.0);
    assert_eq!(vertices.take_tool_colors(), [255, 48, 48]);
    assert_eq!(vertices.take_color_print_colors(), [255, 48, 48]);
    assert!(output.take_moves().take_ids().is_empty());
}

#[test]
fn vertex_buffers_carry_libvgcode_layer_zs() {
    let at = |move_type, layer_id, z| MoveVertex {
        move_type,
        layer_id,
        position: [0.0, 0.0, z],
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves: vec![
                MoveVertex::default(),
                at(EMoveType::Travel, 0, 5.0),
                at(EMoveType::Extrude, 0, 0.2),
                at(EMoveType::Travel, 1, 0.6),
                at(EMoveType::Extrude, 1, 0.4),
                at(EMoveType::Travel, 2, 0.8),
            ]
            .into(),
            spiral_vase_mode: false,
            ..Default::default()
        },
        print_statistics: Default::default(),
    });
    let mut vertices = output.take_vertices();
    assert_eq!(
        vertices.ids_range(0, vertices.count() as u32).unwrap()[1..]
            .iter()
            .step_by(2)
            .copied()
            .collect::<Vec<_>>(),
        [0, 0, 0, 0, 1, 1, 1, 1, 2, 2]
    );
    // Layers::update: the z of the layer's last non-custom extrusion, 0 without one.
    assert_eq!(vertices.take_layer_zs(), [0.2, 0.4, 0.0]);
}

#[test]
fn vertex_buffers_view_range_follows_the_layer_range() {
    let at = |move_type, layer_id| MoveVertex {
        move_type,
        layer_id,
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves: vec![
                MoveVertex::default(),
                at(EMoveType::Travel, 0),
                at(EMoveType::Extrude, 0),
                at(EMoveType::Travel, 1),
                at(EMoveType::Extrude, 1),
                at(EMoveType::Travel, 2),
            ]
            .into(),
            spiral_vase_mode: false,
            ..Default::default()
        },
        print_statistics: Default::default(),
    });
    let mut vertices = output.take_vertices();
    let tags = vertices.tags_range(0, vertices.count() as u32).unwrap();
    let ids = vertices.ids_range(0, vertices.count() as u32).unwrap();
    let all_roles = u32::MAX;
    let (travels, seams) = (1 << 0, 1 << 4);
    for (layers, current_layer) in [([0, 0], 0), ([0, 1], 1)] {
        let [first, last, current] =
            vertices.view_range(layers[0], layers[1], all_roles, seams)[..]
        else {
            panic!("three indices");
        };
        assert_eq!(last, current);
        assert!(first < last);
        // libvgcode `update_view_full_range`: with travels hidden the range ends
        // at the last extrusion of the top layer.
        let current = current as usize;
        assert_eq!(tags[4 * current + 1], 10, "extrusion at {current}");
        assert_eq!(ids[2 * current + 1], current_layer);
    }
    // Top layer only (the GCodeViewer default): a top layer without visible
    // vertices walks the enabled start to the last vertex (ViewerImpl.cpp:1796-1808).
    let [_, last, current] = vertices.view_range(1, 9, all_roles, seams)[..] else {
        panic!("three indices");
    };
    assert_eq!(
        [last, current].map(|id| id as usize),
        [ids.len() / 2 - 1; 2]
    );
    // Visible travels extend the range over the trailing travel of the top layer.
    let [_, last, _] = vertices.view_range(0, 2, all_roles, travels | seams)[..] else {
        panic!("three indices");
    };
    assert_eq!(last as usize, ids.len() / 2 - 1);
    // Hiding every role leaves nothing visible: `Range()`.
    assert_eq!(vertices.view_range(0, 2, 0, seams), [0, 0, 0]);
}
