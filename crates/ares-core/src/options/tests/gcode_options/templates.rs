use super::super::super::{
    AmsCounts, CsvTable, ExtruderType, ExtruderTypes, FilamentGCodeSourceOptions, GCodeOptions,
    NozzleType, NozzleVolumeTypes, Nullable, NullableInts, NullableNozzleTypes, OrcaBool,
    OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts,
    OrcaPercents, OrcaString, OrcaStrings, Percent, Point2d, Point2dList,
    PrinterGCodeSourceOptions, ProcessGCodeSourceOptions, ProjectGCodeSourceOptions,
    RammingParameters, RetractLiftEnforce, RetractLiftEnforces, SpaceTuple, VariantStride,
    ZHopType, ZHopTypes,
};

mod opaque;
mod shapes;
mod strings;

fn project(
    printer: &PrinterGCodeSourceOptions,
    process: &ProcessGCodeSourceOptions,
    filament: &FilamentGCodeSourceOptions,
    project: &ProjectGCodeSourceOptions,
) -> GCodeOptions {
    GCodeOptions::from_sources(printer, process, filament, project)
}

fn owned_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
