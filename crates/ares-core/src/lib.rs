#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "O98-O101 Arachne prerequisites activate with concentric internal fill"
    )
)]
mod arachne;
pub mod arrangement;
mod bridge_support;
mod bridges;
mod brims;
mod contours;
mod extrusion_entity;
mod extrusions;
mod fill;
mod gap_fills;
mod gcode;
mod gcode_adaptive_bed_mesh;
mod gcode_auxiliary_fan;
mod gcode_config_header;
mod gcode_filament_stats;
mod gcode_file_start;
mod gcode_finish;
mod gcode_first_layer_print_placeholders;
mod gcode_format;
mod gcode_head_wrap_detect_zone;
mod gcode_header;
mod gcode_input_shaping;
mod gcode_junction_deviation;
mod gcode_layer_change_retraction;
mod gcode_layer_custom;
mod gcode_layer_diagnostics;
mod gcode_layer_fan;
mod gcode_layer_markers;
mod gcode_lift;
mod gcode_line_numbers;
mod gcode_m73;
mod gcode_machine_limits;
mod gcode_machine_start_placeholders;
mod gcode_machine_start_runtime_placeholders;
mod gcode_machine_start_stat_placeholders;
mod gcode_move_buffer;
mod gcode_move_emit;
mod gcode_object_labels;
mod gcode_placeholders;
mod gcode_power_loss_recovery;
mod gcode_pressure_advance;
mod gcode_print_bed_placeholders;
mod gcode_print_move;
pub mod gcode_processor;
mod gcode_reserved_tags;
mod gcode_role_change;
mod gcode_role_fan;
mod gcode_runtime_options;
mod gcode_scan_first_layer;
mod gcode_spiral_vase;
mod gcode_spiral_vase_transition;
mod gcode_start_custom;
mod gcode_startup;
mod gcode_stat_placeholders;
mod gcode_temperature_transition;
pub mod gcode_thumbnails;
mod gcode_travel_retraction;
mod gcode_wipe_before_external_loop;
mod gcode_wipe_on_loops;
mod gcode_wrapping_detection;
mod gcode_writer;
mod gcode_writer_setup;
mod generation;
mod geometry;
mod infills;
mod mesh_slicer;
mod model;
mod model_shrinkage;
mod moves;

mod options;
mod perimeters;
mod pipeline;
mod planning;
pub mod plater;
mod print;
mod print_apply;
mod print_paths;
mod printable_height;
mod profiles;
mod project;
pub(crate) mod project_slice;
mod segments;
mod skirts;
mod slicer_context;
mod speeds;
mod stl;
mod surface;

use std::fmt;

pub use bridges::BridgeOptions;
pub use brims::{BrimOptions, BrimPath, BrimType, LayerBrims, generate_brims};
pub use geometry::{ConvexHull3d, convex_hull_3d};

