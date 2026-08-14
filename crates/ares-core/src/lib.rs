#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "O98/O99 Arachne prerequisites activate with concentric internal fill"
    )
)]
mod arachne;
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
mod gcode_finish_emit;
mod gcode_first_layer_print_placeholders;
mod gcode_format;
mod gcode_head_wrap_detect_zone;
mod gcode_header;
mod gcode_input_shaping;
mod gcode_junction_deviation;
mod gcode_layer_change_retraction;
mod gcode_layer_custom;
mod gcode_layer_diagnostic_emit;
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
mod print;
mod print_apply;
mod print_paths;
mod printable_height;
mod profiles;
mod project;
mod project_slice;
mod segments;
mod skirts;
mod speeds;
mod stl;
mod surface;

use std::fmt;

pub use bridges::BridgeOptions;
pub use brims::{BrimOptions, BrimPath, BrimType, LayerBrims, generate_brims};
#[cfg(test)]
pub(crate) use contours::make_overhang_printable_contours;
pub(crate) use contours::stitch_printable;
pub use contours::{Contour, LayerContours, stitch_layer_slices};
pub use extrusion_entity::{ExtrusionEntityCollection, ExtrusionPath, ExtrusionRole};
pub use extrusions::{
    ExtrusionMove, ExtrusionOptions, LayerExtrusionMoves, generate_extrusion_moves,
};
pub use gap_fills::{GapFillPath, LayerGapFills, generate_gap_fills};
pub(crate) use gap_fills::{SolidSurfaceGapFillInput, append_solid_surface_gap_fills};
pub use gcode_thumbnails::{
    GCodeThumbnailDefinition, GCodeThumbnailFormat, ThumbnailParseError,
    parse_thumbnail_definitions, thumbnail_error_string,
};
pub use generation::{GenerationMetadata, ORCA_SLICER_COMPATIBILITY_VERSION};
pub use infills::{InfillPath, InfillRole, LayerInfills, generate_infills};
pub use model::{Model, Point3, Triangle, ZBounds};
pub use moves::{LayerToolpathMoves, ToolpathMove, ToolpathMoveKind, generate_toolpath_moves};
pub use options::registry::{
    OptionDefinition, OptionValueKind, extruder_option_keys, extruder_retract_keys,
    filament_option_keys, filament_options_with_variant, filament_retract_keys, option_definition,
    option_definitions, print_options_with_variant, printer_extruder_options,
    printer_options_with_variant_1, printer_options_with_variant_2,
};
pub use options::{
    AmsCounts, AuthorizationType, BedTemperatureFormula, CsvTable, DefaultBedType,
    DifferentExtrudersSupport, ExtruderIndexIdMapLookup, ExtruderType, ExtruderTypes,
    ExtruderVariantLists, FilamentGCodeSourceOptions, FilamentOptions, FilamentPrintSourceOptions,
    FilamentRegionSourceOptions, FilamentRetractOverrideOptions, FilamentTypeDisplay, FlatMatrix,
    FloatOrPercent, GCodeFlavor, GCodeOptions, HardwareOptions, InfillOptions, InfillPattern,
    InputShaperType, MachineEnvelopeOptions, Millimeters, MultiToMulti2Update, MultiToMultiUpdate,
    NozzleType, NozzleVolumeType, NozzleVolumeTypes, Nullable, NullableFloats, NullableInts,
    NullableNozzleTypes, NullablePrinterTechnologies, ObjectOptions, OrcaBool, OrcaBools,
    OrcaFloat, OrcaFloatOrPercents, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaString,
    OrcaStrings, OrcaUInt, Percent, Point2d, Point2dGroups, Point2dList, PowerLossRecoveryMode,
    PresetMetadata, PrintHostType, PrinterExtruderMultipleFilamentUpdate, PrinterExtruderUpdate,
    PrinterGCodeSourceOptions, PrinterModel, PrinterNotes, PrinterOptions, PrinterRemainingOptions,
    PrinterStructure, PrinterTechnologies, PrinterTechnology, ProcessBrimType,
    ProcessCounterboreHoleBridging, ProcessDraftShield, ProcessEnsureVerticalShellThickness,
    ProcessExtraBridgeLayer, ProcessFuzzySkinMode, ProcessFuzzySkinType, ProcessGCodeSourceOptions,
    ProcessGapFillTarget, ProcessInfillPattern, ProcessInternalBridgeFilter, ProcessIroningType,
    ProcessNoiseType, ProcessObjectSourceOptions, ProcessOptions, ProcessPerimeterGenerator,
    ProcessPrintOrder, ProcessPrintSequence, ProcessPrintSourceOptions, ProcessRegionSourceOptions,
    ProcessSeamPosition, ProcessSeamScarfType, ProcessSkirtType, ProcessSlicingMode,
    ProcessSupportBasePattern, ProcessSupportInterfacePattern, ProcessSupportStyle,
    ProcessSupportType, ProcessTimelapseType, ProcessWallDirection, ProcessWallSequence,
    ProcessWipeTowerWallType, ProjectBedType, ProjectFilamentMapMode, ProjectGCodeSourceOptions,
    ProjectPresetSourceOptions, ProjectPrintSourceOptions, ProjectRuntimeOptions, ProjectSettings,
    RammingParameters, RawOverhangFanThreshold, RegionOptions, RetractLiftEnforce,
    RetractLiftEnforces, SliceOptions, SpaceTuple, ThumbnailDefinitions, VariantStride,
    WipeTowerType, ZHopType, ZHopTypes,
};
pub use perimeters::{
    LayerPerimeters, PerimeterOptions, PerimeterPath, PerimeterRole, SeamPosition, WallDirection,
    WallGenerator, WallSequence, generate_perimeters,
};
pub use pipeline::{PipelineDiagnostics, PipelineStage, SlicingPipeline, run_slicing_pipeline};
pub use planning::{Layer, plan_layers};
pub use print::{LayerRegion, Print, PrintLayer, PrintObject, PrintRegion, build_print_domain};
#[cfg(test)]
pub(crate) use print_paths::finalize_print_paths;
pub use print_paths::{
    LayerPrintPaths, PrintPath, PrintPathInput, PrintPathRole, ShellLayerOptions,
    filter_short_gap_fill_paths, generate_print_paths,
};
pub(crate) use print_paths::{
    finalize_print_paths_with_layer_contours, generate_print_paths_with_bridge_policy,
};
pub use profiles::{
    ComposedProfile, MergedProfile, MergedProfileMetadata, ProfileFragment, ProfileGroupMetadata,
    ProfileKind, ProfileSelection, compose_profile_fragments, merge_profile_fragments,
};
pub use project::{
    LayerConfigRange, PlateMetadata, Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel,
    ProjectObject, ProjectVolume, ProjectVolumeType, Transform3d, load_project,
};
pub use project_slice::slice_project;
#[cfg(test)]
pub use project_slice::task22g_browser_oracle;
#[cfg(test)]
pub use project_slice::{task22h_browser_input_oracle, task22h_browser_oracle};
#[cfg(test)]
pub use project_slice::{task22i_browser_input_oracle, task22i_browser_oracle};
#[cfg(test)]
pub use project_slice::{task22j_browser_input_oracle, task22j_browser_oracle};
#[cfg(test)]
pub use project_slice::{
    task22k_browser_input_oracle, task22k_browser_oracle, task22l_browser_input_oracle,
    task22l_browser_oracle,
};
#[cfg(test)]
pub use project_slice::{task22m_browser_input_oracle, task22m_browser_oracle};
#[cfg(any(test, feature = "task22n-browser-oracle"))]
pub use project_slice::{task22n_browser_input_oracle, task22n_browser_oracle};
pub use segments::{LayerSlice, Point2, Segment2, slice_layers};
pub use skirts::{DraftShield, LayerSkirts, SkirtOptions, SkirtPath, SkirtType, generate_skirts};
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
    fn detect(input: &[u8]) -> Self {
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

#[derive(Debug, Eq, PartialEq)]
pub enum SliceError {
    EmptyInput,
    InvalidInput(String),
    ProjectSlicingIncomplete,
    UnsupportedProjectFeature(String),
}

impl fmt::Display for SliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("slice input is empty"),
            Self::InvalidInput(message) => f.write_str(message),
            Self::ProjectSlicingIncomplete => f.write_str("ProjectSlicingIncomplete"),
            Self::UnsupportedProjectFeature(feature) => {
                write!(f, "unsupported project feature: {feature}")
            }
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

#[cfg(test)]
mod tests;
