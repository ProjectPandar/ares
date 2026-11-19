#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "O98-O101 Arachne prerequisites activate with concentric internal fill"
    )
)]
mod arachne;
pub mod arrangement;
mod bridges;
mod brims;
mod contours;
mod extrusion_entity;
mod extrusions;
mod fill;
mod gap_fills;
mod gcode_junction_deviation;
mod gcode_power_loss_recovery;
pub mod gcode_processor;
mod gcode_spiral_vase;
pub mod gcode_thumbnails;
mod generation;
mod geometry;
mod mesh_slicer;
mod model;

mod options;
mod perimeters;
mod planning;
pub mod plater;
mod print;
mod print_apply;
mod profiles;
mod project;
pub(crate) mod project_slice;
mod segments;
mod skirts;
mod slice_error;
mod slicer_context;
mod stl;
mod surface;

pub use bridges::BridgeOptions;
pub use brims::{BrimOptions, BrimPath, BrimType, LayerBrims, generate_brims};
pub use geometry::{ConvexHull3d, convex_hull_3d};

pub use contours::{Contour, LayerContours, stitch_layer_slices};
pub use extrusion_entity::{ExtrusionEntityCollection, ExtrusionPath, ExtrusionRole};
pub use gap_fills::{GapFillPath, LayerGapFills, generate_gap_fills};
pub use gcode_processor::{
    GCodeProcessorResult, MoveVertex, PrintEstimatedStatistics, PrintStatistics, ProcessedGCode,
};
pub use gcode_thumbnails::{
    GCodeThumbnailDefinition, GCodeThumbnailFormat, ThumbnailData, ThumbnailParseError,
    parse_thumbnail_definitions, thumbnail_error_string,
};
pub use generation::{GenerationMetadata, ORCA_SLICER_COMPATIBILITY_VERSION};
pub use model::{Model, Point3, Triangle, ZBounds};
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
    ImexToolLayout, InputShaperType, MachineEnvelopeOptions, Millimeters, MultiToMulti2Update,
    MultiToMultiUpdate, NozzleType, NozzleVolumeType, NozzleVolumeTypes, Nullable, NullableFloats,
    NullableInts, NullableNozzleTypes, NullablePrinterTechnologies, ObjectOptions, OrcaBool,
    OrcaBools, OrcaFloat, OrcaFloatOrPercents, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents,
    OrcaString, OrcaStrings, OrcaUInt, Percent, Point2d, Point2dGroups, Point2dList,
    PowerLossRecoveryMode, PresetMetadata, PrintHostType, PrinterExtruderMultipleFilamentUpdate,
    PrinterExtruderUpdate, PrinterGCodeSourceOptions, PrinterModel, PrinterNotes, PrinterOptions,
    PrinterRemainingOptions, PrinterStructure, PrinterTechnologies, PrinterTechnology,
    ProcessBrimType, ProcessCounterboreHoleBridging, ProcessDraftShield,
    ProcessEnsureVerticalShellThickness, ProcessExtraBridgeLayer, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessGCodeSourceOptions, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessIroningType, ProcessNoiseType, ProcessObjectSourceOptions,
    ProcessOptions, ProcessPerimeterGenerator, ProcessPrintOrder, ProcessPrintSequence,
    ProcessPrintSourceOptions, ProcessRegionSourceOptions, ProcessSeamPosition,
    ProcessSeamScarfType, ProcessSkirtType, ProcessSlicingMode, ProcessSupportBasePattern,
    ProcessSupportInterfacePattern, ProcessSupportStyle, ProcessSupportType, ProcessTimelapseType,
    ProcessTopSurfaceExpansionDirection, ProcessWallDirection, ProcessWallSequence,
    ProcessWipeTowerWallType, ProjectBedType, ProjectFilamentMapMode, ProjectGCodeSourceOptions,
    ProjectPresetSourceOptions, ProjectPrimeVolumeMode, ProjectPrintSourceOptions,
    ProjectRuntimeOptions, ProjectSettings, RammingParameters, RawOverhangFanThreshold,
    RegionOptions, RetractLiftEnforce, RetractLiftEnforces, SliceOptions, SpaceTuple,
    ThumbnailDefinitions, VariantStride, WipeTowerType, ZHopType, ZHopTypes,
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
pub use planning::{Layer, plan_layers};
pub use plater::{
    ArrangeScope, ArrangeSettings, PlaterEdits, PlaterPlate, PlaterSession, PlaterSlice,
    PlaterSnapshot, ScaleEdit, StlFile, StlImportOptions, slice_plater_plate,
    slice_plater_with_profiles,
};
pub use print::validate::{validate_print, validate_project_plate, validate_stl};
pub use print::warnings::{PrintWarning, SlicingNotificationType, WarningLevel};

