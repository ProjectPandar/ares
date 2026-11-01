use super::{
    AmsCounts, ArcFittingMode, BedTemperatureFormula, CsvTable, ExtruderTypes,
    FilamentGCodeSourceOptions, FloatOrPercent, GCodeFlavor, NozzleVolumeTypes, Nullable,
    NullableInts, NullableNozzleTypes, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt,
    OrcaInts, OrcaPercents, OrcaString, OrcaStrings, Percent, Point2dList, PowerLossRecoveryMode,
    PrinterGCodeSourceOptions, PrinterStructure, ProcessGCodeSourceOptions, ProjectFilamentMapMode,
    ProjectGCodeSourceOptions, RammingParameters, RetractLiftEnforces, SpaceTuple, VariantStride,
    WipeTowerType, ZHopTypes, gcode_fields::gcode_option_fields,
};

macro_rules! clone_source_field {
    ($printer:ident, $process:ident, $filament:ident, $project:ident; printer, $field:ident) => {
        $printer.$field.clone()
    };
    ($printer:ident, $process:ident, $filament:ident, $project:ident; process, $field:ident) => {
        $process.$field.clone()
    };
    ($printer:ident, $process:ident, $filament:ident, $project:ident; filament, $field:ident) => {
        $filament.$field.clone()
    };
    ($printer:ident, $process:ident, $filament:ident, $project:ident; project, $field:ident) => {
        $project.$field.clone()
    };
}

macro_rules! declare_gcode_options {
    ($($owner:ident => $field:ident => $key:literal: $ty:ty),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct GCodeOptions {
            $(pub $field: $ty),*
        }

        impl GCodeOptions {
            #[cfg_attr(not(test), allow(dead_code))]
            pub(crate) fn from_sources(
                printer: &PrinterGCodeSourceOptions,
                process: &ProcessGCodeSourceOptions,
                filament: &FilamentGCodeSourceOptions,
                project: &ProjectGCodeSourceOptions,
            ) -> Self {
                Self {
                    $($field: clone_source_field!(
                        printer,
                        process,
                        filament,
                        project;
                        $owner,
                        $field
                    )),*
                }
            }
        }


    };
}

gcode_option_fields!(declare_gcode_options);
