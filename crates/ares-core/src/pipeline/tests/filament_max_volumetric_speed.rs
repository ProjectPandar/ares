use crate::{
    ExtrusionMove, InputFormat, Layer, LayerBrims, LayerContours, LayerExtrusionMoves,
    LayerGapFills, LayerInfills, LayerPerimeters, LayerPrintPaths, LayerSkirts, LayerSlice,
    LayerToolpathMoves, Model, PipelineDiagnostics, PipelineStage, Point2, PrintPath,
    PrintPathRole, SliceOptions, SlicingPipeline, ToolpathMove, ToolpathMoveKind,
    build_print_domain, gcode::format_gcode, generate_speed_moves,
    pipeline::test_support::rectangular_pipeline,
};
use serde_json::json;

#[test]
fn filament_max_volumetric_speed_caps_print_feedrate() {
    let uncapped = options(json!({ "filament_max_volumetric_speed": 0.0 }));
    let capped = options(json!({ "filament_max_volumetric_speed": 1.0 }));

    let uncapped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&uncapped), &uncapped).unwrap())
            .unwrap();
    let capped_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&capped), &capped).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print"),
        6000.0
    );
    assert!(
        first_speed_feedrate(&capped_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&uncapped_gcode, "external_perimeter", "print")
    );
}

#[test]
fn filament_max_volumetric_speed_does_not_cap_travel_feedrate() {
    let capped = options(json!({
        "filament_max_volumetric_speed": 0.1,
        "travel_speed": 120
    }));
    let gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&capped), &capped).unwrap()).unwrap();

    assert_eq!(
        first_speed_feedrate(&gcode, "external_perimeter", "travel"),
        7200.0
    );
}

#[test]
fn filament_flow_ratio_lowers_volumetric_capped_feedrate() {
    let base = options(json!({ "filament_max_volumetric_speed": 1.0 }));
    let higher_flow = options(json!({
        "filament_max_volumetric_speed": 1.0,
        "filament_flow_ratio": 2.0
    }));

    let base_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&base), &base).unwrap()).unwrap();
    let higher_flow_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&higher_flow), &higher_flow).unwrap())
            .unwrap();

    assert!(
        first_speed_feedrate(&higher_flow_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&base_gcode, "external_perimeter", "print")
    );
}

#[test]
fn adaptive_volumetric_speed_lowers_print_feedrate_from_coefficients() {
    let disabled = options(json!({
        "filament_max_volumetric_speed": 10.0,
        "filament_adaptive_volumetric_speed": false,
        "volumetric_speed_coefficients": ["0 0 0 0 0 1"]
    }));
    let enabled = options(json!({
        "filament_max_volumetric_speed": 10.0,
        "filament_adaptive_volumetric_speed": true,
        "volumetric_speed_coefficients": ["0 0 0 0 0 1"]
    }));

    let disabled_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&disabled), &disabled).unwrap())
            .unwrap();
    let enabled_gcode =
        String::from_utf8(format_gcode(&rectangular_pipeline(&enabled), &enabled).unwrap())
            .unwrap();

    assert!(
        first_speed_feedrate(&enabled_gcode, "external_perimeter", "print")
            < first_speed_feedrate(&disabled_gcode, "external_perimeter", "print")
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_speed": 100,
        "initial_layer_speed": 100,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn first_speed_feedrate(gcode: &str, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    gcode
        .lines()
        .find_map(|line| {
            line.starts_with(&target)
                .then(|| line.rsplit(':').next().unwrap().parse().unwrap())
        })
        .unwrap_or_else(|| panic!("missing {kind} {role} speed"))
}

fn last_speed_feedrate(gcode: &str, role: &str, kind: &str) -> f64 {
    let target = format!(";SPEED:{kind}:{role}:");
    gcode
        .lines()
        .filter(|line| line.starts_with(&target))
        .map(|line| line.rsplit(':').next().unwrap().parse().unwrap())
        .next_back()
        .unwrap_or_else(|| panic!("missing {kind} {role} speed"))
}
