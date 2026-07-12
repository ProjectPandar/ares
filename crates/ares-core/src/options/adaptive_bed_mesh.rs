use serde_json::Value;

use super::SliceOptions;
use crate::{Point2, SliceError};

const BED_MESH_MIN_KEY: &str = "bed_mesh_min";
const BED_MESH_MAX_KEY: &str = "bed_mesh_max";
const BED_MESH_PROBE_DISTANCE_KEY: &str = "bed_mesh_probe_distance";
const ADAPTIVE_BED_MESH_MARGIN_KEY: &str = "adaptive_bed_mesh_margin";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AdaptiveBedMeshOptions {
    bed_mesh_min: Point2,
    bed_mesh_max: Point2,
    bed_mesh_probe_distance: Point2,
    adaptive_bed_mesh_margin: f64,
}

impl AdaptiveBedMeshOptions {
    pub(crate) const fn bed_mesh_min(self) -> Point2 {
        self.bed_mesh_min
    }

    pub(crate) const fn bed_mesh_max(self) -> Point2 {
        self.bed_mesh_max
    }

    pub(crate) const fn bed_mesh_probe_distance(self) -> Point2 {
        self.bed_mesh_probe_distance
    }

    pub(crate) const fn adaptive_bed_mesh_margin(self) -> f64 {
        self.adaptive_bed_mesh_margin
    }
}

impl SliceOptions {
    pub(crate) fn adaptive_bed_mesh_options(&self) -> Result<AdaptiveBedMeshOptions, SliceError> {
        Ok(AdaptiveBedMeshOptions {
            bed_mesh_min: parse_point(
                BED_MESH_MIN_KEY,
                self.values().get(BED_MESH_MIN_KEY),
                Point2::new(-99999.0, -99999.0),
            )?,
            bed_mesh_max: parse_point(
                BED_MESH_MAX_KEY,
                self.values().get(BED_MESH_MAX_KEY),
                Point2::new(99999.0, 99999.0),
            )?,
            bed_mesh_probe_distance: parse_point(
                BED_MESH_PROBE_DISTANCE_KEY,
                self.values().get(BED_MESH_PROBE_DISTANCE_KEY),
                Point2::new(50.0, 50.0),
            )?,
            adaptive_bed_mesh_margin: parse_margin(
                self.values().get(ADAPTIVE_BED_MESH_MARGIN_KEY),
            )?,
        })
    }
}

fn parse_point(key: &str, value: Option<&Value>, default: Point2) -> Result<Point2, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value {
        Value::String(text) => parse_point_text(key, text),
        Value::Array(values) => parse_point_array(key, values),
        _ => Err(invalid(key, "must be a point")),
    }
}

fn parse_point_text(key: &str, text: &str) -> Result<Point2, SliceError> {
    let Some((x, y)) = text.trim().split_once('x') else {
        return Err(invalid(key, "must be an XxY point"));
    };
    if y.contains('x') {
        return Err(invalid(key, "must be an XxY point"));
    }
    point(key, parse_number(key, x)?, parse_number(key, y)?)
}

fn parse_point_array(key: &str, values: &[Value]) -> Result<Point2, SliceError> {
    match values {
        [x, y] => point(key, json_number(key, x)?, json_number(key, y)?),
        [Value::Array(pair)] => parse_point_array(key, pair),
        _ => Err(invalid(key, "must contain two coordinates")),
    }
}

fn parse_margin(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(0.0);
    };
    let margin = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .ok_or_else(|| invalid(ADAPTIVE_BED_MESH_MARGIN_KEY, "must be a number"))?;
    if margin.is_finite() && margin >= 0.0 {
        Ok(margin)
    } else {
        Err(invalid(
            ADAPTIVE_BED_MESH_MARGIN_KEY,
            "must be non-negative",
        ))
    }
}

fn parse_number(key: &str, text: &str) -> Result<f64, SliceError> {
    text.trim()
        .parse()
        .map_err(|_| invalid(key, "must contain finite coordinates"))
}

fn json_number(key: &str, value: &Value) -> Result<f64, SliceError> {
    value
        .as_f64()
        .ok_or_else(|| invalid(key, "must contain finite coordinates"))
}

fn point(key: &str, x: f64, y: f64) -> Result<Point2, SliceError> {
    if x.is_finite() && y.is_finite() {
        Ok(Point2::new(x, y))
    } else {
        Err(invalid(key, "must contain finite coordinates"))
    }
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
