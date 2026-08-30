use crate::{FloatOrPercent, Nullable, OrcaFloat};

/// `GCodeWriter::apply_print_config` caps print acceleration by the machine
/// extruding limit; Klipper also clamps against the X/Y axis limits.
pub(super) fn machine_acceleration_limit(full: &crate::options::ProjectSettings) -> u32 {
    let flavor = full.printer.gcode.gcode_flavor;
    if !matches!(
        flavor,
        crate::GCodeFlavor::MarlinLegacy
            | crate::GCodeFlavor::MarlinFirmware
            | crate::GCodeFlavor::Klipper
            | crate::GCodeFlavor::RepRapFirmware
    ) {
        return 0;
    }
    let mut limit = rounded_acceleration(first_float(
        &full.printer.machine.machine_max_acceleration_extruding,
    ));
    if flavor == crate::GCodeFlavor::Klipper {
        let axis_limit = [
            &full.printer.machine.machine_max_acceleration_x,
            &full.printer.machine.machine_max_acceleration_y,
        ]
        .into_iter()
        .map(|axis| rounded_acceleration(first_float(axis)))
        .filter(|limit| *limit > 0)
        .min()
        .unwrap_or(u32::MAX);
        limit = limit.min(axis_limit);
    }
    limit
}

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
