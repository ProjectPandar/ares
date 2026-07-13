mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{
    ProcessDraftShield, ProcessPrintOrder, ProcessPrintSequence, ProcessSkirtType,
    ProcessTimelapseType, ProcessWipeTowerWallType,
};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, OrcaString, OrcaStrings, Percent,
    option_group::declare_option_group,
};

declare_option_group! {
    pub struct ProcessPrintSourceOptions, ProcessPrintSourceOptionsBuilder {
        reduce_crossing_wall => "reduce_crossing_wall": OrcaBool = OrcaBool(false),
        max_travel_detour_distance => "max_travel_detour_distance": FloatOrPercent = FloatOrPercent::Float(0.0),
        print_sequence => "print_sequence": ProcessPrintSequence = ProcessPrintSequence::ByLayer,
        print_order => "print_order": ProcessPrintOrder = ProcessPrintOrder::Default,
        draft_shield => "draft_shield": ProcessDraftShield = ProcessDraftShield::Disabled,
        initial_layer_line_width => "initial_layer_line_width": FloatOrPercent = FloatOrPercent::Float(0.0),
        initial_layer_print_height => "initial_layer_print_height": OrcaFloat = OrcaFloat(0.2),
        initial_layer_speed => "initial_layer_speed": OrcaFloat = OrcaFloat(30.0),
        initial_layer_infill_speed => "initial_layer_infill_speed": OrcaFloat = OrcaFloat(60.0),
        reduce_infill_retraction => "reduce_infill_retraction": OrcaBool = OrcaBool(false),
        ooze_prevention => "ooze_prevention": OrcaBool = OrcaBool(false),
        filename_format => "filename_format": OrcaString = string("{input_filename_base}_{filament_type[initial_tool]}_{print_time}.gcode"),
        post_process => "post_process": OrcaStrings = strings(&[]),
        resolution => "resolution": OrcaFloat = OrcaFloat(0.01),
        skirt_distance => "skirt_distance": OrcaFloat = OrcaFloat(2.0),
        skirt_height => "skirt_height": OrcaInt = OrcaInt(1),
        skirt_loops => "skirt_loops": OrcaInt = OrcaInt(1),
        skirt_type => "skirt_type": ProcessSkirtType = ProcessSkirtType::Combined,
        skirt_speed => "skirt_speed": OrcaFloat = OrcaFloat(50.0),
        single_loop_draft_shield => "single_loop_draft_shield": OrcaBool = OrcaBool(false),
        min_skirt_length => "min_skirt_length": OrcaFloat = OrcaFloat(0.0),
        spiral_mode => "spiral_mode": OrcaBool = OrcaBool(false),
        spiral_mode_smooth => "spiral_mode_smooth": OrcaBool = OrcaBool(false),
        spiral_mode_max_xy_smoothing => "spiral_mode_max_xy_smoothing": FloatOrPercent = FloatOrPercent::Percent(Percent(200.0)),
        spiral_finishing_flow_ratio => "spiral_finishing_flow_ratio": OrcaFloat = OrcaFloat(0.0),
        spiral_starting_flow_ratio => "spiral_starting_flow_ratio": OrcaFloat = OrcaFloat(0.0),
        standby_temperature_delta => "standby_temperature_delta": OrcaInt = OrcaInt(-5),
        preheat_time => "preheat_time": OrcaFloat = OrcaFloat(30.0),
        preheat_steps => "preheat_steps": OrcaInt = OrcaInt(1),
        enable_prime_tower => "enable_prime_tower": OrcaBool = OrcaBool(false),
        prime_tower_enable_framework => "prime_tower_enable_framework": OrcaBool = OrcaBool(false),
        prime_tower_width => "prime_tower_width": OrcaFloat = OrcaFloat(60.0),
        wipe_tower_rotation_angle => "wipe_tower_rotation_angle": OrcaFloat = OrcaFloat(0.0),
        prime_tower_brim_width => "prime_tower_brim_width": OrcaFloat = OrcaFloat(3.0),
        prime_tower_infill_gap => "prime_tower_infill_gap": Percent = Percent(150.0),
        prime_tower_skip_points => "prime_tower_skip_points": OrcaBool = OrcaBool(true),
        prime_tower_flat_ironing => "prime_tower_flat_ironing": OrcaBool = OrcaBool(false),
        enable_tower_interface_features => "enable_tower_interface_features": OrcaBool = OrcaBool(false),
        enable_tower_interface_cooldown_during_tower => "enable_tower_interface_cooldown_during_tower": OrcaBool = OrcaBool(false),
        wipe_tower_bridging => "wipe_tower_bridging": OrcaFloat = OrcaFloat(10.0),
        wipe_tower_extra_flow => "wipe_tower_extra_flow": Percent = Percent(100.0),
        wipe_tower_cone_angle => "wipe_tower_cone_angle": OrcaFloat = OrcaFloat(30.0),
        wipe_tower_extra_spacing => "wipe_tower_extra_spacing": Percent = Percent(100.0),
        wipe_tower_max_purge_speed => "wipe_tower_max_purge_speed": OrcaFloat = OrcaFloat(90.0),
        wipe_tower_wall_type => "wipe_tower_wall_type": ProcessWipeTowerWallType = ProcessWipeTowerWallType::Rib,
        wipe_tower_extra_rib_length => "wipe_tower_extra_rib_length": OrcaFloat = OrcaFloat(0.0),
        wipe_tower_rib_width => "wipe_tower_rib_width": OrcaFloat = OrcaFloat(8.0),
        wipe_tower_fillet_wall => "wipe_tower_fillet_wall": OrcaBool = OrcaBool(true),
        wipe_tower_filament => "wipe_tower_filament": OrcaInt = OrcaInt(0),
        wiping_volumes_extruders => "wiping_volumes_extruders": OrcaFloats = floats(&[70.0; 10]),
        prime_volume => "prime_volume": OrcaFloat = OrcaFloat(45.0),
        timelapse_type => "timelapse_type": ProcessTimelapseType = ProcessTimelapseType::Traditional,
        independent_support_layer_height => "independent_support_layer_height": OrcaBool = OrcaBool(true),
        combine_brims => "combine_brims": OrcaBool = OrcaBool(false),
        gcode_label_objects => "gcode_label_objects": OrcaBool = OrcaBool(true),
        exclude_object => "exclude_object": OrcaBool = OrcaBool(false),
        gcode_comments => "gcode_comments": OrcaBool = OrcaBool(false),
        slow_down_layers => "slow_down_layers": OrcaInt = OrcaInt(0),
        notes => "notes": OrcaString = string(""),
    }
}

