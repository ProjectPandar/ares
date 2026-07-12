use std::collections::BTreeMap;

use serde_json::Value;

use crate::{SliceError, options::infill::patterns::parse_infill_rotate_template};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IroningType {
    NoIroning,
    TopSurfaces,
    TopmostOnly,
    AllSolid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IroningPattern {
    Rectilinear,
    Concentric,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrdinaryIroningConfig {
    ironing_type: IroningType,
    pattern: IroningPattern,
    inset_mm: f64,
    spacing_mm: f64,
    angle_degrees: f64,
    angle_fixed: bool,
    solid_infill_direction_degrees: f64,
    solid_infill_rotate_template_degrees: Vec<f64>,
}

impl OrdinaryIroningConfig {
    pub(crate) const fn ironing_type(&self) -> IroningType {
        self.ironing_type
    }

    pub(crate) const fn pattern(&self) -> IroningPattern {
        self.pattern
    }

    pub(crate) const fn inset_mm(&self) -> f64 {
        self.inset_mm
    }

    pub(crate) const fn spacing_mm(&self) -> f64 {
        self.spacing_mm
    }

    pub(crate) fn rectilinear_angle_degrees(&self, layer_index: usize) -> f64 {
        let base_angle = if self.angle_fixed {
            0.0
        } else if self.solid_infill_rotate_template_degrees.is_empty() {
            self.solid_infill_direction_degrees
                + if layer_index.is_multiple_of(2) {
                    0.0
                } else {
                    90.0
                }
        } else {
            self.solid_infill_rotate_template_degrees
                [layer_index % self.solid_infill_rotate_template_degrees.len()]
        };
        (base_angle + self.angle_degrees) % 360.0
    }
}

pub(crate) fn parse(
    values: &BTreeMap<String, Value>,
    first_nozzle_diameter_mm: f64,
) -> Result<OrdinaryIroningConfig, SliceError> {
    let ironing_type = parse_ironing_type(values)?;
    let pattern = parse_ironing_pattern(values)?;
    let inset_mm = parse_selected_inset_mm(values, first_nozzle_diameter_mm)?;
    let spacing_mm = parse_selected_spacing_mm(values)?;
    let angle_degrees = crate::options::parsing::parse_range_f64(
        "ironing_angle",
        values.get("ironing_angle"),
        0.0,
        0.0,
        359.0,
    )?;
    let angle_fixed = parse_ironing_angle_fixed(values)?;
    let solid_infill_direction_degrees = crate::options::parsing::parse_range_f64(
        "solid_infill_direction",
        values.get("solid_infill_direction"),
        45.0,
        0.0,
        360.0,
    )?;
    let solid_infill_rotate_template_degrees = parse_infill_rotate_template(
        "solid_infill_rotate_template",
        values.get("solid_infill_rotate_template"),
    )?;
    Ok(OrdinaryIroningConfig {
        ironing_type,
        pattern,
        inset_mm,
        spacing_mm,
        angle_degrees,
        angle_fixed,
        solid_infill_direction_degrees,
        solid_infill_rotate_template_degrees,
    })
}

fn parse_ironing_angle_fixed(values: &BTreeMap<String, Value>) -> Result<bool, SliceError> {
    let Some(value) = values.get("ironing_angle_fixed") else {
        return Ok(false);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput("ironing_angle_fixed must be a boolean".to_owned()))
}

fn parse_ironing_pattern(values: &BTreeMap<String, Value>) -> Result<IroningPattern, SliceError> {
    let Some(value) = values.get("ironing_pattern") else {
        return Ok(IroningPattern::Rectilinear);
    };
    let Value::String(text) = value else {
        return Err(SliceError::InvalidInput(
            "ironing_pattern must be a string".to_owned(),
        ));
    };
    match text.as_str() {
        "rectilinear" => Ok(IroningPattern::Rectilinear),
        "concentric" => Ok(IroningPattern::Concentric),
        _ => Err(SliceError::InvalidInput(
            "ironing_pattern has invalid value".to_owned(),
        )),
    }
}

fn parse_selected_spacing_mm(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let ordinary = crate::options::parsing::parse_range_f64(
        "ironing_spacing",
        values.get("ironing_spacing"),
        0.1,
        0.0,
        1.0,
    )?;
    match values.get("filament_ironing_spacing") {
        Some(value) => parse_nullable_filament_spacing_mm(value)?.map_or(Ok(ordinary), Ok),
        None => Ok(ordinary),
    }
}

fn parse_nullable_filament_spacing_mm(value: &Value) -> Result<Option<f64>, SliceError> {
    let value = match value {
        Value::Array(values) => values.first().ok_or_else(|| {
            SliceError::InvalidInput("filament_ironing_spacing must not be empty".to_owned())
        })?,
        value => value,
    };
    if value.is_null() || matches!(value, Value::String(text) if text.trim() == "nil") {
        return Ok(None);
    }
    crate::options::parsing::parse_range_f64("filament_ironing_spacing", Some(value), 0.1, 0.0, 1.0)
        .map(Some)
}

fn parse_selected_inset_mm(
    values: &BTreeMap<String, Value>,
    first_nozzle_diameter_mm: f64,
) -> Result<f64, SliceError> {
    let ordinary = crate::options::parsing::parse_range_f64(
        "ironing_inset",
        values.get("ironing_inset"),
        0.0,
        0.0,
        100.0,
    )?;
    let configured = match values.get("filament_ironing_inset") {
        Some(value) => parse_nullable_filament_inset_mm(value)?.unwrap_or(ordinary),
        None => ordinary,
    };
    Ok(if configured == 0.0 {
        first_nozzle_diameter_mm * 0.5
    } else {
        configured
    })
}

fn parse_nullable_filament_inset_mm(value: &Value) -> Result<Option<f64>, SliceError> {
    let value = match value {
        Value::Array(values) => values.first().ok_or_else(|| {
            SliceError::InvalidInput("filament_ironing_inset must not be empty".to_owned())
        })?,
        value => value,
    };
    if matches!(value, Value::String(text) if text.trim() == "nil") {
        return Ok(None);
    }
    crate::options::parsing::parse_range_f64("filament_ironing_inset", Some(value), 0.0, 0.0, 100.0)
        .map(Some)
}

fn parse_ironing_type(values: &BTreeMap<String, Value>) -> Result<IroningType, SliceError> {
    let Some(value) = values.get("ironing_type") else {
        return Ok(IroningType::NoIroning);
    };
    let Value::String(text) = value else {
        return Err(SliceError::InvalidInput(
            "ironing_type must be a string".to_owned(),
        ));
    };
    match text.as_str() {
        "no ironing" => Ok(IroningType::NoIroning),
        "top" => Ok(IroningType::TopSurfaces),
        "topmost" => Ok(IroningType::TopmostOnly),
        "solid" => Ok(IroningType::AllSolid),
        _ => Err(SliceError::InvalidInput(
            "ironing_type has invalid value".to_owned(),
        )),
    }
}
