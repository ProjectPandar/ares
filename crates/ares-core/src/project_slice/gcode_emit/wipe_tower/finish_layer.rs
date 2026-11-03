//! `WipeTower2::finish_layer`: sparse/solid support, wall, brim and wipe path.
use super::{
    generator::Generator,
    geometry::{BoxCoordinates, ConeOptions, gaps, new_scale, unscaled},
    result::ToolChangeResult,
    writer::Writer,
};
use crate::{
    ProcessWipeTowerWallType,
    geometry::{ClipperError, CoordinateScale, JoinType, Polygon, raw_offset_paths},
};

impl Generator<'_> {
    pub(super) fn finish_layer(&mut self) -> Result<ToolChangeResult, ClipperError> {
        self.layer_finished = true;
        let old_tool = self.current_tool;
        let mut writer = self.writer();
        writer.set_y_shift(self.y_shift);
        let first_layer = self.is_first_layer() || (self.tool_changes <= 1 && self.plan.no_sparse);
        let max_speed = self.settings.process.print.wipe_tower_max_purge_speed.0 as f32;
        let mut feedrate = if first_layer {
            self.first_layer_speed() * 60.0
        } else {
            (max_speed * 60.0).min(self.infill_speed() * 60.0)
        };
        if self
            .settings
            .process
            .print
            .enable_tower_interface_features
            .0
            && self.previous_interface
        {
            feedrate = feedrate.min(1200.0);
        }
        let layer = &self.plan.layers[self.layer_index];
        let layer_height = layer.height;
        let perimeter = self.plan.perimeter_width;
        let current_depth = layer.depth - layer.changes_depth();
        let bounds = BoxCoordinates::new(
            [perimeter, layer.depth - (current_depth - perimeter)],
            [self.plan.width - 2.0 * perimeter, current_depth - perimeter],
        );
        writer.set_initial_position(
            if self.left_to_right {
                bounds.upper
            } else {
                [bounds.lower[0], bounds.upper[1]]
            },
            [self.plan.width, self.plan.depth],
            self.rotation,
        );
        let toolchanges_on_layer = layer.changes_depth() > 1e-3;
        if bounds.size()[1] > perimeter - 1e-3 {
            writer.rectangle(bounds.lower, bounds.size(), feedrate);
        }
        if writer.position[0] > bounds.lower[0] + 1e-4 {
            writer.travel([bounds.lower[0], writer.position[1]], 0.0);
        }
        if writer.position[1] > bounds.lower[1] + 1e-4 {
            writer.travel([writer.position[0], bounds.lower[1]], 0.0);
        }
        if bounds.size()[1] - perimeter > perimeter {
            self.finish_grid(&mut writer, bounds, first_layer, feedrate);
        }
        let spacing = perimeter - self.layer_height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
        feedrate = if first_layer {
            self.first_layer_speed() * 60.0
        } else {
            (max_speed * 60.0).min(self.perimeter_speed() * 60.0)
        };
        let wall_bounds =
            BoxCoordinates::new([0.0, 0.0], [self.plan.width, layer.depth + perimeter]);
        let shape = self.shape();
        let mut polygon = match self.settings.process.print.wipe_tower_wall_type {
            ProcessWipeTowerWallType::Cone => shape.cone_wall(
                wall_bounds,
                &mut writer,
                ConeOptions {
                    angle: self.settings.process.print.wipe_tower_cone_angle.0 as f32,
                    z: f64::from(if self.plan.no_sparse {
                        self.current_height + layer.height
                    } else {
                        layer.z
                    }),
                    spacing,
                    infill: first_layer
                        && self.plan.width > 2.0 * spacing
                        && self.plan.depth > 2.0 * spacing,
                    feedrate,
                    scale: self.scale,
                },
            )?,
            ProcessWipeTowerWallType::Rib | ProcessWipeTowerWallType::Rectangle => {
                let rib = self.settings.process.print.wipe_tower_wall_type
                    == ProcessWipeTowerWallType::Rib;
                let wall = shape.wall_polygon(wall_bounds, rib, self.scale)?;
                // `WipeTower2.cpp:2663-2676`: with the gap wall, the layer's skip points
                // (none during the planning passes) open the wall.
                let skip_points = match self.wall_skip_points.get(self.layer_index) {
                    Some(points) if self.use_gap_wall() => points.as_slice(),
                    _ => &[],
                };
                let (paths, wall) = gaps::split(
                    &wall,
                    skip_points,
                    self.plan.width,
                    (2.5 * f64::from(self.plan.perimeter_width)) as f32,
                    self.scale,
                );
                let filament = &self.filaments[self.current_tool];
                writer.generate_path(
                    &paths,
                    feedrate,
                    (filament.retract_length, filament.retract_speed * 60.0),
                    self.scale,
                );
                // `WipeTower2.cpp:2679-2684`: the shift that puts the first-layer rib wall's
                // minimum corner at the tower position.
                if rib && self.is_first_layer() {
                    let bounds = crate::geometry::BoundingBox::from_polygon(&wall)
                        .expect("the rib wall has points");
                    let factor = self.scale.factor() as f32;
                    self.rib_offset = [
                        -(bounds.min().x() as f32 * factor),
                        -(bounds.min().y() as f32 * factor),
                    ];
                }
                wall
            }
        };
        if first_layer {
            polygon = self.finish_brim(&mut writer, polygon, spacing)?;
        }
        let closest = closest(&polygon, writer.position, self.scale);
        let previous = (closest + polygon.points().len() - 1) % polygon.points().len();
        writer.add_wipe_point(writer.position);
        writer.add_wipe_point(unscaled(polygon.points()[previous], self.scale));
        if !self.plan.no_sparse || toolchanges_on_layer || first_layer {
            self.used_filament[self.current_tool] += writer.take_used_filament();
            self.current_height += layer_height;
        }
        Ok(self.result(writer, old_tool, super::result::Kind::Finish))
    }
    fn finish_grid(
        &self,
        writer: &mut Writer,
        bounds: BoxCoordinates,
        first_layer: bool,
        feedrate: f32,
    ) {
        let perimeter = self.plan.perimeter_width;
        let dy = bounds.size()[1] - perimeter;
        let mut left = bounds.lower[0] + 2.0 * perimeter;
        let mut right = bounds.upper[0] - 2.0 * perimeter;
        writer.travel([bounds.lower[0] + 2.0 * perimeter, bounds.lower[1]], 0.0);
        writer
            .gcode
            .push_str(";--------------------\n; CP EMPTY GRID START\n");
        writer
            .gcode
            .push_str(&format!("; layer #{}\n", self.layer_changes + 1));
        let solid = first_layer
            || self
                .plan
                .layers
                .get(self.layer_index + 1)
                .is_some_and(|next| {
                    next.changes.iter().any(|c| {
                        self.filaments[c.new_tool].soluble || self.filaments[c.old_tool].soluble
                    })
                });
        if solid {
            let mut sparse_factor = 1.5;
            if first_layer {
                left -= perimeter;
                right += perimeter;
                sparse_factor = 1.0;
            }
            let mut y = bounds.lower[1] + perimeter;
            let count = (dy / (perimeter * sparse_factor)) as i32;
            let spacing = (dy - perimeter) / (count - 1) as f32;
            for index in 0..count {
                writer.extrude([writer.position[0], y], feedrate);
                let x = match index % 2 {
                    0 => right,
                    _ => left,
                };
                writer.extrude([x, y], 0.0);
                y += spacing;
            }
            writer.extrude([writer.position[0], bounds.upper[1]], 0.0);
        } else {
            writer.extrude(
                [bounds.lower[0] + 2.0 * perimeter, bounds.upper[1]],
                feedrate,
            );
            let count = 1
                + ((right - left) / self.settings.process.print.wipe_tower_bridging.0 as f32)
                    as i32;
            let dx = (right - left) / count as f32;
            for index in 1..=count {
                let x = left + dx * index as f32;
                writer.travel([x, writer.position[1]], 0.0);
                let y = match index % 2 {
                    0 => bounds.upper[1],
                    _ => bounds.lower[1],
                };
                writer.extrude([x, y], 0.0);
            }
        }
        writer
            .gcode
            .push_str("; CP EMPTY GRID END\n;------------------\n\n\n\n\n\n\n");
    }

    fn finish_brim(
        &mut self,
        writer: &mut Writer,
        mut polygon: Polygon,
        spacing: f32,
    ) -> Result<Polygon, ClipperError> {
        writer.gcode.push_str("; WIPE_TOWER_BRIM_START\n");
        let mut brim = self.settings.process.print.prime_tower_brim_width.0 as f32;
        if brim < 0.0 {
            brim = if self.plan.height < 100.0 {
                self.plan.height / 100.0 * 8.0
            } else {
                8.0
            };
        }
        let count = ((brim + spacing / 2.0) / spacing) as usize;
        for _ in 0..count {
            polygon = raw_offset_paths(
                std::slice::from_ref(&polygon),
                (f64::from(spacing) / self.scale.factor()) as f32,
                JoinType::Miter,
                3.0,
            )?
            .remove(0);
            let start = closest(&polygon, writer.position, self.scale);
            writer.travel(unscaled(polygon.points()[start], self.scale), 0.0);
            for step in 1..=polygon.points().len() {
                writer.extrude(
                    unscaled(
                        polygon.points()[(start + step) % polygon.points().len()],
                        self.scale,
                    ),
                    0.0,
                );
            }
        }
        writer.gcode.push_str("; WIPE_TOWER_BRIM_END\n");
        self.brim_width = count as f32 * spacing;
        self.first_layer_bounds = crate::geometry::BoundingBox::from_polygon(&polygon);
        Ok(polygon)
    }
}

fn closest(polygon: &Polygon, position: [f32; 2], scale: CoordinateScale) -> usize {
    let anchor = new_scale(position, scale);
    polygon
        .points()
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let distance = |p: &&crate::geometry::Point| {
                let dx = (p.x() - anchor.x()) as f64;
                let dy = (p.y() - anchor.y()) as f64;
                dx * dx + dy * dy
            };
            distance(a).total_cmp(&distance(b))
        })
        .unwrap()
        .0
}
