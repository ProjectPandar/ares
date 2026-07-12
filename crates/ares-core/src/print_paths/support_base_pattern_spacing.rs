use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    LayerPrintPaths, PrintPath, PrintPathRole, rotated_rectangle_lines,
    support_rectangle::{
        EPSILON, RectangleBounds, rebuild_path, rectangle_bounds, rectangle_points,
    },
};
use crate::SliceError;

const SUPPORT_BASE_PATTERN_SPACING: &str = "support_base_pattern_spacing";
const SUPPORT_BASE_PATTERN: &str = "support_base_pattern";
const DEFAULT_SUPPORT_BASE_PATTERN_SPACING: f64 = 2.5;

#[derive(Clone, Copy)]
enum SupportBasePattern {
    SingleFamily,
    RectilinearGrid,
}

#[derive(Clone, Copy)]
struct SupportBaseLineConfig {
    pitch: f64,
    support_angle: f64,
    pattern: SupportBasePattern,
    with_sheath: bool,
    support_material_width: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct SupportBaseSpacingConfig {
    support_material_width: f64,
    support_angle: f64,
    raft_first_layer_density_percent: f64,
    tree_support_wall_count: u32,
}

impl SupportBaseSpacingConfig {
    pub(crate) const fn new(
        support_material_width: f64,
        support_angle: f64,
        raft_first_layer_density_percent: f64,
        tree_support_wall_count: u32,
    ) -> Self {
        Self {
            support_material_width,
            support_angle,
            raft_first_layer_density_percent,
            tree_support_wall_count,
        }
    }
}

pub(crate) fn apply_support_base_pattern_spacing(
    layers: Vec<LayerPrintPaths>,
    values: &BTreeMap<String, Value>,
    config: SupportBaseSpacingConfig,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let spacing = parse_support_base_pattern_spacing(values)?;
    let pattern = parse_support_base_pattern(values)?;
    let spacing_pitch = spacing + config.support_material_width;
    let first_layer_pitch =
        config.support_material_width / (config.raft_first_layer_density_percent / 100.0);
    let with_sheath = config.tree_support_wall_count > 0;

    Ok(layers
        .into_iter()
        .map(|layer| {
            let pitch = if layer.layer_id() == 0 {
                first_layer_pitch
            } else {
                spacing_pitch
            };
            let paths = layer
                .paths()
                .iter()
                .flat_map(|path| {
                    support_base_lines(
                        path,
                        SupportBaseLineConfig {
                            pitch,
                            support_angle: config.support_angle,
                            pattern,
                            with_sheath,
                            support_material_width: config.support_material_width,
                        },
                    )
                })
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect())
}

fn parse_support_base_pattern_spacing(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get(SUPPORT_BASE_PATTERN_SPACING) else {
        return Ok(DEFAULT_SUPPORT_BASE_PATTERN_SPACING);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite() && *value >= 0.0)
    .ok_or_else(invalid_support_base_pattern_spacing)
}

fn parse_support_base_pattern(
    values: &BTreeMap<String, Value>,
) -> Result<SupportBasePattern, SliceError> {
    let Some(value) = values.get(SUPPORT_BASE_PATTERN) else {
        return Ok(SupportBasePattern::SingleFamily);
    };
    let Some(text) = value.as_str() else {
        return Err(invalid_support_base_pattern());
    };

    match text {
        "rectilinear-grid" | "grid" => Ok(SupportBasePattern::RectilinearGrid),
        "default" | "rectilinear" | "honeycomb" | "lightning" | "hollow" => {
            Ok(SupportBasePattern::SingleFamily)
        }
        _ => Err(invalid_support_base_pattern()),
    }
}

fn support_base_lines(path: &PrintPath, config: SupportBaseLineConfig) -> Vec<PrintPath> {
    if path.role() != PrintPathRole::SupportMaterial || !path.is_closed() {
        return vec![path.clone()];
    }

    let Some(bounds) = rectangle_bounds(path.points()) else {
        return vec![path.clone()];
    };

    let mut paths = Vec::new();
    let line_bounds = if config.with_sheath {
        paths.push(rebuild_path(
            path,
            PrintPathRole::SupportMaterial,
            rectangle_points(bounds),
            true,
        ));
        inset_bounds(bounds, 0.4 * config.support_material_width)
    } else {
        Some(bounds)
    };
    let Some(line_bounds) = line_bounds else {
        return paths;
    };

    let mut lines = rotated_rectangle_lines(line_bounds, config.pitch, config.support_angle);
    if let SupportBasePattern::RectilinearGrid = config.pattern {
        lines.extend(rotated_rectangle_lines(
            line_bounds,
            config.pitch,
            config.support_angle + 90.0,
        ));
    }

    paths.extend(
        lines
            .into_iter()
            .map(|points| rebuild_path(path, PrintPathRole::SupportMaterial, points.into(), false)),
    );
    paths
}

fn inset_bounds(bounds: RectangleBounds, inset: f64) -> Option<RectangleBounds> {
    let inset_bounds = RectangleBounds {
        min_x: bounds.min_x + inset,
        min_y: bounds.min_y + inset,
        max_x: bounds.max_x - inset,
        max_y: bounds.max_y - inset,
    };
    (inset_bounds.max_x - inset_bounds.min_x > EPSILON
        && inset_bounds.max_y - inset_bounds.min_y > EPSILON)
        .then_some(inset_bounds)
}

fn invalid_support_base_pattern_spacing() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_BASE_PATTERN_SPACING} must be a finite non-negative number"
    ))
}

fn invalid_support_base_pattern() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_BASE_PATTERN} must be one of default, rectilinear, rectilinear-grid, honeycomb, lightning, hollow, or grid"
    ))
}
