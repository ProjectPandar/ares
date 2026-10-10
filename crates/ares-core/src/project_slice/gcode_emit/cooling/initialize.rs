use super::{CoolingState, feedrate};
use crate::{
    options::{PartCoolingFanRamp, PartCoolingFanRampConfig},
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

impl CoolingState {
    pub(in crate::project_slice::gcode_emit) fn from_traversal(
        traversal: &PreparedPostClassicTraversal,
        writer: &super::super::motion::EmitState,
    ) -> Self {
        let full = &traversal.resolved.views.full;
        let filament = &full.filament.print;
        let runtime = &traversal.resolved.views.runtime_gcode;
        let first_bool = |values: &[crate::OrcaBool]| values.first().is_some_and(|value| value.0);
        let max_speed = super::first_percent(&filament.fan_max_speed.0);
        let slow_down_layer_time = filament
            .slow_down_layer_time
            .0
            .first()
            .map_or(0.0, |value| value.0);
        let part_fan_ramp = PartCoolingFanRamp::new(PartCoolingFanRampConfig {
            min_speed: super::first_percent(&filament.fan_min_speed.0),
            max_speed,
            full_speed_layer: super::first_non_negative(&filament.full_fan_speed_layer.0) as u32,
            close_fan_first_layers: super::first_non_negative(
                &filament.close_fan_the_first_x_layers.0,
            ) as u32,
            layer_times_s: [
                slow_down_layer_time,
                filament
                    .fan_cooling_layer_time
                    .0
                    .first()
                    .map_or(0.0, |value| value.0),
            ],
            fan_kickstart_s: runtime.fan_kickstart.0,
            reduce_fan_stop_start_freq: first_bool(&filament.reduce_fan_stop_start_freq.0),
        });
        let gcode = &traversal.resolved.views.runtime_gcode;
        let slope = gcode.max_volumetric_extrusion_rate_slope.0;
        let equalizer = (slope > 0.0).then(|| {
            crate::project_slice::pressure_equalizer::PressureEqualizerPass::new(
                slope,
                gcode.max_volumetric_extrusion_rate_slope_segment_length.0,
                gcode.extrusion_rate_smoothing_external_perimeter_only.0,
                &gcode
                    .filament_diameter
                    .0
                    .iter()
                    .map(|value| value.0)
                    .collect::<Vec<f64>>(),
                gcode.use_relative_e_distances.0,
            )
        });
        Self {
            equalizer,
            part_speed: None,
            physical_part_speed: 0,
            provisional_part_speed: 0,
            pending_layer_index: None,
            equalized_layer_result: None,
            buffered_gcode: Vec::new(),
            part_fan_ramp,
            additional_speed: None,
            additional_fan_speed: super::first_percent_int(
                &filament.additional_cooling_fan_speed.0,
            ),
            auxiliary_fan: runtime.auxiliary_fan.0,
            part_cooling_fan_min_pwm: runtime.part_cooling_fan_min_pwm.0.clamp(0, 100) as u8,
            gcode_comments: full.process.print.gcode_comments.0,
            feedrate: feedrate::State::new(
                writer
                    .extruders
                    .ids()
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            feedrate::Config {
                                enabled: filament
                                    .slow_down_for_layer_cooling
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_for_layer_cooling.0[0])
                                    .0,
                                target_time: filament
                                    .slow_down_layer_time
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_layer_time.0[0])
                                    .0 as f32,
                                minimum_speed: filament
                                    .slow_down_min_speed
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.slow_down_min_speed.0[0])
                                    .0 as f32,
                                keep_outer_wall_speed: filament
                                    .dont_slow_down_outer_wall
                                    .0
                                    .get(id)
                                    .unwrap_or(&filament.dont_slow_down_outer_wall.0[0])
                                    .0,
                                relative_e: runtime.use_relative_e_distances.0,
                                keep_markers: slope > 0.0,
                            },
                        )
                    })
                    .collect(),
                runtime.travel_speed.0,
                super::super::motion::tool_selection::prefix(traversal),
                writer.initial_filaments.initial(),
            ),
        }
    }
}
