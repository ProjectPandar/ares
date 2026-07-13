use super::super::super::{
    AmsCounts, BedTemperatureFormula, CsvTable, ExtruderType, ExtruderTypes,
    FilamentGCodeSourceOptions, FloatOrPercent, GCodeFlavor, GCodeOptions, NozzleType,
    NozzleVolumeType, NozzleVolumeTypes, Nullable, NullableInts, NullableNozzleTypes, OrcaBool,
    OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaString, OrcaStrings,
    Percent, Point2d, Point2dList, PowerLossRecoveryMode, PrinterGCodeSourceOptions,
    PrinterStructure, ProcessGCodeSourceOptions, ProjectFilamentMapMode,
    ProjectGCodeSourceOptions, RammingParameters, RetractLiftEnforce, RetractLiftEnforces,
    SpaceTuple, VariantStride, WipeTowerType, ZHopType, ZHopTypes,
};

macro_rules! assert_printer_projection {
    ($field:ident, $sentinel:expr) => {{
        let mut printer = PrinterGCodeSourceOptions::default();
        printer.$field = $sentinel;
        let projected = GCodeOptions::from_sources(
            &printer,
            &ProcessGCodeSourceOptions::default(),
            &FilamentGCodeSourceOptions::default(),
            &ProjectGCodeSourceOptions::default(),
        );
        assert_eq!(projected.$field, printer.$field, stringify!($field));
    }};
}

macro_rules! assert_process_projection {
    ($field:ident, $sentinel:expr) => {{
        let mut process = ProcessGCodeSourceOptions::default();
        process.$field = $sentinel;
        let projected = GCodeOptions::from_sources(
            &PrinterGCodeSourceOptions::default(),
            &process,
            &FilamentGCodeSourceOptions::default(),
            &ProjectGCodeSourceOptions::default(),
        );
        assert_eq!(projected.$field, process.$field, stringify!($field));
    }};
}

macro_rules! assert_filament_projection {
    ($field:ident, $sentinel:expr) => {{
        let mut filament = FilamentGCodeSourceOptions::default();
        filament.$field = $sentinel;
        let projected = GCodeOptions::from_sources(
            &PrinterGCodeSourceOptions::default(),
            &ProcessGCodeSourceOptions::default(),
            &filament,
            &ProjectGCodeSourceOptions::default(),
        );
        assert_eq!(projected.$field, filament.$field, stringify!($field));
    }};
}

macro_rules! assert_project_projection {
    ($field:ident, $sentinel:expr) => {{
        let mut project = ProjectGCodeSourceOptions::default();
        project.$field = $sentinel;
        let projected = GCodeOptions::from_sources(
            &PrinterGCodeSourceOptions::default(),
            &ProcessGCodeSourceOptions::default(),
            &FilamentGCodeSourceOptions::default(),
            &project,
        );
        assert_eq!(projected.$field, project.$field, stringify!($field));
    }};
}

mod defaults;
mod filament;
mod printer;
mod process;
mod project;
