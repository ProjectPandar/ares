mod enums;
pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::RawOverhangFanThreshold;

use super::super::{
    OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaStrings,
    Percent, option_group::declare_option_group,
};

declare_option_group! {
    pub struct FilamentPrintSourceOptions, FilamentPrintSourceOptionsBuilder {
        additional_cooling_fan_speed => "additional_cooling_fan_speed": OrcaInts = ints(&[0]),
        close_additional_fan_first_x_layers => "close_additional_fan_first_x_layers": OrcaInts = ints(&[1]),
        additional_fan_full_speed_layer => "additional_fan_full_speed_layer": OrcaInts = ints(&[0]),
        first_x_layer_fan_speed => "first_x_layer_fan_speed": OrcaFloats = floats(&[0.0]),
        cool_plate_temp => "cool_plate_temp": OrcaInts = ints(&[35]),
        textured_cool_plate_temp => "textured_cool_plate_temp": OrcaInts = ints(&[40]),
        supertack_plate_temp => "supertack_plate_temp": OrcaInts = ints(&[35]),
        eng_plate_temp => "eng_plate_temp": OrcaInts = ints(&[45]),
        hot_plate_temp => "hot_plate_temp": OrcaInts = ints(&[45]),
        textured_plate_temp => "textured_plate_temp": OrcaInts = ints(&[45]),
        supertack_plate_temp_initial_layer => "supertack_plate_temp_initial_layer": OrcaInts = ints(&[35]),
        cool_plate_temp_initial_layer => "cool_plate_temp_initial_layer": OrcaInts = ints(&[35]),
        textured_cool_plate_temp_initial_layer => "textured_cool_plate_temp_initial_layer": OrcaInts = ints(&[40]),
        eng_plate_temp_initial_layer => "eng_plate_temp_initial_layer": OrcaInts = ints(&[45]),
        hot_plate_temp_initial_layer => "hot_plate_temp_initial_layer": OrcaInts = ints(&[45]),
        textured_plate_temp_initial_layer => "textured_plate_temp_initial_layer": OrcaInts = ints(&[45]),
        enable_overhang_bridge_fan => "enable_overhang_bridge_fan": OrcaBools = bools(&[true]),
        overhang_fan_speed => "overhang_fan_speed": OrcaInts = ints(&[100]),
        overhang_fan_threshold => "overhang_fan_threshold": Vec<RawOverhangFanThreshold> = overhang_thresholds(&[RawOverhangFanThreshold::Percent95]),
        slow_down_for_layer_cooling => "slow_down_for_layer_cooling": OrcaBools = bools(&[true]),
        close_fan_the_first_x_layers => "close_fan_the_first_x_layers": OrcaInts = ints(&[1]),
        reduce_fan_stop_start_freq => "reduce_fan_stop_start_freq": OrcaBools = bools(&[false]),
        dont_slow_down_outer_wall => "dont_slow_down_outer_wall": OrcaBools = bools(&[false]),
        fan_cooling_layer_time => "fan_cooling_layer_time": OrcaFloats = floats(&[60.0]),
        activate_air_filtration => "activate_air_filtration": OrcaBools = bools(&[false]),
        activate_air_filtration_during_print => "activate_air_filtration_during_print": OrcaBools = bools(&[true]),
        activate_air_filtration_on_completion => "activate_air_filtration_on_completion": OrcaBools = bools(&[true]),
        during_print_exhaust_fan_speed => "during_print_exhaust_fan_speed": OrcaInts = ints(&[60]),
        complete_print_exhaust_fan_speed => "complete_print_exhaust_fan_speed": OrcaInts = ints(&[80]),
        nozzle_temperature_initial_layer => "nozzle_temperature_initial_layer": OrcaInts = ints(&[200]),
        full_fan_speed_layer => "full_fan_speed_layer": OrcaInts = ints(&[0]),
        fan_max_speed => "fan_max_speed": OrcaFloats = floats(&[100.0]),
        fan_min_speed => "fan_min_speed": OrcaFloats = floats(&[20.0]),
        slow_down_min_speed => "slow_down_min_speed": OrcaFloats = floats(&[10.0]),
        slow_down_layer_time => "slow_down_layer_time": OrcaFloats = floats(&[5.0]),
        nozzle_temperature => "nozzle_temperature": OrcaInts = ints(&[200]),
        nozzle_temperature_range_low => "nozzle_temperature_range_low": OrcaInts = ints(&[190]),
        nozzle_temperature_range_high => "nozzle_temperature_range_high": OrcaInts = ints(&[240]),
        idle_temperature => "idle_temperature": OrcaInts = ints(&[0]),
        filament_shrink => "filament_shrink": OrcaPercents = percents(&[100.0]),
        filament_shrinkage_compensation_z => "filament_shrinkage_compensation_z": OrcaPercents = percents(&[100.0]),
        support_material_interface_fan_speed => "support_material_interface_fan_speed": OrcaInts = ints(&[-1]),
        internal_bridge_fan_speed => "internal_bridge_fan_speed": OrcaInts = ints(&[-1]),
        ironing_fan_speed => "ironing_fan_speed": OrcaInts = ints(&[-1]),
        filament_notes => "filament_notes": OrcaStrings = strings(&[""]),
        activate_chamber_temp_control => "activate_chamber_temp_control": OrcaBools = bools(&[false]),
        chamber_temperature => "chamber_temperature": OrcaInts = ints(&[0]),
        chamber_minimal_temperature => "chamber_minimal_temperature": OrcaInts = ints(&[0]),
    }
}

