use crate::{PrinterExtruderUpdate, SliceError, SliceOptions};

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn update<'a>(
    target: &mut SliceOptions,
    printer_config: &'a SliceOptions,
    key_set: &'a [&'a str],
    extruder_id: usize,
) -> Result<(), SliceError> {
    target.update_values_to_printer_extruders_string_int_keys(PrinterExtruderUpdate {
        printer_config,
        key_set,
        id_name: "printer_extruder_id",
        variant_name: "printer_extruder_variant",
        stride: 2,
        extruder_id,
    })
}

fn printer(value: serde_json::Value) -> SliceOptions {
    options(value)
}

fn update_multiple_filaments<'a>(
    target: &mut SliceOptions,
    printer_config: &'a SliceOptions,
    key_set: &'a [&'a str],
) -> Result<(), SliceError> {
    target.update_values_to_printer_extruders_for_multiple_filaments_string_int_keys(
        crate::PrinterExtruderMultipleFilamentUpdate {
            printer_config,
            key_set,
            id_name: "filament_self_index",
            variant_name: "filament_extruder_variant",
        },
    )
}

mod bool;
mod enums;
mod float_percent;
mod multiple_filament_bool;
mod multiple_filament_enum;
mod multiple_filament_float_or_percent;
mod multiple_filament_float_percent;
mod multiple_filament_string_int;
mod string_int;
