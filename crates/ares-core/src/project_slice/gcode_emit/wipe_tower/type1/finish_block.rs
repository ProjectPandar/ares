//! Type1 finish-layer/block shared infill (`WipeTower.cpp:3036–3387`).
use super::super::geometry::BoxCoordinates;
use super::{generator::Generator, result::Result, writer::Writer};
use crate::geometry::ClipperError;
use std::fmt::Write;

fn rectangle_loop(
    generator: &Generator<'_>,
    writer: &mut Writer,
    bounds: BoxCoordinates,
    closest: usize,
    speed: f32,
) {
    let corners = bounds.corners();
    let thick = generator.thick_bridge(bounds.lower[1]);
    let mut changed = false;
    for offset in 1..=4 {
        let i = (closest + offset) % 4;
        if thick && i == 1 {
            writer.flow = generator.flow_at(0.2);
            writer.gcode.push_str("; LAYER_HEIGHT: 0.200000\n");
            changed = true;
        } else if thick && i == 2 && changed {
            writer.flow = generator.flow;
            writeln!(writer.gcode, "; LAYER_HEIGHT: {:.6}", generator.height).unwrap();
        }
        writer.extrude(corners[i], speed);
    }
}

impl Generator<'_> {
    pub(super) fn align_box(&self, mut bounds: BoxCoordinates) -> BoxCoordinates {
        let step = self.plan.spacing * self.plan.perimeter;
        let up = (f64::from(bounds.upper[1] - self.plan.perimeter) - 1e-4) as f32;
        bounds.upper[1] = ((up / step).ceil() * step + self.plan.perimeter).min(self.plan.depth);
        let down = (f64::from(bounds.lower[1] - self.plan.perimeter) + 1e-4) as f32;
        bounds.lower[1] = ((down / step).floor() * step + self.plan.perimeter).max(-self.y_shift);
        bounds
    }

    pub(super) fn rectangle_fill(
        &self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        speed: f32,
    ) -> Option<[[f32; 2]; 2]> {
        let height = bounds.size()[1];
        let corners = bounds.corners();
        if height > self.plan.perimeter - 1e-3 {
            let right = writer.position[0] - bounds.lower[0] > bounds.upper[0] - writer.position[0];
            let top = writer.position[1] - bounds.lower[1] > bounds.upper[1] - writer.position[1];
            let closest = match (right, top) {
                (false, false) => 0,
                (true, false) => 1,
                (true, true) => 2,
                (false, true) => 3,
            };
            writer.travel([corners[closest][0], writer.position[1]], 0.0);
            writer.travel(corners[closest], 0.0);
            rectangle_loop(self, writer, bounds, closest, speed);
            Some([writer.position, corners[(closest + 1) % 4]])
        } else if height > 1e-3 {
            let thick = self.thick_bridge(bounds.lower[1]);
            if thick {
                writer.flow = self.flow_at(0.2);
                writer.gcode.push_str("; LAYER_HEIGHT: 0.200000\n");
            }
            let mut points = [corners[0], corners[1]];
            if (writer.position[0] - points[0][0]).abs() > (writer.position[0] - points[1][0]).abs()
            {
                points.swap(0, 1);
            }
            writer.travel([points[0][0], writer.position[1]], 0.0);
            writer.travel(points[0], 0.0);
            writer.extrude(points[1], 0.0);
            if thick {
                writer.flow = self.flow;
                writeln!(writer.gcode, "; LAYER_HEIGHT: {:.6}", self.height).unwrap();
            }
            Some([
                writer.position,
                if writer.position == corners[0] {
                    corners[1]
                } else {
                    corners[0]
                },
            ])
        } else {
            None
        }
    }

    pub(super) fn finish_block(
        &mut self,
        block: usize,
        tool: usize,
        fill: bool,
    ) -> std::result::Result<Result, ClipperError> {
        let block = &self.plan.blocks[block];
        let p = self.plan.perimeter;
        let bounds = BoxCoordinates::new(
            [p, block.current],
            [
                self.plan.width - 2.0 * p,
                block.start + block.layer_depths[self.index] - block.current - p,
            ],
        );
        let mut writer = self.writer(tool);
        writer.gcode.push_str("; WIPE_TOWER_START\n");
        writer.set_position(
            [
                if self.left_to_right {
                    bounds.upper[0]
                } else {
                    bounds.lower[0]
                },
                bounds.upper[1],
            ],
            [self.plan.width, self.plan.depth],
            0.0,
        );
        let wipe = super::grid::fill(
            self,
            &mut writer,
            bounds,
            (fill, true),
            self.finish_speed(tool),
        );
        let outer = self.align_box(BoxCoordinates::new(
            [0.0; 2],
            [self.plan.width, self.plan.layers[self.index].depth + p],
        ));
        let corners = outer.corners();
        let target = wipe.filter(|w| w[0] == writer.position).map_or_else(
            || {
                let i = corners
                    .iter()
                    .position(|&point| point == writer.position)
                    .unwrap_or(3);
                corners[(i + 1) % 4]
            },
            |w| w[1],
        );
        writer.add_wipe(writer.position);
        writer.add_wipe(target);
        writer.gcode.push_str("; WIPE_TOWER_END\n");
        Ok(self.finish_result(writer, tool))
    }

    pub(super) fn finish_block_solid(
        &mut self,
        block: usize,
        tool: usize,
        interface: bool,
    ) -> std::result::Result<Result, ClipperError> {
        let block = &self.plan.blocks[block];
        let p = self.plan.perimeter;
        let bounds = BoxCoordinates::new(
            [p, block.current],
            [
                self.plan.width - 2.0 * p,
                block.start + block.layer_depths[self.index] - block.current - p,
            ],
        );
        let thicker =
            self.index > 1 && !block.solid_infill[self.index - 1] && self.flow < self.flow_at(0.2);
        let mut writer = if thicker {
            let mut w = Writer::new(0.2, p, self.filaments[0].area, tool);
            w.flow = self.flow_at(0.2);
            w.max_e_speed = self.filaments[tool].max_e_speed;
            w.set_y_shift(self.y_shift);
            w
        } else {
            self.writer(tool)
        };
        writer.gcode.push_str("; WIPE_TOWER_START\n");
        writer.set_position(
            [
                if self.left_to_right {
                    bounds.upper[0]
                } else {
                    bounds.lower[0]
                },
                bounds.lower[1],
            ],
            [self.plan.width, self.plan.depth],
            0.0,
        );
        self.left_to_right = !self.left_to_right;
        writeln!(
            writer.gcode,
            ";--------------------\n; CP EMPTY GRID START\n; layer #{}",
            self.index + 2
        )
        .unwrap();
        let count =
            ((f64::from(bounds.size()[1]) + 0.25 * f64::from(p)) / f64::from(p) + 1.0) as i32;
        let speed = if interface {
            1200.0
        } else {
            self.finish_speed(tool)
        };
        let mut y = bounds.lower[1];
        for i in 0..count {
            writer.extrude(
                [
                    if self.left_to_right {
                        bounds.upper[0]
                    } else {
                        bounds.lower[0]
                    },
                    writer.position[1],
                ],
                speed,
            );
            let wipe_x = if self.left_to_right {
                bounds.lower[0]
            } else {
                bounds.upper[0]
            };
            if i == count - 1 {
                writer.add_wipe(writer.position);
                writer.add_wipe([wipe_x, writer.position[1]]);
                break;
            }
            self.left_to_right = !self.left_to_right;
            y += p;
            writer.extrude([writer.position[0], y], speed);
        }
        writer
            .gcode
            .push_str("; CP EMPTY GRID END\n;------------------\n\n\n\n\n\n\n; WIPE_TOWER_END\n");
        Ok(self.finish_result(writer, tool))
    }
}