pub(crate) use contours::stitch_printable;
pub use contours::{Contour, LayerContours, stitch_layer_slices};
pub use extrusion_entity::{ExtrusionEntityCollection, ExtrusionPath, ExtrusionRole};
pub use extrusions::{
    ExtrusionMove, ExtrusionOptions, LayerExtrusionMoves, generate_extrusion_moves,
};
pub use gap_fills::{GapFillPath, LayerGapFills, generate_gap_fills};
pub(crate) use gap_fills::{SolidSurfaceGapFillInput, append_solid_surface_gap_fills};
pub use gcode_processor::{
    GCodeProcessorResult, MoveVertex, PrintEstimatedStatistics, PrintStatistics, ProcessedGCode,
};
pub use gcode_thumbnails::{
    GCodeThumbnailDefinition, GCodeThumbnailFormat, ThumbnailData, ThumbnailParseError,
    parse_thumbnail_definitions, thumbnail_error_string,
};
pub use generation::{GenerationMetadata, ORCA_SLICER_COMPATIBILITY_VERSION};
pub use infills::{InfillPath, InfillRole, LayerInfills, generate_infills};
pub use model::{Model, Point3, Triangle, ZBounds};
pub use moves::{LayerToolpathMoves, ToolpathMove, ToolpathMoveKind, generate_toolpath_moves};
pub use options::bed_exclusion;
pub use options::registry::{
    OptionDefinition, OptionGuiType, OptionMetadata, OptionMode, OptionValueKind,
    extruder_option_keys, extruder_retract_keys, filament_option_keys,
    filament_options_with_variant, filament_retract_keys, option_definition, option_definitions,
    option_metadata, option_metadata_table, print_config_def_json, print_options_with_variant,
    printer_extruder_options, printer_options_with_variant_1, printer_options_with_variant_2,
};
pub use options::{
    AmsCounts, ArcFittingMode, AuthorizationType, BedExcludeVolumeMode, BedTemperatureFormula,
    ConfigOverrides, CsvTable, DefaultBedType, DifferentExtrudersSupport, ExtruderIndexIdMapLookup,
    ExtruderType, ExtruderTypes, ExtruderVariantLists, FilamentGCodeSourceOptions, FilamentOptions,
    FilamentPrintSourceOptions, FilamentRegionSourceOptions, FilamentRetractOverrideOptions,
    FilamentTypeDisplay, FlatMatrix, FloatOrPercent, GCodeFlavor, GCodeOptions, HardwareOptions,
    ImexToolLayout, InfillOptions, InfillPattern, InputShaperType, MachineEnvelopeOptions,
    Millimeters, MultiToMulti2Update, MultiToMultiUpdate, NozzleType, NozzleVolumeType,
    NozzleVolumeTypes, Nullable, NullableFloats, NullableInts, NullableNozzleTypes,
    NullablePrinterTechnologies, ObjectOptions, OrcaBool, OrcaBools, OrcaFloat,
    OrcaFloatOrPercents, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaString, OrcaStrings,
    OrcaUInt, Percent, Point2d, Point2dGroups, Point2dList, PowerLossRecoveryMode, PresetMetadata,
    PrintHostType, PrinterExtruderMultipleFilamentUpdate, PrinterExtruderUpdate,
    PrinterGCodeSourceOptions, PrinterModel, PrinterNotes, PrinterOptions, PrinterRemainingOptions,
    PrinterStructure, PrinterTechnologies, PrinterTechnology, ProcessBrimType,
    ProcessCounterboreHoleBridging, ProcessDraftShield, ProcessEnsureVerticalShellThickness,
    ProcessExtraBridgeLayer, ProcessFuzzySkinMode, ProcessFuzzySkinType, ProcessGCodeSourceOptions,
    ProcessGapFillTarget, ProcessInfillPattern, ProcessInternalBridgeFilter, ProcessIroningType,
    ProcessNoiseType, ProcessObjectSourceOptions, ProcessOptions, ProcessPerimeterGenerator,
    ProcessPrintOrder, ProcessPrintSequence, ProcessPrintSourceOptions, ProcessRegionSourceOptions,
    ProcessSeamPosition, ProcessSeamScarfType, ProcessSkirtType, ProcessSlicingMode,
    ProcessSupportBasePattern, ProcessSupportInterfacePattern, ProcessSupportStyle,
    ProcessSupportType, ProcessTimelapseType, ProcessTopSurfaceExpansionDirection,
    ProcessWallDirection, ProcessWallSequence, ProcessWipeTowerWallType, ProjectBedType,
    ProjectFilamentMapMode, ProjectGCodeSourceOptions, ProjectPresetSourceOptions,
    ProjectPrimeVolumeMode, ProjectPrintSourceOptions, ProjectRuntimeOptions, ProjectSettings,
    RammingParameters, RawOverhangFanThreshold, RegionOptions, RetractLiftEnforce,
    RetractLiftEnforces, SliceOptions, SpaceTuple, ThumbnailDefinitions, VariantStride,
    WipeTowerType, ZHopType, ZHopTypes,
};
pub use options::{
    BeltRotationAxis, FanDirection, FilamentPresetSourceOptions, ImexVizTheme,
    ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode, ProcessCenterOfSurfacePattern,
    ProcessSurfaceFillOrder, ProcessToolChangeOrdering, RemapAxis,
};
pub use options::{
    decode_config_value, decode_config_value_by_key, encode_config_value,
    encode_config_value_by_key,
};
pub use perimeters::{
    LayerPerimeters, PerimeterOptions, PerimeterPath, PerimeterRole, SeamPosition, WallDirection,
    WallGenerator, WallSequence, generate_perimeters,
};
pub use pipeline::slice_stl_validated;
pub use pipeline::{PipelineDiagnostics, PipelineStage, SlicingPipeline, run_slicing_pipeline};
pub use planning::{Layer, plan_layers};
pub use plater::{
    ArrangeScope, ArrangeSettings, PlaterEdits, PlaterPlate, PlaterSession, PlaterSlice,
    PlaterSnapshot, ScaleEdit, StlFile, StlImportOptions, slice_plater_plate,
    slice_plater_with_profiles,
};
pub use print::validate::{validate_print, validate_project_plate, validate_stl};
pub use print::warnings::{PrintWarning, SlicingNotificationType, WarningLevel};
pub use print::{LayerRegion, Print, PrintLayer, PrintObject, PrintRegion, build_print_domain};
pub use project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;

