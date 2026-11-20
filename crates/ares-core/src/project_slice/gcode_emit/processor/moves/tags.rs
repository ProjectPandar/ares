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

/// One parse shared by move recording and timing, including unrecorded slices.
#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit::processor) enum CustomTag<'a> {
    ColorChange {
        filament: usize,
        color: Option<&'a str>,
    },
    PausePrint,
    Custom,
}

impl CustomTag<'_> {
    pub(in crate::project_slice::gcode_emit::processor) fn time_kind(
        self,
        active: usize,
    ) -> Option<CustomGCodeType> {
        match self {
            Self::ColorChange { filament, .. } if filament == active => {
                Some(CustomGCodeType::ColorChange)
            }
            Self::PausePrint => Some(CustomGCodeType::PausePrint),
            Self::ColorChange { .. } | Self::Custom => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit::processor) struct ParsedLine<'a> {
    pub(in crate::project_slice::gcode_emit::processor) raw: &'a str,
    pub(in crate::project_slice::gcode_emit::processor) custom: Option<CustomTag<'a>>,
    pub(in crate::project_slice::gcode_emit::processor) wiping: Option<bool>,
}

impl<'a> ParsedLine<'a> {
    pub(in crate::project_slice::gcode_emit::processor) fn new(raw: &'a str, bbl: bool) -> Self {
        Self {
            raw,
            custom: parse_custom_tag(raw, bbl),
            wiping: parse_wipe_tag(raw, bbl),
        }
    }
}

fn parse_wipe_tag(line: &str, bbl: bool) -> Option<bool> {
    if bbl {
        return match line.trim() {
            "; WIPE_START" | ";WIPE_START" => Some(true),
            "; WIPE_END" | ";WIPE_END" => Some(false),
            _ => None,
        };
    }
    // GCodeProcessor.hpp:375 selects compatible tokens (cpp:81-84);
    // cpp:3146-3155 prefix-matches without stripping comment-leading spaces.
    let comment = line.strip_prefix(';')?;
    if comment.starts_with("WIPE_START") {
        Some(true)
    } else if comment.starts_with("WIPE_END") {
        Some(false)
    } else {
        None
    }
}

fn parse_custom_tag(line: &str, bbl: bool) -> Option<CustomTag<'_>> {
    let comment = line.strip_prefix(';')?;
    let tags = if bbl { &BBL_TAGS } else { &COMPATIBLE_TAGS };
    if comment.starts_with(tags.color_change) {
        // boost::split(token_compress_on) retains the final empty token.
        let mut tokens = comment
            .split(',')
            .filter(|token| !token.is_empty())
            .chain(comment.ends_with(',').then_some(""));
        let mut filament = 0;
        if let Some(tool) = tokens.nth(1).and_then(|token| token.strip_prefix('T')) {
            match tool.parse::<i32>() {
                Ok(tool @ 0..=255) => filament = tool as usize,
                _ => return None,
            }
        }
        Some(CustomTag::ColorChange {
            filament,
            color: tokens.next(),
        })
    } else if comment == tags.pause_print {
        Some(CustomTag::PausePrint)
    } else if comment == tags.custom_code {
        Some(CustomTag::Custom)
    } else {
        None
    }
}

impl MoveRecorder<'_> {
    pub(super) fn tag(
        &mut self,
        source: usize,
        line: ParsedLine<'_>,
        g1_line_id: usize,
        position: [f64; 4],
    ) {
        let comment = &line.raw[1..];
        let custom = line.custom;
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
        } else if let Some(CustomTag::ColorChange { filament, color }) = custom {
            self.color_change(source, filament, color, position);
        } else if matches!(custom, Some(CustomTag::PausePrint)) {
            self.store(source, EMoveType::PausePrint, [position; 2], 0.0);
            self.custom_gcode_per_print_z.push(CustomGCodeItem {
                print_z: position[2],
                kind: CustomGCodeType::PausePrint,
                extruder: self.filament as u32 + 1,
                color: String::new(),
                extra: String::new(),
            });
            self.set_option();
        } else if matches!(custom, Some(CustomTag::Custom)) {
            self.store(source, EMoveType::CustomGCode, [position; 2], 0.0);
            self.custom_gcode_per_print_z.push(CustomGCodeItem {
                print_z: position[2],
                kind: CustomGCodeType::Custom,
                extruder: self.filament as u32 + 1,
                color: String::new(),
                extra: String::new(),
            });
            self.set_option();
        } else if comment == tags.layer_change {
            self.layer_id += 1;
        }
    }

    /// `GCodeProcessor.cpp:3237-3301`: active-filament metadata and color ids.
    fn color_change(
        &mut self,
        source: usize,
        filament: usize,
        color: Option<&str>,
        position: [f64; 4],
    ) {
        const DEFAULT_COLORS: [&str; 6] = [
            "#0B2C7A", "#1C8891", "#AAF200", "#F5CE0A", "#D16830", "#942616",
        ];
        let color = match color {
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
            self.store(source, EMoveType::ColorChange, [position; 2], 0.0);
            self.custom_gcode_per_print_z.push(CustomGCodeItem {
                print_z: position[2],
                kind: CustomGCodeType::ColorChange,
                extruder: filament as u32 + 1,
                color: color.to_owned(),
                extra: String::new(),
            });
            self.set_option();
            self.flush_color_change_volume();
        }
    }
}
