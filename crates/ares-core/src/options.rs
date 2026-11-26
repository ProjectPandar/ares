use crate::SliceError;
macro_rules! option_modules { ($($vis:vis $module:ident),* $(,)?) => { $($vis mod $module;)* }; }
#[rustfmt::skip]
option_modules!(defaults, different_extruders, extruder_count, extruder_index, extruder_variant, fdm_normalization);
pub mod bed_exclusion;
mod config_export;
pub(crate) mod flush_matrix;
mod flush_volume;
pub(crate) use config_export::{
    collect_config_entries, config_block_with_compat_temperatures, escape_scalar_string,
    escape_string_vector_element, write_config_block,
};
pub(crate) use typed_legacy::{LegacyAction, find_explicit_rule, is_obsolete_input};
mod config_types;
mod gcode_fields;
mod gcode_options;
#[rustfmt::skip]
option_modules!(filament_count, filament_type, infill);
pub(crate) mod filament_map_recommend;
mod filament_options;
pub(crate) use filament_options::FilamentOptionsBuilder;
mod bed_temperature;
mod decode;
mod encode;
mod gcode_flavor;
pub(crate) mod json_deserialize;
pub(crate) mod model_config_deserialize;
mod part_cooling_fan;
option_modules!(layer_change_retraction, pub(crate) legacy, parameter_size);
mod object_fields;
mod object_options;
pub(crate) mod option_group;
pub(crate) mod overrides;
pub(crate) mod parsing;
pub(crate) mod power_loss_recovery;
mod preset_metadata;
mod printer_options;
mod process_options;
pub(crate) use {
    printer_options::PrinterOptionsBuilder, process_options::ProcessOptionsBuilder,
    project_runtime_options::ProjectRuntimeOptionsBuilder,
    project_settings::ProjectSettingsBuilder,
};
pub(crate) mod project_config_views;
mod project_deserialize;
pub(crate) mod project_fdm_normalization;
mod project_runtime_options;
mod project_settings;
mod project_variants;
mod region_fields;
mod region_options;

pub mod registry;
mod support_different_extruders;
#[cfg_attr(not(test), allow(dead_code, unused_imports))]
mod typed_legacy;
pub(crate) use typed_legacy::parse_loaded_thumbnail_definitions;
mod update_multi_to_multi;
mod update_printer_extruders;
mod update_single_to_multi;
mod validation;
mod vector_resize;
pub(crate) use bed_temperature::first_layer_bed_temperature_key_for;
pub(crate) use config_types::{format_number, format_number_precision};
mod arc_fitting;
pub use arc_fitting::ArcFittingMode;
pub use decode::{decode_config_value, decode_config_value_by_key};
use defaults::*;
pub use encode::{encode_config_value, encode_config_value_by_key};