pub use print_paths::{
    LayerPrintPaths, PrintPath, PrintPathInput, PrintPathRole, ShellLayerOptions,
    filter_short_gap_fill_paths, generate_print_paths,
};
pub(crate) use print_paths::{
    finalize_print_paths_with_layer_contours, generate_print_paths_with_bridge_policy,
};
pub use profiles::Placement;
pub use profiles::{
    CliProfileSliceRequest, ComposedProfile, CurrentPrinter, FilamentSlotState, FilteredPresets,
    GuideFilament, GuideModel, InstalledPrinters, LoadSettingsFile, LoadSettingsRequest,
    LoadSettingsSliceRequest, MergedProfile, MergedProfileMetadata, MissingParent,
    ParsedPresetBundle, PresetCatalogHandle, PresetValues, PrinterModelInfo, ProfileFragment,
    ProfileGroupMetadata, ProfileKind, ProfileSelection, ProfileSettings, ProfileSliceRequest,
    SelectablePrinter, compose_profile_fragments, filter_compatible_presets, guide_models,
    merge_profile_fragments, parse_preset_bundle, profile_placed_project, profile_settings,
    slice_with_load_settings, slice_with_profiles, slice_with_profiles_processed,
    validate_load_settings_input, validate_with_profiles,
};
pub use profiles::{
    PresetCollectionState, PresetConfig, PresetOutcome, PresetRecord, PresetSelection,
    load_preset_bundle,
};
pub use profiles::{
    slice_cli_with_profiles_validated, slice_with_profiles_processed_validated,
    slice_with_profiles_validated,
};
pub use project::{
    ConfigSubstitution, CustomGCodeInfo, CustomGCodeMode, LayerConfigRange, PlateMetadata, Point3d,
    Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume,
    ProjectVolumeType, Transform3d, UpwardMachineInput, UpwardMachineSource, load_project,
    project_config_substitutions,
};
pub use project_slice::{
    CliProjectSliceRequest, CliUpwardMachine, SlicedPlate, slice_cli_project_plate_validated,
    slice_project_plate_processed_validated, slice_project_plate_validated,
};
pub use project_slice::{slice_project, slice_project_plate, slice_project_plate_processed};
pub use segments::{LayerSlice, Point2, Segment2, slice_layers};
pub use skirts::{DraftShield, LayerSkirts, SkirtOptions, SkirtPath, SkirtType, generate_skirts};
pub use slicer_context::{SlicerContext, SlicingStatus};
pub use speeds::{
    AccelerationOptions, JerkOptions, LayerSpeedMoves, OverhangSpeedBands, SpeedMove,
    SpeedMoveKinematics, SpeedOptions, generate_speed_moves,
};
pub use surface::{Surface, SurfaceType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFormat {
    ThreeMf,
    Stl,
    Unknown,
}

impl InputFormat {
    pub(crate) fn detect(input: &[u8]) -> Self {
        if input.starts_with(b"PK\x03\x04") {
            Self::ThreeMf
        } else if input.trim_ascii_start().starts_with(b"solid") {
            Self::Stl
        } else {
            Self::Unknown
        }
    }

    fn as_gcode_value(self) -> &'static str {
        match self {
            Self::ThreeMf => "3mf",
            Self::Stl => "stl",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SliceError {
    EmptyInput,
    MissingProjectPlate(u32),
    InvalidInput(String),
    /// Resolved non-FFF technology (`OrcaSlicer.cpp:3619,3627-3633`).
    InvalidPrinterTechnology,
    /// `CLI_POSTPROCESS_NOT_SUPPORTED` (`Utils.hpp:44`): a 3MF whose
    /// embedded `post_process` array is non-empty is rejected under the CLI
    /// (`OrcaSlicer.cpp:1682-1690`).
    PostProcessNotSupported,
    /// `CLI_PROCESS_NOT_COMPATIBLE` (`Utils.hpp:42`): a plain CLI 3MF slice
    /// whose embedded process `print_compatible_printers` does not contain
    /// the current printer's system name (`OrcaSlicer.cpp:2653-2654`).
    ProcessNotCompatible,
    /// `CLI_NO_SUITABLE_OBJECTS` (`Utils.hpp:61`): the selected plate is
    /// empty or has no object fully inside it. The payload keeps the
    /// model-read detail for the stderr channel.
    NoSuitableObjects(String),
    /// `CLI_INVALID_VALUES_IN_3MF` (`Utils.hpp:43`): parameter values
    /// embedded in the 3MF/config fail validation; the payload is the
    /// `key: detail` block for the stderr channel.
    InvalidValuesInProject(String),
    /// `CLI_VALIDATE_ERROR` (`Utils.hpp:62`): `Print::validate` rejection.
    /// `sliced_info` is already populated at the validate point
    /// (`OrcaSlicer.cpp:5981-5987`), so the result.json fields ride along:
    /// plate_index, layer_height (the f32-stored value widened to f64),
    /// wall_loops, sparse_infill_density.
    Validate {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    SequentialPrintCollision {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    LayeredPrintCollision {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    CustomGCode {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    /// OrcaSlicer.cpp:6286-6291: a slicing/export exception, returned as CLI -100.
    Slicing {
        message: String,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
    ProjectSlicingIncomplete,
    UnsupportedProjectFeature(String),
    ToolpathOutsidePrintableArea {
        error_code: u8,
        gcode: Vec<u8>,
        plate_index: u32,
        layer_height: f64,
        wall_loops: i32,
        sparse_infill_density: f64,
    },
}

impl fmt::Display for SliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("slice input is empty"),
            Self::MissingProjectPlate(plate) => write!(f, "project has no plate {plate}"),
            Self::InvalidInput(message) => f.write_str(message),
            Self::InvalidPrinterTechnology => f.write_str("invalid printer_technology"),
            Self::PostProcessNotSupported => {
                f.write_str("post_process is not supported under CLI.")
            }
            Self::ProcessNotCompatible => f.write_str(
                "The selected printer is not compatible with the process preset in the 3mf.",
            ),
            Self::NoSuitableObjects(message) => f.write_str(message),
            Self::InvalidValuesInProject(message) => f.write_str(message),
            Self::Validate { message, .. }
            | Self::SequentialPrintCollision { message, .. }
            | Self::LayeredPrintCollision { message, .. }
            | Self::CustomGCode { message, .. }
            // OrcaSlicer.cpp:6286-6291 forwards ex.what() without decoration.
            | Self::Slicing { message, .. } => f.write_str(message),
            Self::ProjectSlicingIncomplete => f.write_str("ProjectSlicingIncomplete"),
            Self::UnsupportedProjectFeature(feature) => {
                write!(f, "unsupported project feature: {feature}")
            }
            Self::ToolpathOutsidePrintableArea { .. } => f.write_str("Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer."),
        }
    }
}

impl std::error::Error for SliceError {}

pub fn load_model(input: impl AsRef<[u8]>) -> Result<Model, SliceError> {
    let input = input.as_ref();
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }

    match InputFormat::detect(input) {
        InputFormat::Stl => stl::load(input),
        InputFormat::ThreeMf => Err(SliceError::InvalidInput(
            "3MF project input must be loaded with load_project".to_owned(),
        )),
        InputFormat::Unknown if stl::looks_like_binary(input) => stl::load(input),
        InputFormat::Unknown => Err(SliceError::InvalidInput(
            "unsupported or malformed model input".to_owned(),
        )),
    }
}

pub async fn slice(input: impl AsRef<[u8]>, options: SliceOptions) -> Result<Vec<u8>, SliceError> {
    let pipeline = run_slicing_pipeline(input, &options)?;
    gcode::format_gcode(&pipeline, pipeline.options())
}
