// Ports AGPL OrcaSlicer GCodeViewer.cpp:3358-3388,4172-4239 display records.
use super::ViewerImpl;
use crate::{Color, ColorPrint, DUMMY_COLOR};
use ares_core::gcode_processor::{CustomGCodeType, GCodeProcessorResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartialTimeKind {
    Print,
    ColorChange,
    Pause,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartialTime {
    pub kind: PartialTimeKind,
    /// One-based CustomGCode::Item extruder id.
    pub extruder_id: u32,
    pub color1: Color,
    pub color2: Color,
    pub duration: f32,
    /// Remaining time at interval start, as in the processor pair.
    pub remaining: f32,
    /// Source-selected model mm³; Pause's preceding Print has no usage cell.
    pub volume: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorPrintRange {
    pub color: Color,
    pub z: [f64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintedColor {
    pub segment: ColorPrint,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtruderColorPrints {
    pub extruder_id: u8,
    pub segments: Vec<PrintedColor>,
    pub ranges: Vec<ColorPrintRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingColorVolume {
    pub missing_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorPrintLegend {
    pub visible: bool,
    pub extruders: Vec<ExtruderColorPrints>,
    pub partial_usage: Result<Vec<PartialTime>, MissingColorVolume>,
}

impl Default for ColorPrintLegend {
    fn default() -> Self {
        Self {
            visible: false,
            extruders: Vec::new(),
            partial_usage: Ok(Vec::new()),
        }
    }
}

/// Keeps printed transitions, layer brackets and partial times distinct.
/// Missing native volume indices make the whole partial table unavailable;
/// the original result and the other two structures are untouched.
pub fn color_print_legend(viewer: &ViewerImpl, result: &GCodeProcessorResult) -> ColorPrintLegend {
    let zs = viewer.layers().get_zs();
    let extruders: Vec<_> = viewer
        .used_extruders
        .iter()
        .map(|(&id, segments)| {
            let mut ranges: Vec<ColorPrintRange> = Vec::new();
            for item in result.custom_gcode_per_print_z.iter().filter(|item| {
                u32::from(id) + 1 == u32::from(item.extruder as u8)
                    && item.kind == CustomGCodeType::ColorChange
            }) {
                let index = zs.partition_point(|z| *z < (item.print_z - 0.0001) as f32);
                if index == zs.len() {
                    continue;
                }
                let z = [
                    if index == 0 {
                        0.0
                    } else {
                        f64::from(zs[index - 1])
                    },
                    f64::from(zs[index]),
                ];
                if ranges.last().is_none_or(|last| last.z != z) {
                    ranges.push(ColorPrintRange {
                        color: raw_color(&item.color),
                        z,
                    });
                }
            }
            ExtruderColorPrints {
                extruder_id: id,
                segments: segments
                    .iter()
                    .map(|segment| PrintedColor {
                        segment: segment.clone(),
                        color: if viewer.color_print_colors.is_empty() {
                            DUMMY_COLOR
                        } else {
                            viewer.color_print_colors
                                [usize::from(segment.color_id) % viewer.color_print_colors.len()]
                        },
                    })
                    .collect(),
                ranges,
            }
        })
        .collect();
    ColorPrintLegend {
        visible: !result.custom_gcode_per_print_z.is_empty()
            || extruders.iter().any(|item| item.segments.len() > 1),
        extruders,
        partial_usage: partial_times(viewer, result),
    }
}

fn partial_times(
    viewer: &ViewerImpl,
    result: &GCodeProcessorResult,
) -> Result<Vec<PartialTime>, MissingColorVolume> {
    // Independent matching iterators preserve find_if/erase order without
    // repeatedly copying/shifting the source custom-item vector.
    let mut changes = result
        .custom_gcode_per_print_z
        .iter()
        .filter(|item| item.kind == CustomGCodeType::ColorChange);
    let mut pauses = result
        .custom_gcode_per_print_z
        .iter()
        .filter(|item| item.kind == CustomGCodeType::PausePrint);
    let mut last_colors = viewer.get_tool_colors().to_vec();
    let mut last_extruder = 1;
    let mut volume_index = 0;
    let mut rows = Vec::new();
    for time in &result.custom_gcode_times {
        let item = match time.kind {
            CustomGCodeType::ColorChange => changes.next(),
            CustomGCodeType::PausePrint => pauses.next(),
            CustomGCodeType::ToolChange
            | CustomGCodeType::Template
            | CustomGCodeType::Custom
            | CustomGCodeType::Unknown => continue,
        };
        if time.kind == CustomGCodeType::PausePrint && item.is_none() {
            continue;
        }
        let extruder_id = item.map_or(last_extruder, |item| item.extruder);
        let volume = if time.kind == CustomGCodeType::ColorChange {
            let value = result
                .print_statistics
                .volumes_per_color_change
                .get(volume_index)
                .ok_or(MissingColorVolume {
                    missing_index: volume_index,
                })?;
            volume_index += 1;
            Some(*value)
        } else {
            None
        };
        let print = PartialTime {
            kind: PartialTimeKind::Print,
            extruder_id,
            color1: last_colors[extruder_id as usize - 1],
            color2: [0; 3],
            duration: time.duration,
            remaining: time.remaining,
            volume,
        };
        rows.push(print.clone());
        let Some(item) = item else {
            continue;
        };
        if time.kind == CustomGCodeType::PausePrint {
            rows.push(PartialTime {
                kind: PartialTimeKind::Pause,
                color1: [0; 3],
                volume: None,
                ..print
            });
            continue;
        }
        let color = raw_color(&item.color);
        rows.push(PartialTime {
            kind: PartialTimeKind::ColorChange,
            color2: color,
            volume: None,
            ..print
        });
        last_colors[extruder_id as usize - 1] = color;
        last_extruder = extruder_id;
    }
    Ok(rows)
}

// GCodeViewer ignores decode_color's failure and retains default black.
// These display swatches do not use the path wrapper's minimum brightness.
fn raw_color(color: &str) -> Color {
    let bytes = color.as_bytes();
    if !matches!(bytes.len(), 7 | 9) || bytes[0] != b'#' {
        return [0; 3];
    }
    std::array::from_fn(|i| {
        match (
            (bytes[1 + 2 * i] as char).to_digit(16),
            (bytes[2 + 2 * i] as char).to_digit(16),
        ) {
            (Some(high), Some(low)) => (high * 16 + low) as u8,
            _ => 0,
        }
    })
}