pub(crate) use infill::ExtraSolidInfills;
pub(crate) use legacy::handle_legacy;
pub(crate) use model_config_deserialize::{
    deserialize_object_model_field, deserialize_region_model_field,
};
pub(crate) use object_options::ObjectOptionOverrides;
pub use overrides::ConfigOverrides;
use parsing::parse_numeric_vector;
pub(crate) use part_cooling_fan::{
    InternalBridgeFanSpeed, PartCoolingFanRamp, PartCoolingFanRampConfig,
};
pub(crate) use project_variants::{MaterializedProjectVariants, materialize_project_variants};
pub(crate) use project_variants::{
    NozzleInventory, ProcessNozzle, mapped_extruder_id, parse_nozzle_counts,
    process_nozzle_config_index, refresh_filament_map, refresh_filament_variants,
    update_static_filament_maps,
};
#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use region_options::{
    RegionBase, RegionOptionOverrides, RegionOverrideSources, unescape_strings_cstyle,
};
pub use support_different_extruders::DifferentExtrudersSupport;
pub use update_multi_to_multi::{MultiToMulti2Update, MultiToMultiUpdate};
pub use update_printer_extruders::{PrinterExtruderMultipleFilamentUpdate, PrinterExtruderUpdate};
pub use {
    bed_temperature::BedTemperatureFormula,
    config_types::{
        AmsCounts, CsvTable, FlatMatrix, FloatOrPercent, Millimeters, Nullable,
        NullablePrinterTechnologies, OrcaBool, OrcaBools, OrcaBoolsNullable, OrcaFloat,
        OrcaFloatOrPercents, OrcaFloatOrPercentsNullable, OrcaFloats, OrcaFloatsNullable, OrcaInt,
        OrcaInts, OrcaPercents, OrcaString, OrcaStrings, OrcaUInt, Percent, Point2d, Point2dGroups,
        Point2dList, PrinterTechnologies, PrinterTechnology, RammingParameters, SpaceTuple,
        VariantStride,
    },
    filament_options::{
        FilamentGCodeSourceOptions, FilamentOptions, FilamentPresetSourceOptions,
        FilamentPrintSourceOptions, FilamentRegionSourceOptions, FilamentRetractOverrideOptions,
        RawOverhangFanThreshold,
    },
    gcode_flavor::GCodeFlavor,
    gcode_options::GCodeOptions,
    layer_change_retraction::RetractLiftEnforce,
    object_options::ObjectOptions,
    power_loss_recovery::PowerLossRecoveryMode,
    preset_metadata::PresetMetadata,
    printer_options::{
        AuthorizationType, BedExcludeVolumeMode, BeltRotationAxis, DefaultBedType, ExtruderType,
        ExtruderTypes, ExtruderVariantLists, FanDirection, ImexToolLayout, ImexVizTheme,
        InputShaperType, MachineEnvelopeOptions, NozzleType, NozzleVolumeType, NozzleVolumeTypes,
        NullableFloats, NullableInts, NullableNozzleTypes, PrintHostType,
        PrinterGCodeSourceOptions, PrinterModel, PrinterNotes, PrinterOptions,
        PrinterRemainingOptions, PrinterStructure, RemapAxis, RetractLiftEnforces,
        ThumbnailDefinitions, WipeTowerType, ZHopType, ZHopTypes,
    },
    process_options::{
        ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode, ProcessBrimType,
        ProcessCenterOfSurfacePattern, ProcessCounterboreHoleBridging, ProcessDraftShield,
        ProcessEnsureVerticalShellThickness, ProcessExtraBridgeLayer, ProcessFuzzySkinMode,
        ProcessFuzzySkinType, ProcessGCodeSourceOptions, ProcessGapFillTarget,
        ProcessInfillPattern, ProcessInternalBridgeFilter, ProcessIroningType, ProcessNoiseType,
        ProcessObjectSourceOptions, ProcessOptions, ProcessPerimeterGenerator, ProcessPrintOrder,
        ProcessPrintSequence, ProcessPrintSourceOptions, ProcessRegionSourceOptions,
        ProcessSeamPosition, ProcessSeamScarfType, ProcessSkirtType, ProcessSlicingMode,
        ProcessSupportBasePattern, ProcessSupportInterfacePattern, ProcessSupportStyle,
        ProcessSupportType, ProcessSurfaceFillOrder, ProcessTimelapseType,
        ProcessToolChangeOrdering, ProcessTopSurfaceExpansionDirection, ProcessWallDirection,
        ProcessWallSequence, ProcessWipeTowerWallType,
    },
    project_runtime_options::{
        ProjectBedType, ProjectFilamentMapMode, ProjectGCodeSourceOptions,
        ProjectPresetSourceOptions, ProjectPrimeVolumeMode, ProjectPrintSourceOptions,
        ProjectRuntimeOptions,
    },
    project_settings::ProjectSettings,
    region_options::RegionOptions,
};
pub use {extruder_index::ExtruderIndexIdMapLookup, filament_type::FilamentTypeDisplay};
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct SliceOptions {
    values: std::collections::BTreeMap<String, serde_json::Value>,
}
impl SliceOptions {
    pub fn values(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.values
    }

    pub fn is_infill_first(&self) -> Result<bool, SliceError> {
        let Some(value) = self.values.get("is_infill_first") else {
            return Ok(DEFAULT_IS_INFILL_FIRST);
        };
        value
            .as_bool()
            .ok_or_else(|| SliceError::InvalidInput("is_infill_first must be a boolean".to_owned()))
    }

    pub fn nozzle_diameters(&self) -> Result<Vec<f64>, SliceError> {
        self.numeric_vector("nozzle_diameter", DEFAULT_NOZZLE_DIAMETERS, |value| {
            value >= 0.005
        })
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
