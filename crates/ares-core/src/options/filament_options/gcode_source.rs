pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::super::{
    CsvTable, Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaStrings,
    RammingParameters, SpaceTuple, VariantStride, option_group::declare_option_group,
};

declare_option_group! {
    append pub struct FilamentGCodeSourceOptions, FilamentGCodeSourceOptionsBuilder {
        filament_end_gcode => "filament_end_gcode": OrcaStrings = strings(&[" "]),
        filament_flow_ratio => "filament_flow_ratio": Vec<Nullable<OrcaFloat>> = nullable_floats(&[1.0]),
        enable_pressure_advance => "enable_pressure_advance": OrcaBools = bools(&[false]),
        pressure_advance => "pressure_advance": OrcaFloats = floats(&[0.02]),
        adaptive_pressure_advance => "adaptive_pressure_advance": OrcaBools = bools(&[false]),
        adaptive_pressure_advance_overhangs => "adaptive_pressure_advance_overhangs": OrcaBools = bools(&[false]),
        adaptive_pressure_advance_model => "adaptive_pressure_advance_model": CsvTable = csv_table(&["0,0,0\n0,0,0"]),
        adaptive_pressure_advance_bridges => "adaptive_pressure_advance_bridges": OrcaFloats = floats(&[0.0]),
        filament_diameter => "filament_diameter": OrcaFloats = floats(&[1.75]),
        filament_adaptive_volumetric_speed => "filament_adaptive_volumetric_speed": Vec<Nullable<OrcaBool>> = nullable_bools(&[false]),
        volumetric_speed_coefficients => "volumetric_speed_coefficients": SpaceTuple = space_tuple(&[""]),
        filament_adhesiveness_category => "filament_adhesiveness_category": OrcaInts = ints(&[0]),
        filament_density => "filament_density": OrcaFloats = floats(&[0.0]),
        filament_type => "filament_type": OrcaStrings = strings(&["PLA"]),
        filament_soluble => "filament_soluble": OrcaBools = bools(&[false]),
        filament_colour => "filament_colour": OrcaStrings = strings(&["#F2754E"]),
        filament_vendor => "filament_vendor": OrcaStrings = strings(&["(Undefined)"]),
        filament_is_support => "filament_is_support": OrcaBools = bools(&[false]),
        filament_printable => "filament_printable": OrcaInts = ints(&[3]),
        filament_change_length => "filament_change_length": OrcaFloats = floats(&[10.0]),
        filament_cost => "filament_cost": OrcaFloats = floats(&[0.0]),
        default_filament_colour => "default_filament_colour": OrcaStrings = strings(&[""]),
        temperature_vitrification => "temperature_vitrification": OrcaInts = ints(&[100]),
        filament_max_volumetric_speed => "filament_max_volumetric_speed": OrcaFloats = floats(&[2.0]),
        required_nozzle_hrc => "required_nozzle_HRC": OrcaInts = ints(&[0]),
        filament_extruder_variant => "filament_extruder_variant": VariantStride = variant_stride(&["Direct Drive Standard"]),
        filament_flush_volumetric_speed => "filament_flush_volumetric_speed": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.0]),
        filament_flush_temp => "filament_flush_temp": Vec<Nullable<OrcaInt>> = nullable_ints(&[0]),
        retraction_distances_when_ec => "retraction_distances_when_ec": Vec<Nullable<OrcaFloat>> = nullable_floats(&[10.0]),
        long_retractions_when_ec => "long_retractions_when_ec": Vec<Nullable<OrcaBool>> = nullable_bools(&[false]),
        filament_start_gcode => "filament_start_gcode": OrcaStrings = strings(&[" "]),
        filament_change_extrusion_role_gcode => "filament_change_extrusion_role_gcode": OrcaStrings = strings(&[""]),
        filament_loading_speed => "filament_loading_speed": OrcaFloats = floats(&[28.0]),
        filament_loading_speed_start => "filament_loading_speed_start": OrcaFloats = floats(&[3.0]),
        filament_unloading_speed => "filament_unloading_speed": OrcaFloats = floats(&[90.0]),
        filament_unloading_speed_start => "filament_unloading_speed_start": OrcaFloats = floats(&[100.0]),
        filament_toolchange_delay => "filament_toolchange_delay": OrcaFloats = floats(&[0.0]),
        filament_cooling_moves => "filament_cooling_moves": OrcaInts = ints(&[4]),
        filament_cooling_initial_speed => "filament_cooling_initial_speed": OrcaFloats = floats(&[2.2]),
        filament_minimal_purge_on_wipe_tower => "filament_minimal_purge_on_wipe_tower": OrcaFloats = floats(&[15.0]),
        filament_cooling_before_tower => "filament_cooling_before_tower": Vec<Nullable<OrcaFloat>> = nullable_floats(&[10.0]),
        filament_tower_interface_pre_extrusion_dist => "filament_tower_interface_pre_extrusion_dist": OrcaFloats = floats(&[10.0]),
        filament_tower_interface_pre_extrusion_length => "filament_tower_interface_pre_extrusion_length": OrcaFloats = floats(&[0.0]),
        filament_tower_ironing_area => "filament_tower_ironing_area": OrcaFloats = floats(&[4.0]),
        filament_tower_interface_purge_volume => "filament_tower_interface_purge_volume": OrcaFloats = floats(&[20.0]),
        filament_tower_interface_print_temp => "filament_tower_interface_print_temp": OrcaInts = ints(&[-1]),
        filament_cooling_final_speed => "filament_cooling_final_speed": OrcaFloats = floats(&[3.4]),
        filament_ramming_parameters => "filament_ramming_parameters": RammingParameters = ramming_parameters(&[concat!(
            "120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0|",
            " 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10",
            " 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6"
        )]),
        filament_multitool_ramming => "filament_multitool_ramming": OrcaBools = bools(&[false]),
        filament_multitool_ramming_volume => "filament_multitool_ramming_volume": OrcaFloats = floats(&[10.0]),
        filament_multitool_ramming_flow => "filament_multitool_ramming_flow": OrcaFloats = floats(&[10.0]),
        filament_stamping_loading_speed => "filament_stamping_loading_speed": OrcaFloats = floats(&[0.0]),
        filament_stamping_distance => "filament_stamping_distance": OrcaFloats = floats(&[0.0]),
    }
}

