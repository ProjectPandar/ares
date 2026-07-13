use crate::{
    AccelerationOptions, BridgeOptions, BrimOptions, ExtrusionOptions, JerkOptions,
    PerimeterOptions, ShellLayerOptions, SliceError, SpeedOptions,
};
macro_rules! option_modules { ($($vis:vis $module:ident),* $(,)?) => { $($vis mod $module;)* }; }
mod acceleration;
#[rustfmt::skip]
option_modules!(adaptive_bed_mesh, bed_excluded_area, defaults, different_extruders, draft_shield, extruder_count, extruder_index, extruder_variant, fdm_normalization);
mod brim;
mod config_types;
#[rustfmt::skip]
option_modules!(pub(crate) filament_config_export, filament_count, filament_override, filament_type, flow_ratios, gap_fill, hardware, infill, small_area_infill_flow);
mod filament_options;
#[rustfmt::skip]
option_modules!(auxiliary_fan, bed_temperature, chamber_temperature, custom_gcode, filament_change, exhaust_fan, fan_speedup, filament_cooling_before_tower, flush_placeholders, gcode_flavor, gcode_output, nozzle_temperature, part_cooling_fan, preheat, timelapse_type, temperature_vector, temperature_vitrification);

#[rustfmt::skip]
option_modules!(input_shaping, pub(crate) ironing_flow, pub(crate) ironing_type);
mod machine_limits;
#[rustfmt::skip]
option_modules!(layer_change_retraction, legacy, initial_layer_print_height, object_distance, overhang_reverse, overhang_speed, parameter_size, raft, skirt_type, support_enable, support_object_skip_flush, support_style, support_placement, pub(crate) support_threshold, support_interface_not_for_body, support_type, pub(crate) support_z_distance, tree_support_options, wall_direction, wall_sequence);
mod object_fields;
mod object_options;
pub(crate) mod option_group;
pub(crate) mod parsing;
mod pellet;
mod physical_extruder_map;
pub(crate) mod power_loss_recovery;
mod preset_metadata;
mod pressure_advance;
mod printer_options;
mod process_options;
mod project_runtime_options;
mod project_settings;
mod relative_e;
mod shell_layers;
mod slow_down_layers;
mod small_perimeter;
mod speed;
mod volumetric_speed;

