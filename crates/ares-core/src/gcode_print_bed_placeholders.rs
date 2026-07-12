use serde_json::Value;

use crate::gcode_format::format_decimal;
use crate::{Point2, SliceError, SliceOptions};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PrintBedPlaceholders {
    min: [String; 2],
    max: [String; 2],
    size: [String; 2],
}

impl PrintBedPlaceholders {
    pub(crate) fn min_list(&self) -> String {
        self.min.join(",")
    }

    pub(crate) fn max_list(&self) -> String {
        self.max.join(",")
    }

    pub(crate) fn size_list(&self) -> String {
        self.size.join(",")
    }
}

pub(crate) fn placeholders(options: &SliceOptions) -> Result<PrintBedPlaceholders, SliceError> {
    let points = printable_area_points(options)?;
    let bounds = Bounds::from_points(&points)?;
    Ok(PrintBedPlaceholders {
        min: [
            format_decimal(bounds.min.x()),
            format_decimal(bounds.min.y()),
        ],
        max: [
            format_decimal(bounds.max.x()),
            format_decimal(bounds.max.y()),
        ],
        size: [
            format_decimal(bounds.max.x() - bounds.min.x()),
            format_decimal(bounds.max.y() - bounds.min.y()),
        ],
    })
}

pub(crate) fn template_contains_placeholder(template: &str) -> bool {
    template.contains("[print_bed_min]")
        || template.contains("[print_bed_max]")
        || template.contains("[print_bed_size]")
}

fn printable_area_points(options: &SliceOptions) -> Result<Vec<Point2>, SliceError> {
    match options.values().get("printable_area") {
        Some(value) => parse_printable_area(value),
        None => parse_printable_area_text(
            crate::options::registry::option_definition("printable_area")
                .expect("printable_area option definition exists")
                .default_value,
        ),
    }
}

fn parse_printable_area(value: &Value) -> Result<Vec<Point2>, SliceError> {
    match value {
        Value::String(text) => parse_printable_area_text(text),
        Value::Array(values) => parse_printable_area_array(values),
        _ => Err(invalid("must be a point list")),
    }
}

fn parse_printable_area_text(text: &str) -> Result<Vec<Point2>, SliceError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(invalid("must contain at least two points"));
    }

    let mut points = Vec::new();
    for token in text.split(',') {
        let token = token.trim();
        let Some((x, y)) = token.split_once('x') else {
            return Err(invalid("has a malformed point"));
        };
        if y.contains('x') || x.trim().is_empty() || y.trim().is_empty() {
            return Err(invalid("has a malformed point"));
        }
        points.push(point(parse_coordinate(x)?, parse_coordinate(y)?)?);
    }
    require_multiple_points(points)
}

fn parse_printable_area_array(values: &[Value]) -> Result<Vec<Point2>, SliceError> {
    let points = values
        .iter()
        .map(parse_point_array)
        .collect::<Result<Vec<_>, _>>()?;
    require_multiple_points(points)
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

fn require_multiple_points(points: Vec<Point2>) -> Result<Vec<Point2>, SliceError> {
    if points.len() >= 2 {
        Ok(points)
    } else {
        Err(invalid("must contain at least two points"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    min: Point2,
    max: Point2,
}

impl Bounds {
    fn from_points(points: &[Point2]) -> Result<Self, SliceError> {
        let Some((first, rest)) = points.split_first() else {
            return Err(invalid("must contain at least two points"));
        };
        if rest.is_empty() {
            return Err(invalid("must contain at least two points"));
        }
        let mut bounds = Self {
            min: *first,
            max: *first,
        };
        for point in rest {
            bounds.include(*point);
        }
        Ok(bounds)
    }

    fn include(&mut self, point: Point2) {
        self.min = Point2::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
        self.max = Point2::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
    }
}

fn invalid(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("printable_area {reason}"))
}
