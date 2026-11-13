//! `WipeTower.cpp:2760–2913`: Type1 tool-change result and stock template slots.
use super::super::{geometry::BoxCoordinates, writer::Writer as CommandWriter};
use super::{generator::Generator, result::Result};
use crate::geometry::ClipperError;
use std::fmt::Write;

impl Generator<'_> {
    pub(super) fn tool_change(
        &mut self,
        new: usize,
        solid: bool,
        solid_nozzle: bool,
    ) -> std::result::Result<Result, ClipperError> {
        self.nozzle_change.gcode.clear();
        if self.plan.nozzle_map[self.current] != self.plan.nozzle_map[new] {
            self.nozzle_change = self.change_nozzle(self.current, new, solid_nozzle)?;
        }
        let old = self.current;
        let change = self.plan.layers[self.index]
            .changes
            .iter()
            .find(|change| change.new == new);
        let (mut length, depth, mut purge, nozzle_depth) = change
            .map_or((0.0, 0.0, 0.0, 0.0), |c| {
                (c.length, c.depth, c.purge, c.nozzle_depth)
            });
        let nozzle_width = 2.0 * self.plan.perimeter;
        let nozzle_lines =
            ((nozzle_depth + 1e-3) / nozzle_width / if self.plan.tpu { 2.0 } else { 1.0 }) as i32;
        let interface = solid
            && self
                .settings
                .process
                .print
                .enable_tower_interface_features
                .0;
        if interface {
            let extra = self.filaments[new].interface_purge;
            if extra > 0.0 {
                purge += extra * self.filaments[new].area;
                length += extra;
            }
        }
        let block_index = self
            .plan
            .blocks
            .iter()
            .position(|b| b.category == self.filaments[new].category)
            .expect("planned change has category block");
        self.current_block = Some(block_index);
        let bounds = BoxCoordinates::new(
            [self.plan.perimeter, self.plan.blocks[block_index].current],
            [
                self.plan.width - 2.0 * self.plan.perimeter,
                depth - self.plan.layers[self.index].spacing * nozzle_depth,
            ],
        );
        let mut writer = self.writer(self.current);
        writer
            .gcode
            .push_str(";--------------------\n; CP TOOLCHANGE START\n");
        writeln!(writer.gcode, "; toolchange #{}", self.changes + 1).unwrap();
        writeln!(
            writer.gcode,
            "; material : {} -> {}\n;--------------------",
            self.filaments[old].material, self.filaments[new].material
        )
        .unwrap();
        if matches!(
            self.settings.printer.gcode.gcode_flavor,
            crate::GCodeFlavor::MarlinLegacy | crate::GCodeFlavor::MarlinFirmware
        ) {
            writer.gcode.push_str("M220 B\n");
        }
        writer.gcode.push_str("M220 S100\n");
        let count = (length / bounds.size()[0]) as i32;
        let offset = count as f32 * (self.plan.layers[self.index].spacing * self.plan.infill_gap);
        let initial = [
            if matches!(self.index % 4, 1 | 2) {
                bounds.upper[0]
            } else {
                bounds.lower[0]
            },
            bounds.lower[1] + if self.index % 2 == 1 { offset } else { 0.0 },
        ];
        writer.set_position(initial, [self.plan.width, self.plan.depth], 0.0);
        writer.gcode.push_str("; WIPE_TOWER_START\n");
        // Unload/Load bodies are #if 0 in Type1; real commands come from stock slots.
        self.used[old] += std::mem::take(&mut writer.used);
        writer.gcode.push_str(
            "[filament_end_gcode]\n[restore_layer_z_before_toolchange]\n[change_filament_gcode]\n",
        );
        writer.tool = new;
        writer.max_e_speed = self.filaments[new].max_e_speed;
        writer.gcode.push_str("[filament_start_gcode]\n");
        CommandWriter::flush_planner_queue(
            &mut writer.gcode,
            self.settings.printer.gcode.gcode_flavor,
        );
        self.current = new;
        let base = if self.first_layer() {
            self.filaments[new].first_temperature
        } else {
            self.filaments[new].temperature
        };
        let interface_temp = self.filaments[new].interface_temperature;
        let cooldown = self
            .settings
            .process
            .print
            .enable_tower_interface_cooldown_during_tower
            .0;
        if interface {
            if interface_temp > 0 && interface_temp != base {
                writer.temperature(interface_temp, true);
            }
            if cooldown && interface_temp > 0 && interface_temp != base {
                writer.temperature(base, false);
            }
            let distance = self.filaments[new].interface_distance;
            let amount = self.filaments[new].interface_length;
            let displacement = if matches!(self.index % 4, 0 | 3) {
                distance
            } else {
                -distance
            };
            if distance > 0.0 && amount > 0.0 {
                let x = writer.position[0] + displacement;
                writer.explicit(
                    [
                        x.clamp(bounds.lower[0], bounds.upper[0]),
                        writer.position[1],
                    ],
                    amount,
                    600.0,
                    (false, true),
                );
            }
        }
        if self.settings.project.print.nozzle_diameter.0.len() > 1
            && self.filaments[new].material == "TPU"
        {
            let step = self.plan.layers[self.index].spacing.max(2.0) * nozzle_width;
            let speed = 60.0 * self.filaments[new].max_e_speed / self.flow * 0.25;
            let start = [
                self.nozzle_change.origin_start[0],
                self.nozzle_change.origin_start[1] + nozzle_width,
            ];
            writer.travel(start, 0.0);
            let count = (nozzle_lines + 1) / 2;
            writer.travel_lines(
                [
                    bounds.lower[0] + self.plan.perimeter,
                    bounds.upper[0] - self.plan.perimeter,
                ],
                (count, true),
                (speed, step),
            );
            writer.travel(initial, 0.0);
        }
        self.wipe(&mut writer, bounds, length, solid);
        if interface && !cooldown && interface_temp > 0 && interface_temp != base {
            writer.temperature(base, false);
        }
        writer.gcode.push_str("; WIPE_TOWER_END\n");
        self.changes += 1;
        self.plan.blocks[block_index].current +=
            depth - nozzle_depth * self.plan.layers[self.index].spacing;
        self.plan.blocks[block_index].last_filament = Some(new);
        if matches!(
            self.settings.printer.gcode.gcode_flavor,
            crate::GCodeFlavor::MarlinLegacy | crate::GCodeFlavor::MarlinFirmware
        ) {
            writer.gcode.push_str("M220 R\n");
        }
        writer.feedrate(self.travel_speed * 60.0);
        CommandWriter::flush_planner_queue(
            &mut writer.gcode,
            self.settings.printer.gcode.gcode_flavor,
        );
        writer
            .gcode
            .push_str("G92 E0\n; CP TOOLCHANGE END\n;------------------\n\n\n");
        self.used[new] += std::mem::take(&mut writer.used);
        let mut result = Result::from_writer(writer, self.z, self.height, old, false);
        result.tool_change = true;
        result.change_start = result.common.start;
        result.common.contact = interface;
        result.purge = purge;
        result.nozzle_change = self.nozzle_change.clone();
        Ok(result)
    }
}
