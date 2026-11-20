//! `Print.cpp:1524-1556,1627-1781::Print::validate` (OrcaSlicer v2.4.2):
//! applied height/width bounds, extruder addressing, then bed compatibility.
mod bed_filament;
mod compacted_tower;
mod estimated_tower;
mod extrusion_width;
mod identity;
mod layered_wrapping;
mod sequential_clearance;

pub(crate) use identity::project_is_bbl;

use std::sync::OnceLock;

use regex::Regex;

use crate::{
    ConfigOverrides, GCodeFlavor, ProcessPrintSequence, Project, ProjectSettings, SliceError,
    SliceOptions,
    project::effective_config::{
        resolve_bounded_project_config, types::BoundedResolvedProjectConfig,
    },
};

/// Applied nozzle, extruder-addressing and bed slices of `Print.hpp:935::Print::validate`.
/// Returns the exact upstream diagnostic in the `Validate` family: the CLI
/// adapter exits with `CLI_VALIDATE_ERROR` (-51, Utils.hpp:62) and the
/// result.json carries the `sliced_info` values populated at the validate
/// point (`OrcaSlicer.cpp:5981-5987`) — the f32-stored layer height and
/// density widen to f64 like upstream's ConfigOptionFloat/Percent reads.
pub fn validate_print(settings: &ProjectSettings, bbl_printer: bool) -> Result<(), SliceError> {
    let into_validate = |error: SliceError| match error {
        SliceError::InvalidInput(message) => SliceError::Validate {
            message,
            plate_index: 1,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        },
        other => other,
    };
    let usage = extrusion_width::flat(settings).map_err(into_validate)?;
    validate_layer_templates(settings, bbl_printer).map_err(into_validate)?;
    bed_filament::validate(settings, bbl_printer, &usage.supported_used_filaments, 1)
        .map_err(into_validate)
}

fn validate_layer_templates(
    settings: &ProjectSettings,
    bbl_printer: bool,
) -> Result<(), SliceError> {
    let gcode = &settings.printer.gcode;
    layer_gcode(
        gcode.use_relative_e_distances.0,
        gcode.gcode_flavor,
        bbl_printer,
        &gcode.before_layer_change_gcode.0,
        &gcode.layer_change_gcode.0,
    )
}

/// Validate an option-only STL using its completed print configuration.
pub fn validate_stl(input: impl AsRef<[u8]>, options: &SliceOptions) -> Result<(), SliceError> {
    use crate::project_slice::option_stl::{self, byte_api_error};

    let (project, _) =
        option_stl::load(input.as_ref(), "input.stl", options).map_err(byte_api_error)?;
    project
        .settings()
        .validate_loaded_config(&project.documents().project_settings_raw, true)?;
    validate_loaded_project(project, None, None).map_err(byte_api_error)
}

/// Apply project configuration and validate before an independent processing call.
pub fn validate_project_plate(
    input: impl AsRef<[u8]>,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
) -> Result<(), SliceError> {
    overrides.validate_cli_config()?;
    let mut project = crate::project::load_project_with(input, overrides)?;
    project
        .settings()
        .validate_loaded_config(&project.documents().project_settings_raw, true)?;
    let filament_count = project.cli_filament_count();
    project.initialize_cli_loaded_project_config(filament_count);
    validate_loaded_project(project, plate, None)
}

pub(crate) fn validate_loaded_project(
    project: Project,
    plate: Option<u32>,
    selected_printer: Option<&str>,
) -> Result<(), SliceError> {
    let project = project.select_plate(plate)?;
    let plate = project.first_plate_id();
    let resolved = resolve_bounded_project_config(&project)?;
    validate_applied_project(&project, &resolved, plate, selected_printer)
}

