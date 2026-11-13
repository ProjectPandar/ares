//! `WipeTower2::toolchange_Unload`, including SEMM and multi-tool ramming.
use super::{
    generator::Generator,
    geometry::BoxCoordinates,
    plan::volume_to_length,
    writer::{Extrude, Writer},
};

impl Generator<'_> {
    pub(super) fn unload(
        &mut self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        old_temperature: i32,
        new_temperature: i32,
    ) {
        let filament = &self.filaments[self.current_tool];
        let config = &self.settings.printer.gcode;
        let semm = config.single_extruder_multi_material.0;
        let enabled = self.plan.ramming;
        let perimeter = self.plan.perimeter_width;
        let xl = bounds.lower[0] + perimeter;
        let xr = bounds.upper[0] - perimeter;
        let line_width = perimeter * filament.ramming_width;
        let y_step = line_width * filament.ramming_step * self.plan.ramming_spacing;
        let start = [xl, bounds.lower[1] + self.depth_traversed + y_step / 2.0];
        writer.gcode.push_str("; CP TOOLCHANGE UNLOAD\n");
        writer.change_width(line_width);
        self.left_to_right = true;
        let mut remaining = xr - xl;
        let mut done = 0.0;
        let do_ramming = (semm && enabled) || filament.multitool_ramming;
        if do_ramming {
            writer.travel(start, 0.0);
            if !self.mk4_mmu3 {
                writer.disable_linear_advance();
            }
            if self.mk4_mmu3 {
                writer.set_temperature(old_temperature - 20, false, true);
            }
        } else {
            writer.position = start;
        }
        // set_layer always selects SHAPE_NORMAL in the source.
        if do_ramming
            && self.layer_index > 1
            && self.layer_index < self.plan.layers.len()
            && let Some(first_line) = self.ramming_first_line(y_step)
        {
            writer.extrude([xl + first_line - perimeter, writer.position[1]], 0.0);
            remaining -= first_line - perimeter;
        }
        if self.mk4_mmu3 {
            writer.switch_filament_monitoring(false);
            writer.wait(1.5);
        }
        let mut index = 0;
        while do_ramming && index < filament.ramming_speeds.len() {
            let step = if semm {
                0.25
            } else {
                filament.multitool_ramming_time
            };
            let length = volume_to_length(
                filament.ramming_speeds[index] * step,
                line_width,
                self.layer_height,
            );
            let extrusion = filament.ramming_speeds[index] * step / self.filaments[0].area;
            let distance = (length - done).min(remaining);
            let time = distance / length * step;
            writer.extrude_explicit(
                writer.position,
                Extrude {
                    amount: 0.0,
                    feedrate: distance / (time / 60.0),
                    record: true,
                    limit: false,
                },
            );
            writer.extrude_explicit(
                [
                    writer.position[0]
                        + if self.left_to_right {
                            distance
                        } else {
                            -distance
                        },
                    writer.position[1],
                ],
                Extrude {
                    amount: extrusion * (distance / length),
                    feedrate: 0.0,
                    record: true,
                    limit: false,
                },
            );
            remaining -= distance;
            if remaining < 1e-3 {
                writer.travel([writer.position[0], writer.position[1] + y_step], 7200.0);
                self.left_to_right = !self.left_to_right;
                remaining = xr - xl;
            }
            done += distance;
            if done > length - 1e-3 {
                index += 1;
                done = 0.0;
            }
        }
        let end = writer.position;
        writer.change_width(perimeter);
        if enabled {
            writer.gcode.push_str("; Ramming start\n");
        }
        let cooling_retract = if semm {
            config.cooling_tube_retraction.0 as f32
        } else {
            0.0
        };
        let tube = if semm {
            config.cooling_tube_length.0 as f32
        } else {
            0.0
        };
        if enabled && semm && (cooling_retract != 0.0 || tube != 0.0) {
            writer.gcode.push_str("; Retract(unload)\n");
            let distance = cooling_retract + tube / 2.0 - 15.0;
            writer.suppress_preview = true;
            writer.load(-15.0, filament.unloading_start * 60.0);
            writer.load(-0.70 * distance, filament.unloading_speed * 60.0);
            writer.load(-0.20 * distance, 0.5 * filament.unloading_speed * 60.0);
            writer.load(-0.10 * distance, 0.3 * filament.unloading_speed * 60.0);
            writer.suppress_preview = false;
        }
        let cooling = enabled && semm && filament.cooling_moves > 0 && tube != 0.0;
        let mut change_later = false;
        if semm
            && new_temperature != 0
            && (new_temperature != self.old_temperature || self.is_first_layer() || self.mk4_mmu3)
        {
            if self.mk4_mmu3 && cooling {
                change_later = true;
            } else {
                writer.set_temperature(new_temperature, false, true);
            }
            self.old_temperature = new_temperature;
        }
        if cooling {
            self.cooling_moves(
                writer,
                bounds,
                y_step,
                change_later.then_some(new_temperature),
            );
        }
        if enabled && semm {
            writer.gcode.push_str("; Cooling park\n");
            writer.wait(filament.delay);
            let amount = -tube / 2.0 + config.parking_pos_retraction.0 as f32 - cooling_retract;
            if amount != 0.0 {
                writer.load(-amount, 2000.0);
            }
        }
        if enabled {
            writer.gcode.push_str("; Ramming end\n");
        }
        if do_ramming {
            writer.travel(
                [
                    end[0],
                    end[1] + (y_step / self.plan.ramming_spacing - perimeter) / 2.0 + perimeter,
                ],
                2400.0,
            );
        } else {
            writer.position = [
                end[0],
                bounds.lower[1]
                    + self.depth_traversed
                    + self.wipe_start_offset_after_ram(0.0, self.is_first_layer())
                    + perimeter,
            ];
        }
        writer.suppress_preview = false;
        writer
            .gcode
            .push_str(Writer::flush_planner_queue_command(self.flavor));
    }

    fn ramming_first_line(&self, y_step: f32) -> Option<f32> {
        let perimeter = self.plan.perimeter_width;
        let previous = &self.plan.layers[self.layer_index - 1];
        let sparse_start = previous.changes_depth() + perimeter;
        let mut depth = 0.0;
        for change in &self.plan.layers[self.layer_index].changes {
            if change.old_tool != self.current_tool {
                depth += change.required_depth;
                continue;
            }
            depth += change.ramming_depth;
            let end_y = depth - (y_step / self.plan.ramming_spacing - perimeter) / 2.0;
            return (end_y > sparse_start + 0.5 * perimeter).then_some(change.first_wipe_line);
        }
        None
    }

    fn cooling_moves(
        &self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        y_step: f32,
        change_temperature: Option<i32>,
    ) {
        let filament = &self.filaments[self.current_tool];
        let perimeter = self.plan.perimeter_width;
        let xl = bounds.lower[0] + perimeter;
        let xr = bounds.upper[0] - perimeter;
        let tube = self.settings.printer.gcode.cooling_tube_length.0 as f32;
        writer.gcode.push_str("; Cooling\n");
        let increment = (filament.cooling_final - filament.cooling_initial)
            / (2.0 * filament.cooling_moves as f32 - 1.0);
        if self.mk4_mmu3 {
            writer.disable_linear_advance();
        }
        writer.suppress_preview = true;
        writer.travel([writer.position[0], writer.position[1] + y_step], 0.0);
        let old_x = writer.position[0];
        let turning = if xr - old_x > old_x - xl { xr } else { xl };
        let stamping = filament.stamping_distance + tube / 2.0;
        for i in 0..filament.cooling_moves {
            if i > 0 && filament.stamping_distance != 0.0 {
                self.cooling_stamp(writer, bounds, [old_x, turning], stamping);
            }
            if i == filament.cooling_moves - 1
                && let Some(temperature) = change_temperature
            {
                writer.set_temperature(temperature, false, true);
            }
            let speed = filament.cooling_initial + increment * 2.0 * i as f32;
            writer.load_move_x(turning, tube, speed, 50.0);
            writer.load_move_x(old_x, -tube, speed + increment, 50.0);
        }
    }

    fn cooling_stamp(
        &self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        stroke: [f32; 2],
        stamping: f32,
    ) {
        let [old_x, turning] = stroke;
        let filament = &self.filaments[self.current_tool];
        let perimeter = self.plan.perimeter_width;
        let xl = bounds.lower[0] + perimeter;
        let xr = bounds.upper[0] - perimeter;
        let travel_speed = self.travel_speed;
        let turn = (old_x + 20.0 * if turning - old_x > 0.0 { 1.0 } else { -1.0 }).clamp(xl, xr);
        if stamping > 5.0 {
            let center = writer.position[0];
            writer.load_move_x(turn, stamping - 5.0, filament.stamping_speed, 200.0);
            writer.load_move_x(center, 5.0, filament.stamping_speed, travel_speed);
            writer.travel([center, writer.position[1]], 0.0);
        } else {
            writer.load_move_x_there_and_back(
                turn,
                stamping,
                filament.stamping_speed,
                travel_speed,
            );
        }
        writer.load(-stamping, filament.unloading_speed * 60.0);
    }
}
