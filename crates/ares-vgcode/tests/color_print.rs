//! ColorPrint.hpp:12-18, ViewerImpl.cpp:1002-1026,1373-1382.
//! Failure inventory before implementation: markers/travel become printed
//! segments, repeated same ids split, returning colors get deduplicated,
//! extruder order changes, timestamps exclude the current vertex, or reload
//! retains old per-extruder state. This isolated public viewer seam complements
//! the native processor and browser E2E fixtures.

use ares_vgcode::{ColorPrint, GCodeInputData, MoveType, PathVertex, ViewerImpl};

#[test]
fn color_prints_follow_per_extruder_transitions_and_inclusive_times() {
    let vertex = |move_type, extruder_id, color_id, layer_id, times| PathVertex {
        move_type,
        extruder_id,
        color_id,
        layer_id,
        times,
        ..Default::default()
    };
    let mut viewer = ViewerImpl::default();
    viewer.load(GCodeInputData {
        vertices: vec![
            vertex(MoveType::Travel, 7, 8, 0, [1.0, 2.0]),
            vertex(MoveType::Extrude, 2, 2, 0, [3.0, 4.0]),
            vertex(MoveType::Extrude, 2, 2, 0, [5.0, 6.0]),
            vertex(MoveType::ColorChange, 2, 4, 1, [0.0, 0.0]),
            vertex(MoveType::Travel, 0, 0, 1, [2.0, 1.0]),
            vertex(MoveType::Extrude, 0, 0, 1, [1.0, 2.0]),
            vertex(MoveType::Extrude, 2, 4, 2, [2.0, 3.0]),
            vertex(MoveType::Extrude, 2, 2, 3, [1.0, 1.0]),
            vertex(MoveType::ColorChange, 2, 5, 4, [0.0, 0.0]),
        ],
        ..Default::default()
    });
    assert_eq!(viewer.get_used_extruders_ids(), [0, 2]);
    assert_eq!(viewer.get_color_prints_count(7), 0);
    assert_eq!(viewer.get_color_prints_count(2), 3);
    assert_eq!(
        viewer.get_color_prints(0),
        &[ColorPrint {
            extruder_id: 0,
            color_id: 0,
            layer_id: 1,
            times: [12.0, 15.0],
        }]
    );
    assert_eq!(
        viewer.get_color_prints(2),
        &[
            ColorPrint {
                extruder_id: 2,
                color_id: 2,
                layer_id: 0,
                times: [4.0, 6.0]
            },
            ColorPrint {
                extruder_id: 2,
                color_id: 4,
                layer_id: 2,
                times: [14.0, 18.0]
            },
            ColorPrint {
                extruder_id: 2,
                color_id: 2,
                layer_id: 3,
                times: [15.0, 19.0]
            },
        ]
    );
    viewer.load(GCodeInputData {
        vertices: vec![PathVertex::default()],
        ..Default::default()
    });
    assert!(viewer.get_used_extruders_ids().is_empty());
    assert!(viewer.get_color_prints(2).is_empty());
    assert_eq!(viewer.get_color_prints_count(2), 0);
}
