//! `WipeTower2::prime` and constructor bed classification.
use super::{
    generator::Generator, geometry::BoxCoordinates, result::ToolChangeResult, writer::Writer,
};

impl Generator<'_> {
    pub(super) fn prime(&mut self, height: f32, tools: &[usize]) -> Vec<ToolChangeResult> {
        self.set_layer(height, height);
        self.current_tool = tools[0];
        let (bed_width, bottom_left, circular) = self.priming_bed();
        let width = (0.9 * bed_width / tools.len() as f32).min(60.0);
        let mut bounds = BoxCoordinates::new(
            [0.02 * bed_width, 0.01 + self.plan.perimeter_width / 2.0],
            [width, 100.0],
        );
        if circular {
            bounds = BoxCoordinates::new([0.0; 2], [width, 100.0]);
            let half_width = tools.len() as f32 * width / 2.0;
            bounds.translate([
                -half_width,
                -((bed_width / 2.0).powi(2) - (1.05 * half_width).powi(2))
                    .max(0.0)
                    .sqrt(),
            ]);
        } else {
            bounds.translate(bottom_left);
        }
        let trimpot = self.settings.printer.gcode.single_extruder_multi_material.0
            && self.settings.printer.gcode.high_current_on_filament_swap.0;
        let mut results: Vec<ToolChangeResult> = Vec::with_capacity(tools.len());
        for (index, &tool) in tools.iter().enumerate() {
            let old_tool = self.current_tool;
            let mut writer = self.writer();
            if index == 0 {
                writer
                    .gcode
                    .push_str(";--------------------\n; CP PRIMING START\n;--------------------\n");
                writer.speed_override_backup();
                writer.speed_override(100);
                writer.set_initial_position([0.0; 2], [0.0; 2], 0.0);
                writer.travel(bounds.lower, 7200.0);
            } else {
                writer.set_initial_position(results.last().unwrap().end, [0.0; 2], 0.0);
            }
            if index == 0 && trimpot {
                writer.trimpot(750);
            }
            self.left_to_right = true;
            self.change(&mut writer, tool);
            self.load(&mut writer, bounds);
            if index + 1 == tools.len() {
                // No color transition exists for a single primed tool. The source
                // indexes tools[-1] here; do not reproduce that out-of-bounds read.
                let previous = tools[index.saturating_sub(1)];
                self.wipe(
                    &mut writer,
                    bounds,
                    self.wipe_volumes[previous][tool],
                    (false, true, false),
                );
            } else {
                self.wipe(&mut writer, bounds, 20.0, (false, true, false));
                let mut unload_box = bounds;
                unload_box.translate([
                    0.0,
                    writer.position[1] - bounds.lower[1] + self.plan.perimeter_width,
                ]);
                self.unload(
                    &mut writer,
                    unload_box,
                    self.filaments[self.current_tool].first_temperature,
                    self.filaments[tools[index + 1]].first_temperature,
                );
                bounds.translate([width, 0.0]);
                writer.travel(bounds.lower, 7200.0);
            }
            self.tool_changes += 1;
            self.used_filament[self.current_tool] += writer.take_used_filament();
            if index + 1 == tools.len() && trimpot {
                writer.trimpot(550);
            }
            if index + 1 == tools.len() {
                writer.speed_override_restore();
                writer.feedrate(self.settings.process.gcode.travel_speed.0 as f32 * 60.0);
                Writer::flush_planner_queue(&mut writer.gcode, self.flavor);
                writer
                    .gcode
                    .push_str("G92 E0\n; CP PRIMING END\n;------------------\n\n\n");
            }
            results.push(self.result(writer, old_tool, super::result::Kind::Priming));
        }
        self.old_temperature = -1;
        results
    }

    fn priming_bed(&self) -> (f32, [f32; 2], bool) {
        let points = &self.settings.printer.remaining.printable_area.0;
        let mut min = [points[0].x, points[0].y];
        let mut max = min;
        for point in points {
            min[0] = min[0].min(point.x);
            min[1] = min[1].min(point.y);
            max[0] = max[0].max(point.x);
            max[1] = max[1].max(point.y);
        }
        let width = (max[0] - min[0]) as f32;
        if points.len() == 4 {
            return (width, [points[0].x as f32, points[0].y as f32], false);
        }
        let radius_squared = f64::from(width / 2.0).powi(2);
        let limit_squared = f64::from(width / 10.0).powi(2);
        let center = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
        let circular = points.iter().all(|p| {
            ((p.x - center[0]).powi(2) + (p.y - center[1]).powi(2) - radius_squared).abs()
                <= limit_squared
        });
        (width, [0.0; 2], circular)
    }
}