pub(crate) fn validate_applied_project(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_number: u32,
    selected_printer: Option<&str>,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    let into_validate = |error: SliceError| into_validate_message(error, settings);
    layered_wrapping::validate(project, resolved, plate_number)?;
    // `Print.cpp:1334-1341`: spiral vase admits one copy unless the print
    // runs by object; the adjacent regions check (`:1342-1353`) stays
    // deferred.
    if settings.process.print.spiral_mode.0 {
        let total_copies_count: usize = project
            .objects()
            .iter()
            .map(|object| object.instances().len())
            .sum();
        if total_copies_count > 1
            && settings.process.print.print_sequence != ProcessPrintSequence::ByObject
        {
            return Err(SliceError::Validate {
                message: "Please select \"By object\" print sequence to print multiple objects in spiral vase mode."
                    .to_owned(),
                plate_index: plate_number,
                layer_height: f64::from(settings.process.object.layer_height.0 as f32),
                wall_loops: settings.process.region.wall_loops.0,
                sparse_infill_density: f64::from(
                    settings.process.region.sparse_infill_density.0 as f32,
                ),
            });
        }
    }
    let bbl = project_is_bbl(
        settings,
        &project.documents().project_settings_raw,
        selected_printer,
    )?;
    sequential_clearance::validate(project, resolved, plate_number, bbl)?;
    crate::project_slice::validate_print_height(project, resolved, plate_number)?;
    extrusion_width::applied(resolved).map_err(into_validate)?;
    validate_layer_templates(settings, bbl).map_err(into_validate)?;
    bed_filament::validate(
        settings,
        bbl,
        &resolved.usage.supported_used_filaments,
        plate_number,
    )
    .map_err(into_validate)?;
    estimated_tower::validate(project, resolved, plate_number)
}

/// Wrap an `InvalidInput` rejection from the applied-project validation in
/// the `Validate` family (`CLI_VALIDATE_ERROR`, -51, Utils.hpp:62): the
/// CLI adapter's result.json carries the sliced_info values populated at
/// the validate point (`OrcaSlicer.cpp:5981-5987`).
fn into_validate_message(error: SliceError, settings: &ProjectSettings) -> SliceError {
    match error {
        SliceError::InvalidInput(message) => SliceError::Validate {
            message,
            plate_index: 1,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        },
        other => other,
    }
}

const MISSING_RESET: &str = "Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add \"G92 E0\" to layer_gcode.";

pub(crate) fn layer_gcode(
    relative: bool,
    flavor: GCodeFlavor,
    bbl_printer: bool,
    before: &str,
    layer: &str,
) -> Result<(), SliceError> {
    static ANY: OnceLock<Regex> = OnceLock::new();
    static EXACT: OnceLock<Regex> = OnceLock::new();
    let any = ANY.get_or_init(|| {
        Regex::new(r"(?ms)^[ \t]*[gG]92[ \t]*[eE](0(\.0*)?|\.0+)[ \t]*(;.*)?$")
            .expect("upstream G92 E0 regex")
    });
    let before_any = any.is_match(before);
    let layer_any = any.is_match(layer);
    if relative {
        let exact = EXACT.get_or_init(|| {
            Regex::new(r"(?ms)^[ \t]*G92[ \t]*E(0(\.0*)?|\.0+)[ \t]*(;.*)?$")
                .expect("upstream uppercase G92 E0 regex")
        });
        let before_exact = exact.is_match(before);
        let layer_exact = exact.is_match(layer);
        if before_any && !before_exact {
            return Err(SliceError::InvalidInput("\"G92 E0\" was found in before_layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\".".to_owned()));
        }
        if layer_any && !layer_exact {
            return Err(SliceError::InvalidInput("\"G92 E0\" was found in layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\".".to_owned()));
        }
        if matches!(
            flavor,
            GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware
        ) && !bbl_printer
            && !before_exact
            && !layer_exact
        {
            return Err(SliceError::InvalidInput(MISSING_RESET.to_owned()));
        }
    } else if before_any {
        return Err(SliceError::InvalidInput("\"G92 E0\" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing.".to_owned()));
    } else if layer_any {
        return Err(SliceError::InvalidInput("\"G92 E0\" was found in layer_change_gcode, which is incompatible with absolute extruder addressing.".to_owned()));
    }
    Ok(())
}

/// `OrcaSlicer.cpp:6043-6059`: configured model, selected printer, system printer.
pub(crate) fn is_bbl_printer(model: &str, selected: &str, system: &str) -> bool {
    let name = if !model.is_empty() {
        model
    } else if !selected.is_empty() {
        selected
    } else {
        system
    };
    name.starts_with("Bambu Lab")
}
