use serde_json::Value;

use crate::gcode_first_layer_print_placeholders::{
    FirstLayerPrintBounds, FirstLayerPrintPlaceholders,
};
use crate::{Point2, SliceError, SliceOptions};

const KEY: &str = "head_wrap_detect_zone";

pub(crate) fn placeholder_value(
    options: &SliceOptions,
    first_layer_print: Option<&FirstLayerPrintPlaceholders>,
) -> Result<&'static str, SliceError> {
    let Some(zone) = parse_zone(options.values().get(KEY))? else {
        return Ok("0");
    };
    let Some(print_bounds) = first_layer_print.and_then(FirstLayerPrintPlaceholders::bounds) else {
        return Ok("0");
    };

    Ok(if intersects(zone, print_bounds) {
        "1"
    } else {
        "0"
    })
}

fn parse_zone(value: Option<&Value>) -> Result<Option<FirstLayerPrintBounds>, SliceError> {
    match value {
        None => Ok(None),
        Some(Value::String(text)) => parse_zone_text(text),
        Some(Value::Array(values)) => parse_zone_array(values),
        Some(_) => Err(invalid("unsupported value")),
    }
}

fn parse_zone_text(text: &str) -> Result<Option<FirstLayerPrintBounds>, SliceError> {
    let text = text.trim();
    if text.is_empty() || text == "0x0" {
        return Ok(None);
    }

    let points = text
        .split(',')
        .map(|token| {
            let token = token.trim();
            let Some((x, y)) = token.split_once('x') else {
                return Err(invalid("has a malformed point"));
            };
            if y.contains('x') || x.trim().is_empty() || y.trim().is_empty() {
                return Err(invalid("has a malformed point"));
            }
            point(parse_coordinate(x)?, parse_coordinate(y)?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    bounds_from_points(points)
}

fn parse_zone_array(values: &[Value]) -> Result<Option<FirstLayerPrintBounds>, SliceError> {
    if values.is_empty() {
        return Ok(None);
    }
    let points = values
        .iter()
        .map(parse_point_array)
        .collect::<Result<Vec<_>, _>>()?;
    bounds_from_points(points)
}

fn parse_point_array(value: &Value) -> Result<Point2, SliceError> {
    let Value::Array(coords) = value else {
        return Err(invalid("point must be [x, y]"));
    };
    let [x, y] = coords.as_slice() else {
        return Err(invalid("point must be [x, y]"));
    };
    point(json_coordinate(x)?, json_coordinate(y)?)
}

fn json_coordinate(value: &Value) -> Result<f64, SliceError> {
    match value {
        Value::Number(number) => finite_coordinate(
            number
                .as_f64()
                .ok_or_else(|| invalid("coordinate must be numeric"))?,
        ),
        Value::String(text) => parse_coordinate(text),
        _ => Err(invalid("coordinate must be numeric")),
    }
}

fn parse_coordinate(text: &str) -> Result<f64, SliceError> {
    let value = text
        .trim()
        .parse()
        .map_err(|_| invalid("coordinate must be numeric"))?;
    finite_coordinate(value)
}

fn finite_coordinate(value: f64) -> Result<f64, SliceError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid("coordinate must be finite"))
    }
}

fn point(x: f64, y: f64) -> Result<Point2, SliceError> {
    Ok(Point2::new(finite_coordinate(x)?, finite_coordinate(y)?))
}

fn bounds_from_points(points: Vec<Point2>) -> Result<Option<FirstLayerPrintBounds>, SliceError> {
    if points.is_empty() {
        return Ok(None);
    }
    if points.len() < 3 {
        return Err(invalid("must contain at least three points"));
    }
    let mut points = points.into_iter();
    let first = points
        .next()
        .expect("non-empty point list has a first point");
    let mut bounds = FirstLayerPrintBounds::new(first, first);
    for point in points {
        bounds.include(point);
    }
    Ok(Some(bounds))
}

fn intersects(zone: FirstLayerPrintBounds, print: FirstLayerPrintBounds) -> bool {
    zone.min().x() <= print.max().x()
        && zone.max().x() >= print.min().x()
        && zone.min().y() <= print.max().y()
        && zone.max().y() >= print.min().y()
}

fn invalid(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{KEY} {reason}"))
}
