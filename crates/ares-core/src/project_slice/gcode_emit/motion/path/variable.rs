use super::super::scarf::Slope;
use super::{
    EmitState, PathProperties, SOURCE_EPSILON_MM, adaptive_pa, arc, extrusion, fan, overhang,
};
use crate::project_slice::gcode_emit::motion::features::{feature_description, is_bridge_feature};
use crate::project_slice::gcode_emit::motion::format::{
    axis as format_axis, extrusion as format_extrusion, z as format_z,
};

pub(super) struct Emission<'a> {
    pub(super) output: &'a mut Vec<u8>,
    pub(super) points: &'a [(f64, f64)],
    pub(super) wipe_points: &'a [(f64, f64)],
    pub(super) processed: &'a [overhang::ProcessedPoint],
    pub(super) original_speed: f64,
    pub(super) acceleration: u32,
    pub(super) slope: Option<Slope>,
    pub(super) properties: PathProperties<'a>,
    pub(super) state: &'a mut EmitState,
}

/// `GCode.cpp:7229-7248`: a `sloped` processed path emits
/// `extrude_to_xyz(dest3d, dE * e_ratio)` per segment — the slope's
/// z/e ratios interpolated over accumulated emitted length — inside the
/// SAME delta-60 `set_speed` loop as ordinary processed paths, so the
/// writer's `last_set_speed`/`m_current_speed` tracks every emitted
/// segment (that value is what `Wipe::wipe` reads at
/// `role_based_wipe_speed=1`).
#[allow(clippy::too_many_arguments)]
fn sloped_segment(
    output: &mut Vec<u8>,
    end: arc::Point,
    length: f64,
    z: f64,
    e_ratio: f64,
    slope: Slope,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let extrusion = extrusion::for_length(
        length,
        properties.mm3_per_mm,
        state.options.filament_flow_ratio,
        state.options.print_flow_ratio,
        state.options.filament_area,
    ) * slope.flow_ratio
        * state
            .small_area_flow
            .multiplier_for_feature(properties.feature, length)
        * e_ratio;
    let extrusion = extrusion::coordinate(state, extrusion);
    let previous_z = state.scarf_z;
    let z_word = if previous_z.is_some_and(|last| (z - last).abs() < 1.0e-9) {
        String::new()
    } else {
        format!(" Z{}", format_z(z))
    };
    output.extend_from_slice(
        format!(
            "G1 X{} Y{}{} E{}{}\n",
            format_axis(end.x),
            format_axis(end.y),
            z_word,
            format_extrusion(extrusion),
            if state.options.gcode_comments {
                let bridge = if is_bridge_feature(properties.feature) {
                    " (bridge)"
                } else {
                    ""
                };
                format!(" ; {}{bridge}", feature_description(properties.feature))
            } else {
                Default::default()
            }
        )
        .as_bytes(),
    );
    state.x = end.x;
    state.y = end.y;
    state.scarf_z = Some(z);
    if !z_word.is_empty() {
        state.record_z(z);
    }
    state.wipe_start = Some(end);
}

pub(super) fn emit(command: Emission<'_>) {
    let Emission {
        output,
        points,
        wipe_points,
        processed,
        original_speed,
        acceleration,
        slope,
        properties,
        state,
    } = command;
    let original_feedrate = original_speed * 60.0;
    let mut last_feedrate = processed[0].speed * 60.0;
    // `GCode.cpp:7149` `line_length = (p - prev).norm()` — `p`/`prev`
    // are `point_to_gcode_quantized` (XYZF_EXPORT_DIGITS=3) coordinates,
    // so per-segment length and the sloped `path_length` accumulate over
    // the QUANTIZED emitted positions, not the raw source geometry.
    let quantize = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut previous = (quantize(points[0].0), quantize(points[0].1));
    // `GCode.cpp:7115-7122`: the sloped length basis is the PROCESSED
    // point polyline (`Polyline3 l(new_points)`), not the raw emitted
    // path.
    let slope_total_length = slope.map(|_| {
        points
            .windows(2)
            .map(|pair| (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1))
            .sum::<f64>()
    });
    let mut slope_path_length = 0.0;
    for index in 1..points.len() {
        let end = arc::Point {
            x: points[index].0,
            y: points[index].1,
        };
        fan::update_for_variable_segment(
            output,
            properties,
            processed[index - 1],
            processed[index],
            state,
        );
        let end_q = (quantize(end.x), quantize(end.y));
        let length = ((end_q.0 - previous.0) * (end_q.0 - previous.0)
            + (end_q.1 - previous.1) * (end_q.1 - previous.1))
            .sqrt();
        if length < SOURCE_EPSILON_MM {
            continue;
        }
        let feedrate = processed[index - 1].speed * 60.0;
        adaptive_pa::ramp(
            output,
            state,
            properties,
            acceleration,
            adaptive_pa::Ramp::Variable {
                previous: last_feedrate,
                next: feedrate,
            },
        );
        if (last_feedrate - feedrate).abs() > 60.0 {
            extrusion::speed(output, feedrate, properties);
            state.current_feedrate = feedrate;
            last_feedrate = feedrate;
        } else if (original_feedrate - feedrate).abs() <= 60.0 {
            extrusion::speed(output, original_feedrate, properties);
            state.current_feedrate = original_feedrate;
            last_feedrate = original_feedrate;
        }
        if let Some(slope) = slope {
            slope_path_length += length;
            let ratio = slope_path_length / slope_total_length.unwrap();
            let z_ratio = slope.z_begin + (slope.z_end - slope.z_begin) * ratio;
            let e_ratio = slope.e_begin + (slope.e_end - slope.e_begin) * ratio;
            let z = state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * z_ratio;
            sloped_segment(output, end, length, z, e_ratio, slope, properties, state);
        } else {
            extrusion::linear_segment(output, end, length, properties, state);
        }
        previous = end_q;
    }
    state.extrusion_feedrate = last_feedrate;
    // The PATH points are local mm (raw scaled points, like upstream
    // `Wipe::path`); `wipe_moves` converts them with the local scale only
    // so the int round-trip is exact. The CURRENT-position anchor stays
    // in the emitted (global) frame — `scaled_position` inverts the
    // emitter's own conversion exactly.
    state.wipe_start = wipe_points.last().map(|&(x, y)| arc::Point {
        x: x + state.origin.0 - state.extruder_offset.0,
        y: y + state.origin.1 - state.extruder_offset.1,
    });
    state.wipe_path = wipe_points
        .iter()
        .rev()
        .map(|&(x, y)| arc::Point { x, y })
        .collect();
}
