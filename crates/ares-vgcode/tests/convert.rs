use ares_core::{
    ExtrusionRole,
    gcode_processor::{
        CustomGCodeItem, CustomGCodeType, EMoveType, GCodeProcessorResult, MoveVertex,
    },
};
use ares_vgcode::{DUMMY_COLOR, MoveType, convert::from_processor_result};

#[test]
fn empty_and_dummy_results_have_no_vertices() {
    for moves in [vec![], vec![MoveVertex::default()]] {
        let input = from_processor_result(&GCodeProcessorResult {
            moves: moves.into(),
            spiral_vase_mode: true,
            ..Default::default()
        });
        assert!(input.vertices.is_empty());
        assert!(input.spiral_vase_mode);
    }
}

#[test]
fn path_starts_use_previous_position_and_zero_time() {
    let a = MoveVertex {
        move_type: EMoveType::Extrude,
        extrusion_role: ExtrusionRole::Perimeter,
        position: [1.0, 2.0, 0.2],
        time: 3.0,
        mm3_per_mm: 0.04,
        gcode_id: 16777217,
        layer_id: 2,
        width: 0.4,
        height: 0.2,
        feedrate: 40.0,
        fan_speed: 50.0,
        temperature: 210.0,
        extruder_id: 1,
        cp_color_id: 3,
        pressure_advance: 0.04,
        acceleration: 1500.0,
        jerk: 8.5,
        ..MoveVertex::default()
    };
    let b = MoveVertex {
        position: [2.0, 2.0, 0.2],
        ..a
    };
    let c = MoveVertex {
        position: [3.0, 2.0, 0.2],
        mm3_per_mm: 0.06,
        ..b
    };
    let d = MoveVertex {
        position: [4.0, 2.0, 0.2],
        extrusion_role: ExtrusionRole::ExternalPerimeter,
        ..c
    };
    let e = MoveVertex {
        position: [5.0, 2.0, 0.2],
        move_type: EMoveType::Travel,
        ..d
    };
    let f = MoveVertex {
        move_type: EMoveType::Retract,
        ..e
    };
    let data = from_processor_result(&GCodeProcessorResult {
        moves: vec![MoveVertex::default(), a, b, c, d, e, f].into(),
        spiral_vase_mode: false,
        ..Default::default()
    });
    assert_eq!(data.vertices.len(), 10);
    for (index, position) in [
        (0, [0.0; 3]),
        (3, b.position),
        (5, c.position),
        (7, d.position),
    ] {
        assert_eq!(data.vertices[index].position, position);
        assert_eq!(data.vertices[index].times, [0.0; 2]);
    }
    assert_eq!(data.vertices[9].move_type, MoveType::Retract);
    let v = &data.vertices[1];
    assert_eq!(
        (v.gcode_id, v.layer_id, v.extruder_id, v.color_id),
        (16777217, 2, 1, 3)
    );
    assert_eq!(
        (v.width, v.height, v.feedrate, v.fan_speed, v.temperature),
        (0.4, 0.2, 40.0, 50.0, 210.0)
    );
    assert_eq!(v.times, [3.0, 0.0]);
    // ORCA PA/acceleration/jerk visualization (`LibVGCodeWrapper.cpp:215-260`):
    // phantom and regular vertices both carry the current move's values.
    for v in &data.vertices[..2] {
        assert_eq!(
            (v.pressure_advance, v.acceleration, v.jerk),
            (0.04, 1500.0, 8.5)
        );
    }
}

#[test]
fn palettes_follow_color_decode_and_minimum_brightness() {
    let tools = ["#000000", "#FF8040", "not-a-color"].map(str::to_owned);
    let mut result = GCodeProcessorResult {
        extruder_colors: tools.into(),
        ..Default::default()
    };
    let data = from_processor_result(&result);
    assert_eq!(
        data.tools_colors,
        vec![[48; 3], [255, 128, 64], DUMMY_COLOR]
    );
    assert_eq!(data.color_print_colors, data.tools_colors);
    // Plater.cpp:16927-16946 + GUI_Preview.cpp:680-688: append event colors
    // to base tools and add gray for any custom item. Keep the wrapper's
    // RGBA decoder coverage using a synthetic result (processor tags use RGB).
    result.custom_gcode_per_print_z.push(CustomGCodeItem {
        print_z: 0.2,
        kind: CustomGCodeType::ColorChange,
        extruder: 1,
        color: "#112233FF".into(),
        extra: String::new(),
    });
    let data = from_processor_result(&result);
    assert_eq!(
        data.color_print_colors,
        vec![[48; 3], [255, 128, 64], DUMMY_COLOR, [48, 48, 51], [128; 3],]
    );
    result.custom_gcode_per_print_z[0].kind = CustomGCodeType::PausePrint;
    let data = from_processor_result(&result);
    assert_eq!(
        data.color_print_colors,
        vec![[48; 3], [255, 128, 64], DUMMY_COLOR, [128; 3],]
    );
    result.custom_gcode_per_print_z.clear();
    let data = from_processor_result(&result);
    assert_eq!(data.color_print_colors, data.tools_colors);
}
