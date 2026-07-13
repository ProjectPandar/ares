use serde::{Serialize, Serializer, ser::SerializeMap};

use super::FilamentPrintSourceOptions;

impl Serialize for FilamentPrintSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(48))?;
        serialize_entries(&mut map, self)?;
        map.end()
    }
}

pub(crate) fn serialize_entries<M>(
    map: &mut M,
    value: &FilamentPrintSourceOptions,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry("activate_air_filtration", &value.activate_air_filtration)?;
    map.serialize_entry(
        "activate_air_filtration_during_print",
        &value.activate_air_filtration_during_print,
    )?;
    map.serialize_entry(
        "activate_air_filtration_on_completion",
        &value.activate_air_filtration_on_completion,
    )?;
    map.serialize_entry(
        "activate_chamber_temp_control",
        &value.activate_chamber_temp_control,
    )?;
    map.serialize_entry(
        "additional_cooling_fan_speed",
        &value.additional_cooling_fan_speed,
    )?;
    map.serialize_entry(
        "additional_fan_full_speed_layer",
        &value.additional_fan_full_speed_layer,
    )?;
    map.serialize_entry(
        "chamber_minimal_temperature",
        &value.chamber_minimal_temperature,
    )?;
    map.serialize_entry("chamber_temperature", &value.chamber_temperature)?;
    map.serialize_entry(
        "close_additional_fan_first_x_layers",
        &value.close_additional_fan_first_x_layers,
    )?;
    map.serialize_entry(
        "close_fan_the_first_x_layers",
        &value.close_fan_the_first_x_layers,
    )?;
    map.serialize_entry(
        "complete_print_exhaust_fan_speed",
        &value.complete_print_exhaust_fan_speed,
    )?;
    map.serialize_entry("cool_plate_temp", &value.cool_plate_temp)?;
    map.serialize_entry(
        "cool_plate_temp_initial_layer",
        &value.cool_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "dont_slow_down_outer_wall",
        &value.dont_slow_down_outer_wall,
    )?;
    map.serialize_entry(
        "during_print_exhaust_fan_speed",
        &value.during_print_exhaust_fan_speed,
    )?;
    map.serialize_entry(
        "enable_overhang_bridge_fan",
        &value.enable_overhang_bridge_fan,
    )?;
    map.serialize_entry("eng_plate_temp", &value.eng_plate_temp)?;
    map.serialize_entry(
        "eng_plate_temp_initial_layer",
        &value.eng_plate_temp_initial_layer,
    )?;
    map.serialize_entry("fan_cooling_layer_time", &value.fan_cooling_layer_time)?;
    map.serialize_entry("fan_max_speed", &value.fan_max_speed)?;
    map.serialize_entry("fan_min_speed", &value.fan_min_speed)?;
    map.serialize_entry("filament_notes", &value.filament_notes)?;
    map.serialize_entry("filament_shrink", &value.filament_shrink)?;
    map.serialize_entry(
        "filament_shrinkage_compensation_z",
        &value.filament_shrinkage_compensation_z,
    )?;
    map.serialize_entry("first_x_layer_fan_speed", &value.first_x_layer_fan_speed)?;
    map.serialize_entry("full_fan_speed_layer", &value.full_fan_speed_layer)?;
    map.serialize_entry("hot_plate_temp", &value.hot_plate_temp)?;
    map.serialize_entry(
        "hot_plate_temp_initial_layer",
        &value.hot_plate_temp_initial_layer,
    )?;
    map.serialize_entry("idle_temperature", &value.idle_temperature)?;
    map.serialize_entry(
        "internal_bridge_fan_speed",
        &value.internal_bridge_fan_speed,
    )?;
    map.serialize_entry("ironing_fan_speed", &value.ironing_fan_speed)?;
    map.serialize_entry("nozzle_temperature", &value.nozzle_temperature)?;
    map.serialize_entry(
        "nozzle_temperature_initial_layer",
        &value.nozzle_temperature_initial_layer,
    )?;
    map.serialize_entry(
        "nozzle_temperature_range_high",
        &value.nozzle_temperature_range_high,
    )?;
    map.serialize_entry(
        "nozzle_temperature_range_low",
        &value.nozzle_temperature_range_low,
    )?;
    map.serialize_entry("overhang_fan_speed", &value.overhang_fan_speed)?;
    map.serialize_entry("overhang_fan_threshold", &value.overhang_fan_threshold)?;
    map.serialize_entry(
        "reduce_fan_stop_start_freq",
        &value.reduce_fan_stop_start_freq,
    )?;
    map.serialize_entry(
        "slow_down_for_layer_cooling",
        &value.slow_down_for_layer_cooling,
    )?;
    map.serialize_entry("slow_down_layer_time", &value.slow_down_layer_time)?;
    map.serialize_entry("slow_down_min_speed", &value.slow_down_min_speed)?;
    map.serialize_entry("supertack_plate_temp", &value.supertack_plate_temp)?;
    map.serialize_entry(
        "supertack_plate_temp_initial_layer",
        &value.supertack_plate_temp_initial_layer,
    )?;
    map.serialize_entry(
        "support_material_interface_fan_speed",
        &value.support_material_interface_fan_speed,
    )?;
    map.serialize_entry("textured_cool_plate_temp", &value.textured_cool_plate_temp)?;
    map.serialize_entry(
        "textured_cool_plate_temp_initial_layer",
        &value.textured_cool_plate_temp_initial_layer,
    )?;
    map.serialize_entry("textured_plate_temp", &value.textured_plate_temp)?;
    map.serialize_entry(
        "textured_plate_temp_initial_layer",
        &value.textured_plate_temp_initial_layer,
    )?;
    Ok(())
}
