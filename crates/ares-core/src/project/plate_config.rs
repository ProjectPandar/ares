//! Plate-scoped configuration overrides from `model_settings.config`.
//!
//! OrcaSlicer stores a small set of options inside each plate's metadata
//! block (`BBS_3MF_Importer::_extract_model_config_from_mb_plate`,
//! `bbs_3mf.cpp:4420-4564`) and applies the plate config over the resolved
//! print config before slicing (`OrcaSlicer.cpp:5979`,
//! `new_print_config.apply(*part_plate->config())`). The plate value
//! therefore wins over `project_settings.config` — e.g. a project-level
//! `filament_map_mode = Manual` is overridden by the plate's
//! `Auto For Flush` and the G-code config block dumps the plate value.

use crate::options::{
    OrcaBool, OrcaInt, OrcaInts, ProcessPrintSequence, ProjectBedType, ProjectFilamentMapMode,
    ProjectSettings,
};

use super::model_settings::Metadata;

/// The plate metadata keys OrcaSlicer deserializes into `PartPlate::config`
/// (`bbs_3mf.cpp:4418-4565`); each maps to the owning settings group field.
pub(super) fn apply_plate_config_overrides(settings: &mut ProjectSettings, metadata: &[Metadata]) {
    let mut bed_type = ProjectBedType::DefaultPlate;
    for item in metadata {
        match item.key.as_str() {
            // `fmmDefault` is never written to the file; every stored value is
            // a concrete mode (`bbs_3mf.cpp:4445-4449` skips only legacy
            // "Auto" — mapped to AutoForFlush here like upstream, which leaves
            // the default for it).
            "filament_map_mode" => {
                if let Some(mode) = parse_map_mode(&item.value) {
                    settings.project.gcode.filament_map_mode = mode;
                }
            }
            // The plate attribute is `filament_maps` (plural;
            // `bbs_3mf.cpp:4451` FILAMENT_MAP_ATTR) applied to the
            // singular `filament_map` config key.
            "filament_maps" | "filament_map" => {
                // Upstream clamps each entry to `>= 1`
                // (`bbs_3mf.cpp:4456-4460`).
                if let Some(map) = parse_int_list(&item.value) {
                    settings.project.gcode.filament_map =
                        OrcaInts(map.iter().map(|&v| OrcaInt(v.max(1))).collect());
                }
            }
            "bed_type" => {
                // bbs_3mf.cpp:4641–4646 initializes btPC before parsing any present value.
                bed_type = parse_bed_type(&item.value).unwrap_or(ProjectBedType::CoolPlate);
            }
            "print_sequence" => {
                if let Some(sequence) = parse_print_sequence(&item.value) {
                    settings.process.print.print_sequence = sequence;
                }
            }
            "first_layer_print_sequence" => {
                if let Some(sequence) = parse_int_list(&item.value) {
                    settings.project.print.first_layer_print_sequence =
                        OrcaInts(sequence.iter().map(|&v| OrcaInt(v)).collect());
                }
            }
            "other_layers_print_sequence" => {
                if let Some(sequence) = parse_int_list(&item.value) {
                    settings.project.print.other_layers_print_sequence =
                        OrcaInts(sequence.iter().map(|&v| OrcaInt(v)).collect());
                }
            }
            "other_layers_print_sequence_nums" => {
                if let Ok(count) = item.value.trim().parse::<i32>() {
                    settings.project.print.other_layers_print_sequence_nums = OrcaInt(count);
                }
            }
            "spiral_mode" => {
                if let Some(vase) = parse_bool(&item.value) {
                    settings.process.print.spiral_mode = OrcaBool(vase);
                }
            }
            _ => {}
        }
    }
    settings.project.print.curr_bed_type = bed_type.resolve(settings.project.print.curr_bed_type);
}

/// The plate's own `filament_map_mode` and `filament_map` (`PartPlate::get_filament_map_mode`,
/// `PartPlate.cpp:5790-5796`, and `get_filament_maps`, `:5815-5822`): `None` (`fmmDefault`) and
/// empty when the plate stores none. The importer clamps each map entry to `>= 1`
/// (`bbs_3mf.cpp:4456-4460`).
pub(super) fn own_filament_map(
    metadata: &[Metadata],
) -> (Option<ProjectFilamentMapMode>, Vec<i32>) {
    let (mut mode, mut map) = (None, Vec::new());
    for item in metadata {
        match item.key.as_str() {
            "filament_map_mode" => mode = parse_map_mode(&item.value).or(mode),
            "filament_maps" | "filament_map" => {
                if let Some(values) = parse_int_list(&item.value) {
                    map = values.into_iter().map(|value| value.max(1)).collect();
                }
            }
            _ => {}
        }
    }
    (mode, map)
}

/// The plate's own `imex_parallel_mode` (`bbs_3mf.cpp:4668-4669`, which also takes the attribute's
/// name from before the IMEX rename, `:381-385`); the last one stored wins.
pub(super) fn own_imex_mode(metadata: &[Metadata]) -> Option<&str> {
    metadata
        .iter()
        .rfind(|item| {
            matches!(
                item.key.as_str(),
                "imex_parallel_mode" | "ixex_parallel_mode"
            )
        })
        .map(|item| item.value.as_str())
}

/// The plate's `locked` attribute (`bbs_3mf.cpp:4637-4640`), read with `std::boolalpha`: only
/// "true" locks; the last one stored wins.
pub(super) fn own_locked(metadata: &[Metadata]) -> bool {
    metadata
        .iter()
        .rfind(|item| item.key == "locked")
        .is_some_and(|item| item.value == "true")
}

fn parse_map_mode(value: &str) -> Option<ProjectFilamentMapMode> {
    match value.trim() {
        "Auto For Flush" | "Auto" => Some(ProjectFilamentMapMode::AutoForFlush),
        "Auto For Match" => Some(ProjectFilamentMapMode::AutoForMatch),
        "Manual" => Some(ProjectFilamentMapMode::Manual),
        _ => None,
    }
}

fn parse_bed_type(value: &str) -> Option<ProjectBedType> {
    match value {
        "Supertack Plate" | "SuperTack Plate" => Some(ProjectBedType::SupertackPlate),
        "Cool Plate" => Some(ProjectBedType::CoolPlate),
        "Engineering Plate" => Some(ProjectBedType::EngineeringPlate),
        "High Temp Plate" => Some(ProjectBedType::HighTempPlate),
        "Textured PEI Plate" => Some(ProjectBedType::TexturedPeiPlate),
        "Textured Cool Plate" => Some(ProjectBedType::TexturedCoolPlate),
        _ => None,
    }
}

fn parse_print_sequence(value: &str) -> Option<ProcessPrintSequence> {
    match value.trim() {
        "by layer" | "by default" => Some(ProcessPrintSequence::ByLayer),
        "by object" => Some(ProcessPrintSequence::ByObject),
        _ => None,
    }
}

fn parse_int_list(value: &str) -> Option<Vec<i32>> {
    let trimmed = value.trim().trim_matches(['[', ']']);
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    // Upstream parses plate lists with `get_vector_from_string`
    // (stringstream `>>` — all ASCII whitespace incl. embedded `&#10;`
    // newlines, `bbs_3mf.cpp:4359-4366`); commas are tolerated.
    trimmed
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| entry.trim().parse::<i32>().ok())
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}
