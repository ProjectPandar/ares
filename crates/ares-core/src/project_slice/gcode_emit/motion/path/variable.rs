use super::{
    EmitState, PathProperties, SOURCE_EPSILON_MM, adaptive_pa, arc, extrusion, fan, overhang,
};

pub(super) struct Emission<'a> {
    pub(super) output: &'a mut Vec<u8>,
    pub(super) points: &'a [(f64, f64)],
    pub(super) wipe_points: &'a [(f64, f64)],
    pub(super) processed: &'a [overhang::ProcessedPoint],
    pub(super) original_speed: f64,
    pub(super) acceleration: u32,
    pub(super) force_no_extrusion: bool,
    pub(super) properties: PathProperties<'a>,
    pub(super) state: &'a mut EmitState,
}

pub(super) fn emit(command: Emission<'_>) {
    let Emission {
        output,
        points,
        wipe_points,
        processed,
        original_speed,
        acceleration,
        force_no_extrusion,
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
    // `GCode.cpp:7114-7121`: the sloped length denominator is the PROCESSED
    // point polyline (`Polyline3 l(new_points)`) built from the RAW
    // `ProcessedPoint.p`, before `point_to_gcode_quantized`; the
    // `path_length` numerator (`GCode.cpp:7149`) still accumulates over the
    // quantized emitted positions, so the cumulative ratio may exceed 1.
    let slope = properties.slope.map(|slope| {
        let total_length = processed
            .windows(2)
            .map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y))
            .sum::<f64>();
        (slope, total_length)
    });
    let mut previous_z = quantize(processed[0].z);
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
            + (end_q.1 - previous.1) * (end_q.1 - previous.1)
            + (quantize(processed[index].z) - previous_z).powi(2))
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
        if properties.contour.is_some() {
            super::contour::segment(
                output,
                crate::Point3d::new(end.x, end.y, processed[index].z),
                length,
                properties,
                state,
            );
        } else if let Some((slope, total_length)) = slope {
            slope_path_length += length;
            let ratio = slope_path_length / total_length;
            let z_ratio = slope.z_begin + (slope.z_end - slope.z_begin) * ratio;
            let e_ratio = slope.e_begin + (slope.e_end - slope.e_begin) * ratio;
            let z = state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * z_ratio;
            // `GCode.cpp:7229-7248`: slope Z/E interpolation stays inside
            // the processed delta-60 speed loop; the writer is shared.
            let delta = extrusion::for_length(
                length,
                extrusion::effective_flow(properties, &state.options, state.on_first_layer),
                state.options.filament_flow_ratio,
                state.options.filament_area,
            ) * extrusion::small_area_multiplier(state, properties.feature, length)
                * e_ratio;
            extrusion::extrude_to_xyz(
                output,
                extrusion::XyzEmission {
                    end,
                    z,
                    delta,
                    properties,
                },
                state,
            );
        } else if force_no_extrusion {
            extrusion::force_no_extrusion_segment(output, end, state);
        } else {
            extrusion::linear_segment(output, end, length, properties, state);
        }
        previous = end_q;
        previous_z = quantize(processed[index].z);
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
