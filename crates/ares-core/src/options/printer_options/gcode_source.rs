mod enums;
mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

pub use enums::{
    ExtruderType, ExtruderTypes, NozzleType, NullableInts, NullableNozzleTypes, PrinterStructure,
    RetractLiftEnforces, WipeTowerType, ZHopType, ZHopTypes,
};

use super::super::{
    BedTemperatureFormula, GCodeFlavor, PowerLossRecoveryMode, RetractLiftEnforce,
    config_types::{
        Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaString,
        OrcaStrings, Point2dList,
    },
    option_group::declare_option_group,
};

declare_option_group! {
    pub struct PrinterGCodeSourceOptions, PrinterGCodeSourceOptionsBuilder {
        before_layer_change_gcode => "before_layer_change_gcode": OrcaString = string(""),
        printing_by_object_gcode => "printing_by_object_gcode": OrcaString = string(""),
        machine_end_gcode => "machine_end_gcode": OrcaString = string("M104 S0 ; turn off temperature\nG28 X0  ; home X axis\nM84     ; disable motors\n"),
        fan_kickstart => "fan_kickstart": OrcaFloat = OrcaFloat(0.0),
        fan_speedup_overhangs => "fan_speedup_overhangs": OrcaBool = OrcaBool(true),
        fan_speedup_time => "fan_speedup_time": OrcaFloat = OrcaFloat(0.0),
        part_cooling_fan_min_pwm => "part_cooling_fan_min_pwm": OrcaInt = OrcaInt(0),
        support_object_skip_flush => "support_object_skip_flush": OrcaBool = OrcaBool(false),
        bed_temperature_formula => "bed_temperature_formula": BedTemperatureFormula = BedTemperatureFormula::HighestTemp,
        physical_extruder_map => "physical_extruder_map": OrcaInts = ints(&[0]),
        nozzle_flush_dataset => "nozzle_flush_dataset": NullableInts = nullable_ints(&[0]),
        scan_first_layer => "scan_first_layer": OrcaBool = OrcaBool(false),
        enable_power_loss_recovery => "enable_power_loss_recovery": PowerLossRecoveryMode = PowerLossRecoveryMode::PrinterConfiguration,
        wrapping_detection_layers => "wrapping_detection_layers": OrcaInt = OrcaInt(20),
        wrapping_exclude_area => "wrapping_exclude_area": Point2dList = Point2dList(Vec::new()),
        gcode_flavor => "gcode_flavor": GCodeFlavor = GCodeFlavor::MarlinLegacy,
        time_cost => "time_cost": OrcaFloat = OrcaFloat(0.0),
        layer_change_gcode => "layer_change_gcode": OrcaString = string(""),
        time_lapse_gcode => "time_lapse_gcode": OrcaString = string(""),
        wrapping_detection_gcode => "wrapping_detection_gcode": OrcaString = string(""),
        enable_long_retraction_when_cut => "enable_long_retraction_when_cut": OrcaInt = OrcaInt(0),
        retraction_distances_when_cut => "retraction_distances_when_cut": OrcaFloats = floats(&[18.0]),
        long_retractions_when_cut => "long_retractions_when_cut": OrcaBools = bools(&[false]),
        z_hop_types => "z_hop_types": ZHopTypes = ZHopTypes(vec![ZHopType::Slope]),
        travel_slope => "travel_slope": OrcaFloats = floats(&[3.0]),
        retract_lift_enforce => "retract_lift_enforce": RetractLiftEnforces = RetractLiftEnforces(vec![RetractLiftEnforce::AllSurfaces]),
        file_start_gcode => "file_start_gcode": OrcaString = string(""),
        machine_start_gcode => "machine_start_gcode": OrcaString = string("G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n"),
        single_extruder_multi_material => "single_extruder_multi_material": OrcaBool = OrcaBool(true),
        manual_filament_change => "manual_filament_change": OrcaBool = OrcaBool(false),
        change_filament_gcode => "change_filament_gcode": OrcaString = string(""),
        change_extrusion_role_gcode => "change_extrusion_role_gcode": OrcaString = string(""),
        silent_mode => "silent_mode": OrcaBool = OrcaBool(false),
        machine_pause_gcode => "machine_pause_gcode": OrcaString = string(""),
        template_custom_gcode => "template_custom_gcode": OrcaString = string(""),
        nozzle_type => "nozzle_type": NullableNozzleTypes = NullableNozzleTypes(vec![Nullable::Value(NozzleType::Undefine)]),
        nozzle_hrc => "nozzle_hrc": OrcaInt = OrcaInt(0),
        auxiliary_fan => "auxiliary_fan": OrcaBool = OrcaBool(false),
        support_air_filtration => "support_air_filtration": OrcaBool = OrcaBool(true),
        printer_structure => "printer_structure": PrinterStructure = PrinterStructure::Undefine,
        support_chamber_temp_control => "support_chamber_temp_control": OrcaBool = OrcaBool(true),
        extruder_type => "extruder_type": ExtruderTypes = ExtruderTypes(vec![ExtruderType::DirectDrive]),
        printer_extruder_id => "printer_extruder_id": OrcaInts = ints(&[1]),
        master_extruder_id => "master_extruder_id": OrcaInt = OrcaInt(1),
        printer_extruder_variant => "printer_extruder_variant": OrcaStrings = strings(&["Direct Drive Standard"]),
        use_firmware_retraction => "use_firmware_retraction": OrcaBool = OrcaBool(false),
        use_relative_e_distances => "use_relative_e_distances": OrcaBool = OrcaBool(true),
        disable_m73 => "disable_m73": OrcaBool = OrcaBool(false),
        cooling_tube_retraction => "cooling_tube_retraction": OrcaFloat = OrcaFloat(91.5),
        cooling_tube_length => "cooling_tube_length": OrcaFloat = OrcaFloat(5.0),
        high_current_on_filament_swap => "high_current_on_filament_swap": OrcaBool = OrcaBool(false),
        parking_pos_retraction => "parking_pos_retraction": OrcaFloat = OrcaFloat(92.0),
        extra_loading_move => "extra_loading_move": OrcaFloat = OrcaFloat(-2.0),
        machine_load_filament_time => "machine_load_filament_time": OrcaFloat = OrcaFloat(0.0),
        machine_tool_change_time => "machine_tool_change_time": OrcaFloat = OrcaFloat(0.0),
        machine_unload_filament_time => "machine_unload_filament_time": OrcaFloat = OrcaFloat(0.0),
        wipe_tower_type => "wipe_tower_type": WipeTowerType = WipeTowerType::Type2,
        purge_in_prime_tower => "purge_in_prime_tower": OrcaBool = OrcaBool(true),
        enable_filament_ramming => "enable_filament_ramming": OrcaBool = OrcaBool(true),
        tool_change_on_wipe_tower => "tool_change_on_wipe_tower": OrcaBool = OrcaBool(false),
        support_multi_bed_types => "support_multi_bed_types": OrcaBool = OrcaBool(false),
        use_3mf => "use_3mf": OrcaBool = OrcaBool(false),
    }
}

