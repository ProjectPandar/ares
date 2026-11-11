//! `WipeTower.cpp:3390–3514`: Type1 zigzag, entry ironing and bridge-flow state.
use super::super::{filament::at, geometry::BoxCoordinates};
use super::{generator::Generator, writer::Writer};
use std::fmt::Write;

fn entry(
    writer: &mut Writer,
    filament: &super::filament::Filament,
    (left_to_right, ironing): (bool, bool),
    (perimeter, speed): (f32, f32),
    end_x: f32,
) {
    let retract = filament.retract_length;
    let retract_speed = filament.retract_speed * 60.0;
    let area = filament.ironing_area;
    let distance = (end_x - writer.position[0]).abs().min(3.0);
    let sign = if left_to_right { 1.0 } else { -1.0 };
    writer.extrude(
        [writer.position[0] + sign * distance, writer.position[1]],
        speed,
    );
    writer.load(-retract, retract_speed);
    writer.travel(
        [
            (f64::from(writer.position[0]) - sign as f64 * 1.5 * f64::from(distance)) as f32,
            writer.position[1],
        ],
        600.0,
    );
    if ironing && area > 0.0 {
        writer.travel(
            [
                writer.position[0] + sign * 0.5 * distance,
                writer.position[1],
            ],
            240.0,
        );
        let destination = [writer.position[0] + sign * distance, writer.position[1]];
        writer.spiral_ironing(area, perimeter, 600.0);
        writer.travel(destination, speed);
    } else {
        writer.travel(
            [
                (f64::from(writer.position[0]) + sign as f64 * 1.5 * f64::from(distance)) as f32,
                writer.position[1],
            ],
            240.0,
        );
    }
    writer.load(retract, retract_speed);
}

impl Generator<'_> {
    pub(super) fn thick_bridge(&self, y: f32) -> bool {
        if self.height >= 0.2 || self.index <= 1 {
            return false;
        }
        self.current_block.is_some_and(|i| {
            let block = &self.plan.blocks[i];
            y > block.finish_depth[self.index - 1]
                && y < block.start + block.depth - self.plan.perimeter
        })
    }

    pub(super) fn wipe(
        &mut self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        length: f32,
        solid: bool,
    ) {
        let perimeter = self.plan.perimeter;
        let first_layer_ratio = if self.settings.process.object.set_other_flow_ratios.0 {
            self.settings.process.region.first_layer_flow_ratio.0 as f32
        } else {
            1.0
        };
        writer.flow = self.flow
            * if self.first_layer() {
                first_layer_ratio
            } else {
                1.0
            };
        writeln!(
            writer.gcode,
            "; CP_TOOLCHANGE_WIPE CT{} FL{}",
            u8::from(solid),
            u8::from(self.first_layer())
        )
        .unwrap();
        if !self.nozzle_change.gcode.is_empty() {
            writeln!(writer.gcode, "; LINE_WIDTH: {perimeter:.6}").unwrap();
        }
        if self.first_layer() {
            writeln!(
                writer.gcode,
                "; LINE_WIDTH: {:.6}",
                first_layer_ratio * perimeter
            )
            .unwrap();
        }
        let filament = &self.filaments[self.current];
        let should_heat = !solid
            && !self.first_layer()
            && matches!(
                at(&self.settings.filament.gcode.filament_cooling_before_tower, self.current),
                crate::Nullable::Value(value) if value.0 as f32 > 1e-4
            );
        let config = &self.settings.process.print;
        let ironing = config.prime_tower_flat_ironing.0
            && config.prime_tower_skip_points.0
            && (!solid || !config.enable_tower_interface_features.0);
        let left = bounds.lower[0];
        let right = bounds.upper[0];
        let step = if solid || self.first_layer() {
            perimeter
        } else {
            self.plan.layers[self.index].spacing * self.plan.infill_gap
        };
        let mut remaining = if solid { f32::MAX } else { length };
        let target = if solid {
            1200.0
        } else if self.first_layer() {
            (self.first_speed() * 60.0).min(self.max_speed())
        } else {
            self.max_speed()
        };
        let speeds = [
            0.33 * target,
            0.375 * target,
            0.458 * target,
            0.875 * target,
            target.min(0.875 * target + 50.0),
        ];
        self.left_to_right = (self.index + 3) % 4 >= 2;
        let from_up = self.index % 2 == 1;
        let mut index = 0_usize;
        loop {
            let speed = speeds[index.min(4)];
            let thick = self.thick_bridge(writer.position[1]);
            if thick {
                writer.flow = self.flow_at(0.2);
                writer.gcode.push_str("; LAYER_HEIGHT: 0.200000\n");
            }
            let end_x = if self.left_to_right { right } else { left };
            if index == 0 && config.prime_tower_skip_points.0 {
                entry(
                    writer,
                    filament,
                    (self.left_to_right, ironing),
                    (perimeter, speed),
                    end_x,
                );
            }
            if index == 0 && should_heat {
                writer.format_line_m104(
                    filament.temperature,
                    self.settings
                        .printer
                        .gcode
                        .physical_extruder_map
                        .0
                        .get(self.plan.nozzle_map[self.current])
                        .map(|tool| tool.0),
                );
            }
            writer.extrude([end_x, writer.position[1]], speed);
            if thick {
                writer.flow = self.flow;
                writeln!(writer.gcode, "; LAYER_HEIGHT: {:.6}", self.height).unwrap();
            }
            if (!from_up && writer.position[1] + step - 1e-4_f32 > bounds.upper[1] - perimeter)
                || (from_up && writer.position[1] - step + 1e-4_f32 < bounds.lower[1])
            {
                break;
            }
            remaining -= right - left;
            if remaining < 1e-3 {
                break;
            }
            writer.extrude(
                [
                    writer.position[0],
                    writer.position[1] + if from_up { -step } else { step },
                ],
                0.0,
            );
            self.left_to_right = !self.left_to_right;
            index += 1;
        }
        writer.add_wipe(writer.position);
        writer.add_wipe([
            if !self.left_to_right {
                self.plan.width
            } else {
                0.0
            },
            writer.position[1],
        ]);
        if self.current != self.plan.layers[self.index].changes.last().unwrap().new {
            self.left_to_right = !self.left_to_right;
        }
        writer.flow = self.flow;
        if self.first_layer() {
            writeln!(writer.gcode, "; LINE_WIDTH: {perimeter:.6}").unwrap();
        }
    }
}
