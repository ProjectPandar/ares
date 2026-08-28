use crate::{FloatOrPercent, Nullable, OrcaFloat};

pub(super) fn acceleration(
    object: Option<&crate::ObjectOptions>,
    fallback: f64,
    value: impl Fn(&crate::ObjectOptions) -> f64,
) -> u32 {
    rounded_acceleration(object.map_or(fallback, value))
}

pub(super) fn absolute(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

pub(super) fn rounded_acceleration(value: f64) -> u32 {
    (value + 0.5).floor() as u32
}

pub(super) fn first_float(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

pub(in crate::project_slice::gcode_emit) fn first_nullable_float(
    values: &[Nullable<OrcaFloat>],
    default: f64,
) -> f64 {
    values
        .iter()
        .find_map(|value| match value {
            Nullable::Value(value) => Some(value.0),
            Nullable::Nil => None,
        })
        .unwrap_or(default)
}
