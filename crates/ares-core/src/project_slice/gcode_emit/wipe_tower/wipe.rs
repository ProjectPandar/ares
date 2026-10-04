//! `WipeTower2::toolchange_Wipe` row stepping and material ironing.
use super::{
    generator::Generator, geometry::BoxCoordinates, plan::volume_to_length, writer::Writer,
};

impl Generator<'_> {
    pub(super) fn wipe(
        &mut self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        volume: f32,
        interface: bool,
    ) {
        writer.gcode.push_str("; CP TOOLCHANGE WIPE\n");
        writer.set_flow(self.extrusion_flow * self.plan.extra_flow);
        let line_width = self.plan.perimeter_width * self.plan.extra_flow;
        writer.change_width(line_width);
        let mut remaining = volume_to_length(volume, self.plan.perimeter_width, self.layer_height)
            / self.plan.extra_flow;
        let dy = if self.is_first_layer() {
            self.plan.extra_flow
        } else {
            self.plan.extra_spacing
        } * self.plan.perimeter_width;
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
                    if self.left_to_right {
                        xr - line_width
                    } else {
                        xl + line_width
                    },
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
                speed = if speed < 0.34 * target {
                    0.375 * target
                } else if f64::from(speed) < 0.377 * f64::from(target) {
                    0.458 * target
                } else if speed < 0.46 * target {
                    0.875 * target
                } else {
                    target.min(speed + 50.0)
                };
            }
            let previous_x = writer.position[0];
            let x = if self.left_to_right {
                xr - if index % 4 == 0 {
                    0.0
                } else {
                    1.5 * line_width
                }
            } else {
                xl + if index % 4 == 1 {
                    0.0
                } else {
                    1.5 * line_width
                }
            };
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
            if remaining < 1e-3 {
                writer.travel(
                    [
                        if self.left_to_right {
                            xl + 1.5 * line_width
                        } else {
                            xr - 1.5 * line_width
                        },
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
            writer.wipe_path.push(writer.rotated(point));
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
