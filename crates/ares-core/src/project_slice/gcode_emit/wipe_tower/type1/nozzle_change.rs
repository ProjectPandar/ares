//! `WipeTower.cpp:2915–3033`: old-nozzle extrusion before Type1 filament change.
use super::{generator::Generator, result::NozzleChange, writer::Writer};
use crate::geometry::ClipperError;
use std::fmt::Write;

impl Generator<'_> {
    pub(super) fn change_nozzle(
        &mut self,
        old: usize,
        new: usize,
        solid: bool,
    ) -> Result<NozzleChange, ClipperError> {
        let width = 2.0 * self.plan.perimeter;
        let change = self.plan.layers[self.index]
            .changes
            .iter()
            .find(|c| c.old == old && c.new == new);
        let step = if self.first_layer() || solid {
            width
        } else {
            self.plan.layers[self.index].spacing * change.map_or(width, |c| c.nozzle_gap)
        };
        let count = change.map_or(0, |c| ((c.nozzle_depth + 1e-3) / step) as i32);
        let flow_at = |height: f32| {
            height * (width - height * (1.0 - std::f32::consts::PI / 4.0)) / self.filaments[0].area
        };
        let flow = flow_at(self.height);
        let bridge_flow = flow_at(0.2);
        let mut speed = if solid {
            2400.0
        } else {
            60.0 * self.filaments[self.current].max_e_speed / flow
        };
        let tpu = self.filaments[self.current].material == "TPU";
        if tpu {
            speed *= 0.25;
        }
        let bridge_speed =
            (60.0 * self.filaments[self.current].max_e_speed / bridge_flow).min(speed);
        let mut writer = Writer::new(self.height, width, self.filaments[0].area, self.current);
        writer.flow = flow;
        writer.max_e_speed = self.filaments[self.current].max_e_speed;
        writer.set_y_shift(self.y_shift);
        writer.gcode.push_str("; Nozzle change start\n");
        let block = self
            .plan
            .blocks
            .iter()
            .position(|b| b.category == self.filaments[old].category)
            .expect("old nozzle has a planned category block");
        self.current_block = Some(block);
        let offset = self.plan.perimeter + (width - self.plan.perimeter) / 2.0;
        let lower = [
            offset,
            self.plan.blocks[block].current + (width - self.plan.perimeter) / 2.0,
        ];
        let right = self.plan.width - offset;
        let upper = lower[1] + count as f32 * step - (width - self.plan.perimeter) / 2.0;
        writer.set_position(lower, [self.plan.width, self.plan.depth], 0.0);
        let count = if solid { i32::MAX } else { count };
        self.left_to_right = true;
        let mut real_count = 0;
        let mut thick = false;
        loop {
            if self.thick_bridge(writer.position[1]) {
                writer.flow = bridge_flow;
                writer.gcode.push_str("; LAYER_HEIGHT: 0.200000\n");
                thick = true;
            }
            writer.extrude(
                [
                    if self.left_to_right { right } else { lower[0] },
                    writer.position[1],
                ],
                if thick { bridge_speed } else { speed },
            );
            real_count += 1;
            if real_count == count
                || writer.position[1] + step - upper + (width + self.plan.perimeter) / 2.0
                    > 1e-4_f32
            {
                break;
            }
            if thick {
                writer.flow = flow;
                writeln!(writer.gcode, "; LAYER_HEIGHT: {:.6}", self.height).unwrap();
                thick = false;
            }
            writer.extrude([writer.position[0], writer.position[1] + step], speed);
            self.left_to_right = !self.left_to_right;
        }
        if thick {
            writeln!(writer.gcode, "; LAYER_HEIGHT: {:.6}", self.height).unwrap();
        }
        writer.flow = flow;
        self.plan.blocks[block].current += real_count as f32 * step;
        self.plan.blocks[block].last_nozzle = Some(old);
        let mut wipe_path = Vec::new();
        if tpu {
            let count = (real_count + 1) / 2;
            speed *= 2.0;
            writer.travel([writer.position[0], writer.position[1] - width], 0.0);
            writer.travel_lines(
                [lower[0] + self.plan.perimeter, right - self.plan.perimeter],
                (count, !self.left_to_right),
                (speed, -step),
            );
        } else {
            let position = writer.rotated(writer.position);
            wipe_path.push(position);
            wipe_path.push([
                if self.left_to_right {
                    0.0
                } else {
                    self.plan.width
                },
                position[1],
            ]);
        }
        writer.gcode.push_str("; Nozzle change end\n");
        Ok(NozzleChange {
            start: writer.start,
            origin_start: lower,
            end: writer.rotated(writer.position),
            gcode: writer.gcode,
            wipe_path,
        })
    }
}