pub mod registry;
mod support_different_extruders;
mod support_ironing;
mod update_diff_values_to_child_config;
mod update_multi_to_multi;
mod update_non_diff_values_to_base_config;
mod update_printer_extruders;
mod update_single_to_multi;
mod validation;
mod vector_resize;
pub(crate) use acceleration::AccelToDecelConfig;
pub(crate) use bed_temperature::FirstLayerBedTemperature;
use defaults::*;
pub(crate) use fan_speedup::FanSpeedupControl;
pub(crate) use gap_fill::GapFillTarget;
pub(crate) use infill::{
    InfillLayerRole, InfillWallBoundaryOptions, InfillWallOverlapOptions, InternalBridgeFilter,
};
pub(crate) use machine_limits::MachineLimits;
pub(crate) use object_options::ObjectOptionOverrides;
use parsing::{parse_extrusion_width_text, parse_numeric_vector};
pub(crate) use part_cooling_fan::{LayerRoleFanControl, PartCoolingFanRamp};
pub use support_different_extruders::DifferentExtrudersSupport;
pub use update_multi_to_multi::{MultiToMulti2Update, MultiToMultiUpdate};
pub use update_printer_extruders::{PrinterExtruderMultipleFilamentUpdate, PrinterExtruderUpdate};
pub use {
    bed_temperature::BedTemperatureFormula,
    config_types::{
        AmsCounts, CsvTable, FlatMatrix, FloatOrPercent, Millimeters, Nullable,
        NullablePrinterTechnologies, OrcaBool, OrcaBools, OrcaFloat, OrcaFloatOrPercents,
        OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaString, OrcaStrings, OrcaUInt, Percent,
        Point2d, Point2dGroups, Point2dList, PrinterTechnologies, PrinterTechnology,
        RammingParameters, SpaceTuple, VariantStride,
    },
    filament_options::{
        FilamentGCodeSourceOptions, FilamentOptions, FilamentPrintSourceOptions,
        FilamentRegionSourceOptions, FilamentRetractOverrideOptions, RawOverhangFanThreshold,
    },
    gcode_flavor::GCodeFlavor,
    hardware::HardwareOptions,
    infill::{InfillOptions, InfillPattern},
    layer_change_retraction::RetractLiftEnforce,
    object_options::ObjectOptions,
    power_loss_recovery::PowerLossRecoveryMode,
    preset_metadata::PresetMetadata,
    printer_options::{
        AuthorizationType, DefaultBedType, ExtruderType, ExtruderTypes, ExtruderVariantLists,
        InputShaperType, MachineEnvelopeOptions, NozzleType, NozzleVolumeType, NozzleVolumeTypes,
        NullableFloats, NullableInts, NullableNozzleTypes, PrintHostType,
        PrinterGCodeSourceOptions, PrinterModel, PrinterNotes, PrinterOptions,
        PrinterRemainingOptions, PrinterStructure, RetractLiftEnforces, ThumbnailDefinitions,
        WipeTowerType, ZHopType, ZHopTypes,
    },
    process_options::{
        ProcessBrimType, ProcessCounterboreHoleBridging, ProcessDraftShield,
        ProcessEnsureVerticalShellThickness, ProcessExtraBridgeLayer, ProcessFuzzySkinMode,
        ProcessFuzzySkinType, ProcessGCodeSourceOptions, ProcessGapFillTarget,
        ProcessInfillPattern, ProcessInternalBridgeFilter, ProcessIroningType, ProcessNoiseType,
        ProcessObjectSourceOptions, ProcessOptions, ProcessPerimeterGenerator, ProcessPrintOrder,
        ProcessPrintSequence, ProcessPrintSourceOptions, ProcessRegionSourceOptions,
        ProcessSeamPosition, ProcessSeamScarfType, ProcessSkirtType, ProcessSlicingMode,
        ProcessSupportBasePattern, ProcessSupportInterfacePattern, ProcessSupportStyle,
        ProcessSupportType, ProcessTimelapseType, ProcessWallDirection, ProcessWallSequence,
        ProcessWipeTowerWallType,
    },
    project_runtime_options::{
        ProjectBedType, ProjectFilamentMapMode, ProjectGCodeSourceOptions,
        ProjectPresetSourceOptions, ProjectPrintSourceOptions, ProjectRuntimeOptions,
    },
    project_settings::ProjectSettings,
};
pub(crate) use {chamber_temperature::ChamberTemperatureControl, exhaust_fan::ExhaustFanControl};
pub use {extruder_index::ExtruderIndexIdMapLookup, filament_type::FilamentTypeDisplay};
#[rustfmt::skip]
pub(crate) use {input_shaping::InputShapingConfig, layer_change_retraction::ZHopLiftMode};
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct SliceOptions {
    values: std::collections::BTreeMap<String, serde_json::Value>,
}
impl SliceOptions {
    #[rustfmt::skip]
    pub fn values(&self) -> &std::collections::BTreeMap<String, serde_json::Value> { &self.values }
    #[rustfmt::skip]
    pub fn known_definition_count(&self) -> usize { self.values.keys().filter(|key| registry::option_definition(key).is_some()).count() }
    #[rustfmt::skip]
    pub fn layer_height(&self) -> Result<f64, SliceError> { self.positive_f64("layer_height", DEFAULT_LAYER_HEIGHT) }
    #[rustfmt::skip]
    pub fn hardware_options(&self) -> Result<HardwareOptions, SliceError> { Ok(HardwareOptions { nozzle_diameters: self.nozzle_diameters()?, filament_diameters: self.filament_diameters()?, min_layer_heights: self.min_layer_heights()?, max_layer_heights: self.max_layer_heights()? }) }
    #[rustfmt::skip]
    pub fn extrusion_options(&self) -> Result<ExtrusionOptions, SliceError> { flow_ratios::parse_extrusion_options(self) }
    #[rustfmt::skip]
    pub fn speed_options(&self) -> Result<SpeedOptions, SliceError> { speed::parse_speed_options(self) }
    #[rustfmt::skip]
    pub fn acceleration_options(&self) -> Result<AccelerationOptions, SliceError> { acceleration::parse_acceleration_options(&self.values) }
    #[rustfmt::skip]
    pub fn jerk_options(&self) -> Result<JerkOptions, SliceError> { acceleration::parse_jerk_options(&self.values) }
    #[rustfmt::skip]
    pub fn skirt_options(&self) -> Result<crate::SkirtOptions, SliceError> { Ok(crate::SkirtOptions::new(self.non_negative_u32("skirt_loops", DEFAULT_SKIRT_LOOPS)?, self.range_f64("skirt_distance", 2.0, 0.0, f64::INFINITY)?, self.non_negative_u32("skirt_height", DEFAULT_SKIRT_HEIGHT)?, self.range_f64("skirt_speed", 50.0, 0.0, f64::INFINITY)?).with_draft_shield(draft_shield::parse_draft_shield(&self.values)?).with_skirt_type(skirt_type::parse_skirt_type(&self.values)?).with_min_skirt_length_mm(self.range_f64("min_skirt_length", 0.0, 0.0, f64::INFINITY)?).with_single_loop_draft_shield(self.bool_option("single_loop_draft_shield", false)?).with_skirt_start_angle_degrees(self.range_f64("skirt_start_angle", -135.0, -180.0, 180.0)?)) }
    #[rustfmt::skip]
    pub fn brim_options(&self) -> Result<BrimOptions, SliceError> { Ok(BrimOptions::new(self.range_f64("brim_width", 0.0, 0.0, 100.0)?, self.range_f64("brim_object_gap", 0.0, 0.0, 2.0)?, brim::parse_brim_type(&self.values)?).with_combine_brims(self.bool_option("combine_brims", false)?).with_brim_ears_max_angle_degrees(brim::parse_brim_ears_max_angle(&self.values)?).with_brim_ears_detection_length_mm(brim::parse_brim_ears_detection_length(&self.values)?).with_efc_outline_offset_mm(brim::parse_efc_outline_offset(self)?)) }
    #[rustfmt::skip]
    pub fn perimeter_options(&self) -> Result<PerimeterOptions, SliceError> { overhang_reverse::parse_perimeter_options(self) }
    #[rustfmt::skip]
    pub fn bridge_options(&self) -> Result<BridgeOptions, SliceError> { crate::bridges::parse_bridge_options(&self.values) }
    #[rustfmt::skip]
    pub fn infill_options(&self) -> Result<InfillOptions, SliceError> { infill::parse_infill_options(self) }
    #[rustfmt::skip]
    pub fn shell_layer_options(&self) -> Result<ShellLayerOptions, SliceError> { shell_layers::parse_shell_layer_options(self) }
    #[rustfmt::skip]
    pub(crate) fn filter_out_gap_fill_mm(&self) -> Result<f64, SliceError> { gap_fill::parse_filter_out_gap_fill(self) }
    #[rustfmt::skip]
    pub(crate) fn gap_fill_target(&self) -> Result<gap_fill::GapFillTarget, SliceError> { gap_fill::parse_gap_fill_target(self) }
    #[rustfmt::skip]
    pub(crate) fn support_ironing(&self) -> Result<bool, SliceError> { support_ironing::parse_support_ironing(self) }
    #[rustfmt::skip]
    pub(crate) fn support_type(&self) -> Result<support_type::SupportType, SliceError> { support_type::parse(self) }
    #[rustfmt::skip]
    pub(crate) fn support_z_distance_options(&self) -> Result<support_z_distance::SupportZDistanceOptions, SliceError> { support_z_distance::parse(self) }
    pub(crate) fn precise_z_height(&self) -> Result<bool, SliceError> {
        self.bool_option("precise_z_height", false)
    }
    #[rustfmt::skip]
    pub(crate) fn z_offset(&self) -> Result<f64, SliceError> { self.range_f64("z_offset", 0.0, f64::NEG_INFINITY, f64::INFINITY) }
    #[rustfmt::skip]
    pub fn is_infill_first(&self) -> Result<bool, SliceError> { let Some(value) = self.values.get("is_infill_first") else { return Ok(DEFAULT_IS_INFILL_FIRST); }; value.as_bool().ok_or_else(|| SliceError::InvalidInput("is_infill_first must be a boolean".to_owned())) }
    pub fn nozzle_diameters(&self) -> Result<Vec<f64>, SliceError> {
        self.numeric_vector("nozzle_diameter", DEFAULT_NOZZLE_DIAMETERS, |value| {
            value >= 0.005
        })
    }
    #[rustfmt::skip]
    pub fn filament_diameters(&self) -> Result<Vec<f64>, SliceError> { pellet::effective_filament_diameters(self) }
    pub fn min_layer_heights(&self) -> Result<Vec<f64>, SliceError> {
        self.numeric_vector("min_layer_height", DEFAULT_MIN_LAYER_HEIGHTS, |value| {
            value >= 0.0
        })
    }
    pub fn max_layer_heights(&self) -> Result<Vec<f64>, SliceError> {
        self.numeric_vector("max_layer_height", DEFAULT_MAX_LAYER_HEIGHTS, |value| {
            value >= 0.0
        })
    }
    fn non_negative_u32(&self, key: &str, default: u32) -> Result<u32, SliceError> {
        let Some(value) = self.values.get(key) else {
            return Ok(default);
        };
        let value = match value {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => text.parse().ok(),
            _ => None,
        }
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer")))?;
        if value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value <= u32::MAX as f64 {
            Ok(value as u32)
        } else {
            Err(SliceError::InvalidInput(format!(
                "{key} must be a non-negative integer"
            )))
        }
    }
    fn positive_f64(&self, key: &str, default: f64) -> Result<f64, SliceError> {
        let Some(value) = self.values.get(key) else {
            return Ok(default);
        };
        let Some(value) = value.as_f64() else {
            return Err(SliceError::InvalidInput(format!("{key} must be a number")));
        };
        if !value.is_finite() || value <= 0.0 {
            Err(SliceError::InvalidInput(format!("{key} must be positive")))
        } else if value <= MIN_LAYER_HEIGHT {
            Err(SliceError::InvalidInput(format!(
                "{key} must be greater than 0.000001"
            )))
        } else {
            Ok(value)
        }
    }
    pub(crate) fn bool_option(&self, key: &str, default: bool) -> Result<bool, SliceError> {
        let Some(value) = self.values.get(key) else {
            return Ok(default);
        };
        value
            .as_bool()
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
    }
    pub(crate) fn sparse_infill_line_width(&self) -> Result<f64, SliceError> {
        let nozzle_diameter = self.nozzle_diameters()?[0];
        let configured = self.extrusion_width("sparse_infill_line_width", 0.0, nozzle_diameter)?;
        if configured == 0.0 {
            Ok(nozzle_diameter)
        } else {
            Ok(configured)
        }
    }
    pub(crate) fn extrusion_width(
        &self,
        key: &str,
        default: f64,
        nozzle_diameter: f64,
    ) -> Result<f64, SliceError> {
        let Some(value) = self.values.get(key) else {
            return Ok(default);
        };
        let value = match value {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => parse_extrusion_width_text(text, nozzle_diameter),
            _ => None,
        }
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
        if value.is_finite() && value >= 0.0 {
            Ok(value)
        } else {
            Err(SliceError::InvalidInput(format!(
                "{key} contains invalid value"
            )))
        }
    }
    pub(crate) fn percent(&self, key: &str, default: f64) -> Result<f64, SliceError> {
        self.range_f64(key, default, 0.0, 100.0)
    }
    pub(crate) fn range_f64(
        &self,
        key: &str,
        default: f64,
        min: f64,
        max: f64,
    ) -> Result<f64, SliceError> {
        parsing::parse_range_f64(key, self.values.get(key), default, min, max)
    }
    fn numeric_vector(
        &self,
        key: &str,
        default: &[f64],
        is_valid: impl Fn(f64) -> bool,
    ) -> Result<Vec<f64>, SliceError> {
        let Some(value) = self.values.get(key) else {
            return Ok(default.to_vec());
        };
        let values = parse_numeric_vector(key, value)?;
        if values
            .iter()
            .all(|value| value.is_finite() && is_valid(*value))
        {
            Ok(values)
        } else {
            Err(SliceError::InvalidInput(format!(
                "{key} contains invalid value"
            )))
        }
    }
}
#[rustfmt::skip] #[cfg(test)] mod tests;