impl ProcessPrintSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 59] = [
        "reduce_crossing_wall",
        "max_travel_detour_distance",
        "print_sequence",
        "print_order",
        "draft_shield",
        "initial_layer_line_width",
        "initial_layer_print_height",
        "initial_layer_speed",
        "initial_layer_infill_speed",
        "reduce_infill_retraction",
        "ooze_prevention",
        "filename_format",
        "post_process",
        "resolution",
        "skirt_distance",
        "skirt_height",
        "skirt_loops",
        "skirt_type",
        "skirt_speed",
        "single_loop_draft_shield",
        "min_skirt_length",
        "spiral_mode",
        "spiral_mode_smooth",
        "spiral_mode_max_xy_smoothing",
        "spiral_finishing_flow_ratio",
        "spiral_starting_flow_ratio",
        "standby_temperature_delta",
        "preheat_time",
        "preheat_steps",
        "enable_prime_tower",
        "prime_tower_enable_framework",
        "prime_tower_width",
        "wipe_tower_rotation_angle",
        "prime_tower_brim_width",
        "prime_tower_infill_gap",
        "prime_tower_skip_points",
        "prime_tower_flat_ironing",
        "enable_tower_interface_features",
        "enable_tower_interface_cooldown_during_tower",
        "wipe_tower_bridging",
        "wipe_tower_extra_flow",
        "wipe_tower_cone_angle",
        "wipe_tower_extra_spacing",
        "wipe_tower_max_purge_speed",
        "wipe_tower_wall_type",
        "wipe_tower_extra_rib_length",
        "wipe_tower_rib_width",
        "wipe_tower_fillet_wall",
        "wipe_tower_filament",
        "wiping_volumes_extruders",
        "prime_volume",
        "timelapse_type",
        "independent_support_layer_height",
        "combine_brims",
        "gcode_label_objects",
        "exclude_object",
        "gcode_comments",
        "slow_down_layers",
        "notes",
    ];
}

impl Default for ProcessPrintSourceOptions {
    fn default() -> Self {
        ProcessPrintSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProcessPrintSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PrintSourceVisitor)
    }
}

struct PrintSourceVisitor;

impl<'de> Visitor<'de> for PrintSourceVisitor {
    type Value = ProcessPrintSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca PrintConfig process options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProcessPrintSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &ProcessPrintSourceOptions::DECLARATION_ORDER,
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

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}
