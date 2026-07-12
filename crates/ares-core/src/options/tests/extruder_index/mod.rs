use crate::{ExtruderIndexIdMapLookup, SliceError, SliceOptions};
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn complete_lookup<'a>(
    extruder_or_filament_id: i32,
    id_and_variant: (&'a str, &'a str),
    extruder_and_nozzle: (&'a str, &'a str),
    stride: usize,
) -> ExtruderIndexIdMapLookup<'a> {
    ExtruderIndexIdMapLookup {
        extruder_or_filament_id,
        id_name: id_and_variant.0,
        extruder_type: extruder_and_nozzle.0,
        nozzle_volume_type: extruder_and_nozzle.1,
        variant_name: id_and_variant.1,
        stride,
    }
}

mod complete_id;
mod generated_id;
mod no_id;