impl FilamentGCodeSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 53] = [
        "filament_end_gcode",
        "filament_flow_ratio",
        "enable_pressure_advance",
        "pressure_advance",
        "adaptive_pressure_advance",
        "adaptive_pressure_advance_overhangs",
        "adaptive_pressure_advance_model",
        "adaptive_pressure_advance_bridges",
        "filament_diameter",
        "filament_adaptive_volumetric_speed",
        "volumetric_speed_coefficients",
        "filament_adhesiveness_category",
        "filament_density",
        "filament_type",
        "filament_soluble",
        "filament_colour",
        "filament_vendor",
        "filament_is_support",
        "filament_printable",
        "filament_change_length",
        "filament_cost",
        "default_filament_colour",
        "temperature_vitrification",
        "filament_max_volumetric_speed",
        "required_nozzle_HRC",
        "filament_extruder_variant",
        "filament_flush_volumetric_speed",
        "filament_flush_temp",
        "retraction_distances_when_ec",
        "long_retractions_when_ec",
        "filament_start_gcode",
        "filament_change_extrusion_role_gcode",
        "filament_loading_speed",
        "filament_loading_speed_start",
        "filament_unloading_speed",
        "filament_unloading_speed_start",
        "filament_toolchange_delay",
        "filament_cooling_moves",
        "filament_cooling_initial_speed",
        "filament_minimal_purge_on_wipe_tower",
        "filament_cooling_before_tower",
        "filament_tower_interface_pre_extrusion_dist",
        "filament_tower_interface_pre_extrusion_length",
        "filament_tower_ironing_area",
        "filament_tower_interface_purge_volume",
        "filament_tower_interface_print_temp",
        "filament_cooling_final_speed",
        "filament_ramming_parameters",
        "filament_multitool_ramming",
        "filament_multitool_ramming_volume",
        "filament_multitool_ramming_flow",
        "filament_stamping_loading_speed",
        "filament_stamping_distance",
    ];
}

impl Default for FilamentGCodeSourceOptions {
    fn default() -> Self {
        FilamentGCodeSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentGCodeSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GCodeSourceVisitor)
    }
}

struct GCodeSourceVisitor;

impl<'de> Visitor<'de> for GCodeSourceVisitor {
    type Value = FilamentGCodeSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca GCodeConfig filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentGCodeSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &FilamentGCodeSourceOptions::DECLARATION_ORDER,
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

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(raw_strings(values))
}

fn csv_table(values: &[&str]) -> CsvTable {
    CsvTable(raw_strings(values))
}

fn variant_stride(values: &[&str]) -> VariantStride {
    VariantStride(raw_strings(values))
}

fn ramming_parameters(values: &[&str]) -> RammingParameters {
    RammingParameters(raw_strings(values))
}

fn space_tuple(values: &[&str]) -> SpaceTuple {
    SpaceTuple(raw_strings(values))
}

fn raw_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn nullable_bools(values: &[bool]) -> Vec<Nullable<OrcaBool>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(OrcaBool(value)))
        .collect()
}

fn nullable_floats(values: &[f64]) -> Vec<Nullable<OrcaFloat>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(OrcaFloat(value)))
        .collect()
}

fn nullable_ints(values: &[i32]) -> Vec<Nullable<OrcaInt>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(OrcaInt(value)))
        .collect()
}