impl PrinterGCodeSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 62] = [
        "before_layer_change_gcode",
        "printing_by_object_gcode",
        "machine_end_gcode",
        "fan_kickstart",
        "fan_speedup_overhangs",
        "fan_speedup_time",
        "part_cooling_fan_min_pwm",
        "support_object_skip_flush",
        "bed_temperature_formula",
        "physical_extruder_map",
        "nozzle_flush_dataset",
        "scan_first_layer",
        "enable_power_loss_recovery",
        "wrapping_detection_layers",
        "wrapping_exclude_area",
        "gcode_flavor",
        "time_cost",
        "layer_change_gcode",
        "time_lapse_gcode",
        "wrapping_detection_gcode",
        "enable_long_retraction_when_cut",
        "retraction_distances_when_cut",
        "long_retractions_when_cut",
        "z_hop_types",
        "travel_slope",
        "retract_lift_enforce",
        "file_start_gcode",
        "machine_start_gcode",
        "single_extruder_multi_material",
        "manual_filament_change",
        "change_filament_gcode",
        "change_extrusion_role_gcode",
        "silent_mode",
        "machine_pause_gcode",
        "template_custom_gcode",
        "nozzle_type",
        "nozzle_hrc",
        "auxiliary_fan",
        "support_air_filtration",
        "printer_structure",
        "support_chamber_temp_control",
        "extruder_type",
        "printer_extruder_id",
        "master_extruder_id",
        "printer_extruder_variant",
        "use_firmware_retraction",
        "use_relative_e_distances",
        "disable_m73",
        "cooling_tube_retraction",
        "cooling_tube_length",
        "high_current_on_filament_swap",
        "parking_pos_retraction",
        "extra_loading_move",
        "machine_load_filament_time",
        "machine_tool_change_time",
        "machine_unload_filament_time",
        "wipe_tower_type",
        "purge_in_prime_tower",
        "enable_filament_ramming",
        "tool_change_on_wipe_tower",
        "support_multi_bed_types",
        "use_3mf",
    ];
}

impl Default for PrinterGCodeSourceOptions {
    fn default() -> Self {
        PrinterGCodeSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for PrinterGCodeSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GCodeSourceVisitor)
    }
}

struct GCodeSourceVisitor;

impl<'de> Visitor<'de> for GCodeSourceVisitor {
    type Value = PrinterGCodeSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca GCodeConfig printer options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = PrinterGCodeSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &PrinterGCodeSourceOptions::DECLARATION_ORDER,
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

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn nullable_ints(values: &[i32]) -> NullableInts {
    NullableInts(
        values
            .iter()
            .copied()
            .map(|value| Nullable::Value(OrcaInt(value)))
            .collect(),
    )
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}