pub use profiles::entry::slice_assembly;
pub use project::assembly::{Assembly, AssemblyError, AssemblyFile, AssemblyList};
pub use project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
pub use project_slice::option_stl::{slice_stl_named_validated, slice_stl_validated};

pub use profiles::LoadedPresetSources;
pub use profiles::Placement;
pub use profiles::{
    CliProfileSliceRequest, ComposedProfile, CurrentPrinter, FilamentSlotState, FilteredPresets,
    GuideFilament, GuideModel, InstalledPrinters, LoadSettingsFile, LoadSettingsRequest,
    LoadSettingsSliceRequest, MergedProfile, MergedProfileMetadata, MissingParent,
    ParsedPresetBundle, PresetCatalogHandle, PresetValues, PrinterModelInfo, ProfileFragment,
    ProfileGroupMetadata, ProfileKind, ProfileSelection, ProfileSettings, ProfileSliceRequest,
    SelectablePrinter, SourceIdentity, compose_profile_fragments, filter_compatible_presets,
    guide_models, merge_profile_fragments, parse_preset_bundle, profile_placed_project,
    profile_settings, slice_with_load_settings, slice_with_profiles, slice_with_profiles_processed,
    validate_load_settings_input, validate_with_profiles,
};
pub use profiles::{NativeRefreshFile, NativeRefreshPresets, slice_with_load_settings_and_refresh};
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
    ProjectVolumeType, Transform3d, UpwardMachineInput, UpwardMachineSource,
    export_project_settings, load_project, project_config_substitutions,
};
pub use project::{StlExport, export_3mf, export_stl};
pub use project_slice::{
    CliPlateFailure, CliPlateFailureKind, CliPlatesRequest, CliPlatesSettings,
    CliProjectSliceRequest, CliUpwardMachine, DirectProjectSliceRequest, SlicedPlate,
    slice_cli_project_plate_validated, slice_cli_project_plate_validated_with_catalog,
    slice_cli_project_plate_with_refresh, slice_cli_project_plates,
    slice_project_plate_processed_validated, slice_project_plate_processed_validated_with_catalog,
    slice_project_plate_validated, slice_project_plate_validated_with_catalog,
};
pub use project_slice::{
    slice_project, slice_project_plate, slice_project_plate_processed,
    slice_project_plate_processed_with_catalog, slice_project_plate_with_catalog,
};
pub use segments::{LayerSlice, Point2, Segment2, slice_layers};
pub use skirts::{DraftShield, LayerSkirts, SkirtOptions, SkirtPath, SkirtType, generate_skirts};
pub use slice_error::{PresetAdmissionError, PresetAdmissionReason, SliceError};
pub use slicer_context::{SlicerContext, SlicingStatus};
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
}

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
    // The byte-only API requires a concrete bed for temperature selection.
    // Validated entries leave Default Plate (PrintConfig.cpp:607) to Print::validate.
    if let Some(value) = options.values().get("curr_bed_type") {
        let bed = value
            .as_str()
            .ok_or_else(|| SliceError::InvalidInput("curr_bed_type must be a string".to_owned()))?;
        if options::first_layer_bed_temperature_key_for(bed).is_none() {
            return Err(SliceError::InvalidInput(
                "curr_bed_type has an invalid value".to_owned(),
            ));
        }
    }
    project_slice::option_stl::slice(input.as_ref(), "input.stl", &options, false)
}
