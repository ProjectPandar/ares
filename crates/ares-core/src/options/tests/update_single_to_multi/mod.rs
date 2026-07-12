use crate::{SliceError, SliceOptions};
use serde_json::Value;

mod bool;
mod float;
mod float_or_percent;
mod string_int;

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn update(
    target: &mut SliceOptions,
    source: &SliceOptions,
    keys: &[&str],
) -> Result<isize, SliceError> {
    target.update_values_from_single_to_multi_string_int_float_percent_bool_keys(
        source,
        keys,
        "printer_extruder_variant",
    )
}
