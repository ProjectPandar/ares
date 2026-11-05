//! `WipeTower2::tool_change`, `toolchange_Change` and `toolchange_Load`.
use super::{
    generator::Generator,
    geometry::BoxCoordinates,
    result::ToolChangeResult,
    writer::{Extrude, Writer},
};

impl Generator<'_> {
    pub(super) fn tool_change(&mut self, tool: Option<usize>) -> ToolChangeResult {
        let old_tool = self.current_tool;
        let interface = self
            .settings
            .process
            .print
            .enable_tower_interface_features
            .0
            && self.current_interface;
        let (mut volume, area) = tool
            .and_then(|tool| {
                self.plan.layers[self.layer_index]
                    .changes
                    .iter()
                    .find(|c| c.new_tool == tool)
            })
            .map_or((0.0, 0.0), |change| {
                (change.wipe_volume, change.required_depth)
            });
        if interface && let Some(tool) = tool {
            let filament = &self.filaments[tool];
            if filament.interface_purge > 0.0 {
                volume += filament.interface_purge * filament.area;
            }
        }
        let perimeter = self.plan.perimeter_width;
        let bounds = BoxCoordinates::new(
            [perimeter / 2.0; 2],
            [
                self.plan.width - perimeter,
                if tool.is_some() {
                    area + self.depth_traversed - 0.5 * perimeter
                } else {
                    self.plan.depth - perimeter
                },
            ],
        );
        let mut writer = self.writer();
        writer.set_y_shift(self.y_shift);
        writer
            .gcode
            .push_str(";--------------------\n; CP TOOLCHANGE START\n");
        if let Some(tool) = tool {
            writer.gcode.push_str(&format!("; toolchange #{}\n; material : {} -> {}\n;--------------------\n; WIPE_TOWER_START\n", self.tool_changes + 1, self.filaments[old_tool].material, self.filaments[tool].material));
        }
        writer.speed_override_backup();
        writer.speed_override(100);
        writer.set_initial_position(
            self.toolchange_entry_pos(self.depth_traversed),
            [self.plan.width, self.plan.depth],
            self.rotation,
        );
        let trimpot = self.settings.printer.gcode.single_extruder_multi_material.0
            && self.settings.printer.gcode.high_current_on_filament_swap.0;
        if trimpot {
            writer.trimpot(750);
        }
        if let Some(tool) = tool {
            let first = self.is_first_layer();
            let temperature = if first {
                self.filaments[tool].first_temperature
            } else {
                self.filaments[tool].temperature
            };
            let old_temperature = if first {
                self.filaments[old_tool].first_temperature
            } else {
                self.filaments[old_tool].temperature
            };
            self.unload(&mut writer, bounds, old_temperature, temperature);
            self.change(&mut writer, tool);
            self.load(&mut writer, bounds);
            writer.travel([writer.position[0], writer.position[1] - perimeter], 0.0);
            let filament = &self.filaments[tool];
            let interface_temp = filament.interface_temperature;
            let cooldown = self
                .settings
                .process
                .print
                .enable_tower_interface_cooldown_during_tower
                .0;
            let change_interface_temperature =
                interface && interface_temp > 0 && interface_temp != temperature;
            if change_interface_temperature {
                writer.set_temperature(interface_temp, true);
            }
            if change_interface_temperature && cooldown {
                writer.set_temperature(temperature, false);
            }
            if interface && filament.interface_distance > 0.0 && filament.interface_length > 0.0 {
                writer.extrude_explicit(
                    [
                        (writer.position[0] + filament.interface_distance)
                            .clamp(bounds.lower[0], bounds.upper[0]),
                        writer.position[1],
                    ],
                    Extrude {
                        amount: filament.interface_length,
                        feedrate: 600.0,
                        record: false,
                        limit: true,
                    },
                );
            }
            self.wipe(&mut writer, bounds, volume, interface);
            if change_interface_temperature && !cooldown {
                writer.set_temperature(temperature, false);
            }
            writer.gcode.push_str("; WIPE_TOWER_END\n");
            self.tool_changes += 1;
        } else {
            let temperature = self.filaments[old_tool].temperature;
            self.unload(&mut writer, bounds, temperature, temperature);
        }
        self.depth_traversed += area;
        if trimpot {
            writer.trimpot(550);
        }
        writer.speed_override_restore();
        writer.feedrate(self.settings.process.gcode.travel_speed.0 as f32 * 60.0);
        Writer::flush_planner_queue(&mut writer.gcode, self.flavor);
        writer
            .gcode
            .push_str("G92 E0\n; CP TOOLCHANGE END\n;------------------\n\n\n");
        self.used_filament[self.current_tool] += writer.take_used_filament();
        self.result(
            writer,
            old_tool,
            super::result::Kind::Change { contact: interface },
        )
    }

    pub(super) fn change(&mut self, writer: &mut Writer, tool: usize) {
        self.used_filament[self.current_tool] += writer.take_used_filament();
        writer.gcode.push_str("[change_filament_gcode]\n");
        if self.mk4_mmu3 {
            writer.switch_filament_monitoring(true);
        }
        let position = writer.rotated(writer.position);
        writer.feedrate(self.settings.process.gcode.travel_speed.0 as f32 * 60.0);
        writer.gcode.push_str(&format!(
            "G1 X{} Y{}_GCODE_WIPE_TOWER_NEVER_SKIP_TAG\n[deretraction_from_wipe_tower_generator]",
            super::super::format_processor_float(f64::from(position[0])),
            super::super::format_processor_float(f64::from(position[1]))
        ));
        writer.set_tool(tool, self.filaments[tool].max_e_speed);
        Writer::flush_planner_queue(&mut writer.gcode, self.flavor);
        self.current_tool = tool;
    }

    pub(super) fn load(&self, writer: &mut Writer, bounds: BoxCoordinates) {
        let config = &self.settings.printer.gcode;
        if config.single_extruder_multi_material.0
            && self.plan.ramming
            && (config.parking_pos_retraction.0 != 0.0 || config.extra_loading_move.0 != 0.0)
        {
            let filament = &self.filaments[self.current_tool];
            let xl = bounds.lower[0] + self.plan.perimeter_width * 0.75;
            let xr = bounds.upper[0] - self.plan.perimeter_width * 0.75;
            let old_x = writer.position[0];
            let turning = if old_x - xl < xr - old_x { xr } else { xl };
            let distance =
                config.parking_pos_retraction.0 as f32 + config.extra_loading_move.0 as f32;
            writer.gcode.push_str("; CP TOOLCHANGE LOAD\n");
            writer.suppress_preview = true;
            writer.load(0.2 * distance, 60.0 * filament.loading_start);
            writer.load_move_x(turning, 0.7 * distance, filament.loading_speed, 50.0);
            writer.load_move_x(old_x, 0.1 * distance, 0.1 * filament.loading_speed, 50.0);
            writer.travel([old_x, writer.position[1]], 0.0);
            writer.suppress_preview = false;
            if config.high_current_on_filament_swap.0 {
                writer.trimpot(550);
            }
        }
    }
}
