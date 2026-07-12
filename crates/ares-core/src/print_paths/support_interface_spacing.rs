use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    LayerPrintPaths, PrintPath, PrintPathRole, rotated_rectangle_lines,
    support_interface::{
        parse_support_interface_bottom_layers, parse_support_interface_top_layers,
    },
    support_rectangle::{rebuild_path, rectangle_bounds},
};
use crate::SliceError;
use crate::options::support_z_distance::SupportZDistanceOptions;

use super::support_rectangle::{EPSILON, RectangleBounds, rectangle_points};

const SUPPORT_INTERFACE_SPACING: &str = "support_interface_spacing";
const SUPPORT_BOTTOM_INTERFACE_SPACING: &str = "support_bottom_interface_spacing";
const SUPPORT_INTERFACE_PATTERN: &str = "support_interface_pattern";
const SUPPORT_INTERFACE_LOOP_PATTERN: &str = "support_interface_loop_pattern";
const DEFAULT_SUPPORT_INTERFACE_SPACING: f64 = 0.5;
const DEFAULT_SUPPORT_BOTTOM_INTERFACE_SPACING: f64 = 0.5;

#[derive(Clone, Copy)]
enum SupportInterfacePattern {
    SingleFamily,
    RectilinearInterlaced,
    Concentric,
    Grid,
}

