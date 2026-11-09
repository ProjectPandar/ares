use crate::{FloatOrPercent, Nullable, OrcaFloat};

/// `GCodeWriter.cpp:32-33`: flavors using machine limits in the writer cache.
pub(super) fn use_mach_limits(flavor: crate::GCodeFlavor) -> bool {
    matches!(
        flavor,
        crate::GCodeFlavor::MarlinLegacy
            | crate::GCodeFlavor::MarlinFirmware
            | crate::GCodeFlavor::Klipper
            | crate::GCodeFlavor::RepRapFirmware
    )
}

/// `GCodeWriter::apply_print_config` caps print acceleration by the machine
/// extruding limit; Klipper also clamps against the X/Y axis limits.
pub(super) fn machine_acceleration_limit(full: &crate::options::ProjectSettings) -> u32 {
    let flavor = full.printer.gcode.gcode_flavor;
    if !use_mach_limits(flavor) {
        return 0;
    }
    let mut limit =
        first_float(&full.printer.machine.machine_max_acceleration_extruding).round() as u32;
    if flavor == crate::GCodeFlavor::Klipper {
        let axis_limit = [
            &full.printer.machine.machine_max_acceleration_x,
            &full.printer.machine.machine_max_acceleration_y,
        ]
        .into_iter()
        .map(|axis| first_float(axis).round() as u32)
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
) -> f64 {
    object.map_or(fallback, value)
}

pub(super) fn absolute(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

/// Final `GCode.cpp` selection or writer machine-limit conversion: floor(+0.5).
pub(in crate::project_slice::gcode_emit::motion) fn rounded_acceleration(value: f64) -> u32 {
    (value + 0.5).floor() as u32
}

pub(super) fn first_float(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

pub(in crate::project_slice::gcode_emit) fn nullable_float_at(
    values: &[Nullable<OrcaFloat>],
    index: usize,
    default: f64,
) -> f64 {
    values
        .get(index)
        .or_else(|| values.first())
        .and_then(|value| match value {
            Nullable::Value(value) => Some(value.0),
            Nullable::Nil => None,
        })
        .unwrap_or(default)
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