impl FilamentPrintSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 48] = [
        "additional_cooling_fan_speed",
        "close_additional_fan_first_x_layers",
        "additional_fan_full_speed_layer",
        "first_x_layer_fan_speed",
        "cool_plate_temp",
        "textured_cool_plate_temp",
        "supertack_plate_temp",
        "eng_plate_temp",
        "hot_plate_temp",
        "textured_plate_temp",
        "supertack_plate_temp_initial_layer",
        "cool_plate_temp_initial_layer",
        "textured_cool_plate_temp_initial_layer",
        "eng_plate_temp_initial_layer",
        "hot_plate_temp_initial_layer",
        "textured_plate_temp_initial_layer",
        "enable_overhang_bridge_fan",
        "overhang_fan_speed",
        "overhang_fan_threshold",
        "slow_down_for_layer_cooling",
        "close_fan_the_first_x_layers",
        "reduce_fan_stop_start_freq",
        "dont_slow_down_outer_wall",
        "fan_cooling_layer_time",
        "activate_air_filtration",
        "activate_air_filtration_during_print",
        "activate_air_filtration_on_completion",
        "during_print_exhaust_fan_speed",
        "complete_print_exhaust_fan_speed",
        "nozzle_temperature_initial_layer",
        "full_fan_speed_layer",
        "fan_max_speed",
        "fan_min_speed",
        "slow_down_min_speed",
        "slow_down_layer_time",
        "nozzle_temperature",
        "nozzle_temperature_range_low",
        "nozzle_temperature_range_high",
        "idle_temperature",
        "filament_shrink",
        "filament_shrinkage_compensation_z",
        "support_material_interface_fan_speed",
        "internal_bridge_fan_speed",
        "ironing_fan_speed",
        "filament_notes",
        "activate_chamber_temp_control",
        "chamber_temperature",
        "chamber_minimal_temperature",
    ];
}

impl Default for FilamentPrintSourceOptions {
    fn default() -> Self {
        FilamentPrintSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentPrintSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PrintSourceVisitor)
    }
}

struct PrintSourceVisitor;

impl<'de> Visitor<'de> for PrintSourceVisitor {
    type Value = FilamentPrintSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca PrintConfig filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentPrintSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &Self::Value::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn percents(values: &[f64]) -> OrcaPercents {
    OrcaPercents(values.iter().copied().map(Percent).collect())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}

fn overhang_thresholds(values: &[RawOverhangFanThreshold]) -> Vec<RawOverhangFanThreshold> {
    values.to_vec()
}
