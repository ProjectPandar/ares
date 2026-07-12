use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    LayerPrintPaths, PrintPath, PrintPathRole,
    support_rectangle::{
        EPSILON, RectangleBounds, rebuild_path, rebuild_path_without_extrusion_role,
        rectangle_bounds,
    },
};
use crate::{Point2, SliceError};

const SUPPORT_INTERFACE_TOP_LAYERS: &str = "support_interface_top_layers";
const DEFAULT_SUPPORT_INTERFACE_TOP_LAYERS: usize = 3;
const SUPPORT_INTERFACE_BOTTOM_LAYERS: &str = "support_interface_bottom_layers";
const DEFAULT_SUPPORT_INTERFACE_BOTTOM_LAYERS: isize = 0;
const SUPPORT_EXPANSION: &str = "support_expansion";

pub(crate) fn apply_support_interface_top_layers(
    layers: Vec<LayerPrintPaths>,
    values: &BTreeMap<String, Value>,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let top_layers = parse_support_interface_top_layers(values)?;
    let bottom_layers = parse_support_interface_bottom_layers(values)?;
    let resolved_bottom_layers = if bottom_layers < 0 {
        top_layers
    } else {
        bottom_layers as usize
    };
    if top_layers > 0 || resolved_bottom_layers > 0 {
        return Ok(layers);
    }

    Ok(layers
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths()
                .iter()
                .map(rewrite_disabled_interface_path)
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect())
}

pub(crate) fn apply_support_expansion(
    layers: Vec<LayerPrintPaths>,
    values: &BTreeMap<String, Value>,
) -> Result<Vec<LayerPrintPaths>, SliceError> {
    let expansion = parse_support_expansion(values)?;
    if expansion == 0.0 {
        return Ok(layers);
    }

    Ok(layers
        .into_iter()
        .map(|layer| {
            let paths = layer
                .paths()
                .iter()
                .filter_map(|path| expand_support_path(path, expansion))
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect())
}

pub(crate) fn apply_raft_expansion(
    layers: Vec<LayerPrintPaths>,
    raft_layers: u32,
    expansion: f64,
) -> Vec<LayerPrintPaths> {
    if raft_layers == 0 || expansion == 0.0 {
        return layers;
    }

    layers
        .into_iter()
        .map(|layer| {
            if layer.layer_id() >= raft_layers as usize {
                return layer;
            }

            let paths = layer
                .paths()
                .iter()
                .filter_map(|path| expand_support_path(path, expansion))
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

pub(crate) fn apply_raft_first_layer_expansion(
    layers: Vec<LayerPrintPaths>,
    has_raft: bool,
    expansion: f64,
) -> Vec<LayerPrintPaths> {
    if !has_raft || expansion == 0.0 {
        return layers;
    }

    layers
        .into_iter()
        .map(|layer| {
            if layer.layer_id() != 0 {
                return layer;
            }

            let paths = layer
                .paths()
                .iter()
                .filter_map(|path| expand_support_path(path, expansion))
                .collect();
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

pub(super) fn parse_support_interface_top_layers(
    values: &BTreeMap<String, Value>,
) -> Result<usize, SliceError> {
    let Some(value) = values.get(SUPPORT_INTERFACE_TOP_LAYERS) else {
        return Ok(DEFAULT_SUPPORT_INTERFACE_TOP_LAYERS);
    };
    match value {
        Value::Number(number) => parse_non_negative_decimal_integer(&number.to_string()),
        Value::String(text) => parse_non_negative_decimal_integer(text),
        _ => None,
    }
    .ok_or_else(invalid_support_interface_top_layers)
}

pub(super) fn parse_support_interface_bottom_layers(
    values: &BTreeMap<String, Value>,
) -> Result<isize, SliceError> {
    let Some(value) = values.get(SUPPORT_INTERFACE_BOTTOM_LAYERS) else {
        return Ok(DEFAULT_SUPPORT_INTERFACE_BOTTOM_LAYERS);
    };
    match value {
        Value::Number(number) => parse_decimal_integer_at_least_negative_one(&number.to_string()),
        Value::String(text) => parse_decimal_integer_at_least_negative_one(text),
        _ => None,
    }
    .ok_or_else(invalid_support_interface_bottom_layers)
}

fn parse_support_expansion(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get(SUPPORT_EXPANSION) else {
        return Ok(0.0);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite())
    .ok_or_else(invalid_support_expansion)
}

fn parse_non_negative_decimal_integer(text: &str) -> Option<usize> {
    let text = text.trim();
    if text.is_empty() || text.starts_with('-') || text.starts_with('+') {
        return None;
    }

    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if !fraction.is_empty() && !fraction.bytes().all(|byte| byte == b'0') {
        return None;
    }

    whole.parse().ok()
}

fn parse_decimal_integer_at_least_negative_one(text: &str) -> Option<isize> {
    let text = text.trim();
    if text.is_empty() || text.starts_with('+') {
        return None;
    }

    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty() || whole == "-" {
        return None;
    }

    let digits = whole.strip_prefix('-').unwrap_or(whole);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if !fraction.is_empty() && !fraction.bytes().all(|byte| byte == b'0') {
        return None;
    }

    whole.parse().ok().filter(|value: &isize| *value >= -1)
}

fn expand_support_path(path: &PrintPath, expansion: f64) -> Option<PrintPath> {
    if !matches!(
        path.role(),
        PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface
    ) || !path.is_closed()
    {
        return Some(path.clone());
    }

    let Some(bounds) = rectangle_bounds(path.points()) else {
        return Some(path.clone());
    };

    let expanded = RectangleBounds {
        min_x: bounds.min_x - expansion,
        min_y: bounds.min_y - expansion,
        max_x: bounds.max_x + expansion,
        max_y: bounds.max_y + expansion,
    };
    if expanded.max_x - expanded.min_x <= EPSILON || expanded.max_y - expanded.min_y <= EPSILON {
        return None;
    }

    Some(rebuild_path(
        path,
        path.role(),
        expand_points(path.points(), bounds, expanded),
        path.is_closed(),
    ))
}

fn expand_points(
    points: &[Point2],
    source: RectangleBounds,
    expanded: RectangleBounds,
) -> Vec<Point2> {
    points
        .iter()
        .map(|point| {
            let x = if scalar_eq(point.x(), source.min_x) {
                expanded.min_x
            } else {
                expanded.max_x
            };
            let y = if scalar_eq(point.y(), source.min_y) {
                expanded.min_y
            } else {
                expanded.max_y
            };
            Point2::new(x, y)
        })
        .collect()
}

fn scalar_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}

fn rewrite_disabled_interface_path(path: &PrintPath) -> PrintPath {
    if path.role() != PrintPathRole::SupportMaterialInterface {
        return path.clone();
    }

    rebuild_path_without_extrusion_role(
        path,
        PrintPathRole::SupportMaterial,
        path.points().to_vec(),
        path.is_closed(),
    )
}

fn invalid_support_interface_top_layers() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_INTERFACE_TOP_LAYERS} must be a non-negative integer"
    ))
}

fn invalid_support_interface_bottom_layers() -> SliceError {
    SliceError::InvalidInput(format!(
        "{SUPPORT_INTERFACE_BOTTOM_LAYERS} must be an integer greater than or equal to -1"
    ))
}

fn invalid_support_expansion() -> SliceError {
    SliceError::InvalidInput(format!("{SUPPORT_EXPANSION} must be a finite number"))
}
