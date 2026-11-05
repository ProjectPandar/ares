//! `Print.cpp:3537–3571`: retain Type1 generated results and accounting.
use super::super::{super::tool_ordering::LayerTools, Data, writer::Writer as CommandWriter};
use super::{generator::Generator, planning::Planned, result::Result};
use crate::{
    ProjectSettings,
    geometry::{BoundingBox, ClipperError, CoordinateScale, Point, Polyline},
};
use std::fmt::Write;

impl Planned {
    pub(in crate::project_slice::gcode_emit) fn generate(
        self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
        last: &LayerTools,
        normal_height: f64,
    ) -> std::result::Result<Data, ClipperError> {
        let mut generator = Generator::new(settings, scale, self);
        let changes = generator
            .generate()?
            .into_iter()
            .map(|row| row.into_iter().map(Result::into_common).collect())
            .collect();
        // Type1's legacy tool_change(-1) has a disabled unload body. Retain its
        // real result/accounting, but finalize dispatch must never emit it.
        if last.wipe_tower_partitions == 0 {
            generator.z = last.print_z as f32;
            generator.height = normal_height as f32;
        } else if generator.finished {
            generator.z = (last.print_z + normal_height) as f32;
            generator.height = normal_height as f32;
        }
        let mut writer = generator.writer(generator.current);
        write!(
            writer.gcode,
            ";--------------------\n; CP TOOLCHANGE START\n; toolchange #{}\nM220 S100\n",
            generator.changes + 1
        )
        .unwrap();
        writer.feedrate(settings.process.gcode.travel_speed.0 as f32 * 60.0);
        CommandWriter::flush_planner_queue(&mut writer.gcode, settings.printer.gcode.gcode_flavor);
        writer
            .gcode
            .push_str("G92 E0\n; CP TOOLCHANGE END\n;------------------\n\n\n");
        let mut final_purge = Result::from_writer(
            writer,
            generator.z,
            generator.height,
            generator.current,
            false,
        );
        final_purge.tool_change = true;
        final_purge.change_start = final_purge.common.start;
        let final_purge = final_purge.into_common();
        let outer_walls = generator
            .plan
            .layers
            .iter()
            .zip(generator.outer_walls)
            .filter(|(_, walls)| !walls.is_empty())
            .map(|(layer, walls)| {
                (
                    layer.z,
                    walls
                        .into_iter()
                        .map(|wall| {
                            let mut points = wall.into_points();
                            points.push(points[0]);
                            Polyline::new(points)
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        // WipeTower::get_bbx uses the first map entry, or a zero-point box.
        let first_layer_bounds = Some(outer_walls.first().map_or_else(
            || BoundingBox::new(Point::new(0, 0), Point::new(0, 0)),
            |(_, walls)| {
                BoundingBox::from_points(
                    walls.iter().flat_map(|wall| wall.points().iter().copied()),
                )
                .unwrap()
            },
        ));
        let mut depth_profile = vec![(0.0, generator.plan.depth)];
        for layer in &generator.plan.layers {
            if layer.depth < depth_profile.last().unwrap().1 - 1e-3 {
                depth_profile.push((layer.z, layer.depth));
            }
        }
        if depth_profile.last().unwrap().0 < generator.plan.height - 1e-3 {
            depth_profile.push((generator.plan.height, 0.0));
        }
        Ok(Data {
            type1: true,
            rib_offset: generator.rib_offset,
            priming: Vec::new(),
            changes,
            final_purge: Some(final_purge),
            width: generator.plan.width,
            depth: generator.plan.depth,
            height: generator.plan.height,
            brim_width: generator.brim,
            rib_width: generator.plan.rib_width,
            rib_length: generator.plan.rib_length,
            depth_profile,
            first_layer_bounds,
            outer_walls,
            used_filament: generator.used,
            tool_changes: generator.changes,
        })
    }
}
