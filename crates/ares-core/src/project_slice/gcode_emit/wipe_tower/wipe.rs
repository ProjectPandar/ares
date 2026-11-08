//! `WipeTower2::toolchange_Wipe` row stepping and material ironing.
use super::{
    generator::Generator, geometry::BoxCoordinates, plan::volume_to_length, writer::Writer,
};

fn edge_x([left, right]: [f32; 2], left_to_right: bool, inset: f32) -> f32 {
    if left_to_right {
        right - inset
    } else {
        left + inset
    }
}

fn next_speed(speed: f32, target: f32) -> f32 {
    if speed < 0.34 * target {
        0.375 * target
    } else if f64::from(speed) < 0.377 * f64::from(target) {
        0.458 * target
    } else if speed < 0.46 * target {
        0.875 * target
    } else {
        target.min(speed + 50.0)
    }
}

impl Generator<'_> {
    pub(super) fn wipe_row_spacing(&self, first_layer: bool) -> f32 {
        (if first_layer {
            self.plan.extra_flow
        } else {
            self.plan.extra_spacing
        }) * self.plan.perimeter_width
    }

    pub(super) fn wipe_start_offset_after_ram(&self, ramming_depth: f32, first_layer: bool) -> f32 {
        ramming_depth + self.wipe_row_spacing(first_layer)
            - (self.plan.perimeter_width + self.plan.perimeter_width * self.plan.extra_flow) / 2.0
    }

    pub(super) fn wipe(
        &mut self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        volume: f32,
        (interface, fill_box): (bool, bool),
    ) {
        writer.gcode.push_str("; CP TOOLCHANGE WIPE\n");
        writer.set_flow(self.extrusion_flow * self.plan.extra_flow);
        let line_width = self.plan.perimeter_width * self.plan.extra_flow;
        writer.change_width(line_width);
        let mut remaining = volume_to_length(volume, self.plan.perimeter_width, self.layer_height)
            / self.plan.extra_flow;
        let dy = self.wipe_row_spacing(self.is_first_layer());
        let target = if self.is_first_layer() || (self.tool_changes <= 1 && self.plan.no_sparse) {
            self.first_layer_speed() * 60.0
        } else {
            (self.settings.process.print.wipe_tower_max_purge_speed.0 as f32 * 60.0)
                .min(self.infill_speed() * 60.0)
        };
        let mut speed = 0.33 * target;
        let [xl, xr] = [bounds.lower[0], bounds.upper[0]];
        let edge_distance = if self.left_to_right {
            xr - writer.position[0]
        } else {
            writer.position[0] - xl
        };
        if edge_distance < 2.5 * line_width {
            writer.travel(
                [
                    edge_x([xl, xr], self.left_to_right, line_width),
                    writer.position[1] + dy,
                ],
                0.0,
            );
            self.left_to_right = !self.left_to_right;
        }
        let ironing = self.settings.process.print.prime_tower_flat_ironing.0
            && (!interface
                || !self
                    .settings
                    .process
                    .print
                    .enable_tower_interface_features
                    .0);
        let ironing_area = self.filaments[self.current_tool].ironing_area;
        let mut index = 0;
        loop {
            if index != 0 {
                speed = next_speed(speed, target);
            }
            let previous_x = writer.position[0];
            let inset = match (self.left_to_right, index % 4) {
                (true, 0) | (false, 1) => 0.0,
                _ => 1.5 * line_width,
            };
            let x = edge_x([xl, xr], self.left_to_right, inset);
            writer.extrude([x, writer.position[1]], speed);
            if index == 0 && ironing && ironing_area > 0.0 {
                writer.travel(writer.position, 600.0);
                writer.spiral_flat_ironing(
                    writer.position,
                    ironing_area,
                    self.plan.perimeter_width,
                    600.0,
                );
            }
            if writer.position[1] + 1e-4 > bounds.upper[1] - 0.5 * line_width {
                break;
            }
            remaining -= (previous_x - writer.position[0]).abs();
            if !fill_box && remaining < 1e-3 {
                writer.travel(
                    [
                        edge_x([xl, xr], !self.left_to_right, 1.5 * line_width),
                        writer.position[1],
                    ],
                    7200.0,
                );
                break;
            }
            let side_step = if index % 4 == 0 {
                -1.0
            } else if index % 4 == 1 {
                1.0
            } else {
                0.0
            };
            writer.extrude(
                [
                    writer.position[0] + side_step * 1.5 * line_width,
                    writer.position[1] + dy,
                ],
                0.0,
            );
            self.left_to_right = !self.left_to_right;
            index += 1;
        }
        for point in [
            writer.position,
            [writer.position[0], writer.position[1] - dy],
            [
                if !self.left_to_right {
                    self.plan.width
                } else {
                    0.0
                },
                writer.position[1] - dy,
            ],
        ] {
            writer.add_wipe_point(point);
        }
        if let Some(layer) = self.plan.layers.get(self.layer_index)
            && self.current_tool != layer.changes.last().unwrap().new_tool
        {
            self.left_to_right = !self.left_to_right;
        }
        writer.set_flow(self.extrusion_flow);
        writer.change_width(self.plan.perimeter_width);
    }
}
