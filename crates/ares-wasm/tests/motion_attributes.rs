//! ORCA Pressure Advance / Acceleration / Jerk view types through the WASM
//! adapter: `LibVGCodeWrapper.cpp:215-260` carries the processor values,
//! `ViewerImpl.cpp:1853-1866` builds the ranges (PA from extrusions only,
//! acceleration/jerk also from visible travels).
//! Failure inventory: view types not accepted or listed; values left zero in
//! the stride-16 transport; travel PA leaking into the PA range; hidden
//! travels still widening acceleration/jerk.
use ares_core::{
    ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::{SliceOutput, view_types};

const ALL_ROLES: u32 = u32::MAX;
const TRAVELS: u16 = 1 << 0;

#[test]
fn motion_view_types_expose_processor_values_and_ranges() {
    assert_eq!(
        view_types(),
        [
            "Summary",
            "FeatureType",
            "ColorPrint",
            "Speed",
            "ActualSpeed",
            "Height",
            "Width",
            "VolumetricFlowRate",
            "ActualVolumetricFlowRate",
            "LayerTimeLinear",
            "LayerTimeLogarithmic",
            "FanSpeed",
            "Temperature",
            "PressureAdvance",
            "Acceleration",
            "Jerk",
        ]
    );
    let at = |move_type, x, pressure_advance, acceleration, jerk| MoveVertex {
        move_type,
        position: [x, 0.0, 0.2],
        width: 0.4,
        height: 0.2,
        mm3_per_mm: 0.05,
        pressure_advance,
        acceleration,
        jerk,
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves: vec![
                MoveVertex::default(),
                at(EMoveType::Extrude, 1.0, 0.02, 1000.0, 8.0),
                at(EMoveType::Extrude, 2.0, 0.05, 1500.0, 10.5),
                at(EMoveType::Travel, 3.0, 0.9, 3000.0, 12.25),
            ]
            .into(),
            extruder_colors: vec!["#FF0000".into()],
            ..Default::default()
        },
        print_statistics: Default::default(),
    });
    let mut vertices = output.take_vertices();
    let values = vertices.take_values();
    let motion: Vec<_> = values
        .as_chunks::<16>()
        .0
        .iter()
        .map(|vertex| [vertex[13], vertex[14], vertex[15]])
        .collect();
    assert_eq!(
        motion,
        [
            [0.02, 1000.0, 8.0],
            [0.02, 1000.0, 8.0],
            [0.05, 1500.0, 10.5],
            // The second extrusion also starts a path: a preview value
            // changed (LibVGCodeWrapper.cpp:238-239 splits on
            // acceleration/jerk differences, not just type/role).
            [0.05, 1500.0, 10.5],
            [0.9, 3000.0, 12.25],
            [0.9, 3000.0, 12.25],
        ]
    );
    let range = |vertices: &mut ares_wasm::VertexBuffers, name: &str, options| {
        vertices.vertex_colors(name, ALL_ROLES, options).unwrap();
        let range: serde_json::Value =
            serde_json::from_str(&vertices.color_range(name).unwrap()).unwrap();
        serde_json::from_value::<[f32; 2]>(range["range"].clone()).unwrap()
    };
    assert_eq!(
        range(&mut vertices, "PressureAdvance", TRAVELS),
        [0.02, 0.05]
    );
    assert_eq!(
        range(&mut vertices, "Acceleration", TRAVELS),
        [1000.0, 3000.0]
    );
    assert_eq!(range(&mut vertices, "Jerk", TRAVELS), [8.0, 12.25]);
    assert_eq!(range(&mut vertices, "Acceleration", 0), [1000.0, 1500.0]);
    assert_eq!(range(&mut vertices, "Jerk", 0), [8.0, 10.5]);
}
