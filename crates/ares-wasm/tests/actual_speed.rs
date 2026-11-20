//! ORCA Actual Speed / Actual Volumetric Flow Rate view types through the WASM
//! adapter: `LibVGCodeWrapper.cpp:215-260` carries `actual_feedrate` (the
//! path-start phantom vertex takes `prev.actual_feedrate`), and
//! `ViewerImpl.cpp:1845-1866` builds the ranges (actual speed from visible
//! travels/wipes and extrusions, actual flow from extrusions binned).
//! Failure inventory: view types not accepted or listed in `EViewType` order;
//! `actual_feedrate` left zero in the stride-16 transport; the phantom vertex
//! using `curr` instead of `prev`; hidden travels still widening actual speed;
//! travels leaking into the actual flow range.
use ares_core::{
    ProcessedGCode,
    gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex},
};
use ares_wasm::{SliceOutput, view_types};

const ALL_ROLES: u32 = u32::MAX;
const TRAVELS: u16 = 1 << 0;

#[test]
fn actual_speed_view_types_expose_processor_values_and_ranges() {
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
    let at = |move_type, x, actual_feedrate| MoveVertex {
        move_type,
        position: [x, 0.0, 0.2],
        width: 0.4,
        height: 0.2,
        mm3_per_mm: 0.05,
        actual_feedrate,
        ..MoveVertex::default()
    };
    let mut output = SliceOutput::from_processed(ProcessedGCode {
        warnings: Vec::new(),
        gcode: Vec::new(),
        result: GCodeProcessorResult {
            moves: vec![
                MoveVertex {
                    actual_feedrate: 5.0,
                    ..MoveVertex::default()
                },
                at(EMoveType::Extrude, 1.0, 20.0),
                at(EMoveType::Extrude, 2.0, 40.0),
                at(EMoveType::Travel, 3.0, 150.0),
            ],
            extruder_colors: vec!["#FF0000".into()],
            ..Default::default()
        },
        print_statistics: Default::default(),
        archive: None,
    });
    let mut vertices = output.take_vertices();
    let values = vertices.take_values();
    let actual: Vec<_> = values
        .as_chunks::<16>()
        .0
        .iter()
        .map(|vertex| vertex[6])
        .collect();
    assert_eq!(actual, [5.0, 20.0, 40.0, 40.0, 150.0]);
    let range = |vertices: &mut ares_wasm::VertexBuffers, name: &str, options| {
        vertices.vertex_colors(name, ALL_ROLES, options).unwrap();
        let range: serde_json::Value =
            serde_json::from_str(&vertices.color_range(name).unwrap()).unwrap();
        serde_json::from_value::<[f32; 2]>(range["range"].clone()).unwrap()
    };
    assert_eq!(range(&mut vertices, "ActualSpeed", TRAVELS), [5.0, 150.0]);
    assert_eq!(range(&mut vertices, "ActualSpeed", 0), [5.0, 40.0]);
    assert_eq!(
        range(&mut vertices, "ActualVolumetricFlowRate", TRAVELS),
        [0.25, 2.0]
    );
    // `append_range_item` prints the float values with `%.*f`: JS must read
    // each legend value as the exact f32, not the f64 of its shortest decimal
    // (f32 2.315 prints as 2.32, f64 2.315 as 2.31).
    // Parsed with correctly rounded `str::parse`, as `JSON.parse` does
    // (serde_json's default float parsing is not).
    let json = vertices.color_range("ActualVolumetricFlowRate").unwrap();
    let values = json.split_once("\"values\":[").unwrap().1;
    let values: Vec<f64> = values[..values.find(']').unwrap()]
        .split(',')
        .map(|value| value.parse().unwrap())
        .collect();
    let exact: Vec<f64> = values.iter().map(|&v| f64::from(v as f32)).collect();
    assert_eq!(values, exact, "color range values carry exact f32");
}