#[derive(Clone, Copy)]
struct SupportInterfaceLineConfig {
    layer_id: usize,
    pitch: f64,
    support_angle: f64,
    pattern: SupportInterfacePattern,
    loop_pattern: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct SupportInterfaceSpacingConfig {
    support_interface_width: f64,
    support_ironing: bool,
    support_angle: f64,
    support_z_distance: SupportZDistanceOptions,
}

impl SupportInterfaceSpacingConfig {
    pub(crate) const fn new(
        support_interface_width: f64,
        support_ironing: bool,
        support_angle: f64,
        support_z_distance: SupportZDistanceOptions,
    ) -> Self {
        Self {
            support_interface_width,
            support_ironing,
            support_angle,
            support_z_distance,
        }
    }
}

pub(crate) fn apply_support_interface_spacing(
    layers: Vec<LayerPrintPaths>,
    values: &BTreeMap<String, Value>,
    config: SupportInterfaceSpacingConfig,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let spacing = parse_support_interface_spacing(values)?;
    let bottom_spacing = parse_support_bottom_interface_spacing(values)?;
    let top_layers = parse_support_interface_top_layers(values)?;
    let bottom_layers = parse_support_interface_bottom_layers(values)?;
    let pattern = parse_support_interface_pattern(
        values,
        config.support_z_distance.zero_gap_interface_top(top_layers),
    )?;
    let loop_pattern = parse_support_interface_loop_pattern(values)?;
    if config.support_ironing {
        return Ok(layers);
    }

    let resolved_bottom_layers = if bottom_layers < 0 {
        top_layers
    } else {
        bottom_layers as usize
    };
    let selected_spacing = if top_layers == 0 && resolved_bottom_layers > 0 {
        bottom_spacing
    } else {
        spacing
    };
    let pitch = selected_spacing + config.support_interface_width;
    Ok(layers
        .into_iter()
        .map(|layer| {
            let line_config = SupportInterfaceLineConfig {
                layer_id: layer.layer_id(),
                pitch,
                support_angle: config.support_angle,
                pattern,
                loop_pattern,
            };
            let paths = layer
                .paths()
                .iter()
                .flat_map(|path| support_interface_lines(path, line_config))
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect())
}

fn parse_support_interface_spacing(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get(SUPPORT_INTERFACE_SPACING) else {
        return Ok(DEFAULT_SUPPORT_INTERFACE_SPACING);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite() && *value >= 0.0)
    .ok_or_else(invalid_support_interface_spacing)
}

fn parse_support_bottom_interface_spacing(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    let Some(value) = values.get(SUPPORT_BOTTOM_INTERFACE_SPACING) else {
        return Ok(DEFAULT_SUPPORT_BOTTOM_INTERFACE_SPACING);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite() && *value >= 0.0)
    .ok_or_else(invalid_support_bottom_interface_spacing)
}

fn parse_support_interface_pattern(
    values: &BTreeMap<String, Value>,
    zero_gap_interface_top: bool,
) -> Result<SupportInterfacePattern, SliceError> {
    let Some(value) = values.get(SUPPORT_INTERFACE_PATTERN) else {
        return Ok(auto_support_interface_pattern(zero_gap_interface_top));
    };
    let Some(text) = value.as_str() else {
        return Err(invalid_support_interface_pattern());
    };

    match text {
        "auto" => Ok(auto_support_interface_pattern(zero_gap_interface_top)),
        "rectilinear" => Ok(SupportInterfacePattern::SingleFamily),
        "rectilinear_interlaced" => Ok(SupportInterfacePattern::RectilinearInterlaced),
        "concentric" => Ok(SupportInterfacePattern::Concentric),
        "grid" => Ok(SupportInterfacePattern::Grid),
        _ => Err(invalid_support_interface_pattern()),
    }
}

fn auto_support_interface_pattern(zero_gap_interface_top: bool) -> SupportInterfacePattern {
    if zero_gap_interface_top {
        SupportInterfacePattern::Concentric
    } else {
        SupportInterfacePattern::SingleFamily
    }
}

fn parse_support_interface_loop_pattern(
    values: &BTreeMap<String, Value>,
) -> Result<bool, SliceError> {
    let Some(value) = values.get(SUPPORT_INTERFACE_LOOP_PATTERN) else {
        return Ok(false);
    };
    value
        .as_bool()
        .ok_or_else(invalid_support_interface_loop_pattern)
}

fn support_interface_lines(path: &PrintPath, config: SupportInterfaceLineConfig) -> Vec<PrintPath> {
    if path.role() != PrintPathRole::SupportMaterialInterface || !path.is_closed() {
        return vec![path.clone()];
    }

    let Some(bounds) = rectangle_bounds(path.points()) else {
        return vec![path.clone()];
    };

    if let SupportInterfacePattern::Concentric = config.pattern {
        return concentric_interface_loops(bounds, config.pitch)
            .into_iter()
            .map(|points| rebuild_path(path, PrintPathRole::SupportMaterialInterface, points, true))
            .collect();
    }

    let line_angle = match config.pattern {
        SupportInterfacePattern::SingleFamily | SupportInterfacePattern::Grid => {
            config.support_angle + 90.0
        }
        SupportInterfacePattern::RectilinearInterlaced if config.layer_id & 1 == 0 => 45.0,
        SupportInterfacePattern::RectilinearInterlaced => -45.0,
        SupportInterfacePattern::Concentric => {
            unreachable!("concentric returns before line generation")
        }
    };
    let mut lines = rotated_rectangle_lines(bounds, config.pitch, line_angle);
    if let SupportInterfacePattern::Grid = config.pattern {
        lines.extend(rotated_rectangle_lines(
            bounds,
            config.pitch,
            config.support_angle,
        ));
    }

    let mut paths = Vec::new();
    if config.loop_pattern {
        let mut loop_points = path.points().to_vec();
        loop_points.push(loop_points[0]);
        paths.push(rebuild_path(
            path,
            PrintPathRole::SupportMaterialInterface,
            loop_points,
            true,
        ));
    }
    paths.extend(lines.into_iter().map(|points| {
        rebuild_path(
            path,
            PrintPathRole::SupportMaterialInterface,
            points.into(),
            false,
        )
    }));
    paths
}

fn concentric_interface_loops(bounds: RectangleBounds, pitch: f64) -> Vec<Vec<crate::Point2>> {
    let mut loops = Vec::new();
    let mut inset = 0.0;
    loop {
        let min_x = bounds.min_x + inset;
        let min_y = bounds.min_y + inset;
        let max_x = bounds.max_x - inset;
        let max_y = bounds.max_y - inset;
        if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
            break;
        }
        let mut points = rectangle_points(RectangleBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        });
        points.push(points[0]);
        loops.push(points);
        let next_inset = inset + pitch;
        if next_inset <= inset {
            break;
        }
        inset = next_inset;
    }
    loops
}

fn invalid_support_interface_spacing() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_INTERFACE_SPACING} must be a finite non-negative number"
    ))
}

fn invalid_support_bottom_interface_spacing() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_BOTTOM_INTERFACE_SPACING} must be a finite non-negative number"
    ))
}

fn invalid_support_interface_pattern() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_INTERFACE_PATTERN} must be one of auto, rectilinear, concentric, rectilinear_interlaced, or grid"
    ))
}

fn invalid_support_interface_loop_pattern() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_INTERFACE_LOOP_PATTERN} must be a boolean"
    ))
}
