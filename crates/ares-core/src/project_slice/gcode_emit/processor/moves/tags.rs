//! Tag comments (`GCodeProcessor.cpp:57-102` reserved tags,
//! `:3113-3330` `process_tags`).

use super::MoveRecorder;
use crate::ExtrusionRole;
use crate::gcode_processor::{CustomGCodeItem, CustomGCodeType, EMoveType};

pub(super) struct Tags {
    role: &'static str,
    height: &'static str,
    width: &'static str,
    layer_change: &'static str,
    color_change: &'static str,
    pause_print: &'static str,
    custom_code: &'static str,
}

/// `Reserved_Tags` / `Reserved_Tags_compatible` (`GCodeProcessor.cpp:57-102`).
pub(super) const BBL_TAGS: Tags = Tags {
    role: " FEATURE: ",
    height: " LAYER_HEIGHT: ",
    width: " LINE_WIDTH: ",
    layer_change: " CHANGE_LAYER",
    color_change: " COLOR_CHANGE",
    pause_print: " PAUSE_PRINTING",
    custom_code: " CUSTOM_GCODE",
};

pub(super) const COMPATIBLE_TAGS: Tags = Tags {
    role: "TYPE:",
    height: "HEIGHT:",
    width: "WIDTH:",
    layer_change: "LAYER_CHANGE",
    color_change: "COLOR_CHANGE",
    pause_print: "PAUSE_PRINT",
    custom_code: "CUSTOM_GCODE",
};

impl MoveRecorder {
    pub(super) fn tag(
        &mut self,
        source: usize,
        comment: &str,
        g1_line_id: usize,
        position: [f64; 4],
    ) {
        let tags = self.tags;
        if let Some(role) = comment.strip_prefix(tags.role) {
            self.role = ExtrusionRole::from_name(role);
            self.processing_start_custom_gcode =
                self.role == ExtrusionRole::Custom && g1_line_id == 0;
        } else if let Some(height) = comment.strip_prefix(tags.height) {
            if let Ok(height) = height.parse() {
                self.forced_height = height;
            }
        } else if let Some(width) = comment.strip_prefix(tags.width) {
            if let Ok(width) = width.parse() {
                self.forced_width = width;
            }
        } else if comment.starts_with(tags.color_change) {
            self.color_change(source, comment, position);
        } else if comment == tags.pause_print {
            self.store(source, EMoveType::PausePrint, position, 0.0);
        } else if comment == tags.custom_code {
            self.store(source, EMoveType::CustomGCode, position, 0.0);
        } else if comment == tags.layer_change {
            self.layer_id += 1;
        }
    }

    /// `GCodeProcessor.cpp:3237-3301`: active-filament metadata and color ids.
    fn color_change(&mut self, source: usize, comment: &str, position: [f64; 4]) {
        const DEFAULT_COLORS: [&str; 6] = [
            "#0B2C7A", "#1C8891", "#AAF200", "#F5CE0A", "#D16830", "#942616",
        ];
        // boost::split(token_compress_on) retains the final empty token.
        let mut tokens = comment
            .split(',')
            .filter(|token| !token.is_empty())
            .chain(comment.ends_with(',').then_some(""));
        let mut filament = 0;
        if let Some(tool) = tokens.nth(1).and_then(|token| token.strip_prefix('T')) {
            match tool.parse::<i32>() {
                Ok(tool @ 0..=255) => filament = tool as usize,
                _ => return,
            }
        }
        let color = match tokens.next() {
            Some(color)
                if color.len() == 7
                    && color.starts_with('#')
                    && color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit) =>
            {
                color
            }
            Some(_) => DEFAULT_COLORS[0],
            None => {
                let color = DEFAULT_COLORS[self.last_default_color];
                self.last_default_color = (self.last_default_color + 1) % DEFAULT_COLORS.len();
                color
            }
        };
        if let Some(color) = self.extruder_colors.get_mut(filament) {
            *color = (self.extruder_offsets.len() as u8).wrapping_add(self.cp_color_counter);
        }
        self.cp_color_counter = self.cp_color_counter.wrapping_add(1);
        if self.cp_color_counter == u8::MAX {
            self.cp_color_counter = 0;
        }
        if self.filament == filament {
            self.cp_color_current = self.extruder_colors[filament];
            self.store(source, EMoveType::ColorChange, position, 0.0);
            self.custom_gcode_per_print_z.push(CustomGCodeItem {
                print_z: position[2],
                kind: CustomGCodeType::ColorChange,
                extruder: filament as u32 + 1,
                color: color.to_owned(),
                extra: String::new(),
            });
            self.pending_option = Some((
                self.moves.len() - 1,
                self.custom_gcode_per_print_z.len() - 1,
            ));
            self.flush_color_change_volume();
        }
    }
}
