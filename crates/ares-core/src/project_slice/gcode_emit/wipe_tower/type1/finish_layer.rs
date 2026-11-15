//! `WipeTower.cpp:3036–3211,4190–4246`: Type1 outer wall and brim consumers.
use super::super::geometry::BoxCoordinates;
use super::{generator::Generator, result::Result};
use crate::geometry::{ClipperError, JoinType, Point, Polygon, raw_offset_paths};

impl Generator<'_> {
    pub(super) fn finish_layer(
        &mut self,
        perimeter: bool,
        fill: bool,
        fill_wall: bool,
    ) -> std::result::Result<Result, ClipperError> {
        self.finished = true;
        let p = self.plan.perimeter;
        let depth = if self.plan.blocks.len() == 1 {
            self.plan.layers[self.index].depth
        } else {
            self.plan.depth
        };
        let bounds = BoxCoordinates::new([p, p], [self.plan.width - 2.0 * p, depth - 2.0 * p]);
        let mut writer = self.writer(self.current);
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
        let speed = self.finish_speed(self.current);
        let wipe = super::grid::fill(self, &mut writer, bounds, (fill, fill_wall), speed);
        let depth = if self.plan.blocks.len() == 1 {
            self.plan.layers[self.index].depth + p
        } else {
            self.plan.depth
        };
        let outer = self.align_box(BoxCoordinates::new([0.0; 2], [self.plan.width, depth]));
        let mut polygon = self.support_wall(&mut writer, outer, speed, perimeter)?;
        if perimeter {
            let shift = (f64::from(self.y_shift) / self.scale.factor()) as i64;
            self.outer_walls[self.index].push(Polygon::new(
                polygon
                    .points()
                    .iter()
                    .map(|point| Point::new(point.x(), point.y() + shift))
                    .collect(),
            ));
        }
        let spacing = p - self.height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
        let mut loops = ((self.plan.brim + spacing / 2.0) / spacing) as i32;
        if !self.first_layer() {
            if self.plan.layers[self.index].depth != self.plan.layers[0].depth {
                loops = 0;
            } else {
                // Upstream narrows the size_t subtraction, including its -1 sentinel, to int.
                let distance = self
                    .index
                    .wrapping_sub(self.plan.first_layer.unwrap_or(usize::MAX))
                    as i32;
                loops = loops.min((3.0 / spacing) as i32) - distance;
            }
        }
        if loops > 0 {
            // `scaled(spacing)` uses float division, then truncates to coord_t.
            let delta = (spacing / self.scale.factor() as f32) as i64;
            for _ in 0..loops {
                polygon = raw_offset_paths(
                    std::slice::from_ref(&polygon),
                    delta as f32,
                    JoinType::Miter,
                    3.0,
                )?
                .remove(0);
                writer.polygon(&polygon, speed, self.scale);
                self.outer_walls[self.index].push(polygon.clone());
            }
            if self.first_layer() {
                self.brim = loops as f32 * spacing + spacing / 2.0;
            }
        }
        if perimeter || loops > 0 {
            writer.polygon_wipe(
                &polygon,
                self.filaments[self.current].wipe_distance,
                self.scale,
            );
        } else {
            let corners = outer.corners();
            let target = wipe.filter(|path| path[0] == writer.position).map_or_else(
                || {
                    let index = corners
                        .iter()
                        .position(|&p| p == writer.position)
                        .unwrap_or(3);
                    corners[(index + 1) % 4]
                },
                |path| path[1],
            );
            writer.add_wipe(writer.position);
            writer.add_wipe(target);
        }
        writer.gcode.push_str("; WIPE_TOWER_END\n");
        self.nozzle_change.gcode.clear();
        Ok(self.finish_result(writer, self.current))
    }

    pub(super) fn only_outer_wall(&mut self) -> std::result::Result<Result, ClipperError> {
        let p = self.plan.perimeter;
        let layer = &self.plan.layers[self.index];
        let fill_y = layer.changes.iter().map(|c| c.depth).sum::<f32>() + p;
        let fill = BoxCoordinates::new(
            [p, fill_y],
            [self.plan.width - 2.0 * p, layer.depth - fill_y],
        );
        let mut writer = self.writer(self.current);
        writer.set_position(
            [
                if self.left_to_right {
                    fill.upper[0]
                } else {
                    fill.lower[0]
                },
                fill.upper[1],
            ],
            [self.plan.width, self.plan.depth],
            0.0,
        );
        writer.gcode.push_str("; WIPE_TOWER_START\n");
        let depth = if self.plan.smooth || self.plan.wrapping {
            self.plan.depth
        } else {
            layer.depth + p
        };
        let bounds = self.align_box(BoxCoordinates::new([0.0; 2], [self.plan.width, depth]));
        let wall = self.support_wall(&mut writer, bounds, self.finish_speed(self.current), true)?;
        self.outer_walls[self.index].push(wall.clone());
        writer.polygon_wipe(
            &wall,
            self.filaments[self.current].wipe_distance,
            self.scale,
        );
        writer.gcode.push_str("; WIPE_TOWER_END\n");
        let mut result = self.finish_result(writer, self.current);
        result.nozzle_change = self.nozzle_change.clone();
        Ok(result)
    }
}
