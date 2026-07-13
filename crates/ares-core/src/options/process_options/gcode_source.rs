mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaString, OrcaStrings, Percent,
    option_group::declare_option_group,
};

declare_option_group! {
    pub struct ProcessGCodeSourceOptions, ProcessGCodeSourceOptionsBuilder {
        enable_arc_fitting => "enable_arc_fitting": OrcaBool = OrcaBool(false),
        enable_wrapping_detection => "enable_wrapping_detection": OrcaBool = OrcaBool(false),
        gcode_add_line_number => "gcode_add_line_number": OrcaBool = OrcaBool(false),
        max_volumetric_extrusion_rate_slope => "max_volumetric_extrusion_rate_slope": OrcaFloat = OrcaFloat(0.0),
        max_volumetric_extrusion_rate_slope_segment_length => "max_volumetric_extrusion_rate_slope_segment_length": OrcaFloat = OrcaFloat(3.0),
        extrusion_rate_smoothing_external_perimeter_only => "extrusion_rate_smoothing_external_perimeter_only": OrcaBool = OrcaBool(false),
        single_extruder_multi_material_priming => "single_extruder_multi_material_priming": OrcaBool = OrcaBool(false),
        wipe_tower_no_sparse_layers => "wipe_tower_no_sparse_layers": OrcaBool = OrcaBool(false),
        process_change_extrusion_role_gcode => "process_change_extrusion_role_gcode": OrcaString = string(""),
        travel_speed => "travel_speed": OrcaFloat = OrcaFloat(120.0),
        travel_speed_z => "travel_speed_z": OrcaFloat = OrcaFloat(0.0),
        accel_to_decel_enable => "accel_to_decel_enable": OrcaBool = OrcaBool(true),
        accel_to_decel_factor => "accel_to_decel_factor": Percent = Percent(50.0),
        initial_layer_travel_speed => "initial_layer_travel_speed": FloatOrPercent = FloatOrPercent::Percent(Percent(100.0)),
        initial_layer_travel_acceleration => "initial_layer_travel_acceleration": FloatOrPercent = FloatOrPercent::Percent(Percent(100.0)),
        initial_layer_travel_jerk => "initial_layer_travel_jerk": FloatOrPercent = FloatOrPercent::Percent(Percent(100.0)),
        small_area_infill_flow_compensation_model => "small_area_infill_flow_compensation_model": OrcaStrings = strings(&[
            "0,0", "\n0.2,0.4444", "\n0.4,0.6145", "\n0.6,0.7059", "\n0.8,0.7619",
            "\n1.5,0.8571", "\n2,0.8889", "\n3,0.9231", "\n5,0.9520", "\n10,1",
        ]),
    }
}

impl ProcessGCodeSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 17] = [
        "enable_arc_fitting",
        "enable_wrapping_detection",
        "gcode_add_line_number",
        "max_volumetric_extrusion_rate_slope",
        "max_volumetric_extrusion_rate_slope_segment_length",
        "extrusion_rate_smoothing_external_perimeter_only",
        "single_extruder_multi_material_priming",
        "wipe_tower_no_sparse_layers",
        "process_change_extrusion_role_gcode",
        "travel_speed",
        "travel_speed_z",
        "accel_to_decel_enable",
        "accel_to_decel_factor",
        "initial_layer_travel_speed",
        "initial_layer_travel_acceleration",
        "initial_layer_travel_jerk",
        "small_area_infill_flow_compensation_model",
    ];
}

impl Default for ProcessGCodeSourceOptions {
    fn default() -> Self {
        ProcessGCodeSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessGCodeSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GCodeSourceVisitor)
    }
}

struct GCodeSourceVisitor;

impl<'de> Visitor<'de> for GCodeSourceVisitor {
    type Value = ProcessGCodeSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca GCodeConfig process options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessGCodeSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &ProcessGCodeSourceOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn string(value: &str) -> OrcaString {
    OrcaString(value.to_owned())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}
